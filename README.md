⬡
Quantum Loom
Procedural World Simulation — User Manual
QUANTUM ATLAS v1.0 · 16-MODULE EDITION
1
Introduction to Quantum Loom
What Is Quantum Loom?
Quantum Loom is a procedural world simulation platform built for world-builders, narrative designers, game developers, and creative technologists. It provides a complete pipeline from raw terrain generation all the way through to networked multi-agent simulation — the entire lifecycle of a synthetic world managed in one application.

The core philosophy is that every element of a world — geography, climate, architecture, society, story, sound, logic — exists in deterministic relationship to every other element. Quantum Loom makes those relationships explicit through its 16-module Atlas pipeline and its container-based save format.

The Quantum Atlas Pipeline
The Quantum Atlas is the 16-module procedural engine at the heart of Quantum Loom. Modules are organized into four departments, each handling a distinct layer of world simulation:

WorldConstruction (01–04): Physical world — terrain, environment, architecture, lighting
EntitySystems (05–08): Inhabitants — character models, movement choreography, behavior, society
NarrativeSystems (09–12): Story layer — event sequencing, story arcs, memory/history, sound design
PipelineSystems (13–16): Engine layer — logic rules, simulation engine, asset forge, network/agents
Data flows downstream through the pipeline: Terrain feeds Environment, Environment feeds Architect, and so on, forming a dependency chain that produces coherent worlds from parameterized seeds.

Key Concepts at a Glance
Genesis Container (.qgenesis): A world definition file — analogous to an audio soundbank. Contains all 16 module states for a complete world.
Project File (.qgcp): A save format for engineering sessions. Contains directory structures, media assets, p5.js sketches, and an encrypted manifest.
B-DNA: A deterministic identity hash (16-character hex) assigned to every entity in a world, derived from its lineage and spawn parameters.
Wire Types: 16 typed data channels connecting modules and agents. Each wire carries a specific data class (terrain data, audio, narrative events, etc.).
Capsule: The atomic unit of content inside a Genesis Container. Capsules hold one piece of world data — a character, an event, a soundscape, a mesh.
System Requirements
OS: Windows 10/11 (64-bit), macOS 12+, Ubuntu 20.04+
RAM: 8 GB minimum, 16 GB recommended for large worlds
GPU: WebGL 2.0 capable (for hex-grid and 3D previews)
Storage: 500 MB for application; 1–20 GB per Genesis project depending on media assets
Node.js / Electron: Bundled with the application — no separate install required
Installation and First Launch
Download the Quantum Loom installer for your platform from the releases page. Run the installer and follow on-screen prompts. On first launch:

The application creates a local database at ~/.quantum/quantum-agent.db for mount registry tracking.
A default theme (Dark/Venturan) is applied. Themes can be changed from Settings → Appearance.
An empty workspace is shown. Create your first Genesis Container via File → New Genesis or Ctrl+N.
Tip
Import a .qgenesis demo file to explore a fully populated world before building your own. Demo files are available in the examples/ folder of the installation directory.
2
The Quantum Loom Interface
Hub Layout Overview
The main window is divided into four zones:

Header Navigation Bar (52px): Centered pill-tab navigation for Workspace, Library, Zones, Scenes, Sequencer, Rack, IO Nodes, and Output views.
Left Sidebar (48px icon strip): Loom engine tools — Settings, Themes, Assets, Agents, Search, Notifications, Help.
Right Properties Panel (280px, collapsible): Contextual panel showing the Active Layer, Snapshot Data preview, and Quick Actions for the currently selected module.
Atlas Footer Bar (72px): The 16 Atlas module buttons arranged in a horizontal strip at the bottom. Click any module glyph to open that module in the main content area.
Header Navigation — Main Views
Tab	Description
Workspace	The primary world-building canvas. Shows the currently active Atlas module or the hub overview.
Library	Browse and manage Genesis Containers, imported assets, soundbanks, and template worlds.
Zones	Geographic zone editor. Define named regions on the hex grid with custom biome, climate, and faction rules.
Scenes	Scene compositor. Assemble multi-layer scenes from modules and schedule them in the Sequencer.
Sequencer	The Loom Track Sequencer — timeline-based playback of world events, animations, and narrative beats.
Rack	The Instrument Rack — audio synthesis and sound design workspace linked to the Sound module.
IO Nodes	Node graph for wiring module outputs to inputs using the 16 wire types.
Output	Preview and export the world simulation as video, data files, or interactive web exports.
Left Sidebar — Loom Tools
Icon	Tool	Shortcut
⚙	Settings	Ctrl+,
◑	Theme / Appearance	—
◫	Asset Browser	Ctrl+B
⬡	Agent Manager	Ctrl+G
⌕	Global Search	Ctrl+F
🔔	Notifications	—
?	Help / Manual	F1
Right Properties Panel
The collapsible right panel (toggle with the › button or Ctrl+]) contains three sections:

