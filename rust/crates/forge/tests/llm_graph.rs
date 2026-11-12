//! End to end: RUN on the Node Graph calls a (mock) Ollama from a worker
//! thread and the result lands on the node. No GPU needed.

use forge::ForgeApp;
use rack_core::graph::Graph;
use rack_core::store::Session;
use std::io::{Read, Write};
use std::net::TcpListener;

fn mock_ollama(reply: &'static str) -> String {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap();
    std::thread::spawn(move || {
        if let Ok((mut s, _)) = l.accept() {
            let mut req = vec![];
            let mut buf = [0u8; 4096];
            loop {
                let n = s.read(&mut buf).unwrap_or(0);
                req.extend_from_slice(&buf[..n]);
                let t = String::from_utf8_lossy(&req).to_lowercase();
                if let Some(h) = t.find("\r\n\r\n") {
                    let want = t
                        .lines()
                        .find_map(|l| {
                            l.strip_prefix("content-length:")
                                .and_then(|v| v.trim().parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                    if req.len() >= h + 4 + want {
                        break;
                    }
                }
                if n == 0 {
                    break;
                }
            }
            let _ = write!(
                s,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}",
                reply.len()
            );
        }
    });
    format!("http://{addr}")
}

fn frame(ctx: &egui::Context, app: &mut ForgeApp) {
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(1680.0, 1000.0),
        )),
        ..Default::default()
    };
    let mut out = ctx.run_ui(input, |ui| app.ui(ui));
    // No renderer here: the texture uploads are intentionally dropped.
    out.textures_delta.clear();
}

fn app(tag: &str, endpoint: String) -> (egui::Context, ForgeApp) {
    let ctx = egui::Context::default();
    let root = std::env::temp_dir().join(format!("forge-llm-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut a = ForgeApp::new(&ctx, root);
    a.session = Some(Session {
        user: "T".into(),
        conn: "local".into(),
        host: String::new(),
        vault: Some("v-main".into()),
    });
    a.settings.llm.endpoint = endpoint;
    a.settings.llm.timeout_s = 5;
    (ctx, a)
}

fn graph() -> Graph {
    let mut g = Graph::default();
    let p = g.add("prompt", 0.0, 0.0);
    g.node_mut(&p).unwrap().text = "say pong".into();
    let m = g.add("model", 0.0, 0.0);
    let o = g.add("textout", 0.0, 0.0);
    g.connect(&p, 0, &m, 0).unwrap();
    g.connect(&m, 0, &o, 0).unwrap();
    g
}

fn wait(ctx: &egui::Context, a: &mut ForgeApp) {
    for _ in 0..200 {
        frame(ctx, a);
        if a.jobs.is_empty() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("job did not finish");
}

#[test]
fn run_calls_ollama_and_fills_the_node() {
    let ep = mock_ollama("{\"message\":{\"content\":\"po\"},\"done\":false}\n{\"message\":{\"content\":\"ng\"},\"done\":true}\n");
    let (ctx, mut a) = app("ok", ep);
    a.graph = graph();
    a.run_graph();
    wait(&ctx, &mut a);
    let out = a.graph.nodes.iter().find(|n| n.kind == "textout").unwrap();
    assert_eq!(out.out.as_ref().map(|v| v.text()).as_deref(), Some("pong"));
    assert!(a.fui.msg.starts_with("ran 3 nodes"), "{}", a.fui.msg);
}

#[test]
fn unreachable_ollama_marks_the_node_instead_of_hanging() {
    let (ctx, mut a) = app("down", "http://127.0.0.1:1".into());
    a.graph = graph();
    a.run_graph();
    wait(&ctx, &mut a);
    let m = a.graph.nodes.iter().find(|n| n.kind == "model").unwrap();
    assert!(
        m.error.as_deref().unwrap_or("").contains("not reachable"),
        "{:?}",
        m.error
    );
    assert!(
        a.graph
            .nodes
            .iter()
            .find(|n| n.kind == "textout")
            .unwrap()
            .skipped
    );
    assert!(a.fui.msg.contains("failed"), "{}", a.fui.msg);
}

#[test]
fn test_connection_lists_models() {
    let ep = mock_ollama("{\"models\":[{\"name\":\"llama3.2:latest\"}]}");
    let (ctx, mut a) = app("tags", ep);
    a.refresh_models();
    wait(&ctx, &mut a);
    assert_eq!(a.llm_models, vec!["llama3.2:latest"]);
    assert!(a.llm_status.starts_with("connected"), "{}", a.llm_status);
}

#[test]
fn settings_persist_across_launches() {
    let (ctx, mut a) = app("persist", "http://127.0.0.1:1".into());
    a.settings.llm.model = "qwen2.5:7b".into();
    a.autosave();
    let root = a.ws.root().to_path_buf();
    drop(a);
    let b = ForgeApp::new(&ctx, root);
    assert_eq!(b.settings.llm.model, "qwen2.5:7b");
}
