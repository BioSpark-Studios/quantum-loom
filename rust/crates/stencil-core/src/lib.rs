//! Stencil-Stack data model.
//!
//! * [`bsp`] — the bones: a recursive binary space partition resolved to rectangles.
//! * [`manifest`] — the `.qgcp` layout manifest: layer stacks, the instrument
//!   template library ("behind the curtain") and the Meso voxel substrate.
//! * [`laws`] — the validation laws (octave capacity, three-way alignment,
//!   crest border weights).
//!
//! Everything here is plain serde data. It deliberately knows nothing about
//! egui or wgpu, so the Theater and the client can read the same files.

pub mod bsp;
pub mod faceplate;
pub mod laws;
pub mod manifest;

pub use bsp::{BspNode, Rect, ResolvedPanel, SplitAxis};
pub use manifest::QgcpLayoutManifest;