Active Layer: Shows the currently active Atlas module — its name, crest, department, and status.
Snapshot Data: A live preview of the module's stored snapshot, including key parameter values and last-modified timestamp.
Quick Actions: Context-sensitive action buttons — Save Snapshot, Import Genesis, Export Genesis, and Reset to Defaults.
Atlas Footer Bar
The 72px footer strip shows all 16 Atlas modules as icon buttons. Each button displays the module's glyph in its canonical color. Clicking a button activates that module in the main content area and highlights it in the Active Layer panel.

Note
Right-click any footer module button to get a context menu with: Open Module, View Snapshot, Reset Module, and Wire To... options.
Tooltips
Quantum Loom uses a unified tooltip system. Hover over any node, glyph, icon, or interactive control for 600ms to see its tooltip. Tooltips show a title and description. Some tooltips on complex nodes also show parameter ranges, wire compatibility, and keyboard shortcuts.

Themes and Appearance
Open Settings → Appearance or click the theme icon in the left sidebar. Available theme tokens:

Background (--bg): Primary canvas color
Surface / Surface2: Panel and card backgrounds
Edge / Edge-Hi: Border and divider colors
Accent colors: Venturan (blue), Amber (gold), Emerald (green), Fusion (cyan)
Custom themes can be exported as JSON token sets and shared via .qltheme files.

3
Genesis Containers & File Formats
The Genesis Container Hierarchy
A Genesis Container is the top-level world definition unit. Its internal hierarchy follows a strict capacity law designed to keep world complexity manageable and deterministic:

1 Genesis
↓ contains up to 16
Mythos Containers
↓ each contains up to 16
Containers
↓ each contains up to 16
Capsules
= Maximum 4,096 Capsules per Genesis
A Capsule is the atomic unit of world content — a character, location, event, soundscape, or any discrete world element. Capsules carry a B-DNA identifier, metadata tags, and typed wire connections.

The .qgenesis Format
A .qgenesis file is a world definition — analogous to an audio soundbank. It contains the complete parameterized state of all 16 Atlas modules, the container hierarchy, B-DNA registries, and world metadata. It does not contain media assets or user project files.

Structure
{
  "genesis": {
    "id": "GEN-xxxxxxxxxxxxxxxx",
    "name": "My World",
    "created": "ISO-8601 timestamp",
    "version": "1.0",
    "atlas": { /* 16 module snapshots, keyed by module id */ },
    "mythos": [ /* up to 16 MythosContainer objects */ ],
    "metadata": { "author": "...", "tags": [], "description": "..." }
  }
}
Importing a Genesis Container
Click Import Genesis in the Quick Actions panel, or drag a .qgenesis file onto the hub canvas.
The Genesis Import modal opens, showing container metadata and checkboxes to select which module sections to apply.
Select the modules you want to import. Deselected modules keep their current state.
Click Apply. Each selected module loads its snapshot data and refreshes its UI.
Exporting a Genesis Container
Click Export Genesis in the Quick Actions panel or via Ctrl+E.
Enter a world name. All 16 module snapshots are collected and bundled.
The file downloads as <worldname>.qgenesis.
The .qgcp Format
A .qgcp (Quantum Creative Project) file is what you save when working on a creative session. Think of it as the audio engineer's session file — it references a Genesis Container but also holds your working assets, custom scripts, and media.

What .qgcp Contains
Encrypted manifest: AES-256-GCM encryption with argon2id key derivation. Contains project identity, bound Genesis reference, and lifecycle state.
Directory structure: Organized folders for media assets, p5.js sketches, audio files, textures, and narrative documents.
Lifecycle state: Each project exists in one of five states: seeding, active, sealed, archived, deprecated.
Mount registry: Tracked in the local SQLite database at ~/.quantum/quantum-agent.db.
Lifecycle States
State	Description
seeding	Project is being initialized. Genesis Container not yet bound.
active	Normal working state. All edits and imports are permitted.
sealed	Project locked for distribution or archival. Read-only.
archived	Compressed and stored. Can be unshelved back to active.
deprecated	Superseded by a newer version. Preserved but not editable.
Encryption
The manifest is encrypted using argon2id key derivation from a user-provided seal file passphrase, with AES-256-GCM for the manifest payload. Keep your seal file secure — a lost seal file cannot be recovered.

