//! Binary space partitioning layout (the "bones").
//!
//! Axis convention, as in the Stencil-Stack source: `Vertical` divides the
//! **width** (children sit side by side), `Horizontal` divides the **height**
//! (children are stacked). `ratio` is the share given to `child_a`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

/// A node of the layout tree.
///
/// A leaf is a terminal region. Two generations of the format exist and both
/// load: the `.qgcp` form points at a template in the instrument library
/// (`active_template_id`); the earlier standalone form carried `palette`,
/// `content_type` and `elements` directly. All of those fields are optional
/// here, and absent ones are not written back, so files round-trip unchanged.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BspNode {
    Branch {
        axis: SplitAxis,
        ratio: f32,
        child_a: Box<BspNode>,
        child_b: Box<BspNode>,
    },
    Leaf {
        id: String,
        /// Template in the instrument library shown in this leaf.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        active_template_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        palette: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        elements: Vec<String>,
    },
}

/// A leaf resolved to absolute coordinates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolvedPanel {
    pub id: String,
    pub rect: Rect,
    pub active_template_id: Option<String>,
    pub palette: Option<String>,
    pub content_type: Option<String>,
    pub elements: Vec<String>,
}

impl BspNode {
    pub fn leaf(id: &str) -> Self {
        BspNode::Leaf {
            id: id.into(),
            active_template_id: None,
            palette: None,
            content_type: None,
            elements: vec![],
        }
    }

    pub fn branch(axis: SplitAxis, ratio: f32, a: BspNode, b: BspNode) -> Self {
        BspNode::Branch {
            axis,
            ratio,
            child_a: Box::new(a),
            child_b: Box::new(b),
        }
    }

    /// Resolve every leaf to its rectangle inside `rect`. Ratios are clamped
    /// to 0..1 so a bad file cannot invert the layout.
    pub fn resolve_layout(&self, rect: Rect, out: &mut Vec<ResolvedPanel>) {
        match self {
            BspNode::Leaf {
                id,
                active_template_id,
                palette,
                content_type,
                elements,
            } => out.push(ResolvedPanel {
                id: id.clone(),
                rect,
                active_template_id: active_template_id.clone(),
                palette: palette.clone(),
                content_type: content_type.clone(),
                elements: elements.clone(),
            }),
            BspNode::Branch {
                axis,
                ratio,
                child_a,
                child_b,
            } => {
                let r = ratio.clamp(0.0, 1.0);
                let (a, b) = match axis {
                    SplitAxis::Vertical => {
                        let wa = rect.width * r;
                        (
                            Rect::new(rect.x, rect.y, wa, rect.height),
                            Rect::new(rect.x + wa, rect.y, rect.width - wa, rect.height),
                        )
                    }
                    SplitAxis::Horizontal => {
                        let ha = rect.height * r;
                        (
                            Rect::new(rect.x, rect.y, rect.width, ha),
                            Rect::new(rect.x, rect.y + ha, rect.width, rect.height - ha),
                        )
                    }
                };
                child_a.resolve_layout(a, out);
                child_b.resolve_layout(b, out);
            }
        }
    }

    pub fn resolved(&self, rect: Rect) -> Vec<ResolvedPanel> {
        let mut v = vec![];
        self.resolve_layout(rect, &mut v);
        v
    }

    pub fn leaf_ids(&self) -> Vec<&str> {
        match self {
            BspNode::Leaf { id, .. } => vec![id.as_str()],
            BspNode::Branch {
                child_a, child_b, ..
            } => {
                let mut v = child_a.leaf_ids();
                v.extend(child_b.leaf_ids());
                v
            }
        }
    }

    /// Point a leaf at a library template. The layout itself is untouched:
    /// swapping what a leaf shows never moves anything.
    pub fn set_template(&mut self, leaf: &str, template: Option<&str>) -> bool {
        match self {
            BspNode::Leaf {
                id,
                active_template_id,
                ..
            } if id == leaf => {
                *active_template_id = template.map(str::to_owned);
                true
            }
            BspNode::Leaf { .. } => false,
            BspNode::Branch {
                child_a, child_b, ..
            } => child_a.set_template(leaf, template) || child_b.set_template(leaf, template),
        }
    }

    /// Split a leaf in two; the original keeps `child_a`, the new leaf is `child_b`.
    pub fn split_leaf(&mut self, leaf: &str, axis: SplitAxis, new_id: &str) -> bool {
        match self {
            BspNode::Leaf { id, .. } if id == leaf => {
                let old = std::mem::replace(self, BspNode::leaf(new_id));
                *self = BspNode::branch(axis, 0.5, old, BspNode::leaf(new_id));
                true
            }
            BspNode::Leaf { .. } => false,
            BspNode::Branch {
                child_a, child_b, ..
            } => child_a.split_leaf(leaf, axis, new_id) || child_b.split_leaf(leaf, axis, new_id),
        }
    }

