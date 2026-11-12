use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// CORE DATA MODELS FOR THE UNIFIED QGCP ECOSYSTEM COMPOSITOR
// ============================================================================

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
    pub parent_ids: Option<Vec<String>>,
    pub root_id: Option<String>,
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
// BSP TREE SKELETON (THE BONES)
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BspNode {
    Branch {
        axis: SplitAxis,
        ratio: f32,
        #[serde(rename = "child_a")]
        child_a: Box<BspNode>,
        #[serde(rename = "child_b")]
        child_b: Box<BspNode>,
    },
    Leaf {
        id: String,
        /// The ID of the visual instrument design to pull "from behind the curtain"
        active_template_id: Option<String>,
    },
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
    Particles { system_type: String, particle_count: usize },
    WgpuShader { shader_id: String },
    ExternalEngine { viewport_id: String },
    NestedBsp(Box<BspNode>),
}

// ============================================================================
// SYSTEM VALIDATION ENGINE
// ============================================================================

pub struct ValidationEngine;

impl ValidationEngine {
    /// Enforces the Octave Capacity Law on container insertions.
    pub fn assert_capacity(meta: &mut CapacityMetadata) -> Result<(), &'static str> {
        let max_limit = 2u32.pow(meta.current_octave);
        if meta.child_count >= max_limit {
            match meta.growth_mode {
                GrowthMode::Dynamic => {
                    meta.current_octave += 1;
                    meta.max_children_current = 2u32.pow(meta.current_octave);
                    Ok(())
                }
                GrowthMode::Fixed => Err("Capacity Violation: Container is full at fixed octave ceiling."),
            }
        } else {
            Ok(())
        }
    }

    /// Verifies three-way alignment during creation birth state.
    /// Dimension Alignment Rules:
    /// - Level 0: GenesisContainer -> must align with birth type Seal.
    /// - Level 1: MythosContainer  -> must align with birth type Crest.
    /// - Level 2: Container        -> must align with birth type Glyph/Device/Emblem.
    /// - Level 3: Capsule          -> must align with birth type Trait/Mark/Token/Sigil.
    pub fn validate_three_way_alignment(
        structural_level: u32,
        symbolic_birth_type: HeraldicPosition,
    ) -> Result<(), &'static str> {
        match (structural_level, symbolic_birth_type) {
            (0, HeraldicPosition::Seal) => Ok(()),
            (1, HeraldicPosition::Crest) => Ok(()),
            (2, HeraldicPosition::Glyph) | (2, HeraldicPosition::Device) | (2, HeraldicPosition::Emblem) => Ok(()),
            (3, HeraldicPosition::Trait) | (3, HeraldicPosition::Mark) | (3, HeraldicPosition::Token) | (3, HeraldicPosition::Sigil) => Ok(()),
            _ => Err("Three-Way Alignment Failure: Structural depth level mismatches the symbolic birth authority!"),
        }
    }

    /// Enforces Crest Border Weight Law.
    /// Lore parameters are strict and cannot be compromised for aesthetics.
    pub fn validate_crest_lore(crest: FactionCrest, border_weight: f32) -> Result<(), &'static str> {
        if (border_weight - crest.border_weight()).abs() > f32::EPSILON {
            Err("Lore Violation: Border weight does not align with canonical faction specifications.")
        } else {
            Ok(())
        }
    }
}

// ============================================================================
// MAIN SETUP AND ENTRY POINT FOR TESTING
// ============================================================================

fn main() {
    println!("--- Initializing Quantum Genesis Schema Validator ---");

    // Test Case 1: Lore border verification
    println!("Testing Faction Crest Lore rules...");
    let result_syntaran = ValidationEngine::validate_crest_lore(FactionCrest::Syntaran, 2.0);
    assert_eq!(result_syntaran, Ok(()));
    println!("✅ Syntaran Crest Border 2.0pt weight validated.");

    let result_invalid = ValidationEngine::validate_crest_lore(FactionCrest::Luminarite, 1.5);
    assert!(result_invalid.is_err());
    println!("✅ Invalid Luminarite border weight (1.5pt instead of 0.25pt) successfully caught and rejected.");

    // Test Case 2: Three-Way Alignment Validation
    println!("\nTesting Three-Way Alignment logic...");
    let alignment_ok = ValidationEngine::validate_three_way_alignment(3, HeraldicPosition::Sigil);
    assert_eq!(alignment_ok, Ok(()));
    println!("✅ Alignment of Capsule (Level 3) with Sigil birth weight successfully validated.");

    let alignment_err = ValidationEngine::validate_three_way_alignment(3, HeraldicPosition::Crest);
    assert!(alignment_err.is_err());
    println!("✅ Alignment breach (born at Crest-weight on Capsule-level structure) successfully rejected.");

    // Test Case 3: Octave Capacity Check
    println!("\nTesting Octave Capacity Law on Dynamic Containers...");
    let mut capacity = CapacityMetadata {
        growth_mode: GrowthMode::Dynamic,
        current_octave: 4, // 16 children ceiling
        max_children_current: 16,
        child_count: 16, // full
    };

    println!("Triggering insertion to full dynamic container...");
    let cap_result = ValidationEngine::assert_capacity(&mut capacity);
    assert!(cap_result.is_ok());
    println!("✅ Dynamic container automatically scaled to Octave 5 (Ceiling limit: {}).", capacity.max_children_current);

    println!("\n🎉 All unified structural and symbolic constraints validated successfully!");
}