B-DNA — Entity Identity
Every entity in a Genesis Container has a B-DNA — a deterministic 16-character hex identifier prefixed with 0x. B-DNA is derived via a Linear Congruential Generator seeded by the entity's lineage path and spawn parameters. This means the same world seed will always produce the same B-DNA for the same entity, enabling reproducible simulation runs and diff-able container versions.

// B-DNA format: 0x + 16 hex chars
// Example: 0x3a7f1c92b04e68d5
The Mount Registry
The local SQLite database at ~/.quantum/quantum-agent.db tracks all mounted Genesis Containers and QGCP projects on your machine. This allows the Agent Manager and Network module to discover and connect to worlds running on the same system. The registry is managed automatically — you do not need to edit it directly.

4
Wire Types — The 16 Data Channels
What Are Wires?
Wires are typed data channels that connect modules, capsules, and agents in the IO Nodes graph. Every connection has a declared wire type, which enforces data compatibility and controls how downstream modules interpret the incoming data. Attempting to connect incompatible wire types shows a validation error.

Wire connections are drawn in the IO Nodes view. Drag from an output port to an input port — the system highlights compatible ports as you drag.

Wire Type Reference
Code	Name	Carries	Primary Modules
DAT	Data	Generic structured data, parameters, snapshots	All modules (default)
CTL	Control	Signals, triggers, enable/disable commands	Logic, Simulation, Sequencer
AUD	Audio	Sound events, waveform references, mix data	Sound, Rack, Sequencer
NAR	Narrative	Story beats, dialogue trees, arc triggers	Story, Memory, Sequencer
TMP	Temporal	Time-indexed event streams, timeline markers	Sequencer, Simulation, Memory
AGT	Agent	Agent spawn commands, behavior payloads	Network, Behavior, Society
VIS	Visual	Mesh data, texture references, lighting params	Modeling, Architect, Lighting
SPA	Spatial	Hex coordinates, zone boundaries, terrain heightmap	Terrain, Environment, Architect
BHV	Behavior	Behavior trees, state machines, instinct weights	Behavior, Choreography, Society
SOC	Social	Faction data, relationship graphs, social events	Society, Story, Memory
ENR	Energy	Resource flows, entropy values, simulation energy	Simulation, Terrain, Environment
IDN	Identity	B-DNA hashes, entity metadata, lineage chains	All modules (entity tracking)
EVT	Event	Discrete world events, triggers, callbacks	Sequencer, Story, Logic
AST	Asset	Media file references, p5.js sketch paths, mesh URIs	Forge, Architect, Modeling
MET	Meta	Schema definitions, type declarations, module configs	Logic, Network, Genesis system
LGC	Logic	Predicate expressions, rule sets, condition trees	Logic, Behavior, Simulation
Wiring Best Practices
Use DAT for general data passing when no more specific type applies.
Use IDN wires whenever you need to track an entity's identity across module boundaries.
Use EVT for one-shot triggers (e.g., "character dies") and CTL for persistent enable/disable states.
Audio wires (AUD) carry references, not raw audio data — keep large audio files in the project's /media/audio/ folder and reference them by path.
Avoid circular wires unless the Logic module's cycle-detection is explicitly enabled for that subgraph.
5
The 16 Atlas Modules
Module Quick Reference
⬡
01
Terrain
Atlas · WorldConstruction
◉
02
Environment
Mythos · WorldConstruction
⊓
03
Architect
Architect · WorldConstruction
◐
04
Lighting
Prism · WorldConstruction
✦
05
Modeling
Animus · EntitySystems
⟳
06
Choreography
Loom · EntitySystems
⊙
07
Behavior
Instinct · EntitySystems
⚑
08
Society
Order · EntitySystems
▶
09
Sequencer
Chronicle · NarrativeSystems
✎
10
Story
Quill · NarrativeSystems
◎
11
Memory
Codex · NarrativeSystems
♪
12
Sound
Composer · NarrativeSystems
⊢
13
Logic
Axiom · PipelineSystems
∿
14
Simulation
Continuum · PipelineSystems
⚒
15
Forge
Forge · PipelineSystems
⇋
16
Network
Nexus · PipelineSystems
01 — Terrain (Atlas · WorldConstruction)
The Terrain module is the foundation of every world. It generates and manages the hex grid — a pointy-top axial coordinate system with radius 14 (631 hexes) that defines the world's surface geometry, elevation, and biome base layer.

