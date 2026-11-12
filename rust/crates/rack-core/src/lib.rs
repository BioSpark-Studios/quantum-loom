//! Headless core of the BioSpark Main Rack.
//!
//! No UI, GPU or async dependencies: `cargo tree -p rack-core` must never show
//! egui, wgpu or a runtime. The UI talks to this crate through addresses and
//! numbers; output adapters interpret the same [`model::CoreFrame`].

pub mod bus;
pub mod catalog;
pub mod decks;
pub mod engine;
pub mod genesis;
pub mod graph;
pub mod model;
pub mod panel;
pub mod store;

pub use bus::{Bus, Value};
pub use engine::Engine;
pub use model::{Device, Instrument, RackState, Route};
pub use store::{Session, Vault, Workspace};
