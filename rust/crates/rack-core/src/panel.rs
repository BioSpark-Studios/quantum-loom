//! Faceplates: a binary-space-partition tree of panes, each holding placed
//! components. A faceplate attached to an instrument becomes its folded front
//! panel in the rack.

use serde::{Deserialize, Serialize};
use stencil_core as sc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dir {
    /// Children side by side.
    Row,
    /// Children stacked.
    Col,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Comp {
    pub id: String,
    /// Library asset key (`knob`, `fader`, `scope`, …).
    pub kind: String,
    pub label: String,
    /// Position of the component centre inside its pane, 0..1.
    pub x: f32,
    pub y: f32,
    /// Bus address override; defaults to `plate.<plate>.<comp>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pane {
    pub id: String,
    pub title: String,
    pub show_header: bool,
    /// Material key (`inherit`, `alloy`, `void`, `glass`, `screen`, `brass`, …).
    pub material: String,
    /// Signal-colour key of the pane accent.
    pub accent: String,
    pub components: Vec<Comp>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Node {
    Split {
        dir: Dir,
        ratio: f32,
        a: Box<Node>,
        b: Box<Node>,
    },
    Leaf(Pane),
}

/// Normalised rectangle: `[x, y, w, h]` in 0..1 of the faceplate.
pub type NRect = [f32; 4];

impl Node {
    pub fn leaf(id: &str, title: &str, accent: &str) -> Self {
        Node::Leaf(Pane {
            id: id.into(),
            title: title.into(),
            show_header: !title.is_empty(),
            material: "inherit".into(),
            accent: accent.into(),
            components: vec![],
        })
    }

    pub fn split(dir: Dir, ratio: f32, a: Node, b: Node) -> Self {
        Node::Split {
            dir,
            ratio,
            a: Box::new(a),
            b: Box::new(b),
        }
    }

    pub fn panes(&self) -> Vec<&Pane> {
        match self {
            Node::Leaf(p) => vec![p],
            Node::Split { a, b, .. } => {
                let mut v = a.panes();
                v.extend(b.panes());
                v
            }
        }
    }

    pub fn pane_mut(&mut self, id: &str) -> Option<&mut Pane> {
        match self {
            Node::Leaf(p) => (p.id == id).then_some(p),
            Node::Split { a, b, .. } => a.pane_mut(id).or_else(|| b.pane_mut(id)),
        }
    }

    pub fn pane(&self, id: &str) -> Option<&Pane> {
        self.panes().into_iter().find(|p| p.id == id)
    }

    /// Every pane with its normalised rectangle.
    pub fn layout(&self, r: NRect) -> Vec<(&Pane, NRect)> {
        match self {
            Node::Leaf(p) => vec![(p, r)],
            Node::Split { dir, ratio, a, b } => {
                let t = ratio.clamp(0.05, 0.95);
                let (ra, rb) = match dir {
                    Dir::Row => (
                        [r[0], r[1], r[2] * t, r[3]],
                        [r[0] + r[2] * t, r[1], r[2] * (1.0 - t), r[3]],
                    ),
                    Dir::Col => (
                        [r[0], r[1], r[2], r[3] * t],
                        [r[0], r[1] + r[3] * t, r[2], r[3] * (1.0 - t)],
                    ),
                };
                let mut v = a.layout(ra);
                v.extend(b.layout(rb));
                v
            }
        }
    }

    /// Split pane `id` in two; the new pane gets `new_id`. Returns true if found.
    pub fn split_pane(&mut self, id: &str, dir: Dir, new_id: &str) -> bool {
        match self {
            Node::Leaf(p) if p.id == id => {
                let old = Node::Leaf(p.clone());
                let mut fresh = p.clone();
                fresh.id = new_id.into();
                fresh.title = String::new();
                fresh.show_header = false;
                fresh.components.clear();
                *self = Node::split(dir, 0.5, old, Node::Leaf(fresh));
                true
            }
            Node::Leaf(_) => false,
            Node::Split { a, b, .. } => {
                a.split_pane(id, dir, new_id) || b.split_pane(id, dir, new_id)
            }
        }
    }

    /// Remove pane `id`; its sibling takes the freed space. The last pane stays.
    pub fn remove_pane(&mut self, id: &str) -> bool {
        let Node::Split { a, b, .. } = self else {
            return false;
        };
        if matches!(&**a, Node::Leaf(p) if p.id == id) {
            *self = (**b).clone();
            return true;
        }
        if matches!(&**b, Node::Leaf(p) if p.id == id) {
            *self = (**a).clone();
            return true;
        }
        a.remove_pane(id) || b.remove_pane(id)
    }

    /// Set the ratio of the split whose first child contains pane `id` directly.
    pub fn set_ratio_for(&mut self, id: &str, ratio: f32) -> bool {
        match self {
            Node::Leaf(_) => false,
            Node::Split { a, b, ratio: r, .. } => {
                if a.panes().last().is_some_and(|p| p.id == id) {
                    *r = ratio.clamp(0.1, 0.9);
                    true
                } else {
                    a.set_ratio_for(id, ratio) || b.set_ratio_for(id, ratio)
                }
            }
        }
    }
}

impl Node {
    fn to_bsp(
        &self,
        leaves: &mut std::collections::BTreeMap<String, sc::faceplate::LeafContent>,
    ) -> sc::BspNode {
        match self {
            Node::Leaf(p) => {
                leaves.insert(
                    p.id.clone(),
                    sc::faceplate::LeafContent {
                        title: p.title.clone(),
                        show_header: p.show_header,
                        material: p.material.clone(),
                        accent: p.accent.clone(),
                        components: p
                            .components
                            .iter()
                            .map(|c| sc::faceplate::Placed {
                                id: c.id.clone(),
                                asset: c.kind.clone(),
                                label: c.label.clone(),
                                x: c.x,
                                y: c.y,
                                param: c.param.clone(),
                            })
                            .collect(),
                    },
                );
                sc::BspNode::leaf(&p.id)
            }
            Node::Split { dir, ratio, a, b } => sc::BspNode::branch(
                match dir {
                    // children side by side = the width is divided
                    Dir::Row => sc::SplitAxis::Vertical,
                    Dir::Col => sc::SplitAxis::Horizontal,
                },
                *ratio,
                a.to_bsp(leaves),
                b.to_bsp(leaves),
            ),
        }
    }

    fn from_bsp(
        n: &sc::BspNode,
        leaves: &std::collections::BTreeMap<String, sc::faceplate::LeafContent>,
    ) -> Node {
        match n {
            sc::BspNode::Leaf { id, .. } => {
                let c = leaves.get(id).cloned().unwrap_or_default();
                Node::Leaf(Pane {
                    id: id.clone(),
                    title: c.title,
                    show_header: c.show_header,
                    material: if c.material.is_empty() {
                        "inherit".into()
                    } else {
                        c.material
                    },
                    accent: c.accent,
                    components: c
                        .components
                        .into_iter()
                        .map(|p| Comp {
                            id: p.id,
                            kind: p.asset,
                            label: p.label,
                            x: p.x,
                            y: p.y,
                            param: p.param,
                        })
                        .collect(),
                })
            }
            sc::BspNode::Branch {
                axis,
                ratio,
                child_a,
                child_b,
            } => Node::Split {
                dir: match axis {
                    sc::SplitAxis::Vertical => Dir::Row,
                    sc::SplitAxis::Horizontal => Dir::Col,
                },
                ratio: *ratio,
                a: Box::new(Node::from_bsp(child_a, leaves)),
                b: Box::new(Node::from_bsp(child_b, leaves)),
            },
        }
    }
}

impl Faceplate {
    /// The same faceplate as a Stencil document (bones + leaf content).
    pub fn to_stencil(&self) -> sc::faceplate::Faceplate {
        let mut leaves = Default::default();
        let layout = self.tree.to_bsp(&mut leaves);
        sc::faceplate::Faceplate {
            id: self.id.clone(),
            name: self.name.clone(),
            preset: self.preset.clone(),
            material: self.material.clone(),
            layout,
            leaves,
            bind: self.bind.clone(),
        }
    }

    /// Rebuild from a Stencil document. `next` is recomputed from the ids in use.
    pub fn from_stencil(d: &sc::faceplate::Faceplate) -> Self {
        let tree = Node::from_bsp(&d.layout, &d.leaves);
        let next = tree
            .panes()
            .iter()
            .flat_map(|p| {
                std::iter::once(p.id.as_str()).chain(p.components.iter().map(|c| c.id.as_str()))
            })
            .filter_map(|id| id.get(1..)?.parse::<u32>().ok())
            .max()
            .unwrap_or(0);
        Faceplate {
            id: d.id.clone(),
            name: d.name.clone(),
            preset: d.preset.clone(),
            material: d.material.clone(),
            tree,
            bind: d.bind.clone(),
            next,
        }
    }
}

/// `(key, label, aspect w/h)`; `auto` fills the stage.
pub const PRESETS: [(&str, &str, f32); 5] = [
    ("rack1u", "RACK 1U", 19.0 / 1.75),
    ("rack2u", "RACK 2U", 19.0 / 3.5),
    ("theater", "THEATER 16:9", 16.0 / 9.0),
    ("euro", "EURORACK 12HP", 0.48),
    ("auto", "FILL", 0.0),
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Faceplate {
    pub id: String,
    pub name: String,
    pub preset: String,
    /// Device material the panes inherit.
    pub material: String,
    pub tree: Node,
    /// Instrument this faceplate is the front panel of.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bind: Option<String>,
    #[serde(default)]
    pub next: u32,
}

impl Faceplate {
    pub fn aspect(&self) -> f32 {
        PRESETS
            .iter()
            .find(|p| p.0 == self.preset)
            .map(|p| p.2)
            .unwrap_or(0.0)
    }

    pub fn fresh_id(&mut self, prefix: &str) -> String {
        self.next += 1;
        format!("{prefix}{}", self.next)
    }

    pub fn split(&mut self, pane: &str, dir: Dir) -> Option<String> {
        let id = self.fresh_id("p");
        self.tree.split_pane(pane, dir, &id).then_some(id)
    }

    pub fn add_component(&mut self, pane: &str, kind: &str, label: &str) -> Option<String> {
        let id = self.fresh_id("c");
        let p = self.tree.pane_mut(pane)?;
        // Stagger new components so they do not land on top of each other.
        let n = p.components.len() as f32;
        p.components.push(Comp {
            id: id.clone(),
            kind: kind.into(),
            label: label.into(),
            x: if n < 1.0 {
                0.5
            } else {
                (0.2 + (n % 3.0) * 0.3).min(0.85)
            },
            y: (0.5 + (n / 3.0).floor() * 0.3).min(0.85),
            param: None,
        });
        Some(id)
    }

    pub fn remove_component(&mut self, comp: &str) {
        fn walk(n: &mut Node, comp: &str) {
            match n {
                Node::Leaf(p) => p.components.retain(|c| c.id != comp),
                Node::Split { a, b, .. } => {
                    walk(a, comp);
                    walk(b, comp);
                }
            }
        }
        walk(&mut self.tree, comp);
    }

    pub fn component_count(&self) -> usize {
        self.tree.panes().iter().map(|p| p.components.len()).sum()
    }

    /// Bus address a component writes to.
    pub fn address(&self, c: &Comp) -> String {
        c.param
            .clone()
            .unwrap_or_else(|| format!("plate.{}.{}", self.id, c.id))
    }
}

/// Partition templates offered on the LAYOUTS stage: `(key, name, group)`.
pub const TEMPLATES: [(&str, &str); 7] = [
    ("strip1u", "1U UTILITY STRIP"),
    ("voice2u", "2U SYNTH VOICE"),
    ("screen", "SCREEN + CONTROL DECK"),
    ("mixer4", "4-CH MIXER"),
    ("euro12", "EURORACK 12HP"),
    ("stepseq", "STEP SEQUENCER"),
    ("blank", "BLANK"),
];

pub fn template(key: &str) -> Faceplate {
    use Dir::{Col, Row};
    let leaf = Node::leaf;
    let (preset, tree) = match key {
        "strip1u" => (
            "rack1u",
            Node::split(
                Row,
                0.18,
                leaf("p1", "IN", "cool"),
                Node::split(
                    Row,
                    0.78,
                    leaf("p2", "UTILITY", "amber"),
                    leaf("p3", "OUT", "life"),
                ),
            ),
        ),
        "voice2u" => (
            "rack2u",
            Node::split(
                Row,
                0.5,
                Node::split(
                    Row,
                    0.5,
                    leaf("p1", "OSCILLATOR", "cool"),
                    leaf("p2", "FILTER", "myth"),
                ),
                Node::split(
                    Row,
                    0.5,
                    leaf("p3", "ENVELOPE", "amber"),
                    leaf("p4", "AMP", "life"),
                ),
            ),
        ),
        "screen" => (
            "theater",
            Node::split(
                Col,
                0.62,
                leaf("p1", "SCREEN", "life"),
                Node::split(
                    Row,
                    0.5,
                    leaf("p2", "CONTROL A", "cool"),
                    leaf("p3", "CONTROL B", "rose"),
                ),
            ),
        ),
        "mixer4" => (
            "rack2u",
            Node::split(
                Row,
                0.5,
                Node::split(
                    Row,
                    0.5,
                    leaf("p1", "CH 1", "cool"),
                    leaf("p2", "CH 2", "myth"),
                ),
                Node::split(
                    Row,
                    0.5,
                    leaf("p3", "CH 3", "amber"),
                    leaf("p4", "CH 4", "bone"),
                ),
            ),
        ),
        "euro12" => (
            "euro",
            Node::split(
                Col,
                0.3,
                leaf("p1", "SCREEN", "cool"),
                Node::split(
                    Col,
                    0.72,
                    leaf("p2", "CONTROLS", "cool"),
                    leaf("p3", "PATCH", "amber"),
                ),
            ),
        ),
        "stepseq" => (
            "rack1u",
            Node::split(
                Row,
                0.14,
                leaf("p1", "CLOCK", "amber"),
                Node::split(
                    Row,
                    0.82,
                    leaf("p2", "STEPS", "amber"),
                    leaf("p3", "OUT", "life"),
                ),
            ),
        ),
        _ => ("rack2u", leaf("p1", "", "cool")),
    };
    let name = TEMPLATES
        .iter()
        .find(|t| t.0 == key)
        .map(|t| t.1)
        .unwrap_or("BLANK");
    Faceplate {
        id: String::new(),
        name: name.into(),
        preset: preset.into(),
        material: "alloy".into(),
        tree,
        bind: None,
        next: 10,
    }
}

/// The Forge working set: the faceplate being edited and the saved library.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForgeDoc {
    pub current: Faceplate,
    pub plates: Vec<Faceplate>,
    #[serde(default)]
    pub ids: u32,
}

impl Default for ForgeDoc {
    fn default() -> Self {
        let mut current = template("voice2u");
        current.name = "Untitled Faceplate".into();
        Self {
            current,
            plates: vec![],
            ids: 0,
        }
    }
}

impl ForgeDoc {
    /// Save the current faceplate into the library; returns its id.
    pub fn save_current(&mut self) -> String {
        if self.current.id.is_empty() {
            self.ids += 1;
            self.current.id = format!("fp{}", self.ids);
        }
        let id = self.current.id.clone();
        match self.plates.iter_mut().find(|p| p.id == id) {
            Some(slot) => *slot = self.current.clone(),
            None => self.plates.push(self.current.clone()),
        }
        id
    }

    pub fn plate(&self, id: &str) -> Option<&Faceplate> {
        self.plates.iter().find(|p| p.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_tile_the_unit_square() {
        for (key, _) in TEMPLATES {
            let f = template(key);
            let area: f32 = f
                .tree
                .layout([0.0, 0.0, 1.0, 1.0])
                .iter()
                .map(|(_, r)| r[2] * r[3])
                .sum();
            assert!((area - 1.0).abs() < 1e-4, "{key}: {area}");
        }
        assert_eq!(template("voice2u").tree.panes().len(), 4);
    }

    #[test]
    fn split_and_remove_round_trip() {
        let mut f = template("blank");
        let before = f.tree.clone();
        let new = f.split("p1", Dir::Row).unwrap();
        assert_eq!(f.tree.panes().len(), 2);
        assert!(f.split("nope", Dir::Col).is_none());
        assert!(f.tree.remove_pane(&new));
        assert_eq!(f.tree, before);
        // the last pane cannot be removed
        assert!(!f.tree.remove_pane("p1"));
    }

    #[test]
    fn components_get_addresses() {
        let mut d = ForgeDoc::default();
        let c = d.current.add_component("p1", "knob", "DRIVE").unwrap();
        let id = d.save_current();
        let p = d.plate(&id).unwrap();
        let comp = &p.tree.pane("p1").unwrap().components[0];
        assert_eq!(p.address(comp), format!("plate.{id}.{c}"));
        d.current.remove_component(&c);
        assert_eq!(d.current.component_count(), 0);
        // saving again updates in place
        d.save_current();
        assert_eq!(d.plates.len(), 1);
    }

    #[test]
    fn serializes() {
        let d = ForgeDoc::default();
        let s = serde_json::to_string(&d).unwrap();
        assert_eq!(serde_json::from_str::<ForgeDoc>(&s).unwrap(), d);
    }
}

#[cfg(test)]
mod stencil_tests {
    use super::*;

    #[test]
    fn every_template_round_trips_through_stencil() {
        for (key, _) in TEMPLATES {
            let plate = template(key);
            let back = Faceplate::from_stencil(&plate.to_stencil());
            assert_eq!(back.tree, plate.tree, "{key}");
            assert_eq!(back.preset, plate.preset);
            assert_eq!(back.component_count(), plate.component_count());
        }
    }

    #[test]
    fn manifest_is_valid_and_resolves_every_pane() {
        for (key, _) in TEMPLATES {
            let plate = template(key);
            let m = plate.to_stencil().to_manifest();
            assert!(m.validate().is_empty(), "{key}: {:?}", m.validate());
            let r = m.layout_tree.resolved(sc::Rect::new(0.0, 0.0, 1.0, 1.0));
            assert_eq!(r.len(), plate.tree.panes().len(), "{key}");
            // JSON file survives a round trip
            let json = sc::bsp::export_layout_to_json(&m.layout_tree).unwrap();
            assert_eq!(
                sc::bsp::load_layout_from_json(&json).unwrap(),
                m.layout_tree
            );
        }
    }

    #[test]
    fn row_is_width_division() {
        let mut plate = template("strip1u");
        let a = plate.tree.panes()[0].id.clone();
        let before = plate
            .to_stencil()
            .layout
            .resolved(sc::Rect::new(0.0, 0.0, 1.0, 1.0))
            .into_iter()
            .find(|p| p.id == a)
            .unwrap()
            .rect;
        let b = plate.split(&a, Dir::Row).unwrap();
        let r = plate
            .to_stencil()
            .layout
            .resolved(sc::Rect::new(0.0, 0.0, 1.0, 1.0));
        let (ra, rb) = (
            r.iter().find(|p| p.id == a).unwrap().rect,
            r.iter().find(|p| p.id == b).unwrap().rect,
        );
        assert_eq!(ra.y, rb.y);
        assert_eq!(ra.height, rb.height);
        assert!((ra.width + rb.width - before.width).abs() < 1e-6);
        assert!((ra.height - before.height).abs() < 1e-6);
        assert!(
            rb.x >= ra.x + ra.width - 1e-6,
            "second pane sits to the right"
        );
    }
}