Key Controls
World Seed: Integer seed for deterministic terrain generation. Same seed = same terrain, always.
Elevation Scale: Multiplier for height variance. Low values produce flat worlds; high values produce mountainous terrain.
Erosion Passes: Number of hydraulic erosion simulation passes applied to the heightmap.
Biome Distribution: Controls the Voronoi cell sizes used to assign biome types to hex regions.
Hex Grid: The interactive hex grid canvas. Click a hex to select it and view its elevation, biome, and moisture values.
Outputs (SPA wire)
The Terrain module outputs a SPA (Spatial) wire carrying the full heightmap, biome map, and hex coordinate registry. This feeds directly into the Environment module (02).

02 — Environment (Mythos · WorldConstruction)
Environment receives the terrain SPA wire and layers climate systems on top of the physical world. Wind, precipitation, temperature gradients, and seasonal cycles are all defined here.

Key Controls
Climate Zone: Global climate preset (Tropical, Temperate, Arid, Polar, Exotic).
Wind Systems: Direction and intensity of prevailing wind patterns. Wind affects precipitation distribution and erosion in real-time.
Precipitation Map: Rainfall distribution across biomes. High rainfall hexes develop vegetation and water features.
Seasonal Cycle: Enable/disable seasons and configure cycle length in simulation ticks.
Atmospheric Density: Affects sky color, fog, and the effective range of the Lighting module's sun/moon.
03 — Architect (Architect · WorldConstruction)
The Architect module places and manages structural elements — buildings, roads, bridges, fortifications, and infrastructure — on the terrain. It reads spatial data from Terrain and environmental constraints from Environment.

Key Controls
Settlement Templates: Pre-built architectural templates for villages, cities, outposts, and ruins.
Road Network: Pathfinding-based road generator. Define origin/destination hex pairs and the system finds optimal paths respecting terrain slope.
Building Density: Controls how many structures spawn per settled hex.
Era Selector: Architectural style preset by historical era (Ancient, Medieval, Industrial, Future, Alien).
Ruination Factor: 0–1 slider adding decay, rubble, and overgrowth to structures.
04 — Lighting (Prism · WorldConstruction)
Lighting controls all light sources in the world — the sun, moon, ambient sky, and artificial light points. It reads the atmospheric density from Environment to calculate scatter and fog correctly.

Key Controls
Sun Position: Azimuth and elevation of the primary star. Drives day/night cycle when linked to the Sequencer.
Color Temperature: Kelvin value for the sun's color. Low values (warm/sunset), high values (cool/noon).
Moon Phase: Controls secondary light intensity and night atmosphere.
Ambient Fill: Base light level for areas not in direct sunlight.
Light Points: Place individual light sources on the hex grid (fires, torches, windows, streetlamps).
05 — Modeling (Animus · EntitySystems)
The Modeling module is where entities are designed. It provides a parametric character and creature builder based on the B-DNA system — define an entity's physical attributes and the system generates a deterministic visual representation.

Key Controls
Entity Archetypes: Base templates (Humanoid, Creature, Construct, Spirit, Hybrid).
Morphology Sliders: Height, mass, limb ratios, facial structure — all feed into B-DNA generation.
Trait Tags: Keyword tags (e.g., armored, luminescent, aquatic) that activate specific geometry modules.
Variant Generation: Spin up N variants of an entity with genetic drift — useful for populating a species.
Export to Forge: Send the entity mesh to the Forge module (15) for asset packaging.
06 — Choreography (Loom · EntitySystems)
Choreography manages how entities move through the world — locomotion systems, animation state machines, formation logic, and path following. The module crest "Loom" reflects its role weaving movement into the world fabric.

Key Controls
Locomotion Type: Biped, Quadruped, Flight, Aquatic, Crawler — changes the base movement graph.
Animation States: Define states (Idle, Walk, Run, Combat, Sleep) and the conditions that transition between them.
Formation Editor: Group movement patterns for squads and flocks.
Path Constraints: Restrict movement to roads, avoid terrain types, or prefer cover.
Motion Capture Import: Import BVH or JSON motion data from external tools.
07 — Behavior (Instinct · EntitySystems)
Behavior defines the decision-making logic of entities using behavior trees and instinct weights. The "Instinct" crest reflects the module's role giving entities drives that feel organic rather than scripted.

Key Controls
Behavior Tree Editor: Visual node-based tree editor. Sequence, Selector, Parallel, and Decorator nodes available.
Instinct Weights: Sliders for primary drives: Survival, Curiosity, Aggression, Sociality, Loyalty. These weights influence tree branch selection.
Perception Range: How far an entity can "see" other entities and events on the hex grid.
Memory Horizon: How many simulation ticks the entity remembers past events (feeds Memory module 11).
Goal Stack: Priority-ordered goal list that the behavior tree works to satisfy.
08 — Society (Order · EntitySystems)
Society manages the macro-level organization of entities into factions, settlements, trade networks, and political structures. The "Order" crest represents the module's role imposing structure on individual behavior.

