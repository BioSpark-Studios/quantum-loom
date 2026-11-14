//! A faceplate in Stencil terms: the BSP bones plus what is placed on each
//! leaf. Placed components are references into the asset library (`asset`),
//! positioned inside their leaf; the library owns what they look like.
//!
//! The `.qgcp` layout manifest has no slot for placed components yet, so they
//! travel in this sidecar document beside it. When the manifest grows one,
//! only [`Faceplate::to_manifest`] needs to change.

use crate::bsp::BspNode;
use crate::manifest::{
    CapacityMetadata, GrowthMode, InstrumentTemplate, LifecycleState, QgcpLayoutManifest,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// A library asset placed on a leaf.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Placed {
    pub id: String,
    /// Asset-library key (`knob`, `fader`, `scope`, ...).
    pub asset: String,
    pub label: String,
    /// Centre of the component inside the leaf, 0..1.
    pub x: f32,
    pub y: f32,
    /// Bus address override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
}

/// Everything a leaf holds besides its position.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LeafContent {
    pub title: String,
    pub show_header: bool,
    pub material: String,
    pub accent: String,
    pub components: Vec<Placed>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Faceplate {
    pub id: String,
    pub name: String,
    pub preset: String,
    pub material: String,
    pub layout: BspNode,
    /// Content by leaf id.
    pub leaves: BTreeMap<String, LeafContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bind: Option<String>,
}

/// Template key a leaf points at.
pub fn template_key(leaf: &str) -> String {
    format!("leaf.{leaf}")
}

impl Faceplate {
    /// The bones as a `.qgcp` layout manifest. Every leaf points at a
    /// template named for it; the templates carry no layers because choosing a
    /// faction crest for a leaf is a design decision, not something to infer.
    pub fn to_manifest(&self) -> QgcpLayoutManifest {
        let mut layout = self.layout.clone();
        let ids: Vec<String> = layout.leaf_ids().into_iter().map(String::from).collect();
        for id in &ids {
            layout.set_template(id, Some(&template_key(id)));
        }
        let library: HashMap<String, InstrumentTemplate> = ids
            .iter()
            .map(|id| {
                let name = self
                    .leaves
                    .get(id)
                    .map(|c| c.title.clone())
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| id.clone());
                (
                    template_key(id),
                    InstrumentTemplate {
                        name,
                        layers: vec![],
                    },
                )
            })
            .collect();
        QgcpLayoutManifest {
            genesis_id: self.id.clone(),
            name: self.name.clone(),
            domain: "faceplate".into(),
            lifecycle: LifecycleState::Seeding,
            capacity: CapacityMetadata {
                growth_mode: GrowthMode::Dynamic,
                current_octave: 4,
                max_children_current: 16,
                child_count: library.len() as u32,
            },
            bdna_signature: String::new(),
            layout_tree: layout,
            instrument_library: library,
            meso_voxels: vec![],
        }
    }
}
