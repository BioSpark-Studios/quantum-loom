//! Atom graph: a small prompt-chain / logic graph. Wires carry text or
//! numbers; RUN walks the nodes in topological order. Model nodes are stubs
//! until the local LLM router lands — they pass their prompt through, marked.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Wire {
    Text,
    Num,
    Any,
}

pub struct KindDef {
    pub key: &'static str,
    pub name: &'static str,
    pub family: &'static str,
    pub ch: &'static str,
    pub ins: &'static [(&'static str, Wire)],
    pub outs: &'static [(&'static str, Wire)],
    /// Label of the node's editable field, if it has one.
    pub field: Option<&'static str>,
}

use Wire::{Any, Num, Text};

pub const FAMILIES: [(&str, &str, &str); 4] = [
    ("text", "PROMPT · TEXT", "quill"),
    ("model", "ROUTER · MODEL", "nexus"),
    ("logic", "LOGIC", "axiom"),
    ("modules", "QUANTUM MODULES", "forge"),
];

pub const KINDS: &[KindDef] = &[
    KindDef {
        key: "prompt",
        name: "PROMPT",
        family: "text",
        ch: "myth",
        ins: &[],
        outs: &[("out", Text)],
        field: Some("TEXT"),
    },
    KindDef {
        key: "template",
        name: "TEMPLATE",
        family: "text",
        ch: "myth",
        ins: &[("a", Any), ("b", Any)],
        outs: &[("out", Text)],
        field: Some("TPL"),
    },
    KindDef {
        key: "join",
        name: "JOIN",
        family: "text",
        ch: "myth",
        ins: &[("a", Any), ("b", Any)],
        outs: &[("out", Text)],
        field: Some("SEP"),
    },
    KindDef {
        key: "textout",
        name: "TEXT OUT",
        family: "text",
        ch: "myth",
        ins: &[("in", Any)],
        outs: &[],
        field: None,
    },
    KindDef {
        key: "model",
        name: "MODEL",
        family: "model",
        ch: "bone",
        ins: &[("prompt", Text), ("system", Text)],
        outs: &[("out", Text)],
        field: Some("MODEL"),
    },
    KindDef {
        key: "num",
        name: "POT",
        family: "logic",
        ch: "cool",
        ins: &[],
        outs: &[("out", Num)],
        field: Some("VALUE"),
    },
    KindDef {
        key: "resistor",
        name: "RESISTOR",
        family: "logic",
        ch: "cool",
        ins: &[("in", Num)],
        outs: &[("out", Num)],
        field: Some("SCALE"),
    },
    KindDef {
        key: "diode",
        name: "DIODE",
        family: "logic",
        ch: "cool",
        ins: &[("in", Num)],
        outs: &[("out", Num)],
        field: Some("THRESH"),
    },
    KindDef {
        key: "capacitor",
        name: "CAPACITOR",
        family: "logic",
        ch: "cool",
        ins: &[("in", Num)],
        outs: &[("out", Num)],
        field: Some("SMOOTH"),
    },
    KindDef {
        key: "busout",
        name: "BUS OUT",
        family: "logic",
        ch: "life",
        ins: &[("in", Num)],
        outs: &[],
        field: Some("BUS"),
    },
    // One op node per department; `dept` on the node picks which.
    KindDef {
        key: "dept",
        name: "MODULE",
        family: "modules",
        ch: "forge",
        ins: &[("in", Any), ("mod", Num)],
        outs: &[("out", Any), ("cv", Num)],
        field: Some("BUS"),
    },
];

pub fn kind(key: &str) -> Option<&'static KindDef> {
    KINDS.iter().find(|k| k.key == key)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Val {
    Num(f32),
    Text(String),
}

