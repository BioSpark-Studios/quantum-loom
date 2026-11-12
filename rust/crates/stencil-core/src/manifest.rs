//! The `.qgcp` layout manifest, as supplied by the project owner
//! (`reference/stencil/qgcp-manifest-builder.rs`). The types are unchanged
//! except that `BspNode` / `SplitAxis` come from [`crate::bsp`] (which accepts
//! both generations of the leaf) and optional fields are not written when
//! absent, so a file round-trips without gaining `null`s.

use crate::bsp::BspNode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the global Quantum Genesis Container Package (.qgcp) manifest.
/// Combines the structural layout bones, backstage library templates, and
/// the spatial Meso Voxel 3D simulation coordinates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QgcpLayoutManifest {
    pub genesis_id: String,
    pub name: String,
    pub domain: String,
    pub lifecycle: LifecycleState,
    pub capacity: CapacityMetadata,
    pub bdna_signature: String,

    /// The structural division of screen space (The visual layout "Bones")
    pub layout_tree: BspNode,

    /// Backstage library templates representing the visual designs "behind the curtain"
    pub instrument_library: HashMap<String, InstrumentTemplate>,

    /// The spatial Meso simulation substrate representing localized 3D coordinate grids
    pub meso_voxels: Vec<MesoVoxel>,
}

// ============================================================================
// SYSTEM LIFECYCLE & CAPACITY METADATA (OCTAVE CAPACITY LAW)
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LifecycleState {
    Seeding,
    Active,
    Sealed,
    Archived,
    Deprecated,
}

/// Dynamic capacity tracking complying with the Octave Capacity Law.
/// Binary recursive scaling defines depths (Octave 4 -> 16 default limit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityMetadata {
    pub growth_mode: GrowthMode,
    pub current_octave: u32,
    pub max_children_current: u32,
    pub child_count: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GrowthMode {
    Fixed,
    Dynamic,
}

// ============================================================================
// GENETIC B-DNA & CONTRACTUAL COVENANTS
// ============================================================================

/// Tracks genetic lineage, copy depth, and immutable contractual covenants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BDnaRecord {
    pub signature: String,
    pub generation: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub covenant: Option<Covenant>,
    pub sealed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Covenant {
    pub bindings: Vec<CovenantBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CovenantBinding {
    pub name: String,
    pub scope: CovenantScope,
    pub enforceable: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CovenantScope {
    Local,
    Inherited,
    CrossContext,
}

// ============================================================================
// SHADER COMPOSITING & BLEND MODES
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Dodge,
    Burn,
}

// ============================================================================
// THREE-SCALE VOXEL SYSTEM (MESO 3D SPACE)
// ============================================================================

/// Binds 3D physical layout coordinates with cognitive/behavioral state data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MesoVoxel {
    pub voxel_id: String, // format: "x,y,z"
    pub coordinates: VoxelPosition,
    pub layer: String, // e.g. "ground", "underground", "cloud_peaks"
    pub faction_influence: HashMap<String, f32>,
    pub bound_nodes: Vec<VoxelNode>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoxelPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// A computational/narrative node floating within the 3D voxel substrate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelNode {
    pub node_id: String,
    pub name: String,
    pub heraldry: HeraldryState,
    pub bdna: BDnaRecord,
}

// ============================================================================
// HERALDRY SYSTEM & CANONICAL FACTIONS
// ============================================================================

/// Tracks birth and current active positions on the dynamic resonance scale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeraldryState {
    pub birth: HeraldicPosition,
    pub current: HeraldicPosition,
    pub peak: HeraldicPosition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nadir: Option<HeraldicPosition>,
    pub crest: FactionCrest,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HeraldicPosition {
    Seal,
    Crest,
    Glyph,
    Device,
    Emblem,
    Sigil,
    Trait,
    Mark,
    Token,
    Trace,
}

/// The Five Canonical Factions serving as high-level visual and mechanical knobs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FactionCrest {
    Luminarite,
    Venturan,
    Sylvanid,
    Hydralis,
    Syntaran,
}

impl FactionCrest {
    /// Returns the lore-defined border weight in points (pt).
    pub fn border_weight(&self) -> f32 {
        match self {
            FactionCrest::Luminarite => 0.25,
            FactionCrest::Venturan => 1.2,
            FactionCrest::Sylvanid => 1.6,
            FactionCrest::Hydralis => 1.8,
            FactionCrest::Syntaran => 2.0,
        }
    }

    /// Returns the canonical Stratum index.
    pub fn stratum(&self) -> i32 {
        match self {
            FactionCrest::Luminarite => 2,
            FactionCrest::Venturan => 1,
            FactionCrest::Sylvanid => 0,
            FactionCrest::Hydralis => -1,
            FactionCrest::Syntaran => -2,
        }
    }

    /// Returns the primary alchemical hex color code.
    pub fn hex_color(&self) -> &'static str {
        match self {
            FactionCrest::Luminarite => "#EDD878",
            FactionCrest::Venturan => "#00BFFF",
            FactionCrest::Sylvanid => "#2ECC71",
            FactionCrest::Hydralis => "#1A8FD1",
            FactionCrest::Syntaran => "#D4890A",
        }
    }
}

// ============================================================================
// BACKSTAGE INSTRUMENT LIBRARY (THE SKIN & SOUL)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentTemplate {
    pub name: String,
    pub layers: Vec<UiLayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayer {
    pub name: String,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub renderer: RendererSource,
    pub faction_crest: FactionCrest,
    pub border_weight: f32,
    pub color_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RendererSource {
    StaticUiPanel,
    InfiniteCanvas,
    Particles {
        system_type: String,
        particle_count: usize,
    },
    WgpuShader {
        shader_id: String,
    },
    ExternalEngine {
        viewport_id: String,
    },
    NestedBsp(Box<BspNode>),
}

impl QgcpLayoutManifest {
    /// Every leaf must name a template that exists, and every layer must
    /// carry its faction's border weight. Returns all problems, not just the first.
    pub fn validate(&self) -> Vec<String> {
        let mut problems = vec![];
        for id in self.layout_tree.leaf_ids() {
            let panel = self
                .layout_tree
                .resolved(crate::bsp::Rect::new(0.0, 0.0, 1.0, 1.0));
            let tpl = panel
                .iter()
                .find(|p| p.id == id)
                .and_then(|p| p.active_template_id.clone());
            if let Some(t) = tpl {
                if !self.instrument_library.contains_key(&t) {
                    problems.push(format!("leaf '{id}' points at missing template '{t}'"));
                }
            }
        }
        for (key, tpl) in &self.instrument_library {
            for l in &tpl.layers {
                if let Err(e) = crate::laws::validate_crest_lore(l.faction_crest, l.border_weight) {
                    problems.push(format!("template '{key}' layer '{}': {e}", l.name));
                }
                if !(0.0..=1.0).contains(&l.opacity) {
                    problems.push(format!(
                        "template '{key}' layer '{}': opacity {} outside 0..1",
                        l.name, l.opacity
                    ));
                }
            }
        }
        for v in &self.meso_voxels {
            for n in &v.bound_nodes {
                if n.heraldry.birth != n.heraldry.current
                    && n.heraldry.nadir.is_none()
                    && n.heraldry.peak == n.heraldry.birth
                {
                    // birth and current differ but history says nothing moved
                    problems.push(format!(
                        "node '{}': heraldry history is inconsistent",
                        n.node_id
                    ));
                }
            }
        }
        problems
    }
}