Key Controls
Faction Editor: Create named factions with flags, doctrines, and territory hexes.
Relationship Matrix: Define alliance/hostility relationships between factions.
Economic Model: Simple resource-flow model: production, trade routes, taxation.
Government Type: Monarchy, Oligarchy, Democracy, Theocracy, Anarchy — affects decision-making rules in the Logic module.
Population Dynamics: Birth rate, death rate, migration patterns driven by Environment and Simulation outputs.
09 — Sequencer (Chronicle · NarrativeSystems)
The Atlas Sequencer (distinct from the Loom Track Sequencer in the header nav) is a timeline-based event scheduler for world history. Chronicle records the past while scheduling the future.

Key Controls
World Timeline: A horizontal timeline measured in world-ticks. Drag events onto the timeline to schedule them.
Event Library: Categorized events (Natural Disasters, Wars, Discoveries, Migrations, etc.) that can be dropped onto the timeline.
Causality Chains: Link events so that Event A triggering causes Event B to become more/less likely.
History Playback: Step through world history tick by tick, watching modules update in response.
Chronicle Export: Export the world's event history as a structured JSON narrative document.
10 — Story (Quill · NarrativeSystems)
Story is the narrative authoring module. It provides a dialogue tree editor, story arc builder, and branching narrative composer powered by the world's entity and faction data from modules 05–08.

Key Controls
Story Arc Canvas: Visual canvas for designing branching story arcs with node-based chapters.
Dialogue Editor: Write conversations with entity-specific voice parameters. Supports condition branches based on B-DNA tags and instinct weights.
Quest Designer: Define objectives, prerequisites, and rewards linked to world-state conditions.
Arc Variables: Named variables tracked across an arc's progress (flags, counters, inventory).
Narrative Export: Export arcs as Ink/Twine-compatible JSON or as raw NAR wire data.
11 — Memory (Codex · NarrativeSystems)
Memory is the world's persistent history store. Every event that passes through the Sequencer and Story modules is recorded here, indexed by entity B-DNA, location, time, and type. The Codex holds the world's accumulated knowledge.

Key Controls
Event Log: Scrollable, filterable log of all recorded world events.
Entity Memory View: Select an entity by B-DNA and see all events it has witnessed or participated in.
Codex Search: Full-text search across all recorded events and narrative documents.
Memory Decay: Optional forgetting curve — older events fade in influence over time if not reinforced.
Codex Export: Export as structured Markdown, HTML, or JSON for use in external wikis or game databases.
12 — Sound (Composer · NarrativeSystems)
The Sound module — crest Composer — is the world's sonic layer. It maps soundscapes to terrain zones, attaches audio events to narrative triggers, and feeds the Instrument Rack for real-time synthesis.

Key Controls
Soundscape Map: Assign ambient sound profiles to biome types and architectural zones on the hex grid.
Event Audio: Link audio cues to Sequencer events (battle sounds, weather audio, ambient transitions).
Dynamic Mix: Real-time mix parameters that blend soundscapes as the simulation runs.
Rack Send: Route synthesized sound to the Instrument Rack for live performance and recording.
Audio Asset Browser: Browse /media/audio/ project folder for WAV, MP3, OGG files to assign.
13 — Logic (Axiom · PipelineSystems)
The Logic module is the rule engine of the world. It evaluates LGC-wire predicates and conditions, drives conditional events, and enforces world-law constraints. Axiom is where you define what is possible in your world.

Key Controls
Rule Editor: Define IF/THEN rules using a simple predicate language. Rules evaluate against world-state each simulation tick.
Constraint Library: Pre-built world-law constraints (physics limits, magic rules, technology restrictions).
Cycle Detection: Toggle to enable/disable circular dependency detection in logic graphs.
Debug Log: Real-time log of which rules are firing and their evaluated results.
Schema Editor: Define custom data schemas for MET-wire metadata exchanged between modules.
14 — Simulation (Continuum · PipelineSystems)
Simulation runs the world forward in time. It orchestrates all module systems, steps the simulation clock, evaluates entity behavior trees, processes economic flows, and fires scheduled events from the Sequencer.

Key Controls
Tick Rate: Simulation clock speed in ticks/second. Higher rates = faster world evolution, more CPU usage.
Simulation Depth: Which modules participate in each tick cycle. Deselect modules to freeze their state while the rest of the world advances.
Run / Pause / Step: Standard playback controls for the simulation clock.
Entropy Meter: Displays current world entropy — high entropy indicates chaotic, unpredictable emergent states.
Snapshot at Tick: Automatically save a Genesis snapshot at specified tick intervals.
15 — Forge (Forge · PipelineSystems)
The Forge is the asset production pipeline. It takes entity models from Modeling (05), architectural structures from Architect (03), and world data from all modules and packages them into exportable asset formats.

