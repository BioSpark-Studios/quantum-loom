# Handoff: FORGE / Stencil Rust port

Written for the next agent. Read this first, then `rust/README.md` and `rust/reference/stencil/README.md`.

> **Repo warning.** This work was done inside the **Quantum Loom Web App** repo
> (`BioSpark-Studios/quantum-loom`, branch `claude/design-implementation-0wt4j2`, PR #32) because that was
> where the session started. The owner says it likely belongs somewhere else (a new repo, or local).
> Everything lives under `rust/` and is self-contained, so it can be lifted out as a unit. Nothing outside
> `rust/` was changed, except `.gitignore` (one line for `rust/target/`) and `.github/workflows/rust.yml`.

## 1. What this is

The owner supplied a Claude Design export ("BioSpark Main Rack" / **FORGE** client: 19 pages, about 70 JSX/CSS
files, a `design.md` spec) and asked for it to be implemented. It was converted to a **native Rust app
(egui/eframe + wgpu)** instead of web code.

The goal has since been clarified by the owner:

- **The Forge UI and the Asset Creator are the product being built.** The rack/eurorack was the original goal,
  but an asset/component library plus the Stencil compositor now let users drag and drop to build instruments
  and UI.
- Finished racks and instruments may later move to their own public app. The ecosystem may hold millions of
  assets; a creator market is "not for a while".
- 16 **Canon Instruments** (ATLAS, MYTHOS, ...) are the only ones that ship. They are instruments and also
  workspaces for creating assets (audio, gltf, fbx, ...). Other instruments will be plugins acquired later.
- The shipped product is "a completely procedural simulation... we are building the factory that builds the
  tools."
- For now: **a faithful port of the original design**, then adjust.

## 2. Hard rules from the owner (do not break these)

1. **Naming is canon. Never map, rename or "fix" names.** "Quantum ATLAS is not Terrain! Quantum MYTHOS is not
   Environment!" A previous agent "toned down" the mythic naming and the owner treats that as dangerous. Do not
   assert that a design name equals a canon module (an earlier claim "Eidolon Stage is Animus" was wrong and
   retracted). Flag risky guesses to the owner instead of deciding.
2. **Do not build the Core Instruments yet.** "They need to be built a certain way." Priority is the
   instrument builder, tools and asset library.
3. **No Bevy in this project.** Connect to the Theater (the decoupled rendering engine) only through **typed
   wire packets**. wgpu and egui are fine here. Bevy may only be added if the owner releases publicly, for 3D.
4. **Local-first.** Assume a local Ollama setup. No Firebase/Firestore or other cloud providers unless there is
   absolutely no other way.
5. **No shortcuts or partial builds** where a proper one is possible ("Shortcuts make the trip twice as long").
   Real **WASM plugins** are a requirement eventually.
6. **Confirm before token-intensive builds**, or schedule them. **Do not create Artifacts** unless told to; put
   tables in normal chat.
7. The owner cannot see the app (only screenshots have been sent). Treat anything visual as unverified until
   they have seen it.

## 3. Layout of `rust/`

Cargo workspace. egui/eframe **0.37**, wgpu **30**, rustc 1.97 locally. CI's clippy is **newer (1.99)** than
local, so it flags lints local clippy does not (this already broke CI once: `chunks_exact_to_as_chunks`).

| Crate | Role |
|---|---|
| `rack-core` | Headless core, serde only. Parameter **Bus** (address to Value), 16-step **Engine** with an `AudioBackend` trait seam, catalog (`DEPTS`, `MODULES`, `FX`, `PRESETS`...), rack model and routing, workspace/vault storage, Genesis reader, decks, faceplate `panel`, typed node `graph`, `midi` (pure parsing), `timeline`. |
| `rack-llm` | Ollama client over `ureq` 3: streaming `/api/chat`, `/api/tags`. Tested against a mock TCP server. |
| `stencil-core` | Stencil-Stack data model: `bsp` (layout "bones"), `manifest` (the owner's `.qgcp` types, verbatim), `laws` (octave capacity, three-way alignment, crest border weights), `faceplate` (BSP + per-leaf content). Plain serde so the Theater can share it. |
| `stencil-gpu` | wgpu layer compositor: ping-pong blending (normal, multiply, screen, overlay, dodge, burn) plus procedural WGSL fills (solid, gradient, glow, brushed metal). `blend.rs` is a CPU reference the GPU output is tested against. |
| `instrument-kit` | Design tokens, painter primitives, 90 icons (resvg from `assets/icons.json`), widgets (knobs, faders, pads, ...), visualisers, bundled OFL fonts. |
| `forge` | The app: shell, rack, sections, matrix, drawer, settings, decks, assets, forge designer (five stages), timeline UI. `gpu.rs` bridges the compositor into egui. |

`rust/reference/stencil/` holds the five files the owner uploaded verbatim (`stencil-bsp.rs`,
`stencil-egui-wgpu-v2.rs`, `qgcp-manifest-builder.rs`, `qgcp-layout-blueprint.qgcp`, `stencil-design-chat.txt`).
Reference only; not built. The owner said "you'll need those eventually".

## 4. Architecture decisions worth knowing

- **Headless core, thin UI.** Everything is addressed through the Bus; devices use the prefix
  `devId.moduleUid.key`; shared engine modules use global keys. Routes run at 20 Hz with a bipolar amount.
  The "core frame" is OSC-style addresses and numbers for output adapters.
- **Genesis "disc" model:** Genesis, Mythos, Container, Capsule. One deck plays one Mythos; Containers are the
  track list.
- **Stencil-Stack:** BSP bones plus layer skin. Axis convention: **`Vertical` divides the width** (children
  side by side), `Horizontal` divides the height; `ratio` goes to `child_a`. The Forge editor's `Dir::Row` maps
  to `Vertical`, `Dir::Col` to `Horizontal`. Instrument templates live in `instrument_library`; a leaf points
  at one by `active_template_id`, so swapping a template never moves the layout.
- **Faceplates:** components are **asset-library references placed on leaves** (owner's confirmation). The
  `.qgcp` manifest has no slot for placed components, so they travel in a **sidecar** (`stencil_core::faceplate`)
  next to the manifest. SAVE on the Panel stage writes `stencil-<id>-layout` (valid manifest) and
  `stencil-<id>-content` (sidecar) into the workspace. If the manifest gains a slot, only
  `Faceplate::to_manifest()` changes. The editor still edits `rack_core::panel::Node`; `to_stencil()` /
  `from_stencil()` convert both ways (round-trip tested for all 7 templates).
- **Crest is deliberately not assigned.** Leaf templates have no layers, so no faction crest is chosen. The
  owner wants users to be able to assign the Crest later; do not infer one.
- **GPU path:** `forge::gpu::GpuView` composes on eframe's own device and registers the texture with egui.
  Pane plates (material gradient, brushed grain, accent glow) use it, cached per pane. Without a GPU
  (`app.gpu == None`, as in headless tests) the old painter path is used.
- **`.qgcp`:** a zip with `HEADER.json`, `MANIFEST.json`, `SEAL.sig`, `DEPS.json`, `CHECKSUM.sha256`, `assets/`.
  Only plain-JSON Genesis reading is implemented. Sealing (Argon2id to AES-256-GCM) belongs to the shared `qgcp`
  crate and is not implemented here.
- **egui 0.37 gotcha:** `ui.scope_builder` resets the parent's cursor upward. Use `ui.new_child`
  (`util::col` / `util::row`) for non-advancing child UIs.
- **Name clash:** a module called `body` shadows `instrument_kit::theme::body` (font), so those calls are fully
  qualified in `forge.rs` and `assets.rs`.

## 5. Build, test, verify

```bash
cd rust
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

GPU and screenshot tests are **gated** so plain `cargo test` needs no GPU:

```bash
FORGE_GPU_TESTS=1 \
VK_ICD_FILENAMES=/opt/pw-browsers/chromium-1243/chrome-linux64/vk_swiftshader_icd.json \
FORGE_SHOTS=/some/dir \
cargo test -p forge -p stencil-gpu
```

(`VK_ICD_FILENAMES` is the software Vulkan driver in the cloud container; use a real GPU locally and drop it.)
`FORGE_SHOTS` saves PNGs of the rendered screens. Headless rendering uses `egui_kittest` (`wgpu` feature).
In non-GPU harness tests call `out.textures_delta.clear()` or egui panics with "Dropped TexturesDelta".

**Run the clippy line before every push, and do not chain it with `;`** (a `;` once let a lint failure through
to a push).

## 6. State at handoff

- PR #32 on `BioSpark-Studios/quantum-loom`, branch `claude/design-implementation-0wt4j2`. CI green at
  `5ff3565` (the GPU plate commit) when this was written.
- Verified headlessly (swiftshader) against the design's screenshots; the GPU plate was checked by rendering one
  to an image. **The live app window has never been seen by the owner or by me.**
- The original plan replaced the Electron rack page. That was **undone**: `app/quantum-loom-rack.html`, the
  hub, the library page and the root README are restored to their pre-PR state, so the Loom app is untouched.
  The Rust app is a separate port that sits beside it; nothing in the Loom links to it. The root README no
  longer mentions it.

## 7. Not done, and why

**Waiting on the owner:**
- Naming audit: list every legacy label in `rust/` (for example OROS-V7 TERRAIN/ENVIRONMENT names in the Genesis
  tree and decks, module blurbs like "Terrain source", the design's `law` strings). **List only; change nothing
  without direction.**
- Which BSP leaf shape is canonical. Currently the `.qgcp` form is canonical and the older
  `palette`/`content_type`/`elements` fields load as optional.
- Where the plugin interface and draw protocol should live (to be designed together with the Stencil/Theater).

**Deliberately deferred (owner said not yet):**
- Bespoke faces for Eidolon Stage, Chronoscriptorium, Orchestration Nexus, Hearthstone, Herald, Axiomatic Loom,
  Atom Graph and Atmosphere. These are Core Instrument territory.

**Planned next, suggested order** (confirm token-heavy items first):
1. Component halos and LED glows drawn as compositor layers.
2. A layer-stack editor on the leaf (add, reorder, blend, opacity), then a crest picker.
3. Asset model: content-addressed ids, kind, version, deps, provenance, tags; format abstract until the Vault
   defines it. A scalable asset browser (index plus virtualised lists) to replace the draw-everything Library
   stage.
4. Module Builder (256 BSQM components from the design's `genesis/bsqm-modules.registry.json`, typed ports,
   graph to module definition).
5. Plugin-shaped module registry: built-ins register through the same interface a plugin would. Then WASM host
   groundwork (check `wasmtime` builds first; headless/procedural modules before UI ones). Native Rust plugins
   have no stable ABI, which is why WASM is the target; data-package plugins (manifest + faceplate/template +
   graph) come first and suit the market.
6. Optional CI job producing Linux/Windows/macOS binaries, since the owner cannot otherwise run the app.

**Unverifiable in the cloud container:** real audio (`cpal`) and MIDI (`midir`) device I/O need ALSA headers the
container lacks. Only pure logic exists: the `AudioBackend` trait and `Midi::handle`. Also not built: controller
profiles, the EXTENDED deck layout, art-heavy asset families, drawer tabs PERFORM/STAGE/BROWSER, `.qgcp` zip and
sealed reading.

## 8. Tips for the next agent

- The owner's "design phase" material is proof-of-concept demos. Ask rather than invent when a spec is silent
  (component placement was one such case; the answer was "assets in the library, placed on leaves").
- Things to review together with the owner once the repo is settled: HERALD, STENCIL, and the other Quantum
  Ecosystem apps.
- Full original session transcript, if still available:
  `/root/.claude/projects/-home-user-quantum-loom/6e37b21b-456e-5610-bb17-703fba22f740.jsonl` (container-local;
  it will not survive the container).
