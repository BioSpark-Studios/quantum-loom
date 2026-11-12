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

## Not ported yet

- Faceplates are saved and bound, but a bound instrument still shows its macro knobs in the rack
  (`assets::plate` draws a faceplate live and is ready to hook in).
- Bespoke faces for Main Timeline, Eidolon Stage, Chronoscriptorium, Orchestration Nexus,
  Hearthstone, Herald, Axiomatic Loom, Atom Graph, Atmosphere (control surface for now).
- The EXTENDED deck layout; the asset families that are mostly art (scenes, icon atlas, QMF / Alchemy sets).
- Drawer tabs other than Console; Settings; the LLM router (Ollama first) — model nodes pass through, marked.
- Audio output and MIDI input (controller profiles, MIDI Learn). `rack_core::engine::AudioBackend` is the seam.
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