Key Controls
Asset Queue: Items queued for processing — models, textures, audio, narrative documents.
Export Formats: GLB/GLTF (3D), PNG/EXR (textures), WAV/OGG (audio), JSON (data), HTML (narrative pages).
Batch Processor: Process the entire asset queue in one operation.
p5.js Sketch Compiler: Compile p5.js sketches from the project's /media/sketches/ folder into standalone HTML canvases.
Capsule Packager: Wrap a processed asset set into a Capsule with B-DNA metadata for insertion into a Genesis Container.
16 — Network (Nexus · PipelineSystems)
The Network module manages multi-agent communication, external connections, and real-time data exchange between running world instances. Nexus is where your world connects to other worlds and to external systems.

Key Controls
Agent Registry: Shows all agents mounted in the local quantum-agent.db registry.
Connection Graph: Visual graph of inter-world wire connections and their current data flow.
Protocol Selector: WebSocket, WebRTC, or SharedMemory for local/remote agent communication.
Broadcast Rules: Define which wire types are broadcast to connected worlds vs. kept local.
Sync Monitor: Live monitor of incoming/outgoing data packets across all active connections.
6
The Instrument Rack
Overview
The Instrument Rack is a modular audio synthesis workspace accessible via the Rack tab in the header navigation. It is tightly coupled with the Sound module (12) but can also operate standalone as a synthesis environment.

The Rack uses a channel-strip metaphor: each instrument occupies a vertical strip with source, effects chain, and mixer controls. Up to 16 instruments can be loaded simultaneously.

Loading Instruments
Click the + button in the rack header to add a new instrument slot.
Choose an instrument type: Synthesizer, Sampler, Drum Machine, Soundscape Generator, or Ambient Pad.
Drag audio assets from the Asset Browser into an instrument slot to load samples.
World-generated soundscapes from the Sound module appear in the World Sources panel on the left side of the Rack.
The Effects Chain
Each channel strip has a 6-slot effects chain. Available effect types:

Reverb / Convolution: Room, Hall, Cathedral, Custom IR
Delay: Tempo-synced or free-running, with feedback control
Filter: Low-pass, High-pass, Band-pass, Notch with resonance
Compressor: Threshold, ratio, attack, release
EQ: 8-band parametric
Distortion / Saturation: Soft clip, hard clip, bit crush
The Master Bus
The rightmost channel strip is the Master Bus. It controls the final output level and hosts the master effects chain. The Master Bus also shows the current world state indicators: simulation tick, active biome, and faction count — allowing you to compose music that responds to world events.

Rack & Sound Module Integration
When the Rack is open alongside an active simulation, the Sound module (12) sends real-time audio events via AUD wires to the Rack. The Rack's World Trigger system maps these events to MIDI-like note triggers on loaded instruments. Configure the mapping in the Sound module's Rack Send panel.