    /// Remove a leaf; its sibling takes the space. The root leaf cannot be removed.
    pub fn remove_leaf(&mut self, leaf: &str) -> bool {
        let BspNode::Branch {
            child_a, child_b, ..
        } = self
        else {
            return false;
        };
        if matches!(&**child_a, BspNode::Leaf { id, .. } if id == leaf) {
            *self = (**child_b).clone();
            return true;
        }
        if matches!(&**child_b, BspNode::Leaf { id, .. } if id == leaf) {
            *self = (**child_a).clone();
            return true;
        }
        child_a.remove_leaf(leaf) || child_b.remove_leaf(leaf)
    }
}

pub fn load_layout_from_json(s: &str) -> Result<BspNode, serde_json::Error> {
    serde_json::from_str(s)
}

pub fn export_layout_to_json(n: &BspNode) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(id: &str, palette: &str, ct: &str, el: &str) -> BspNode {
        BspNode::Leaf {
            id: id.into(),
            active_template_id: None,
            palette: Some(palette.into()),
            content_type: Some(ct.into()),
            elements: vec![el.into()],
        }
    }

    /// The unit test shipped with the original `stencil-bsp.rs`.
    #[test]
    fn original_source_test_passes() {
        let layout = BspNode::branch(
            SplitAxis::Vertical,
            0.25,
            named("sidebar", "Umbraeth Black", "nav", "logo"),
            named("workspace", "Light", "canvas", "grid"),
        );
        let r = layout.resolved(Rect::new(0.0, 0.0, 1000.0, 600.0));
        assert_eq!(r.len(), 2);
        assert_eq!(
            (r[0].id.as_str(), r[0].rect),
            ("sidebar", Rect::new(0.0, 0.0, 250.0, 600.0))
        );
        assert_eq!(
            (r[1].id.as_str(), r[1].rect),
            ("workspace", Rect::new(250.0, 0.0, 750.0, 600.0))
        );
    }

    #[test]
    fn horizontal_splits_the_height_and_ratios_clamp() {
        let l = BspNode::branch(
            SplitAxis::Horizontal,
            2.0,
            BspNode::leaf("a"),
            BspNode::leaf("b"),
        );
        let r = l.resolved(Rect::new(10.0, 20.0, 100.0, 50.0));
        assert_eq!(r[0].rect, Rect::new(10.0, 20.0, 100.0, 50.0));
        assert_eq!(r[1].rect.height, 0.0);
    }

    #[test]
    fn leaves_tile_exactly() {
        let mut t = BspNode::leaf("root");
        t.split_leaf("root", SplitAxis::Vertical, "r");
        t.split_leaf("r", SplitAxis::Horizontal, "rb");
        let area: f32 = t
            .resolved(Rect::new(0.0, 0.0, 800.0, 600.0))
            .iter()
            .map(|p| p.rect.width * p.rect.height)
            .sum();
        assert!((area - 480_000.0).abs() < 1.0, "{area}");
        assert_eq!(t.leaf_ids(), vec!["root", "r", "rb"]);
    }

    #[test]
    fn swapping_a_template_never_moves_the_layout() {
        let mut t = BspNode::branch(
            SplitAxis::Vertical,
            0.22,
            BspNode::leaf("side"),
            BspNode::leaf("main"),
        );
        let before = t.resolved(Rect::new(0.0, 0.0, 1000.0, 500.0));
        assert!(t.set_template("main", Some("voxel_canvas")));
        assert!(!t.set_template("nope", Some("x")));
        let after = t.resolved(Rect::new(0.0, 0.0, 1000.0, 500.0));
        assert_eq!(before[0].rect, after[0].rect);
        assert_eq!(before[1].rect, after[1].rect);
        assert_eq!(after[1].active_template_id.as_deref(), Some("voxel_canvas"));
    }

    #[test]
    fn removing_a_leaf_gives_its_space_to_the_sibling() {
        let mut t = BspNode::branch(
            SplitAxis::Vertical,
            0.5,
            BspNode::leaf("a"),
            BspNode::leaf("b"),
        );
        assert!(t.remove_leaf("a"));
        assert_eq!(t, BspNode::leaf("b"));
        assert!(!t.remove_leaf("b"));
    }

    #[test]
    fn json_round_trips() {
        let t = BspNode::branch(
            SplitAxis::Vertical,
            0.3,
            named("a", "p", "c", "e"),
            BspNode::leaf("b"),
        );
        let s = export_layout_to_json(&t).unwrap();
        assert_eq!(load_layout_from_json(&s).unwrap(), t);
    }
}