impl Val {
    pub fn text(&self) -> String {
        match self {
            Val::Text(s) => s.clone(),
            Val::Num(n) => format!("{n:.3}"),
        }
    }
    pub fn num(&self) -> f32 {
        match self {
            Val::Num(n) => *n,
            Val::Text(s) => s.trim().parse().unwrap_or(0.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GNode {
    pub id: String,
    pub kind: String,
    pub x: f32,
    pub y: f32,
    /// The node's editable field.
    #[serde(default)]
    pub text: String,
    /// Department index for `dept` nodes.
    #[serde(default)]
    pub dept: usize,
    /// Houdini-style flags: display, bypass, lock.
    #[serde(default)]
    pub display: bool,
    #[serde(default)]
    pub bypass: bool,
    #[serde(default)]
    pub lock: bool,
    #[serde(skip)]
    pub out: Option<Val>,
    #[serde(skip)]
    pub skipped: bool,
    /// Why a node produced nothing (e.g. the model call failed).
    #[serde(skip)]
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GWire {
    pub from: String,
    pub out: usize,
    pub to: String,
    pub inp: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: Vec<GNode>,
    pub wires: Vec<GWire>,
    #[serde(default)]
    pub next: u32,
}

impl Graph {
    pub fn add(&mut self, kind_key: &str, x: f32, y: f32) -> String {
        self.next += 1;
        let id = format!("n{}", self.next);
        let text = match kind_key {
            "join" => "\\n\\n",
            "template" => "{a} — {b}",
            "num" => "0.5",
            "resistor" => "0.5",
            "diode" => "0.5",
            "capacitor" => "0.8",
            "model" => "local",
            _ => "",
        };
        self.nodes.push(GNode {
            id: id.clone(),
            kind: kind_key.into(),
            x,
            y,
            text: text.into(),
            dept: 0,
            display: false,
            bypass: false,
            lock: false,
            out: None,
            skipped: false,
            error: None,
        });
        id
    }

    pub fn node(&self, id: &str) -> Option<&GNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: &str) -> Option<&mut GNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn remove(&mut self, id: &str) {
        self.nodes.retain(|n| n.id != id);
        self.wires.retain(|w| w.from != id && w.to != id);
    }

    fn wire_type(&self, node: &str, port: usize, output: bool) -> Option<Wire> {
        let k = kind(&self.node(node)?.kind)?;
        if output {
            k.outs.get(port)
        } else {
            k.ins.get(port)
        }
        .map(|p| p.1)
    }

    /// Connect an output to an input. Types must match (or either side be
    /// `Any`), the wire must not close a cycle, and an input takes one wire.
    pub fn connect(
        &mut self,
        from: &str,
        out: usize,
        to: &str,
        inp: usize,
    ) -> Result<(), &'static str> {
        if from == to {
            return Err("a node cannot feed itself");
        }
        let a = self.wire_type(from, out, true).ok_or("no such output")?;
        let b = self.wire_type(to, inp, false).ok_or("no such input")?;
        if a != b && a != Any && b != Any {
            return Err("wire types do not match");
        }
        if self.reaches(to, from) {
            return Err("that wire would close a loop");
        }
        self.wires.retain(|w| !(w.to == to && w.inp == inp));
        self.wires.push(GWire {
            from: from.into(),
            out,
            to: to.into(),
            inp,
        });
        Ok(())
    }

    fn reaches(&self, from: &str, target: &str) -> bool {
        let mut stack = vec![from.to_owned()];
        let mut seen = vec![];
        while let Some(n) = stack.pop() {
            if n == target {
                return true;
            }
            if seen.contains(&n) {
                continue;
            }
            for w in self.wires.iter().filter(|w| w.from == n) {
                stack.push(w.to.clone());
            }
            seen.push(n);
        }
        false
    }

    /// Node ids in dependency order.
    pub fn order(&self) -> Vec<String> {
        let mut indeg: HashMap<&str, usize> =
            self.nodes.iter().map(|n| (n.id.as_str(), 0)).collect();
        for w in &self.wires {
            if let Some(d) = indeg.get_mut(w.to.as_str()) {
                *d += 1;
            }
        }
        let mut ready: Vec<&str> = self
            .nodes
            .iter()
            .map(|n| n.id.as_str())
            .filter(|id| indeg[id] == 0)
            .collect();
        let mut out = vec![];
        while let Some(id) = ready.pop() {
            out.push(id.to_owned());
            for w in self.wires.iter().filter(|w| w.from == id) {
                if let Some(d) = indeg.get_mut(w.to.as_str()) {
                    *d -= 1;
                    if *d == 0 {
                        ready.push(w.to.as_str());
                    }
                }
            }
        }
        out
    }

    /// Evaluate the graph. Returns `(bus address, value)` pairs written by
    /// BUS OUT and module nodes. A bypassed node passes its first input
    /// through; a node whose input was skipped is skipped too.
    pub fn run(&mut self) -> Vec<(String, f32)> {
        self.run_with(&mut |model, _system, prompt| {
            Ok(format!("[{model} · router not connected]\n{prompt}"))
        })
    }

    /// Like [`Graph::run`], with `llm(model, system, prompt)` answering MODEL
    /// nodes. A failed call grounds the node (and everything downstream of it)
    /// and leaves the error text on the node.
    pub fn run_with(
        &mut self,
        llm: &mut dyn FnMut(&str, &str, &str) -> Result<String, String>,
    ) -> Vec<(String, f32)> {
        let mut writes = vec![];
        let mut vals: HashMap<(String, usize), Val> = HashMap::new();
        let mut dead: Vec<String> = vec![];
        for id in self.order() {
            let Some(i) = self.nodes.iter().position(|n| n.id == id) else {
                continue;
            };
            let Some(k) = kind(&self.nodes[i].kind) else {
                continue;
            };
            let mut ins: Vec<Option<Val>> = vec![None; k.ins.len()];
            let mut upstream_dead = false;
            for w in self.wires.iter().filter(|w| w.to == id) {
                upstream_dead |= dead.contains(&w.from);
                if let Some(slot) = ins.get_mut(w.inp) {
                    *slot = vals.get(&(w.from.clone(), w.out)).cloned();
                }
            }
            let n = &mut self.nodes[i];
            n.skipped = upstream_dead;
            n.error = None;
            if upstream_dead {
                n.out = None;
                dead.push(id);
                continue;
            }
            let field = n.text.clone();
            let t = |j: usize| {
                ins.get(j)
                    .cloned()
                    .flatten()
                    .map(|v| v.text())
                    .unwrap_or_default()
            };
            let f = |j: usize| {
                ins.get(j)
                    .cloned()
                    .flatten()
                    .map(|v| v.num())
                    .unwrap_or(0.0)
            };
            let k_field = field.trim().parse::<f32>().unwrap_or(0.5);
            let out = if n.bypass {
                ins.first().cloned().flatten()
            } else {
                match n.kind.as_str() {
                    "prompt" => Some(Val::Text(field.clone())),
                    "template" => {
                        Some(Val::Text(field.replace("{a}", &t(0)).replace("{b}", &t(1))))
                    }
                    "join" => {
                        let sep = field.replace("\\n", "\n");
                        let parts: Vec<String> =
                            [t(0), t(1)].into_iter().filter(|s| !s.is_empty()).collect();
                        Some(Val::Text(parts.join(&sep)))
                    }
                    "textout" => ins.first().cloned().flatten(),
                    "model" => match llm(&field, &t(1), &t(0)) {
                        Ok(text) => Some(Val::Text(text)),
                        Err(e) => {
                            n.error = Some(e);
                            None
                        }
                    },
                    "num" => Some(Val::Num(k_field.clamp(0.0, 1.0))),
                    "resistor" => Some(Val::Num(f(0) * k_field)),
                    "diode" => {
                        if f(0) >= k_field {
                            Some(Val::Num(f(0)))
                        } else {
                            // A diode below threshold grounds everything downstream.
                            None
                        }
                    }
                    "capacitor" => {
                        let prev = n.out.as_ref().map(Val::num).unwrap_or(0.0);
                        Some(Val::Num(
                            prev + (f(0) - prev) * (1.0 - k_field.clamp(0.0, 0.99)),
                        ))
                    }
                    "busout" => ins.first().cloned().flatten(),
                    "dept" => ins
                        .first()
                        .cloned()
                        .flatten()
                        .or(Some(Val::Text(String::new()))),
                    _ => None,
                }
            };
            if out.is_none() && !matches!(n.kind.as_str(), "textout" | "busout") {
                dead.push(id.clone());
            }
            match n.kind.as_str() {
                "busout" if !field.trim().is_empty() => {
                    if let Some(v) = &out {
                        writes.push((field.trim().to_owned(), v.num().clamp(0.0, 1.0)));
                    }
                }
                "dept" => {
                    let cv = f(1);
                    vals.insert((id.clone(), 1), Val::Num(cv));
                    if !field.trim().is_empty() {
                        writes.push((field.trim().to_owned(), cv.clamp(0.0, 1.0)));
                    }
                }
                _ => {}
            }
            if let Some(v) = &out {
                vals.insert((id.clone(), 0), v.clone());
            }
            n.out = out;
        }
        writes
    }

    /// The demo chain: a prompt threaded through department ops into a join.
    pub fn demo() -> Self {
        let mut g = Graph::default();
        let p = g.add("prompt", 20.0, 40.0);
        g.node_mut(&p).unwrap().text = "Tornado over the Hydralis coast at dusk".into();
        let dept = |g: &mut Graph, d: usize, bus: &str, x: f32, y: f32| {
            let id = g.add("dept", x, y);
            let n = g.node_mut(&id).unwrap();
            n.dept = d;
            n.text = bus.into();
            id
        };
        let atlas = dept(&mut g, 0, "atlas.out", 230.0, 30.0);
        let cont = dept(&mut g, 13, "continuum.out", 440.0, 110.0);
        let inst = dept(&mut g, 6, "instinct.out", 230.0, 250.0);
        let join = g.add("join", 650.0, 140.0);
        let comp = dept(&mut g, 11, "composer.out", 860.0, 40.0);
        let quill = dept(&mut g, 9, "quill.out", 860.0, 260.0);
        let out = g.add("textout", 1070.0, 150.0);
        for (a, o, b, i) in [
            (&p, 0, &atlas, 0),
            (&atlas, 0, &cont, 0),
            (&inst, 1, &cont, 1),
            (&cont, 0, &join, 0),
            (&inst, 0, &join, 1),
            (&join, 0, &comp, 0),
            (&join, 0, &quill, 0),
            (&quill, 0, &out, 0),
        ] {
            let _ = g.connect(a, o, b, i);
        }
        g
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_runs_end_to_end() {
        let mut g = Graph::demo();
        assert_eq!(g.wires.len(), 8);
        let writes = g.run();
        let out = g.nodes.iter().find(|n| n.kind == "textout").unwrap();
        assert_eq!(
            out.out.as_ref().unwrap().text(),
            "Tornado over the Hydralis coast at dusk"
        );
        assert!(writes.iter().any(|w| w.0 == "atlas.out"));
    }

    #[test]
    fn rejects_cycles_and_type_mismatches() {
        let mut g = Graph::default();
        let a = g.add("resistor", 0.0, 0.0);
        let b = g.add("resistor", 0.0, 0.0);
        let p = g.add("prompt", 0.0, 0.0);
        g.connect(&a, 0, &b, 0).unwrap();
        assert!(g.connect(&b, 0, &a, 0).is_err());
        assert!(g.connect(&p, 0, &a, 0).is_err());
        assert!(g.connect(&a, 0, &a, 0).is_err());
    }

    #[test]
    fn logic_chain_and_grounding() {
        let mut g = Graph::default();
        let pot = g.add("num", 0.0, 0.0);
        let res = g.add("resistor", 0.0, 0.0);
        let dio = g.add("diode", 0.0, 0.0);
        let out = g.add("busout", 0.0, 0.0);
        g.node_mut(&pot).unwrap().text = "0.8".into();
        g.node_mut(&res).unwrap().text = "0.5".into();
        g.node_mut(&dio).unwrap().text = "0.3".into();
        g.node_mut(&out).unwrap().text = "d0.x.a".into();
        g.connect(&pot, 0, &res, 0).unwrap();
        g.connect(&res, 0, &dio, 0).unwrap();
        g.connect(&dio, 0, &out, 0).unwrap();
        assert_eq!(g.run(), vec![("d0.x.a".to_owned(), 0.4)]);
        // raise the threshold: the diode grounds the bus write
        g.node_mut(&dio).unwrap().text = "0.9".into();
        assert!(g.run().is_empty());
        assert!(g.node(&out).unwrap().skipped);
        // bypass passes the input straight through
        g.node_mut(&dio).unwrap().bypass = true;
        assert_eq!(g.run().len(), 1);
    }

    #[test]
    fn model_nodes_use_the_callback_and_failures_ground_downstream() {
        let mut g = Graph::default();
        let p = g.add("prompt", 0.0, 0.0);
        g.node_mut(&p).unwrap().text = "hi".into();
        let m = g.add("model", 0.0, 0.0);
        g.node_mut(&m).unwrap().text = "llama3.2".into();
        let out = g.add("textout", 0.0, 0.0);
        g.connect(&p, 0, &m, 0).unwrap();
        g.connect(&m, 0, &out, 0).unwrap();
        let mut seen = vec![];
        g.run_with(&mut |model, _sys, prompt| {
            seen.push((model.to_owned(), prompt.to_owned()));
            Ok("pong".into())
        });
        assert_eq!(seen, vec![("llama3.2".to_owned(), "hi".to_owned())]);
        assert_eq!(g.node(&out).unwrap().out.as_ref().unwrap().text(), "pong");
        g.run_with(&mut |_, _, _| Err("Ollama is not reachable".into()));
        assert_eq!(
            g.node(&m).unwrap().error.as_deref(),
            Some("Ollama is not reachable")
        );
        assert!(g.node(&out).unwrap().skipped);
        assert!(g.node(&out).unwrap().out.is_none());
    }

    #[test]
    fn an_input_takes_one_wire() {
        let mut g = Graph::default();
        let a = g.add("num", 0.0, 0.0);
        let b = g.add("num", 0.0, 0.0);
        let r = g.add("resistor", 0.0, 0.0);
        g.connect(&a, 0, &r, 0).unwrap();
        g.connect(&b, 0, &r, 0).unwrap();
        assert_eq!(g.wires.len(), 1);
        assert_eq!(g.wires[0].from, b);
        g.remove(&b);
        assert!(g.wires.is_empty());
    }
}
