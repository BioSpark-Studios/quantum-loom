///! Stencil-Stack BSP (Binary Space Partitioning) Layout Engine
/// This module implements the core spatial partitioning logic of the Stencil-Stack
/// GPU Compositing Architecture. It allows recursive, non-overlapping layouts to be
/// defined mathematically, parsed from JSON, and resolved into concrete coordinate boundaries
/// which can then be passed to rendering shaders or downstream AI agents.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a physical bounding box in 2D space.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// The axis along which a parent bounding box is divided.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

/// A node in the Binary Space Partitioning tree.
/// In Rust, recursive types must be wrapped in an indirection pointer like `Box<T>`
/// so that the compiler can calculate a finite, deterministic size at compile-time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BspNode {
    /// A branch node splits its allocated space into two sub-regions (child_a and child_b)
    /// along a specified axis, using a normalized split ratio (0.0 to 1.0).
    Branch {
        axis: SplitAxis,
        ratio: f32,
        #[serde(rename = "child_a")]
        child_a: Box<BspNode>,
        #[serde(rename = "child_b")]
        child_b: Box<BspNode>,
    },
    /// A leaf node is a terminal region of space that hosts actual UI elements,
    /// a visual palette style, and maintains custom boundaries.
    Leaf {
        id: String,
        palette: String,
        content_type: String,
        elements: Vec<String>,
    },
}

/// Represents a resolved UI panel with actual absolute pixel boundaries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolvedPanel {
    pub id: String,
    pub rect: Rect,
    pub palette: String,
    pub content_type: String,
    pub elements: Vec<String>,
}

impl BspNode {
    /// Recursively computes the absolute coordinate boundaries (`Rect`) for every
    /// terminal Leaf node in the BSP tree based on an initial root boundary.
    pub fn resolve_layout(&self, current_rect: Rect, resolved_panels: &mut Vec<ResolvedPanel>) {
        match self {
            BspNode::Leaf { id, palette, content_type, elements } => {
                resolved_panels.push(ResolvedPanel {
                    id: id.clone(),
                    rect: current_rect,
                    palette: palette.clone(),
                    content_type: content_type.clone(),
                    elements: elements.clone(),
                });
            }
            BspNode::Branch { axis, ratio, child_a, child_b } => {
                // Ensure the ratio is clamped between 0.0 and 1.0 to prevent layout inversion
                let clamped_ratio = ratio.clamp(0.0, 1.0);

                match axis {
                    SplitAxis::Vertical => {
                        // Split along the X-axis (width is divided)
                        let width_a = current_rect.width * clamped_ratio;
                        let width_b = current_rect.width - width_a;

                        let rect_a = Rect {
                            x: current_rect.x,
                            y: current_rect.y,
                            width: width_a,
                            height: current_rect.height,
                        };

                        let rect_b = Rect {
                            x: current_rect.x + width_a,
                            y: current_rect.y,
                            width: width_b,
                            height: current_rect.height,
                        };

                        child_a.resolve_layout(rect_a, resolved_panels);
                        child_b.resolve_layout(rect_b, resolved_panels);
                    }
                    SplitAxis::Horizontal => {
                        // Split along the Y-axis (height is divided)
                        let height_a = current_rect.height * clamped_ratio;
                        let height_b = current_rect.height - height_a;

                        let rect_a = Rect {
                            x: current_rect.x,
                            y: current_rect.y,
                            width: current_rect.width,
                            height: height_a,
                        };

                        let rect_b = Rect {
                            x: current_rect.x,
                            y: current_rect.y + height_a,
                            width: current_rect.width,
                            height: height_b,
                        };

                        child_a.resolve_layout(rect_a, resolved_panels);
                        child_b.resolve_layout(rect_b, resolved_panels);
                    }
                }
            }
        }
    }
}

/// Helper function to load a layout directly from a JSON string.
pub fn load_layout_from_json(json_str: &str) -> Result<BspNode, serde_json::Error> {
    serde_json::from_str(json_str)
}

/// Helper function to serialize a layout structure to a formatted JSON string.
pub fn export_layout_to_json(root_node: &BspNode) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(root_node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bsp_layout_resolution() {
        // Construct a simple vertical split layout
        let layout = BspNode::Branch {
            axis: SplitAxis::Vertical,
            ratio: 0.25,
            child_a: Box::new(BspNode::Leaf {
                id: "sidebar".to_string(),
                palette: "Umbraeth Black".to_string(),
                content_type: "nav".to_string(),
                elements: vec!["logo".to_string()],
            }),
            child_b: Box::new(BspNode::Leaf {
                id: "workspace".to_string(),
                palette: "Light".to_string(),
                content_type: "canvas".to_string(),
                elements: vec!["grid".to_string()],
            }),
        };

        let initial_rect = Rect { x: 0.0, y: 0.0, width: 1000.0, height: 600.0 };
        let mut resolved = Vec::new();
        layout.resolve_layout(initial_rect, &mut resolved);

        assert_eq!(resolved.len(), 2);
        
        // Sidebar asserts
        assert_eq!(resolved[0].id, "sidebar");
        assert_eq!(resolved[0].rect, Rect { x: 0.0, y: 0.0, width: 250.0, height: 600.0 });

        // Workspace asserts
        assert_eq!(resolved[1].id, "workspace");
        assert_eq!(resolved[1].rect, Rect { x: 250.0, y: 0.0, width: 750.0, height: 600.0 });
    }
}
