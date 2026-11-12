# FORGE · BioSpark Main Rack (Rust · egui · wgpu)

Native port of the **BioSpark Main Rack** design (`BioSpark Main Rack.html` in the
Claude Design project). It replaces the Electron `quantum-loom-rack.html` page.

```bash
cd rust
cargo run -p forge --release
```

State is kept under `$FORGE_HOME`, or the platform data directory
(`~/.local/share/biospark/forge` on Linux) when that is not set.

## Crates

| Crate | Role | Depends on |
|---|---|---|
| `rack-core` | Headless core: parameter bus, transport clock, module / effect catalog, rack model, routing engine, core output frame, Genesis reading, workspace persistence, vaults. No UI, GPU or async. | `serde` |
| `stencil-core` | Stencil-Stack data model: BSP layout, layer stacks, the instrument template library and the `.qgcp` layout manifest with its three laws (octave capacity, three-way alignment, crest border weights). Plain serde data so the Theater can share it. Tested against the real package in `reference/stencil/`. | `serde` |
| `instrument-kit` | Shared egui kit: design tokens, materials, knobs, faders, pads, readouts, meters, glyph tiles, icon set, glass-screen visualizers. App-neutral. | `egui`, `resvg` |
| `forge` | The client: sign-in gate, project vaults, three-row header, icon rails, Main Rack, Routing Matrix, drawer. Thin binary over a library. | both |

`cargo tree -p rack-core` shows no egui or wgpu. Token names in
`instrument-kit::theme` mirror `tokens.css` so the design document stays the
single source of truth.

## What is ported

- **Shell** — sign-in gate, project vaults (new vault + faction, versions, restore), header rows
  (vault · launcher · workspace tools) with the narrow-width behaviour, icon rails with flyouts.
- **Main Rack** — instrument browser by department, effects browser, drag to add / reorder devices,
  2U devices with bypass emblem, macros, MUTE / SOLO / level / pan / meter, four insert-effect slots,
  unfolded 12-column sections with VIEW fill sections, six panel skins, device inspector.
- **Routing Matrix** — source → destination table with live meters and bipolar amounts, grid overview;
  routes run at 20 Hz on every screen.
- **Headless core output** — the frame adapters see (addresses + numbers), DMX preview, log.
- **Genesis** — the bundled OROS-V7 world mounts by default; drop a Genesis `.json` on the window to mount another.

- **Performance · DJ** — two jog decks with orb platters, narrative waveform, loop section, transport,
  quantum-event pads; four-channel mixer with master and the timeline crossfader; FX rack; 16 decks in
  4 banks playing the mounted Genesis (one deck = one Mythos, Containers are the track list). The core
  publishes `deck.<id>.out` / `clock` and `mix.intensity` on the bus, and the routing matrix lists deck
  controls as destinations.
- **FORGE stages** — LIBRARY (36 live assets by family, search, + PANEL), MODULES (eurorack case, 29 modules,
  macros, skins, save to library / add to rack), PANEL (pane tree: split, remove, place, drag, play mode,
  materials, accent, bind and save a faceplate to an instrument), LAYOUTS (7 partition templates, faction
  resonance), NODE GRAPH (typed wires, cycle check, RUN in dependency order, D/R/L flags, bus writes).

## Pass 3 so far

- **Local LLM** (`rack-llm`) — Ollama over plain http, streaming chat and model list, run on worker threads.
  Node Graph MODEL nodes call it; an unreachable server marks the node and grounds what is downstream.
  Settings drawer tab: endpoint, model picker, connection test. Local only, no cloud providers.
- **Faceplates in the rack** — a faceplate bound to an instrument is its folded front panel.
- **MIDI logic** (`rack_core::midi`) — message parsing, CC → `midi.cc.<n>` on the bus, LEARN for routes.
  No device backend yet (see below).
- **Main Timeline** — real face: preview, track inspector, transport, draggable clip lanes, concurrent
  Quill / Loom / Chronicle timelines; publishes `aft.time`, `<prefix>.t<i>` and `aft.alt.*`.

## Not ported yet

- Bespoke faces for Eidolon Stage, Chronoscriptorium, Orchestration Nexus, Hearthstone, Herald,
  Axiomatic Loom, Atom Graph, Atmosphere (control surface for now).
- Audio output and MIDI **device** input. `midir` and `cpal` need ALSA headers, which the build
  container lacks, so they could not be compiled or tested; `AudioBackend` and `Midi::handle` are the seams.
- Controller profiles for the decks (needs the MIDI backend).
- The EXTENDED deck layout; asset families that are mostly art (scenes, icon atlas, QMF / Alchemy sets).
- Drawer tabs PERFORM, STAGE and BROWSER.
- `.qgcp` (zip / sealed) packages — only plaintext `{header, manifest}` JSON is read.

## Tests

```bash
cargo test --workspace
```

Render tests draw every workspace headlessly and need a GPU adapter, so they only
run with `FORGE_GPU_TESTS=1` (add `FORGE_SHOTS=<dir>` to keep the PNGs).

## Fonts

Orbitron, JetBrains Mono, Inter and Cinzel are bundled under the SIL Open Font
License 1.1 — see `crates/instrument-kit/assets/fonts/OFL.md`.
