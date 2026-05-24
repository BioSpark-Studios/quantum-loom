claude/build-new-feature-eEUNS
# quantum-loom
Procedural world simulation platform — build living, breathing worlds from terrain to narrative.

Quantum Loom is a creative operating system for world-builders. It treats a world the way a producer treats music: terrain is the foundation track, societies are arrangement blocks, story arcs are automation lanes, and the whole simulation is a project you sequence, remix, and run in real time. The 16-module Quantum Atlas engine handles every layer of a synthetic world — from raw geology to multi-agent networks — all wired together through a live signal graph and saved in a portable Genesis Container.

Built by BioSpark Studios as part of the Quantum Ecosystem — a modular platform where every creative tool speaks the same language.

## The Quantum Atlas — 16 Modules

| # | Module | Crest | Department |
|---|--------|-------|------------|
| 01 | Terrain | Atlas | WorldConstruction |
| 02 | Environment | Mythos | WorldConstruction |
| 03 | Architect | Architect | WorldConstruction |
| 04 | Lighting | Prism | WorldConstruction |
| 05 | Modeling | Animus | EntitySystems |
| 06 | Choreography | Loom | EntitySystems |
| 07 | Behavior | Instinct | EntitySystems |
| 08 | Society | Order | EntitySystems |
| 09 | Sequencer | Chronicle | NarrativeSystems |
| 10 | Story | Quill | NarrativeSystems |
| 11 | Memory | Codex | NarrativeSystems |
| 12 | Sound | Composer | NarrativeSystems |
| 13 | Logic | Axiom | PipelineSystems |
| 14 | Simulation | Continuum | PipelineSystems |
| 15 | Forge | Forge | PipelineSystems |
| 16 | Network | Nexus | PipelineSystems |

## Features

- **16-module Atlas pipeline** — WorldConstruction → EntitySystems → NarrativeSystems → PipelineSystems
- **Genesis Container format** (`.qgenesis`) — portable world definition, analogous to an audio soundbank
- **Project format** (`.qgcp`) — encrypted creative session with media assets, p5.js sketches, and lifecycle states
- **16 wire types** — typed data channels (DAT, CTL, AUD, NAR, TMP, AGT, VIS, SPA, BHV, SOC, ENR, IDN, EVT, AST, MET, LGC)
- **B-DNA** — deterministic entity identity hash (LCG-derived 16-char hex)
- **Hex grid** — pointy-top axial coordinates, radius 14, 631 hexes
- **Instrument Rack** — modular audio synthesis linked to the Sound module
- **AI Draft mode** — streams prose from Ollama, Claude, Gemini, OpenAI, or Azure
- **Firebase cloud sync** with real-time collaboration
- **Electron desktop app** with native file dialogs
- **In-app User Manual** — F1 to open, full 8-chapter reference with glossary

## File Formats

- `.qgenesis` — world definition container (all 16 Atlas module states, up to 4,096 capsules)
- `.qgcp` — creative project file (encrypted manifest via argon2id + AES-256-GCM, media assets, p5.js sketches)

## Getting Started

```bash
npm install
npm start
```

Open Genesis import via the **↑ Import** button in the header, or drag a `.qgenesis` file onto the canvas.
Press **F1** for the in-app User Manual.

<div align="center">
<img width="1200" height="475" alt="GHBanner" src="https://ai.google.dev/static/site-assets/images/share-ais-513315318.png" />
</div>

# Run and deploy your AI Studio app

This contains everything you need to run your app locally.

View your app in AI Studio: https://ai.studio/apps/1fb3ad3b-a334-43ca-b1b5-9ded58c5f92b

## Run Locally

**Prerequisites:**  Node.js


1. Install dependencies:
   `npm install`
2. Set the `GEMINI_API_KEY` in [.env.local](.env.local) to your Gemini API key
3. Run the app:
   `npm run dev`
main
