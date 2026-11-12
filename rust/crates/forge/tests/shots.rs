//! Headless render of each workspace. Needs a GPU adapter (a software Vulkan
//! one is fine), so the tests only run when `FORGE_GPU_TESTS=1`.
//! Set `FORGE_SHOTS=<dir>` to keep the PNGs.

use egui::{pos2, Vec2};
use egui_kittest::Harness;
use forge::ForgeApp;
use rack_core::store::Session;

fn fresh_root(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("forge-shots-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn gpu() -> bool {
    std::env::var("FORGE_GPU_TESTS").is_ok_and(|v| v == "1")
}

fn harness(
    tag: &'static str,
    size: Vec2,
    setup: impl Fn(&mut ForgeApp) + 'static,
) -> Harness<'static, Option<ForgeApp>> {
    let root = fresh_root(tag);
    Harness::builder()
        .with_size(size)
        .with_max_steps(8)
        .with_step_dt(1.0 / 30.0)
        .wgpu()
        .build_ui_state(
            move |ui, app: &mut Option<ForgeApp>| {
                let app = app.get_or_insert_with(|| {
                    let mut a = ForgeApp::new(ui.ctx(), root.clone());
                    setup(&mut a);
                    a
                });
                app.ui(ui);
            },
            None,
        )
}

fn snap(h: &mut Harness<'static, Option<ForgeApp>>, name: &str) {
    h.run_steps(6);
    let img = h.render().expect("render");
    let lit = img
        .pixels()
        .filter(|p| p.0[0] as u32 + p.0[1] as u32 + p.0[2] as u32 > 120)
        .count();
    assert!(
        lit > 2000,
        "{name}: frame is nearly blank ({lit} lit pixels)"
    );
    if let Ok(dir) = std::env::var("FORGE_SHOTS") {
        std::fs::create_dir_all(&dir).unwrap();
        img.save(format!("{dir}/{name}.png")).unwrap();
    }
}

fn signed_in(a: &mut ForgeApp, app: &str) {
    a.session = Some(Session {
        user: "Operator One".into(),
        conn: "local".into(),
        host: String::new(),
        vault: Some("v-main".into()),
    });
    a.prefs.app = app.into();
}

const SIZE: Vec2 = Vec2::new(1680.0, 1000.0);

#[test]
fn gate() {
    if !gpu() {
        return;
    }
    let mut h = harness("gate", SIZE, |_| {});
    snap(&mut h, "00-gate");
}

#[test]
fn vaults() {
    if !gpu() {
        return;
    }
    let mut h = harness("vaults", SIZE, |a| {
        a.session = Some(Session {
            user: "Operator One".into(),
            conn: "local".into(),
            host: String::new(),
            vault: None,
        })
    });
    snap(&mut h, "01-vaults");
}

#[test]
fn rack() {
    if !gpu() {
        return;
    }
    let mut h = harness("rack", SIZE, |a| signed_in(a, "rack"));
    snap(&mut h, "04-rack");
    // scroll the rack down to the folded devices
    h.hover_at(pos2(900.0, 600.0));
    for _ in 0..6 {
        h.event(egui::Event::MouseWheel {
            source: egui::MouseWheelSource::Wheel,
            unit: egui::MouseWheelUnit::Point,
            delta: Vec2::new(0.0, -160.0),
            modifiers: Default::default(),
            phase: egui::TouchPhase::Move,
        });
        h.run_steps(2);
    }
    snap(&mut h, "05-rack-scrolled");
}

#[test]
fn rack_narrow() {
    if !gpu() {
        return;
    }
    let mut h = harness("narrow", Vec2::new(900.0, 700.0), |a| signed_in(a, "rack"));
    snap(&mut h, "06-rack-narrow");
}

#[test]
fn matrix_and_drawer() {
    if !gpu() {
        return;
    }
    let mut h = harness("matrix", SIZE, |a| {
        signed_in(a, "modular");
        a.prefs.drawer_open = true;
    });
    snap(&mut h, "07-matrix-drawer");
}

#[test]
fn forge_stages() {
    if !gpu() {
        return;
    }
    for (stage, name) in [
        ("library", "02-forge-library"),
        ("case", "02-forge-modules"),
        ("panel", "02-forge-panel"),
        ("layouts", "02-forge-layouts"),
        ("graph", "02-forge-graph"),
    ] {
        let mut h = harness("forge", SIZE, move |a| {
            signed_in(a, "forge");
            a.prefs.stage = stage.into();
            if stage == "case" {
                for k in ["chronicle", "quill", "composer", "dmx"] {
                    let uid = a.state.ids.next("m");
                    a.draft.modules.push(rack_core::model::ModuleRef {
                        uid,
                        kind: k.into(),
                    });
                }
            }
            if stage == "panel" {
                let fp = &mut a.forge_doc.current;
                fp.add_component("p1", "scope", "SIGNAL");
                fp.add_component("p2", "knob", "DRIVE");
                fp.add_component("p3", "envelope", "ENV");
                fp.add_component("p4", "fader", "LEVEL");
            }
            if stage == "graph" {
                a.graph.run();
            }
        });
        snap(&mut h, name);
    }
}

#[test]
fn perform() {
    if !gpu() {
        return;
    }
    let mut h = harness("perform", SIZE, |a| {
        signed_in(a, "perform");
        a.decks.decks[0].playing = true;
        a.decks.decks[0].pos = 40.0;
    });
    snap(&mut h, "03-perform");
}

#[test]
fn settings_drawer_and_faceplate_in_rack() {
    if !gpu() {
        return;
    }
    let mut h = harness("settings", SIZE, |a| {
        signed_in(a, "rack");
        a.prefs.drawer_open = true;
        a.prefs.drawer_tab = "settings".into();
        a.llm_status = "Ollama is not reachable: connection refused".into();
    });
    snap(&mut h, "08-settings");
    let mut h = harness("plate", SIZE, |a| {
        signed_in(a, "rack");
        let fp = &mut a.forge_doc.current;
        fp.add_component("p1", "scope", "SIGNAL");
        fp.add_component("p2", "knob", "DRIVE");
        fp.add_component("p3", "envelope", "ENV");
        fp.add_component("p4", "fader", "LEVEL");
        fp.bind = Some("i-lights".into());
        let id = a.forge_doc.save_current();
        a.state
            .lib
            .iter_mut()
            .find(|i| i.id == "i-lights")
            .unwrap()
            .faceplate = Some(id);
        a.state.rack.retain(|d| d.inst == "i-lights");
    });
    snap(&mut h, "09-faceplate-in-rack");
}
