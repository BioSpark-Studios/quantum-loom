# Quantum Quill → Rust (egui + wgpu) — Revised Conversion Plan

## Context / corrected identity
- This repo is **Quantum Quill** (narrative remixing DAW), mislabeled "Quantum Loom". **Loom** is a different app (autonomous agents + patterning). There is one Atlas, one Loom, one Quill.
- Everything in `app/` is Quill **except** the 16 `quantum-atlas-*.html` modules, which are removed: Atlas becomes its own cartography/terrain app, and the other modules (Environment/Mythos, Architect, …; written Module/Crest) are likewise standalone apps. Naming rule: Atlas is the *crest* of the Terrain module and Mythos is the *crest* of the Environment module, so always say module and crest together when it could be ambiguous in the same ecosystem.
- **Naming rule:** the crest name is the real identity (Quill, Atlas, Mythos, Chronicle, Composer, Loom, …). The functional labels (Story, Terrain, Environment, Sequence, Sound, …) were added later without your knowledge and are kept only for registry compatibility. In all new Rust code, crate names, UI and docs use the crest name; the legacy label appears only where the registry/manifest needs it (e.g. `mythos_slot`).
- **Quill sits in the registry's "Story" slot** (legacy label): MYTH-10, Dept III NarrativeSystems, crest `Quill`, WireOut `NAR`, accent `#8c50ff`, emblem Hexfeather. Its four layers are Observation, Generation, Performance and Export (e.g. Plot Arc Composer, Narrative Beat Publisher, Prose Formatter, Studio Scribe Dispatcher). `quantum-atlas-story.html` in this repo is therefore **reference for Quill's own Story components, not something to discard**; the other 15 Atlas-era module pages are still out. Quill's data model should line up with the module's 16 standard components (the `10-story.json` companion in the quantum-modules skill has the 256 sub-modules). Quill's module assets live at `assets/MYTH-10/_meta/` (icon, cover, banner, crest.svg, splash).
- Quill = hub shell + workspace, library, zones, scenes, rack, sequencer, subgraph, io-nodes, output + manual. `quill/` (React voice/video studio) is a separate thing; decide later.
- New Rust app is built in a **separate repo**; this repo is untouched. Planning only.
- Rename throughout: `quantum-loom-*` → `quill-*`, `ql-*` storage keys → `quill-*`, `loom-*.js` → `quill-*`, `.loom` project format → `.quill`. Don't carry the Loom name into the new repo.

## What gets dropped / deferred
- Dropped: all Atlas modules, `ecology/civilization/bridge`, the Atlas snapshot keys (`qa-*`), Firebase sync, cloud LLM providers (kept only as opt-in cargo features, off by default; Ollama is the default and only required backend).
- Deferred: real synthesis (the JS never had any). Your Rust DSP/Composer app owns that later.