7
Workflows & Common Tasks
Creating a World from Scratch
New Genesis: Ctrl+N → Name your world and set a seed. The seed can be any integer.
Terrain (01): Set elevation scale, erosion passes, and biome distribution. Hit Generate. Inspect the hex grid and adjust until satisfied.
Environment (02): Choose a climate zone and configure wind and precipitation. Watch the terrain update with vegetation and water.
Architect (03): Place settlements. Use the road generator to connect them.
Lighting (04): Set sun position and color temperature for your world's sun. Adjust seasons if enabled.
Modeling (05): Design the primary entity archetype for your world's dominant species.
Continue downstream: Work through Choreography → Behavior → Society → Sequencer → Story → Memory → Sound → Logic → Simulation → Forge → Network as your world complexity grows.
Export: Ctrl+E to export a .qgenesis file at any stage.
Importing and Continuing an Existing World
Drag a .qgenesis file onto the hub canvas, or click Import Genesis in the Quick Actions panel.
In the import modal, review the container metadata. Select all modules (or only specific ones) and click Apply.
Each module's UI will populate with the imported data. Open modules from the Atlas Footer to inspect and edit.
Resume simulation or editing normally. Export a new .qgenesis when done.
Running a Simulation
Ensure all upstream modules (01–13) are configured and have valid snapshots.
Open Simulation (14) from the Atlas Footer.
Set the tick rate and simulation depth (which modules participate).
Click Run. Watch entity behavior, faction dynamics, and events evolve in real-time.
Use the Step button to advance one tick at a time for careful inspection.
Pause and export a genesis snapshot at any point to save the simulation state.
Building a Narrative Arc
Populate Behavior (07) and Society (08) first — arcs reference entity instincts and faction relationships.
Open Sequencer (09) and lay out the historical events that precede your story.
Open Story (10) and design your arc canvas. Connect story beats to Sequencer events using EVT wires.
Write dialogue in the Dialogue Editor, referencing entity B-DNA for speaker identity.
Connect arc triggers to Behavior changes — story choices can shift entity instinct weights.
Export the arc via Narrative Export in the Story module.
Using p5.js Sketches in a Project
Place .js sketch files in your project's /media/sketches/ folder.
In the Forge module (15), open the p5.js Sketch Compiler.
Select the sketch files to compile. Configure canvas dimensions and background color.
Click Compile. The Forge outputs standalone HTML canvas pages.
These pages can be embedded in narrative documents (Story module) or packaged into Capsules.
Note
p5.js sketch files are stored in your .qgcp project directory, not in the .qgenesis container. The genesis contains only compiled references; the source files live in your project.
Connecting Worlds via the Network Module
Open both worlds in separate Quantum Loom instances on the same machine (or networked machines).
In each world, open Network (16) and verify the Agent Registry shows both world instances.
In the Connection Graph, drag a wire from one world's output port to the other world's input port.
Select the wire type (AGT, DAT, EVT, etc.) for the connection.
Configure Broadcast Rules to control what data flows across the connection.
Start both simulations. Real-time data will flow between the worlds.
8
Shortcuts, Glossary & Reference
Keyboard Shortcuts
Shortcut	Action	Context
Ctrl+N	New Genesis	Global
Ctrl+O	Open .qgenesis / .qgcp	Global
Ctrl+S	Save Snapshot	Active Module
Ctrl+E	Export Genesis	Global
Ctrl+I	Import Genesis	Global
Ctrl+Z	Undo	Module Editors
Ctrl+Y	Redo	Module Editors
Ctrl+B	Toggle Asset Browser	Global
Ctrl+F	Global Search	Global
Ctrl+G	Agent Manager	Global
Ctrl+]	Toggle Right Panel	Global
Ctrl+,	Settings	Global
F1	Open Manual	Global
F5	Run Simulation	Simulation Module
F6	Pause Simulation	Simulation Module
F7	Step One Tick	Simulation Module
Space	Play / Pause (Rack)	Instrument Rack
1–9	Switch to Atlas Module 1–9	Atlas Footer focused
Escape	Close Modal / Deselect	Global
Tab	Cycle through module panels	Module Editors
Ctrl+D	Duplicate selected node/entity	IO Nodes, Editors
Delete	Delete selected node/entity	Editors
Ctrl+A	Select all	Editors
Atlas Module Quick Reference
#	Module	Crest	Glyph	Color	Department	Primary Output Wire
01	Terrain	Atlas	⬡	#1e8cff	WorldConstruction	SPA
02	Environment	Mythos	◉	#8050e0	WorldConstruction	SPA, ENR
03	Architect	Architect	⊓	#64b4ff	WorldConstruction	VIS, AST
04	Lighting	Prism	◐	#e0d8ff	WorldConstruction	VIS
05	Modeling	Animus	✦	#f4c025	EntitySystems	VIS, IDN
06	Choreography	Loom	⟳	#dc3c78	EntitySystems	BHV
07	Behavior	Instinct	⊙	#9030d0	EntitySystems	BHV, LGC
08	Society	Order	⚑	#c8a860	EntitySystems	SOC, DAT
09	Sequencer	Chronicle	▶	#b08030	NarrativeSystems	TMP, EVT
10	Story	Quill	✎	#8c50ff	NarrativeSystems	NAR, EVT
11	Memory	Codex	◎	#00c060	NarrativeSystems	DAT, NAR
12	Sound	Composer	♪	#dc8c1e	NarrativeSystems	AUD
13	Logic	Axiom	⊢	#20c8d0	PipelineSystems	LGC, CTL
14	Simulation	Continuum	∿	#30e060	PipelineSystems	ENR, CTL
15	Forge	Forge	⚒	#ff6400	PipelineSystems	AST, MET
16	Network	Nexus	⇋	#ffffff	PipelineSystems	AGT, DAT
Glossary
Term	Definition
Agent	An autonomous world entity or external process registered in the mount registry and capable of sending/receiving wire data.
AES-256-GCM	Symmetric encryption algorithm used to protect the QGCP manifest. Requires the seal file for decryption.
argon2id	Memory-hard key derivation function used to derive the AES key from the seal file passphrase.
Atlas	The 16-module procedural world simulation engine at the core of Quantum Loom.
Atlas Footer	The 72px strip at the bottom of the hub containing buttons for all 16 Atlas modules.
B-DNA	Deterministic entity identity hash — 0x + 16 hex characters derived via LCG from entity lineage and spawn params.
Behavior Tree	Hierarchical decision structure used by the Behavior module (07) to drive entity AI.
Biome	A geographic climate region on the hex grid (Forest, Desert, Arctic, Ocean, etc.) assigned by the Terrain and Environment modules.
Capsule	The atomic content unit in the Genesis hierarchy. Holds one world element with B-DNA, metadata, and wire connections.
Chronicle	The crest name of the Sequencer module (09). Also refers to the world's recorded event history.
Codex	The crest name of the Memory module (11). Also refers to the world's knowledge store.
Container	The third level of the Genesis hierarchy. Contains up to 16 Capsules.
Crest	The proper name of each module's identity system (e.g., "Atlas" for Terrain, "Mythos" for Environment).
Genesis Container	The top-level world definition unit. Contains all Atlas module states and up to 4,096 Capsules.
Hex Grid	The world surface geometry — pointy-top axial coordinates, radius 14, 631 hexes total.
Hub	The main Quantum Loom application window, hosting all modules in an iframe architecture.
IDN wire	Identity wire type. Carries B-DNA hashes and entity metadata between modules.
Instinct Weight	A 0–1 value in the Behavior module representing the strength of a primal drive (Survival, Curiosity, etc.).
IO Nodes	The node-graph view for visually wiring module outputs to inputs.
LCG	Linear Congruential Generator — the algorithm used to derive B-DNA deterministically.
Loom Track Sequencer	The timeline-based playback tool in the header nav Sequencer tab. Distinct from Atlas module 09 (Chronicle).
Manifest	The encrypted metadata record inside a .qgcp file containing project identity, genesis binding, and lifecycle state.
Mount Registry	SQLite database at ~/.quantum/quantum-agent.db tracking all active Genesis and QGCP instances.
Mythos Container	The second level of the Genesis hierarchy. Contains up to 16 Containers.
p5.js Sketch	A creative coding script (p5.js library) stored in the project's /media/sketches/ folder and compiled by the Forge module.
Pipeline	The ordered chain of 16 Atlas modules through which world data flows from Terrain (01) to Network (16).
QGCP	Quantum Creative Project — the .qgcp save format for creative sessions. Contains assets, media, and an encrypted manifest.
QGenesis	The .qgenesis world definition format — analogous to an audio soundbank.
Rack	The Instrument Rack — the audio synthesis workspace. Accessible via the Rack tab in the header nav.
Seal File	A user-held keyfile whose passphrase is used to derive the AES-256-GCM key for QGCP manifest encryption.
Simulation Tick	One discrete time step in the world simulation. All entity behaviors, economic flows, and scheduled events are evaluated per tick.
Snapshot	The serialized state of one Atlas module stored in localStorage. Snapshots feed downstream modules and are bundled into .qgenesis exports.
SPA wire	Spatial wire type. Carries hex coordinates, zone boundaries, and terrain heightmaps.
Theme Tokens	CSS custom property values (color, font) synchronized across all modules via the theme sync IIFE and postMessage events.
Wire	A typed data channel connecting module outputs to module inputs in the IO Nodes graph.
World Seed	An integer value that deterministically initializes the Terrain module's procedural generation.
Troubleshooting
Module shows empty / no data: Open the module and check if a snapshot exists in the Quick Actions → Snapshot Data panel. If empty, configure the module and hit Save Snapshot.
Genesis import does nothing: Ensure the .qgenesis file is valid JSON with a genesis.atlas key. Open browser dev tools and check the console for parse errors.
Themes not applying to a module: The module may be missing the theme sync IIFE. Open the module page and verify the ql-theme localStorage key is present.
Simulation runs but entities don't move: Check that Choreography (06) and Behavior (07) modules have valid snapshots and are included in the Simulation Depth selection.
QGCP manifest decryption fails: Ensure you are using the correct seal file for this project. The manifest cannot be decrypted with a different seal file.
p5.js sketch not compiling: Verify the sketch uses p5.js instance mode (new p5(sketch)) — global mode sketches may conflict with the Forge compiler.
Wire connection rejected: Check wire type compatibility. DAT wires are accepted by all modules. Specialized wire types require the receiving module to declare that input type.
Tip
Open the browser developer tools (F12) in any module for a live view of localStorage snapshot data and postMessage events.
