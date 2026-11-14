# Stencil reference material

Supplied by the project owner as the starting point for the Stencil-Stack
compositor. These are **reference sources, not built code**: they are not part
of the Cargo workspace and are kept verbatim.

| File | What it is |
|---|---|
| `stencil-bsp.rs` | The BSP layout engine (the first generation of the leaf shape). |
| `stencil-egui-wgpu-v2.rs` | egui 0.27 prototype: BSP walker, infinite canvas, nested nodes and zones. Its wgpu pipeline is a stub. |
| `qgcp-manifest-builder.rs` | The unified `.qgcp` layout manifest types and validation laws. |
| `qgcp-layout-blueprint.qgcp` | A real package built from those types (zip: `HEADER.json`, `MANIFEST.json`, `SEAL.sig`, …). |
| `stencil-design-chat.txt` | The design conversation the above came out of: Core / Theater / client split, the "behind the curtain" template library, the plugin factory. |

`crates/stencil-core` adopts the data types from these files and is tested
against the real package. Where the sources disagree (three shapes of a BSP
leaf), the `.qgcp` form is canonical and the others load as optional fields.
The owner says the Theater side may differ and will be reconciled through typed
wire packets, and that the real Stencil crate will replace or refine this.