## Architecture (follows myth-os-architecture law)
Quill is a **module (Layer 2)**: library crates first, thin binary, talks to other apps via WirePackets only, runs standalone.
- `quill-core` (lib, no UI/GPU/async): data model (characters, capsules, acts, scenes, zones, clips, arcs, graphs), sequencer model, snapshot builder, signal bus, B-DNA. Headless-testable (`cargo build -p quill-core` has zero renderer deps).
- `quill-genesis` (lib): **Genesis Container import/export**, built on the shared `qgcp` / `myth-wire` crates (don't reimplement the format; follow the qgcp-format skill: `.qgenesis`, `.qgcp`, argon2id + AES-256-GCM, Seal key derivation, mount semantics). Quill reads containers into its model and writes its narrative layers back (capsule lineage_hash/B-DNA preserved). Handles both Seal-level kinds: World and Actor/Artist.
- `quill-audio` (lib, feature `audio`): minimal audio track support for the sequencer (see below). Behind a trait so your DSP crate can replace it.
- `quill-llm` (lib): Ollama client (reqwest, streaming NDJSON `/api/chat`).
- `quill-midi` (lib): `midir`, CC map, learn mode.
- `quill-ui` (lib, egui/wgpu, feature-gated): all panels and widgets.
- `quill` (bin, ≤100 lines): wiring. Process boundary to the engine/theater is TCP via `EngineTransport` per the architecture law.
Replace `postMessage`/`localStorage` with one `QuillState` + event channel; background jobs (LLM, file IO, waveform peaks) on threads/tokio, results back with `ctx.request_repaint()`.

## UI: reuse the stencil-stack compositor ("bones and skin")
Reuse your existing stencil-stack compositor (egui-based, recursive stencil tree + N-layer stack). Not set in stone, but it is the plan of record. The separate **Theater compositor** (monitor/projector) is out of scope for now; see "Theater hook".
- **Bones = stencil tree** (recursive splits, normalized rects; user can split/merge/resize). **Skin = layer stack**. Layout and look stay separate, so Quill can re-skin a layout without touching it.
- **Layers:** N per stack, add/remove/duplicate/reorder, each with `blend_mode` (Normal, Multiply, Screen, Overlay, Dodge, Burn), `opacity`, visibility, optional `shader_id`. Each layer owns its own stencil tree and per-leaf content, and each section can carry its own sub-stack.
- **Existing layer kinds kept:** Background (gradient/frame), Particles (boid flocking), Nodes (glow/expand on hover), Components (individual widget kinds, palette from the background zone underneath).
- **New Quill layer kinds:**
  - `Panel`: hosts a Quill view as a leaf via `trait QuillPanel { id, title, ui(&mut Ctx) }` (Workspace, Library, Zones, Scenes, Rack, Sequencer, Subgraph, Io-nodes, Output, Inspector, Manual).
  - `Skin`: PNG/shader knobs, faders, pads, LEDs, meters (the realistic controls below).
  - `Video`: a decoded video/image texture, so the video track's layer lanes and the UI stack share one blend/opacity model.
  - `Waveform`: audio peaks drawn by shader.
- **Rules carried over from your notes:** every leaf's content is clipped to its stencil rect (`ui.set_clip_rect(clip.intersect(rect))`), per-leaf controls sit in a bottom-right bar, and the content cycle offers individual components only, not whole packs.
- **Opacity/blend reality check (important):** egui widgets can't be drawn to an offscreen texture easily. So:
  - Shader/painter layers (Background, Particles, Nodes, Skin, Video, Waveform) render to per-layer wgpu textures and get true blend modes plus opacity in a final GPU composite.
  - Widget layers (Panel/Components) use alpha-multiplied colors (approximate opacity, Normal blend only).
  - Pin `eframe`/`egui`/`wgpu` to the compositor's versions (your notes mention egui 0.27) so the paint-callback API matches, then upgrade together.
- **Persistence:** the whole stack (stencil trees, layers, per-leaf content, active layer) serializes with serde into the user's workspace prefs, so arrangements are saved and shareable.
- **Theater hook (later):** because the stack renders to textures, its final composite can be handed to the Theater as a channel (monitor/projector) via `EngineTransport` and WirePackets without changing Quill. No wiring now.
- The compositor is a shared crate. Quill depends on it rather than copying it, so the Loom, Atlas and Quill UIs stay consistent. Its API gets wired in once I can see the source.

## Shared instrument kit (decided: knobs, faders, sequencer etc. are ecosystem assets, not Quill's)
You're right. Controls and instruments are shared, and Quill only decides which ones to use and what they mean.
- **Three shared layers, none Quill-specific:**
  1. `compositor` (stencil tree + layer stack, Phase 0).
  2. `instrument-kit` (working name): knob, fader, pad grid, meter, XY pad, LED, toggle, plus the **Skin pipeline** (PNG layers + shaders + `Skin` manifest), and bigger instruments built from them: **timeline sequencer**, **DJ mixer**, **sampler pad grid**, step grid, node/patch canvas. Each instrument is a compositor leaf/layer kind that takes a config and a data model.
  3. App crates (Quill, Atlas, Loom, Composer, …) that compose instruments and give them meaning.
- **Quill's job:** the narrative semantics, e.g. a sequencer configured with *story-arc lanes* (arc, beat, scene, act clips, tension curve automation), or a sampler-pad grid wired to scene triggers, or a DJ-style mixer for crossfading story threads. Quill does not own a sequencer implementation. The one in this repo is kept because it lines up story arcs well, and becomes the first timeline preset.
- **Rack becomes an instrument too:** the current MIDI controller surface is just a compositor layout of kit widgets plus `midir` mapping, so it is not a Quill-only feature.
- **Skins are shared, accents are per crest:** one knob/fader art set, tinted per app/crest (Quill `#8c50ff`, other crests use their registry colors), so the whole ecosystem looks consistent.
- **Layering vs the architecture law:** the kit is egui/wgpu, so it lives in the output/UI layer (an adapters-style crate with the renderer deps). `quill-core` and the headless `sequencer` *data model* stay UI-free; the timeline *widget* is in the kit and renders that model. Kit crates import `myth-wire` types only, never an app crate.
- **Build order impact:** Phase 0 compositor, then a standalone **instrument-kit repo** (knob/fader/meter/pads first, then sequencer, then mixer/sampler), then Quill uses it. Quill never needs to wait for every instrument; it starts with the sequencer preset.

## Realistic controls (you make the PNGs)
- **Do not rotate one pre-lit knob PNG** (baked highlight rotates → fake). Preferred pipeline, layered:
  - Knob: base+shadow PNG, cap PNG with baked lighting that stays fixed, small rotating indicator layer, procedural glowing value arc (shader).
  - Alternative: 64–128 frame filmstrip per knob type (best fidelity, larger assets).
  - Fader: cap PNG moves by translation only (safe), slot/track procedural, soft shadow.
  - Pads/buttons/LEDs: housing PNG + additive glow when lit.
  - Meters, XY pad, waveforms: procedural shader, no PNGs.
- Art spec to give yourself: 2x resolution, consistent top-left key light, transparent PNG, power-of-two atlas, a `Skin` manifest (RON) mapping control type → layers, so skins can follow the 8 existing themes / faction palettes.
- Loading: `egui_extras::install_image_loaders` or `ctx.load_texture` into one atlas.

## Audio (simple, replaceable)
- Sequencer gets an **audio track type**: clips reference a decoded file; timeline shows waveform; supports audiobook narration and **lyric timing** (markers/regions with text, tap-to-time, nudge, export as LRC/SRT/JSON).
- Crates: `symphonia` (decode), `cpal` (output) or `rodio`/`kira` (simpler), `rubato` (resample), peak cache for waveform (min/max per N samples, computed on a worker thread), drawn with `egui_plot` or a wgpu callback.
- Boundary: `trait AudioBackend { load, play, pause, seek, position }` so your Composer/DSP crate drops in later with no sequencer changes.
- Rack "synth" mode (note-on → narrative events) needs no audio; it just emits events on the signal bus.

## Video (simple, same pattern as audio)
- Sequencer gets a **video track type** (clips with in/out, trim, move) plus **layered video lanes**: lane order = z-order, each clip/layer has opacity, blend mode (normal/add/multiply/screen), and optional alpha handling.
- **Transitions/fades** on clip edges (drag handles, like audio fades): fade in/out, cross-dissolve between adjacent clips, dip to black/white, wipe, slide/push, iris. Each is a small WGSL shader taking `progress 0..1` + two textures, so adding more later is just another shader file.
- **Alpha / background removal** per layer, as shader effects:
  - Native alpha (premultiplied) from PNG sequences, WebM/VP9 alpha, ProRes 4444.
  - Chroma key (green/blue screen): key color, similarity, smoothness, spill suppression (YCbCr or HSV distance).
  - Luma key and a simple garbage-matte/mask (rect/ellipse/polygon).
- **Decode:** no mature pure-Rust video decoder, so use FFmpeg: `ffmpeg-next` (linked) or `ffmpeg-sidecar` (spawn `ffmpeg`, pipe raw frames; simplest to ship, avoids linking issues). Mind FFmpeg LGPL/GPL build choice. Decode on a worker thread into a small frame cache; scrub uses low-res proxies; audio clock is the master for A/V sync.
- **Render:** frames upload as wgpu textures; compositing is done on the GPU in lane order with the blend/key/transition shaders. Same render path serves preview and export.
- **Export:** render offscreen, read back frames, pipe to FFmpeg with the mixed audio. Presets: H.264 MP4, plus alpha-capable (ProRes 4444 / VP9 WebM / PNG sequence).
- Boundary: `trait VideoBackend { open, frame_at(t), duration, size }` so the decoder is swappable. Crate `quill-video` (feature `video`), alongside `quill-audio`.
- Decided: video layer lanes reuse the stencil-stack compositor's layer model (same `BlendMode`/opacity/`shader_id`, `Video` layer kind), so there is one compositor for UI and video. Blend modes in the sequencer match the UI's list.

## Ecosystem context (known now, wired later)
Quill is one standalone app in the Quantum Genesis Ecosystem alongside Atlas, Loom, Composer and others. Facts that shape the design:
- **Quantum Core** is a headless multi-client game engine that runs the simulation from Genesis Containers. **Quantum Vault** is a sandboxed, directory-isolated runtime (Master Vault, up to 16 Vaults, each with up to 16 Sub-Vaults; 2D/3D spaces, notepads, virtual media studios) with Portals, Gates, Heraldry and autonomous actors contesting for control. Both exist with or without Quill.
- Every app is a **plugin to the Vault** and can have its own **addons**. Apps share Heraldry, Genesis Containers, Capsules, ATOMs, CELLs and other assets.
- **Live remix mode:** Quill edits the *running* simulation. The narrative being authored is the world the actors live in, and they write their own stories inside it. Not wired now.
Design consequences for Quill (cheap now, painful to retrofit):
1. **Shared types come from shared crates** (`myth-wire`, `qgcp`, and heraldry/ATOM/CELL crates when you have them). Quill must not define its own copies. Anything not yet in a shared crate gets a stub behind a trait and is flagged for you.
2. **Two run modes:** standalone bin (default) and a `quill-vault` adapter crate (see "Vault plugin adapter"). Same core either way; `quill-core` never depends on `qshell`.
3. **Directory isolation:** all file IO goes through a `VaultPaths`/`Workspace` abstraction (root dir injected at startup). No hardcoded home or Documents paths, no absolute paths in saved files. Standalone mode just supplies its own root.
4. **Manifest = `PluginManifest`** (see below), not a bespoke JSON. Addons (≤16 per plugin) are **not built in the Vault yet**, so Quill only keeps its internal extension points (compositor layer kinds, panels, export formats, node types) addon-ready and builds nothing addon-specific now.
5. **Edits are commands, not direct mutation.** Sequencer, rack and scene changes go through a command/event layer in `quill-core` (undo/redo falls out for free). In live mode the same commands become CTL WirePackets sent to the Core, and Core state flows back as packets. Offline, commands apply to the local model.
6. **`EngineTransport` stub:** Quill talks to the Core only through the trait; default impl is a loopback no-op (local model), TCP impl later.
7. **Vault context:** a small `VaultContext` trait (vault id, workspace root, LLM provider) with a standalone default; the `quill-vault` adapter fills it from qshell's `Cx`, so `quill-core` and `quill-ui` never depend on the Vault crates.

## Vault plugin adapter (against the qshell contract as it stands today)
The plugin architecture is still being corrected (the goal is real plugins, not compiled-in stand-ins), so Quill isolates every Vault-specific detail in one thin crate, `quill-vault`, and codes to today's documented contract. When the plugin system changes, only this crate changes.
- **Quill is a Path #1 `VaultPlugin`** (UI + live state). A `.qplugin` is data-only and not an option for Quill. A `.qplugin` could later carry exported Genesis containers/capsules as read-only data, which is a separate nice-to-have. Skills (generators) and `.qvault` (the packaged container) are different things and not Quill.
- **Trait mapping:**
  - `manifest()`: `PluginManifest::new("quill", "Quill", icon, crest = Some("Quill"), VaultTier::Free, mythos_slot = Some("MYTH-10"))`.
  - `ui()`: renders the compositor's stencil-stack inside the middle canvas. The shell's master header/footer and rails stay fixed; Quill's BSP layout is *inside* the canvas.
  - `rails()`/`panel_ui()`: Quill's Properties/inspector, library and per-layer controls as slide-in panels. `panel_ui` has no `Cx`, so use the `last_key` idiom (store `cx.vault_id` at the top of `ui()`).
  - `header_ui()`/`footer_ui()`: transport (play/BPM/timecode) as an in-canvas strip, not in the master header.
- **State is vault-scoped:** all mutable Quill state lives in `VaultScoped<QuillState>`; never a bare global field. Option to key by a *project* (like `AnimusView`) if one Quill project should span several child vaults; default is per-vault.
- **File IO only through a `qchamber::Chamber`** jailed to `vault_workspace_root(cx.vault_id)`. Project files, audio/video media, caches and exports all live under `vaults/<id>/`, so a `.qvault` export carries them. No absolute paths in saved data.
- **No network assumed** (only the Master Vault reaches the network): LLM goes through a `LlmProvider` trait. Plugin mode uses the shell's `cx.llm` / `cx.system_prompt`; standalone mode uses direct Ollama. So `quill-llm` is the standalone impl only.
- **No game engine in the client** (rule 12): no Bevy in Quill. Any external process (e.g. FFmpeg sidecar, a viewer) talks over a file/socket of plain data in the workspace, launched from beside `current_exe()`. Prefer linked `ffmpeg-next` in plugin mode to avoid spawning processes outside the Chamber policy; sidecar stays a standalone-mode option.
- **Palette:** paint from `cx.p` (active theme), never hardcode colors. The eight Quill themes become one extra `Palette` mapping in standalone mode.
- **Registration:** one line in `QShellApp::new()`'s plugin vec, mounted per vault via `plugin_ids` in `vaults/vaults.kdl`.
- **egui version:** the compositor, Quill and qshell must share one `egui`/`eframe` version. Pin to qshell's and align the compositor rebuild (Phase 0) to it.
- **Ownership (decided):** the Vault owns its header, ribbon, footer and footer ribbon. The plugin owns the **entire middle canvas**. So Quill's compositor stack, its Layers panel and its own side panels live inside the canvas, and the host's right-rail Layers slot is not used by Quill.
- **Forward path (Vault upgraded to Stencil):** the Vault will later be built on the same stencil engine, and a plugin will just declare the leaves it needs. To be ready, `quill-vault` exposes a static `LeafSpec` list (id, title, preferred role/size, e.g. `sequencer`, `rack`, `library`, `inspector`, `graph`, `output`) alongside the current `ui()`. Today the adapter ignores it and lays the leaves out in its own canvas stack; later the Vault reads it and hands Quill its leaves. `QuillPanel` is already leaf-shaped, so the move is a swap of who does the layout, not a rewrite.
- **Order of work:** a working standalone Quill comes first (the Vault is not required to run it). `quill-vault` follows once Quill is usable, and the compositor crate is written so the Vault can adopt it.

## Heraldry in Quill (from the quill-heraldry skill)
Quill handles heraldry as data it carries and validates, never as rules it owns.
- **Types come from a shared `heraldry` crate** (with `myth-wire`/`qgcp`), not from Quill: `HeraldryState { birth, current, peak, nadir: Option }`, the position scale (Seal > Crest > Glyph > Sigil > Mark > Trace), and the 20 symbol types. Until that crate exists Quill uses a stub behind a trait. Threshold values, ascent/descent and the resonance-weight formula belong to the quantum-condition-engine (and the Core), so Quill reads `current`/`peak`/`nadir` and does not compute them.
- **`birth` is immutable.** The command layer rejects any edit to `birth`, and `.qgenesis`/`.qgcp` import and export must round-trip all four fields unchanged. Only the Core/condition engine moves `current`, `peak` and `nadir`, so in live remix mode Quill sends commands and the Core updates the fields.
- **Birth assignment validator** (`quill-core::heraldry`): when the author creates an entity, check three-way alignment (structural level, functional role, symbolic type). Defaults: GenesisContainer→Seal, MythosContainer→Crest, Container→Glyph/Device/Emblem, Capsule→Mark. An override needs a logged justification and passes the TVG. Initial state is `birth = current = peak = assigned`, `nadir = null`. Hard errors: Seal on a non-Genesis container, Crest at Container/Capsule birth, Sigil on a group, Token treated as permanent. It outputs the line `[Name] — [ContainerType] / [QuantumComponent] / [SymbolicType] / birth:[Position]`.
- **UI:** a Heraldry section in the Inspector shows the four fields (an arc such as `Seal → Glyph (peak Seal, nadir Mark)`), highlights birth-vs-current divergence (the "where the story lives" cases), and shows the validator result at creation. Sigil, crest and charge art uses `resvg` rasterized textures on the compositor's Skin/Icon layers, and the existing PNG/JPG sigils in `assets/` carry over.
- **Race crests** (`birth_crest` vs `current_crest`) appear as separate inspector fields on Actor entities and as gate-condition inputs, but they are display only in Quill.
- **Quill's own heraldry (decided):** Quill is a **MythosContainer born at Crest**, like the other modules. That is its identity in the Genesis system. Its Vault mount is a separate fact: in the Vault's plugin hierarchy a plugin slot sits at Glyph/Device level. Both are true, so `quill-vault` sets `PluginManifest.crest` to Quill's crest token (theming) and does not conflate the two. Quill's `HeraldryState` for itself is `birth = current = peak = Crest`, `nadir = null`. The quantum-modules registry already lists Quill as the crest of Story (MYTH-10), so the 16 canonical crests there are Atlas, Mythos, Architect, Prism, Animus, Loom, Instinct, Order, Chronicle, Quill, Codex, Composer, Axiom, Continuum, Forge, Nexus.
- **Genesis Seal level:** only two kinds of Seal-level Genesis Container exist today: **World** Genesis Containers and **Actor/Artist** Genesis Containers. `quill-genesis` must import and export both kinds. Vaults are outside the Genesis system, though they reuse heraldry.
- **Vault gating is the Vault's job, not Quill's.** Vaults use heraldry as gate-keeping conditionals, for example `actor.crest == Syntaran && actor has Glyph(movement)` to enter a vault type. Quill's only obligation is that entity heraldry it writes is valid and readable by those gate conditions. Possible later feature: a "gate preview" in the Inspector showing which vault types an Actor could enter. That needs the Vault's gate definitions, so it is deferred.

## Crate shortlist
| Need | Crates |
|---|---|
| App/UI | `eframe` (wgpu backend), `egui`, `egui_extras`, `egui-notify`, `egui_commonmark` (manual) |
| GPU | `egui_wgpu`, `wgpu`, `bytemuck`, `glam` |
| Plots/waveform | `egui_plot`, `realfft`/`rustfft` (spectrum), custom `Painter`/shader for dense waveforms |
| Node graphs | `egui-snarl` (or `egui_node_graph2`), `petgraph` |
| Vector/SVG sigils | `resvg`/`usvg`+`tiny-skia` (rasterize existing SVG/PNG crests to textures) |
| MIDI | `midir`, `wmidi` |
| LLM | `reqwest`, `tokio`, `futures-util` (or `ollama-rs`) |
| Persistence | `serde`, `serde_json`/`ron`, `rusqlite` (local-first), `directories` |
| Crypto/format | `argon2`, `aes-gcm`, `zip`, shared `qgcp`/`myth-wire` |
| Video | `ffmpeg-next` or `ffmpeg-sidecar`, `image` (PNG sequences), WGSL shaders via `wgpu` |
| Subtitles/lyrics | `srtlib` or hand-rolled LRC/SRT writer (small) |
| Misc | `rfd`, `keyring`, `tracing`, `thiserror`, `image` |
Fonts (Orbitron, Space Grotesk, JetBrains Mono) bundled via `include_bytes!`. Themes: `QL_THEMES` (8 token sets) → `Theme` struct → egui `Visuals` + shader uniforms.

## Mapping from existing code
- Rack (`quantum-loom-rack.html`): device array (~L771) → data-driven `DeviceDef`; MIDI IIFE (~L1913) → `quill-midi`; patch-cable SVG overlay → Painter bezier layer.
- Sequencer: ruler/playhead, lanes, clips, envelope "arcs", step grid → custom timeline widget; add audio lane.
- Subgraph / io-nodes → `egui-snarl` with typed pins (16 wire types).
- Workspace/library/zones/scenes/output → data-driven list/card panels over `quill-core`.
- `loom-snapshot.js` → `quill-core::snapshot`; `loom-llm.js` → `quill-llm`; `loom-broadcast.js` → WirePacket emission over `EngineTransport`.
- Export dialogs → `rfd` + `quill-genesis`.

## Phases
0. **Rebuild the stencil-stack compositor first, as its own standalone crate/repo** (not inside Quill). Scope: stencil tree (split/merge/resize, normalized rects, serde), N-layer stack (add/remove/duplicate/reorder, blend modes, opacity, `shader_id`, per-section sub-stacks), clipping, the existing layer kinds, and **render-to-texture per layer with a GPU blend composite** (decided now rather than retrofitted). Exposes a `LayerKind` extension trait so apps register their own kinds (Quill's `Panel`, `Skin`, `Video`, `Waveform`). Has a demo bin, runs with no other app, and is reusable by Loom, Atlas and the Theater later. Quill starts only after this API settles; use path deps until then.
1. Repo + workspace skeleton, naming cleanup, theme, bundled fonts, `QuillPanel` trait + compositor hook.
2. **Instrument kit** as its own repo/crate (after Phase 0): knob, fader, meter, XY pad, pads, LED with the Skin pipeline (lock it with your first PNGs), then the timeline sequencer, then mixer/sampler. Quill starts consuming it as pieces land.
3. `quill-core` model + `.quill` save/open.
4. **Genesis Container import/export** (`quill-genesis`), round-trip tests. Prioritized early since it's mandatory.
5. Library/workspace/scenes/zones panels.
6. Sequencer (+ audio track, lyric timing), then video tracks (decode, layers, fades, transitions, keying), then video export.
7. Rack + MIDI.
8. Subgraph + io-nodes.
9. Output/export, Ollama integration, manual, packaging.

## Verification (when built)
- `cargo tree -p quill-core | grep -E "bevy|egui|wgpu"` returns nothing.
- Genesis round-trip: import a `.qgenesis`/`.qgcp`, export, re-import, compare (incl. encrypted + wrong-key cases).
- Sequencer: audio load/seek/sync tests; lyric timing export matches expected LRC/SRT.
- Video: golden-frame tests for each transition and the chroma key (fixed inputs → compared output), A/V sync drift check, alpha export plays back with transparency.
- Visual check of widget test-bed across all themes; MIDI loopback; Ollama stream test.
- Each crate: `cargo run -p quill` runs standalone with no other apps present.

## Open items (not blocking the plan)
0. For later: any plugin API changes as the real-plugin work lands (isolated in `quill-vault`), the Core's wire protocol, and the shared Heraldry/ATOM/CELL crates. Share them when ready.
1. Compositor is being rebuilt first (Phase 0), so its API is designed fresh. Your old source is still useful as a reference for the layer kinds, boid and glow-node behavior, and the clipping and controls rules.
1b. **Crest token = `"Quill"`, mythos slot = `"MYTH-10"`** (Story). Settled, and no 17th crest is needed: the quantum-modules registry already has Quill. The quill-heraldry skill's own "16 canonical module crests" list is out of sync with that registry (it names Core, Vault, Mind, Soul, Terrain, Environment, Lighting, Network, which are infrastructure or module names, not crests). Its list should be replaced with the registry's CrestId list above. The skill file is edited outside this plan.
1c. **Sequencer ≠ Sequence module (decided).** A *sequencer* is a tool/widget (timeline, lanes, clips). The *Sequence* module is Chronicle (MYTH-09), a different thing that could embed a sequencer if it needs one. So the sequencer lives in the shared instrument kit (app-neutral model + widget, no Quill types inside), Quill configures it for story arcs, and Chronicle (or anything else) can reuse it later. Quill's audio track and lyric timing stay in Quill for now; audio crosses to Composer as `AUD` and timeline events as `TMP` WirePackets.
2. Exact Genesis Container spec version Quill must read/write (taken from the shared `qgcp` crate).
4. `quill/` (React music-video creator prototype): **decided, reference only.** It stays in this repo untouched and is not ported. Its features are being built into Quill natively through the audio/video tracks, lyric timing and layer compositor already planned. Worth skimming `quill/components/MusicVideoStudio.tsx` and `quill/services/` for workflow ideas when building Phase 6. Its Gemini calls are not carried over (Ollama only), and `loomBridge.ts` carries the old Loom name.
