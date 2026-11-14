//! Shared instrument kit: design tokens, materials, atomic controls, glass
//! screens and the icon set. App-neutral — it imports no app crate, and every
//! control binds to plain values the caller owns.

pub mod icons;
pub mod paint;
pub mod theme;
pub mod viz;
pub mod widgets;

pub use theme::Channel;
