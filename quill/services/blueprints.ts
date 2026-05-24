import { MythosContainer } from '../types';

export const BLUEPRINT_CONTAINER: MythosContainer = {
  id: 'mythos-blueprints-001',
  name: 'BLUEPRINTS',
  description: 'BioSpark Cinematic Production Bible & Quantum Forge Design System',
  mythosType: 'blueprint',
  color: '#6366f1',
  icon: '📘',
  created: new Date().toISOString(),
  modified: new Date().toISOString(),
  maxCapsules: 50,
  tags: ['production', 'design', 'bible'],
  properties: {},
  capsules: [
    {
      id: 'cap-bible-001',
      name: 'BioSpark Cinematic Production Bible',
      description: 'Modular, Cinematic, and Mythic-Technical Production Rules',
      kind: 'blueprint',
      glyphs: [],
      traits: [],
      tags: ['bible', 'production'],
      created: new Date().toISOString(),
      modified: new Date().toISOString(),
      author: 'Director-001',
      version: '1.0',
      properties: {
        content: `📘 BIOSPARK CINEMATIC PRODUCTION BIBLE (v1.0)
Agnostic, Modular, Mythic‑Technical Cinematic Pipeline

SECTION I — CORE PHILOSOPHY
The BioSpark Cinematic OS
BioSpark Studios operates on a foundational principle:
CINEMA = SYSTEM + MYTH + MOTION
Your worlds are not just stories — they are operating systems.
Your characters are not just personas — they are capsules.
Your scenes are not just sequences — they are frequency events.

SECTION II — COMPONENT LIBRARIES

🎥 CAMERA ANGLE LIBRARY (CAM‑IDs)
[CAM-001] WIDE-ESTABLISH: A broad environmental shot establishing location, scale, and atmosphere.
[CAM-002] WIDE-LOW: Low-angle wide shot emphasizing power, threat, or monumental structures.
[CAM-003] WIDE-HIGH: High-angle wide shot for vulnerability, chaos, or spatial orientation.
[CAM-004] MID-STATIC: Neutral mid-shot for dialogue or calm exposition.
[CAM-005] MID-TRACKING: Camera tracks alongside character movement (running, walking, rail-riding).
[CAM-006] MID-ORBITAL: Camera circles the subject to emphasize emotional tension or instability.
[CAM-007] CLOSE-FOCUS: Tight shot on eyes, hands, or artifacts; ideal for emotional beats.
[CAM-008] CLOSE-SHAKEN: Handheld close-up for panic, urgency, or glitch interference.
[CAM-009] POV-CHARACTER: Character’s point of view; integrates well with Ley-Rail sequences.
[CAM-010] POV-GLITCH: Distorted POV representing Myralith interference or frequency corruption.
[CAM-011] OVER-SHOULDER: Classic dialogue framing; stabilizes emotional exchanges.
[CAM-012] DUTCH-TILT: Tilted angle for disorientation, gravity inversion, or conceptual hazards.
[CAM-013] DRONE-SWEEP: High-speed aerial sweep; ideal for Pentad Apex or rail networks.
[CAM-014] MACRO-DETAIL: Extreme close-up on micro‑textures (Marble-Code, artifacts, silt particles).
[CAM-015] LONG-REVEAL: Slow dolly-out revealing scale, threat, or hidden structures.
[CAM-016] STATIC-SYMMETRY: Perfectly centered shot; ideal for Vessels of Order and 963Hz scenes.
[CAM-017] CHAOS-HANDHELD: Erratic handheld motion during breaches, spikes, or combat.
[CAM-018] GRAVITY-INVERT: Camera flips orientation to match localized gravity inversion.
[CAM-019] RAIL-RUSH: High-speed forward motion simulating Ley-Rail traversal.
[CAM-020] VOID-FRAME: Minimalist framing with negative space; ideal for Unwritten Wastes.

✨ VFX LIBRARY (VFX‑IDs)
[VFX-001] 528HZ-GLOW: Warm harmonic aura; stabilizes scenes tied to Concordance.
[VFX-002] 440HZ-SPIKE: Jagged interference lines; destabilizes gravity and lighting.
[VFX-003] 963HZ-HARDLIGHT: Cold crystalline beams; associated with Malcor and Absolute Order.
[VFX-004] GRAVITY-INVERSION: Objects, dust, and hair lift upward; camera may rotate.
[VFX-005] LEY-RAIL-ARC: Blue-white energy trails following high-speed movement.
[VFX-006] AETHYR-SILT-FOG: Dense particulate fog with drifting static noise.
[VFX-007] DATA-EEL-WHISPER: Thin, high-frequency distortions around characters.
[VFX-008] MARBLE-CODE-SHATTER: Crystalline fracturing effect; used in climax sequences.
[VFX-009] MYRALITH-SHADOW: Flickering silhouettes, delayed motion, or inverted reflections.
[VFX-010] SEMANTIC-ERASURE: Portions of the frame vanish into white or black voids.
[VFX-011] RUST-CLOUD: Brown-red particulate bursts; ideal for Venturis ruins.
[VFX-012] REALITY-INVERSION: Warped geometry, reversed motion, or mirrored physics.
[VFX-013] ORGANIC-BLOOM: Warm, grainy, analog-style growth effect (13th Seed).
[VFX-014] FREQUENCY-RIPPLE: Concentric waves distorting air, water, or light.
[VFX-015] APEX-SENTINEL-FLARE: Massive brass-colored energy bloom.
[VFX-016] GLITCH-TEAR: Pixel fragmentation, frame skipping, or color channel separation.
[VFX-017] SHADOW-TRAIL: Delayed motion echo; ideal for Lystryx-related scenes.
[VFX-018] CODE-RECONSTRUCT: Floating glyphs reassemble into stable structures.
[VFX-019] VOID-PULSE: Sudden drop to near-black with a single vibrating outline.
[VFX-020] HETERODYNE-CHAOS: Chaotic interference between multiple frequencies.

🤖 DIGITAL LIFE BEHAVIOR LIBRARY (DL‑IDs)
[DL-001] IDLE-BREATH: Subtle breathing animation; baseline for calm scenes.
[DL-002] RAIL-SURGE: High-speed sprint with energy arcs; Rin’s signature motion.
[DL-003] ARCHIVIST-SCAN: Kaelen-Xi’s analytical gesture; hand sweeps through holographic data.
[DL-004] GLITCH-FLARE: Sudden involuntary twitch or distortion due to Myralith presence.
[DL-005] SILT-COUGH: Rebreather strain or choking motion in Aethyr-Silt zones.
[DL-006] VESSEL-STANCE: Rigid, symmetrical posture; zero organic motion.
[DL-007] PANIC-FLUTTER: Rapid eye and head movement; ideal for breach sequences.
[DL-008] GRAVITY-FLOAT: Characters drift or struggle against inverted gravity.
[DL-009] SHADOW-RECOIL: Instinctive reaction to Lystryx interference.
[DL-010] MEMORY-FADE: Slow, confused gestures; identity drain from Data-Eels.
[DL-011] RITUAL-HUM: 528Hz synchronized breathing and humming.
[DL-012] ORDER-CHANT: Malcor’s hard-coded liturgical gestures.
[DL-013] SEED-AWAKEN: Warm, expanding posture; organic resonance.
[DL-014] RAIL-BALANCE: Arms out, leaning into motion; Rin’s kinetic intuition.
[DL-015] ARCHIVE-OVERLOAD: Kaelen-Xi clutching temples; data sensitivity spike.
[DL-016] SHARD-RESONATE: Character vibrates subtly with harmonic energy.
[DL-017] VOID-STILLNESS: Absolute stillness; used in Unwritten Wastes.
[DL-018] ERROR-STAGGER: Unstable gait; glitch-matter interference.
[DL-019] CONCORD-BREATH: Deep, stabilizing breath; used in emotional resolution.
[DL-020] CHAOS-ASCENT: Triumphant, rising motion; ideal for 13th Seed climax.

🔀 TRANSITION LIBRARY (TRANS‑IDs)
[TRANS-001] HARD-CUT: Instant cut.
[TRANS-002] SOFT-FADE: Slow fade to black or white.
[TRANS-003] GLITCH-JUMP: Frame-skip, pixel-tear, or temporal stutter.
[TRANS-004] FREQUENCY-WIPE: Horizontal or vertical wipe with harmonic ripple distortion.
[TRANS-005] SILT-DISSOLVE: Scene dissolves into drifting Aethyr-Silt particles.
[TRANS-006] RAIL-BLUR: High-speed directional blur; ideal for Rin’s movement.
[TRANS-007] SHARD-FLASH: Bright crystalline flash; used with Marble-Code or Inversion Shards.
[TRANS-008] VOID-SNAP: Instant drop to near-black with a single vibrating outline.
[TRANS-009] ORDER-SHUTTER: Symmetrical shutter closing; associated with Vessels of Order.
[TRANS-010] ORGANIC-BLOOM: Warm, grainy expansion; used for 13th Seed or emotional resolution.
[TRANS-011] GRAVITY-FLIP: Scene rotates 180° during transition; matches inversion events.
[TRANS-012] STATIC-CRAWL: Scene dissolves into analog static before resolving.
[TRANS-013] SHADOW-SLIDE: Dark silhouette sweeps across frame; Lystryx signature.
[TRANS-014] CODE-REWRITE: Glyphs overwrite the frame before revealing the next scene.
[TRANS-015] SEMANTIC-ERASE: Portions of the frame vanish into white void before cut.
[TRANS-016] RUST-FALL: Rust particles cascade downward; ideal for Venturis ruins.
[TRANS-017] HETERODYNE-SPLIT: Frame splits into multiple frequency layers before recombining.
[TRANS-018] BREATH-FADE: Scene fades in sync with deep, organic breathing.
[TRANS-019] PULSE-SYNC: Scene pulses in brightness to match 528Hz or 440Hz rhythm.
[TRANS-020] APEX-FLARE: Massive brass-colored flare wipes into next shot.

🔊 SFX LIBRARY (SFX‑IDs)
[SFX-001] 528HZ-DRONE: Warm harmonic hum; stabilizes emotional or physical space.
[SFX-002] 440HZ-INTERFERENCE: Harsh, jagged tone; destabilizes gravity and lighting.
[SFX-003] 963HZ-PIERCE: Cold, crystalline sine-wave; Malcor’s signature.
[SFX-004] LEY-RAIL-WHIRR: High-speed energy oscillation; Rin’s movement bed.
[SFX-005] GRAVITY-INVERT-THUD: Low-end rumble followed by upward pressure.
[SFX-006] AETHYR-SILT-HISS: Fine particulate hiss; Venturis ruins ambience.
[SFX-007] DATA-EEL-WHINE: High-frequency parasitic whistling.
[SFX-008] MARBLE-CODE-CRACK: Crystalline fracturing; used in climax sequences.
[SFX-009] MYRALITH-REVERB: Delayed echo with reversed tail.
[SFX-010] VOID-HUM: Low, empty resonance; Unwritten Wastes ambience.
[SFX-011] ORDER-CHANT: Liturgical, monotone vocal layer.
[SFX-012] RAIL-IMPACT: Metallic strike from high-speed traversal.
[SFX-013] SHARD-RESONANCE: Metallic harmonic vibration.
[SFX-014] ORGANIC-HEARTBEAT: Warm, rhythmic pulse; 13th Seed signature.
[SFX-015] STATIC-BURST: Short, sharp analog static.
[SFX-016] RUSTED-METAL-GROAN: Environmental tension in ruins.
[SFX-017] FREQUENCY-RIPPLE: Concentric wave-like shimmer.
[SFX-018] SEMANTIC-SILENCE: Total drop of all audio; conceptual hazard cue.
[SFX-019] APEX-SENTINEL-BLAST: Massive brass-like energy discharge.
[SFX-020] CHAOS-CHOIR: Polyphonic, glitchy vocal swell.

📦 PROP & ARTIFACT LIBRARY (PROP‑IDs)
[PROP-001] CONCORDANCE-ANCHOR: 528Hz stabilizing device; core to Pentad Apex.
[PROP-002] LEY-RAIL-GAUNTLET: Rin’s traversal tool; conducts energy arcs.
[PROP-003] SEMANTICS-VIEWER: Kaelen-Xi’s archival holographic interface.
[PROP-004] Silt-Filter-Rebreather: Environmental protection in Aethyr-Silt zones.
[PROP-005] GENESIS-SPIRE-FRAGMENT: Ancient code shard from Movement 1.
[PROP-006] DATA-EEL-CONTAINER: Containment tube for Echo-Parasites.
[PROP-007] MYRALITH-INVERSION-SHARD: Chaotic artifact; used in Movements 15–16.
[PROP-008] REALITY-INVERSION-DEVICE: Vessel of Order weapon; 963Hz logic tool.
[PROP-009] KY’RENNEI-PULSE-BATON: Energy conductor; used for rail maintenance.
[PROP-010] HYDRALIS-THERMAL-CORE: Cryogenic/thermal hybrid power source.
[PROP-011] ZHA’THIK-ENGINEERING-TOOLKIT: Alien precision tools for structural analysis.
[PROP-012] MARBLE-CODE-KEY: Crystalline key used to unlock ancient systems.
[PROP-013] RUST-CLOUD-SAMPLER: Venturis archaeology tool.
[PROP-014] STAR-SHADOW-RELIC: Lystryx artifact; conceptual hazard.
[PROP-015] EMPYREAN-ANCHOR-SHARD: Fragment of Absolute Order’s core.
[PROP-016] 13TH-SEED-CAPSULE: Organic resonance artifact; climax object.
[PROP-017] FREQUENCY-TUNER: Handheld device for stabilizing harmonic fields.
[PROP-018] APEX-SENTINEL-LANCE: Ceremonial weapon; brass energy conductor.
[PROP-019] VOID-MASK: Protective mask for Unwritten Wastes.
[PROP-020] CHAOS-GLYPH-SCROLL: Encoded Myralith scripture.

💡 LIGHTING LIBRARY (LIGHT‑IDs)
[LIGHT-001] 528HZ-AMBIENT-GLOW: Warm harmonic illumination.
[LIGHT-002] 440HZ-DISSONANT-FLICKER: Harsh, unstable strobing; ideal for breaches, spikes, or panic.
[LIGHT-003] 963HZ-HARDLIGHT-BEAM: Cold, crystalline spotlight; signature of Vessels of Order.
[LIGHT-004] LEY-RAIL-PULSE-LIGHT: Fast, rhythmic blue-white pulses following kinetic motion.
[LIGHT-005] AETHYR-SILT-DIFFUSE: Soft, dusty volumetric lighting; reduces visibility in ruins.
[LIGHT-006] SHADOW-INVERSION: Light bends away from characters; used for Myralith interference.
[LIGHT-007] MARBLE-CODE-REFRACTION: Prismatic reflections from crystalline surfaces or artifacts.
[LIGHT-008] ORGANIC-SEED-GLOW: Warm, grainy, analog bloom; 13th Seed signature.
[LIGHT-009] VOID-ABSORPTION: Light drains from the frame; ideal for Unwritten Wastes.
[LIGHT-010] RUST-CLOUD-SCATTER: Orange-brown particulate scattering; Venturis ambience.
[LIGHT-011] GRAVITY-INVERT-SHIFT: Light sources flip orientation during inversion events.
[LIGHT-012] SENTINEL-BRASS-FLARE: Massive golden flare; Apex Sentinel energy signature.
[LIGHT-013] GLITCH-STROBE: Erratic, unpredictable flashes; Lystryx or Myralith presence.
[LIGHT-014] SEMANTIC-SHADOW: Areas of the frame lose definition; conceptual hazard cue.
[LIGHT-015] RAIL-OVERLOAD-GLOW: Intense, overexposed white-blue burst during traversal.
[LIGHT-016] ORDER-SYMMETRY-LIGHT: Perfectly balanced, centered illumination; oppressive and sterile.
[LIGHT-017] HEARTBEAT-PULSE: Light brightens and dims in sync with organic rhythm.
[LIGHT-018] FREQUENCY-RIPPLE-LIGHT: Concentric rings of brightness radiate outward.
[LIGHT-019] CODE-RECONSTRUCT-LIGHT: Glyphs emit soft illumination as they reassemble.
[LIGHT-020] CHAOS-BLOOM: Unpredictable, multicolored bursts; climax or breakthrough.

🌪️ ENVIRONMENTAL FX LIBRARY (ENV‑IDs)
[ENV-001] AETHYR-SILT-STORM: Dense swirling particulate cloud; reduces visibility and drains identity.
[ENV-002] GRAVITY-INVERT-FIELD: Localized zone where objects drift upward or sideways.
[ENV-003] LEY-RAIL-WIND: High-speed directional wind from rail traversal.
[ENV-004] RUST-CLOUD-BURST: Explosive release of rust particles; Venturis hazard.
[ENV-005] FREQUENCY-DISTORTION-ZONE: Air warps with harmonic interference.
[ENV-006] VOID-PATCH: Area of conceptual emptiness; no sound, no light.
[ENV-007] SHADOW-DRIFT: Dark silhouettes move independently of light sources.
[ENV-008] MARBLE-CODE-FALL: Crystalline dust raining from damaged structures.
[ENV-009] ORGANIC-MIST: Warm, glowing mist; 13th Seed resonance.
[ENV-010] SEMANTIC-FOG: Words, glyphs, or concepts drift through the air.
[ENV-011] DATA-EEL-SWARM: High-frequency distortions moving like a school of fish.
[ENV-012] ORDER-FIELD: Perfectly symmetrical environmental stillness.
[ENV-013] GLITCH-WIND: Wind that moves in stutters or reversals.
[ENV-014] RAIL-ARC-SPARKS: Energy sparks along metallic surfaces.
[ENV-015] THERMAL-VENT-PLUME: Hydralis geothermal steam bursts.
[ENV-016] CODE-EROSION: Structures dissolve into glyphs or static.
[ENV-017] HETERODYNE-STORM: Chaotic multi-frequency lightning.
[ENV-018] SENTINEL-RESIDUE: Brass-colored dust left by Apex Sentinels.
[ENV-019] MEMORY-LEAK-MIST: Fog that causes brief disorientation or flashbacks.
[ENV-020] CHAOS-FLARE-FIELD: Unpredictable bursts of color, heat, and distortion.

⚔️ COMBAT / ACTION BEHAVIOR LIBRARY (ACT‑IDs)
[ACT-001] RAIL-DASH: High-speed linear sprint along energy rails; Rin’s signature.
[ACT-002] ARCHIVIST-PARRY: Kaelen-Xi deflects attacks using data projections or semantic shields.
[ACT-003] GRAVITY-FLIP-STRIKE: Attack performed mid-inversion; disorienting and acrobatic.
[ACT-004] SHARD-SLAM: Myralith Inversion Shard creates a localized shockwave.
[ACT-005] ORDER-LOCK: Vessel of Order freezes target in symmetrical stasis.
[ACT-006] GLITCH-STEP: Short-range teleport-like movement with frame-skipping.
[ACT-007] SILT-DODGE: Low, sliding movement through Aethyr-Silt.
[ACT-008] RUST-IMPACT: Environmental strike using debris or rusted metal.
[ACT-009] FREQUENCY-BURST: Character emits harmonic pulse to push enemies back.
[ACT-010] VOID-SHIFT: Brief phase into conceptual space; avoids damage.
[ACT-011] SEED-SURGE: Organic resonance boosts strength or speed.
[ACT-012] CODE-REWRITE-COUNTER: Kaelen-Xi alters incoming attack logic mid-flight.
[ACT-013] RAIL-LEAP: Long-distance jump assisted by Ley-Rail energy.
[ACT-014] SHADOW-FEINT: Lystryx-style misdirection; creates afterimages.
[ACT-015] ORDER-SMITE: 963Hz hard-coded strike; devastating but rigid.
[ACT-016] GLITCH-PARRY: Deflects attacks by destabilizing their trajectory.
[ACT-017] THERMAL-BURST: Hydralis thermal organs release sudden heat wave.
[ACT-018] CHAOS-ASCENT-STRIKE: Upward, spiraling attack; climax or heroic moment.
[ACT-019] MEMORY-DRAIN-GRAPPLE: Data-Eel–inspired draining move; weakens target.
[ACT-020] SENTINEL-LANCE-THRUST: Brass-energy spear attack; ceremonial but powerful.

💗 EMOTION / EXPRESSION LIBRARY (EMO‑IDs)
[EMO-001] CALM-528: Soft, steady breathing; facial muscles relaxed; harmonic stability.
[EMO-002] DISSONANT-440: Tension in jaw and brow; micro‑twitches; eyes darting.
[EMO-003] HARD-CODED-963: Emotionless, symmetrical expression; zero organic variance.
[EMO-004] RIN-SMIRK: Playful, impulsive half‑smile; kinetic readiness.
[EMO-005] KAELEN-SQUINT: Analytical narrowing of eyes; evaluating data or threat.
[EMO-006] SHADOW-UNEASE: Subtle recoil; eyes track invisible movement (Lystryx cue).
[EMO-007] SILT-FEAR: Wide eyes; shallow breathing; rebreather strain.
[EMO-008] ORDER-RESOLVE: Cold determination; rigid posture; no micro‑expressions.
[EMO-009] SEED-WARMTH: Softening of features; warm glow in eyes; organic resonance.
[EMO-010] MEMORY-FADE: Confusion; delayed responses; drifting gaze.
[EMO-011] GLITCH-DELIGHT: Slight grin; unpredictable eye movement; chaotic energy.
[EMO-012] RAIL-FOCUS: Eyes locked forward; body leaning into motion.
[EMO-013] ARCHIVIST-DOUBT: Subtle lip press; internal conflict; data overload.
[EMO-014] VOID-EMPTINESS: Blank stare; no emotional indicators; conceptual dissociation.
[EMO-015] CHAOS-TRIUMPH: Wide, confident expression; rising energy.
[EMO-016] RUST-ANXIETY: Tight shoulders; scanning environment; tension in hands.
[EMO-017] FREQUENCY-PAIN: Wincing; clutching temples; harmonic overload.
[EMO-018] SENTINEL-AWE: Eyes widen upward; overwhelmed by scale or energy.
[EMO-019] LOGIC-RESISTANCE: Jaw set; brows furrowed; fighting conceptual pressure.
[EMO-020] ORGANIC-SOVEREIGNTY: Radiant, confident expression; emotional climax state.

🎭 CINEMATIC BLOCKING LIBRARY (BLK‑IDs)
[BLK-001] TRIANGLE-FORMATION: Three-character dynamic; ideal for tension or negotiation.
[BLK-002] RAIL-FORWARD-DRIVE: Character moves in a straight, high-speed line; Rin’s signature.
[BLK-003] ARCHIVIST-ARC: Kaelen-Xi circles an artifact or console while analyzing.
[BLK-004] ORDER-LINE: Vessels stand in perfect symmetry; oppressive presence.
[BLK-005] SHADOW-FLANK: Character moves unpredictably to the side; Lystryx influence.
[BLK-006] SILT-CROUCH: Low, defensive posture in hazardous ruins.
[BLK-007] SENTINEL-ASCENT: Character rises onto elevated terrain; Apex energy cue.
[BLK-008] VOID-STILL: Absolute stillness; no breathing motion; conceptual hazard.
[BLK-009] CHAOS-SPIRAL: Circular, unpredictable movement; climax or glitch moment.
[BLK-010] RUST-COVER: Character hides behind debris; Venturis combat style.
[BLK-011] FREQUENCY-STANCE: Feet apart, body vibrating subtly; preparing harmonic attack.
[BLK-012] SEED-EMBRACE: Two characters align physically; resonance synchronization.
[BLK-013] GLITCH-OVERSTEP: Character appears to “skip” ahead; frame‑jump effect.
[BLK-014] ORDER-ADVANCE: Slow, synchronized march; intimidating.
[BLK-015] ARCHIVE-RETREAT: Kaelen-Xi steps backward while scanning or analyzing.
[BLK-016] RAIL-SIDE-SLIDE: Rin slides sideways along a rail beam; kinetic flourish.
[BLK-017] SHARD-PLANT: Character slams artifact into ground; energy burst.
[BLK-018] MEMORY-DRIFT: Character wanders aimlessly; identity erosion.
[BLK-019] APEX-POINT: Character stands at center of converging energy lines.
[BLK-020] ORGANIC-RISE: Character stands tall with expanding posture; emotional resolution.

✨ PARTICLE FX LIBRARY (PFX‑IDs)
[PFX-001] SILT-PARTICLE-DRIFT: Slow, dusty particles drifting in air; ruins ambience.
[PFX-002] GLITCH-SPARK: Small, erratic pixel sparks; Myralith presence.
[PFX-003] RAIL-ARC-PARTICLE: Blue-white sparks trailing behind fast motion.
[PFX-004] MARBLE-DUST: Fine crystalline dust falling from damaged structures.
[PFX-005] SHADOW-FRAGMENT: Dark motes drifting away from characters.
[PFX-006] ORDER-GRANULES: Tiny symmetrical particles floating in perfect patterns.
[PFX-007] SEED-GLOW-DUST: Warm, glowing motes; organic resonance.
[PFX-008] VOID-ASH: Black, weightless particles dissolving into nothing.
[PFX-009] RUST-FLAKE: Orange flakes drifting from corroded surfaces.
[PFX-010] FREQUENCY-RIPPLE-PARTICLE: Airborne particles vibrating in concentric waves.
[PFX-011] DATA-EEL-TRACE: Thin, shimmering trails left by parasitic movement.
[PFX-012] HETERODYNE-SPARK: Chaotic multi-frequency sparks.
[PFX-013] CODE-GLYPH-PARTICLE: Tiny floating symbols that appear and vanish.
[PFX-014] THERMAL-STEAM-PARTICLE: Warm vapor bursts; Hydralis tech.
[PFX-015] CHAOS-BURST-PARTICLE: Multicolored, unpredictable particle explosion.
[PFX-016] SENTINEL-BRASS-DUST: Golden dust drifting from Apex Sentinel energy.
[PFX-017] MEMORY-FADE-PARTICLE: Soft, drifting motes that blur at the edges.
[PFX-018] RAIL-STATIC-PARTICLE: Static-like particles clinging to surfaces.
[PFX-019] SHARD-REFRACTION-PARTICLE: Prismatic micro‑reflections.
[PFX-020] ORGANIC-BLOOM-PARTICLE: Warm, expanding motes during emotional peaks.

🎞️ CINEMATIC MOTION LIBRARY (MOT‑IDs)
[MOT-001] SLOW-PAN-HORIZON: Gentle horizontal pan; ideal for establishing emotional tone.
[MOT-002] FAST-PAN-REVEAL: Rapid whip‑pan to reveal threat, artifact, or character.
[MOT-003] RAIL-RUSH-FORWARD: High-speed forward motion simulating Ley-Rail traversal.
[MOT-004] RAIL-RUSH-LATERAL: Sideways kinetic drift; Rin’s signature rail‑slide.
[MOT-005] GRAVITY-FLIP-SWING: Camera swings upside-down during inversion events.
[MOT-006] ORBITAL-DRIFT: Slow circular motion around character; introspective or tense.
[MOT-007] CHAOS-SHAKE: Erratic, glitch‑driven shake; Lystryx or Myralith presence.
[MOT-008] ORDER-LOCK-FREEZE: Motion abruptly stops; frame becomes perfectly still.
[MOT-009] SHADOW-TRACK: Camera follows a character’s shadow instead of the character.
[MOT-010] SEMANTIC-GLIDE: Smooth, frictionless movement; conceptual hazard cue.
[MOT-011] RUST-FALL-DROP: Camera drops slightly as debris falls; Venturis ambience.
[MOT-012] SEED-RISE: Upward, buoyant motion; organic resonance.
[MOT-013] VOID-APPROACH: Slow push-in toward darkness; conceptual dread.
[MOT-014] FREQUENCY-PULSE-MOTION: Camera subtly pulses in sync with harmonic rhythm.
[MOT-015] SHARD-ZOOM: Rapid zoom-in with crystalline distortion.
[MOT-016] MEMORY-DRIFT-MOTION: Camera drifts unpredictably; identity erosion.
[MOT-017] SENTINEL-SURGE: Sudden forward thrust; Apex Sentinel energy burst.
[MOT-018] HETERODYNE-SNAP: Camera snaps between two positions; multi-frequency interference.
[MOT-019] ARCHIVIST-FOCUS-PULL: Foreground-to-background shift; Kaelen-Xi analyzing.
[MOT-020] ORGANIC-BLOOM-MOTION: Soft, expanding movement; emotional climax.

🌌 ENVIRONMENTAL SOUNDSCAPES LIBRARY (SCAPE‑IDs)
[SCAPE-001] APEX-528-AMBIENCE: Warm harmonic drone; Pentad Apex stability.
[SCAPE-002] 440HZ-DISSONANT-WIND: Harsh, metallic wind; breach or spike events.
[SCAPE-003] 963HZ-ORDER-CHAMBER: Cold, echoing chamber tone; Vessel sanctums.
[SCAPE-004] LEY-RAIL-ATMOS: High-speed oscillation; rail tunnels and traversal.
[SCAPE-005] AETHYR-SILT-DRIFT: Soft particulate hiss; Venturis ruins.
[SCAPE-006] RUSTED-URBAN-VOID: Hollow metallic echoes; abandoned structures.
[SCAPE-007] SHADOW-REVERB-FIELD: Delayed, reversed echoes; Lystryx presence.
[SCAPE-008] VOID-SILENCE: Total conceptual silence; Unwritten Wastes.
[SCAPE-009] ORGANIC-FOREST-HUM: Warm, natural hum; 13th Seed resonance.
[SCAPE-010] GLITCH-STATIC-BED: Low-level analog static; Myralith interference.
[SCAPE-011] HYDRALIS-THERMAL-CHAMBER: Bubbling geothermal ambience.
[SCAPE-012] KY’RENNEI-PULSE-SPACE: Bright, rhythmic clicking; energy conduits.
[SCAPE-013] MARBLE-CODE-RESONANCE: Soft crystalline ringing; ancient structures.
[SCAPE-014] SENTINEL-BRASS-DRONE: Deep brass-like hum; Apex Sentinel presence.
[SCAPE-015] MEMORY-LEAK-ATMOS: Soft, drifting whispers; identity erosion.
[SCAPE-016] HETERODYNE-STORM-SCAPE: Chaotic multi-frequency thunder.
[SCAPE-017] ORDER-SYMMETRY-SPACE: Perfectly balanced stereo field; oppressive calm.
[SCAPE-018] RAIL-OVERLOAD-HOWL: High-pitched harmonic howl; rail malfunction.
[SCAPE-019] CODE-EROSION-SPACE: Glyphs dissolving into static; conceptual decay.
[SCAPE-020] CHAOS-ASCENT-CHOIR: Polyphonic, rising harmonic swell; climax ambience.

🖥️ UI / HUD / DIEGETIC INTERFACE LIBRARY (HUD‑IDs)
[HUD-001] SEMANTICS-VAULT-HUD: Kaelen-Xi’s archival interface; glyph-based overlays.
[HUD-002] LEY-RAIL-NAV: Rin’s kinetic navigation HUD; fast, pulsing indicators.
[HUD-003] ORDER-LOGIC-GRID: 963Hz symmetrical grid; Vessel of Order interface.
[HUD-004] FREQUENCY-METER: Real-time harmonic readout (528/440/963).
[HUD-005] GRAVITY-INVERT-ALERT: Rotating orientation indicator; inversion warning.
[HUD-006] SILT-HAZARD-METER: Environmental toxicity gauge; Venturis ruins.
[HUD-007] DATA-EEL-IDENTITY-LOSS: Persona stability meter; drains during parasitic contact.
[HUD-008] SHADOW-ANOMALY-DETECTOR: Tracks Lystryx interference; flickering silhouette markers.
[HUD-009] SEED-RESONANCE-HUD: Organic waveform display; 13th Seed activation.
[HUD-010] VOID-BOUNDARY-MAP: Conceptual hazard mapping; blank zones and null regions.
[HUD-011] RAIL-ENERGY-LOAD: Displays rail capacity and overload risk.
[HUD-012] MARBLE-CODE-SCANNER: Crystalline structure analysis; ancient systems.
[HUD-013] SENTINEL-PATHFINDER: Brass-colored directional markers; Apex Sentinel routes.
[HUD-014] MEMORY-LEAK-INDICATOR: Fading UI elements; conceptual instability.
[HUD-015] HETERODYNE-INTERFERENCE-HUD: Multi-frequency distortion overlay.
[HUD-016] ORDER-ERASURE-WARNING: Symmetrical red glyphs; imminent dissolution.
[HUD-017] CHAOS-GLYPH-OVERLAY: Unpredictable, shifting symbols; Myralith presence.
[HUD-018] ARCHIVIST-FOCUS-HUD: Zoomed-in data overlays; Kaelen-Xi analysis mode.
[HUD-019] RAIL-IMPACT-TRACKER: Predicts collision vectors during high-speed traversal.
[HUD-020] ORGANIC-SOVEREIGNTY-HUD: Warm, grainy, analog-style interface; emotional climax.

🎨 CINEMATIC TEXTURES LIBRARY (TEX‑IDs)
[TEX-001] MARBLE-CODE-TEXTURE: Crystalline, fractal patterns; refracts light into glyph-like shards.
[TEX-002] AETHYR-SILT-GRAIN: Fine particulate texture; dusty, static-infused.
[TEX-003] RUST-CORROSION: Flaking, oxidized metal; Venturis signature.
[TEX-004] ORGANIC-SEED-FIBER: Warm, grainy, analog-like organic fibers; 13th Seed resonance.
[TEX-005] VOID-MATTE: Perfectly light-absorbing black; no reflections.
[TEX-006] ORDER-SYMMETRY-SURFACE: Smooth, flawless, mathematically perfect texture.
[TEX-007] GLITCH-PATTERN: Pixelated, shifting micro‑artifacts; Myralith presence.
[TEX-008] LEY-RAIL-ENERGY-SKIN: Blue-white pulsing lines; conductive surface.
[TEX-009] HYDRALIS-THERMAL-SCALE: Iridescent, heat-reactive micro‑scales.
[TEX-010] KY’RENNEI-PULSE-MESH: Fine, vibrating mesh pattern; energy conduction.
[TEX-011] SHADOW-FRAY: Dark, frayed edges; Lystryx interference.
[TEX-012] SENTINEL-BRASS-PLATE: Polished brass with faint harmonic etching.
[TEX-013] MEMORY-FADE-BLUR: Soft, drifting blur; identity erosion.
[TEX-014] CODE-GLYPH-ETCH: Tiny glyphs etched into surface; ancient systems.
[TEX-015] HETERODYNE-STRIATION: Chaotic multi-frequency lines.
[TEX-016] RAIL-STATIC-FILM: Thin static layer; rail overload.
[TEX-017] THERMAL-VENT-STONE: Porous, heat-scarred rock.
[TEX-018] SHARD-REFRACTION: Prismatic micro‑fractures.
[TEX-019] ORGANIC-BLOOM-TEXTURE: Soft, expanding grain; emotional peaks.
[TEX-020] CHAOS-MULTI-LAYER: Unpredictable, shifting multi-texture blend.

🪨 ENVIRONMENTAL MATERIALS LIBRARY (MAT‑IDs)
[MAT-001] MARBLE-CODE-CRYSTAL: Hard crystalline material; ancient structures and artifacts.
[MAT-002] AETHYR-SILT-SOIL: Fine, toxic particulate ground; Venturis ruins.
[MAT-003] RUSTED-ALLOY: Old Venturan metal; brittle and corroded.
[MAT-004] ORGANIC-SEED-WOOD: Warm, living wood-like material; 13th Seed growth.
[MAT-005] VOID-GLASS: Transparent but light-absorbing; conceptual hazard.
[MAT-006] ORDER-STEEL: Perfectly symmetrical alloy; Vessel architecture.
[MAT-007] GLITCH-MATTER: Unstable, semi-solid material; Myralith anomalies.
[MAT-008] LEY-RAIL-CONDUIT: Energy-conductive metal; rail infrastructure.
[MAT-009] HYDRALIS-THERMAL-CORE-METAL: Heat-reactive alloy; geothermal tech.
[MAT-010] KY’RENNEI-PULSE-CRYSTAL: Vibrating energy crystal; rail systems.
[MAT-011] SHADOW-RESIN: Dark, semi-liquid resin; Lystryx influence.
[MAT-012] SENTINEL-BRASS: Heavy, ceremonial brass; Apex structures.
[MAT-013] MEMORY-FOG-FILM: Thin conceptual residue; identity erosion zones.
[MAT-014] CODE-GLYPH-STONE: Stone etched with ancient glyphs.
[MAT-015] HETERODYNE-ALLOY: Chaotic multi-frequency metal.
[MAT-016] RAIL-STATIC-PLATE: Metal plate that accumulates static charge.
[MAT-017] THERMAL-VENT-ROCK: Heat-scarred volcanic stone.
[MAT-018] SHARD-GLASS: Prismatic, razor-sharp crystalline glass.
[MAT-019] ORGANIC-BLOOM-VINE: Living vine-like material; emotional resonance.
[MAT-020] CHAOS-FLUX-METAL: Unstable alloy that shifts shape subtly.

⚙️ CINEMATIC PHYSICS LIBRARY (PHY‑IDs)
[PHY-001] GRAVITY-INVERT: Local gravity flips orientation; objects drift upward.
[PHY-002] FREQUENCY-STABILIZE-528: 528Hz field stabilizes motion, light, and emotional tone.
[PHY-003] FREQUENCY-DISRUPT-440: 440Hz field destabilizes gravity, light, and mental clarity.
[PHY-004] HARD-CODE-963: 963Hz field enforces symmetry and removes organic variance.
[PHY-005] LEY-RAIL-ACCELERATION: High-speed linear acceleration; near-frictionless.
[PHY-006] AETHYR-SILT-DRAG: Movement slowed by particulate density.
[PHY-007] SHADOW-INVERSION: Shadows detach or move independently.
[PHY-008] GLITCH-STEP-PHYSICS: Short-range displacement; frame-skipping effect.
[PHY-009] ORGANIC-RESONANCE: Warm, expanding force; boosts physical capability.
[PHY-010] VOID-NULL-FIELD: Removes sound, light, and kinetic force.
[PHY-011] MEMORY-DRAIN-FIELD: Weakens persona stability; slows reaction time.
[PHY-012] RUST-FRICTION: High friction; unstable footing.
[PHY-013] HETERODYNE-CHAOS-FIELD: Chaotic multi-frequency interference; unpredictable motion.
[PHY-014] SENTINEL-BRASS-FORCE: Heavy downward pressure; Apex energy signature.
[PHY-015] CODE-REWRITE-PHYSICS: Physical rules temporarily overwritten by glyph logic.
[PHY-016] THERMAL-SURGE: Sudden heat burst; expands materials.
[PHY-017] RAIL-STATIC-DISCHARGE: Electric arcs jump between surfaces.
[PHY-018] SHARD-RESONANCE-FIELD: Prismatic energy field; refracts motion.
[PHY-019] ORGANIC-BLOOM-FIELD: Soft, buoyant force; emotional uplift.
[PHY-020] CHAOS-ASCENT-FORCE: Upward spiraling force; climax physics.

🎨 CINEMATIC COLOR PALETTE LIBRARY (COL‑IDs)
[COL-001] 528HZ-HARMONIC-GOLD: Warm golds, soft ambers, pale greens; Concordance stability.
[COL-002] 440HZ-DISSONANT-VIOLET: Harsh violets, electric blues, cold magentas; breach energy.
[COL-003] 963HZ-ORDER-WHITE: Sterile whites, cold silvers, pale blues; Vessel purity.
[COL-004] LEY-RAIL-CYAN: Bright cyan, neon blue, white streaks; kinetic traversal.
[COL-005] AETHYR-SILT-BROWN: Dusty browns, muted oranges, rusted reds; Venturis ruins.
[COL-006] SHADOW-INDIGO: Deep indigos, void blacks, flickering purples; Lystryx presence.
[COL-007] ORGANIC-SEED-GREEN: Warm greens, soft yellows, grainy golds; 13th Seed resonance.
[COL-008] VOID-BLACK: Absolute black with no reflectivity; conceptual emptiness.
[COL-009] RUST-ORANGE: Oxidized oranges, burnt reds, metallic browns.
[COL-010] GLITCH-MULTI: Unstable RGB shifts; Myralith interference.
[COL-011] HYDRALIS-TEAL: Cool teals, deep blues, aquatic gradients.
[COL-012] KY’RENNEI-PULSE-PINK: Bright pinks, neon magentas, rhythmic accents.
[COL-013] MARBLE-CODE-PRISM: Prismatic whites, crystalline blues, refracted rainbows.
[COL-014] SENTINEL-BRASS: Rich brass, gold, and deep metallic yellows.
[COL-015] MEMORY-FADE-GRAY: Soft grays, washed-out tones, desaturated palettes.
[COL-016] HETERODYNE-CHAOS: Clashing neons, unpredictable color shifts.
[COL-017] RAIL-STATIC-WHITE: Overexposed whites with blue static edges.
[COL-018] SHARD-REFRACTION: Sharp, high-contrast prismatic colors.
[COL-019] ORGANIC-BLOOM-PASTEL: Soft pastels with warm grain; emotional peaks.
[COL-020] CHAOS-ASCENT-SPECTRUM: Full-spectrum burst; climax palette.

🌦️ ENVIRONMENTAL WEATHER LIBRARY (WEA‑IDs)
[WEA-001] 528HZ-CLEAR-SKY: Warm, stable atmosphere; harmonic calm.
[WEA-002] 440HZ-STORM: Chaotic lightning, violet clouds, unstable winds.
[WEA-003] 963HZ-FROST-FOG: Cold, symmetrical fog; Vessel sanctum weather.
[WEA-004] AETHYR-SILT-STORM: Dense particulate storm; ruins hazard.
[WEA-005] RUST-WIND: Dry, metallic wind carrying rust flakes.
[WEA-006] SHADOW-MIST: Dark mist that moves against wind direction.
[WEA-007] ORGANIC-BLOOM-RAIN: Warm, glowing droplets; 13th Seed resonance.
[WEA-008] VOID-STILLNESS: No wind, no sound, no movement; conceptual hazard.
[WEA-009] LEY-RAIL-ION-RAIN: Electrically charged rain near rail conduits.
[WEA-010] HYDRALIS-THERMAL-STEAM: Geothermal steam plumes; underwater vents.
[WEA-011] KY’RENNEI-PULSE-DRIZZLE: Fast, rhythmic droplets; energy-rich rain.
[WEA-012] MARBLE-CODE-HAIL: Crystalline hailstones; ancient structure collapse.
[WEA-013] MEMORY-LEAK-FOG: Fog that causes brief disorientation.
[WEA-014] HETERODYNE-LIGHTNING: Chaotic multi-frequency lightning strikes.
[WEA-015] SENTINEL-BRASS-SUN: Golden, oppressive sunlight; Apex energy.
[WEA-016] GLITCH-WIND: Wind that stutters or reverses direction.
[WEA-017] RAIL-STATIC-STORM: Static-charged storm around rail networks.
[WEA-018] SHARD-RAIN: Prismatic crystalline rain; dangerous but beautiful.
[WEA-019] ORGANIC-MIST: Warm, glowing mist; emotional resonance.
[WEA-020] CHAOS-ASCENT-WHIRLWIND: Upward spiraling storm; climax weather.

⏱️ CINEMATIC TIMING & RHYTHM LIBRARY (TIM‑IDs)
[TIM-001] 528HZ-SLOW-PULSE: Steady, calming rhythm; stabilizes pacing.
[TIM-002] 440HZ-ERRATIC-PULSE: Unpredictable beats; tension and panic.
[TIM-003] 963HZ-METRONOME: Cold, perfect timing; Vessel sequences.
[TIM-004] RAIL-RUSH-RHYTHM: Fast, linear rhythm; traversal scenes.
[TIM-005] AETHYR-SILT-DRAG: Slowed pacing; heavy atmosphere.
[TIM-006] SHADOW-LAG: Delayed beats; uncanny timing.
[TIM-007] ORGANIC-BLOOM-RISE: Gradual acceleration; emotional uplift.
[TIM-008] VOID-PAUSE: Extended silence; conceptual tension.
[TIM-009] RUST-STAGGER: Uneven, metallic rhythm; ruins ambience.
[TIM-010] GLITCH-SKIP: Skipped beats; Myralith interference.
[TIM-011] HYDRALIS-FLOW: Smooth, wave-like rhythm.
[TIM-012] KY’RENNEI-PULSE: Fast, jittery micro-beats.
[TIM-013] MARBLE-CODE-CHIME: Soft, crystalline timing cues.
[TIM-014] SENTINEL-BRASS-MARCH: Heavy, ceremonial rhythm.
[TIM-015] MEMORY-FADE-SLOW: Slowing tempo; identity erosion.
[TIM-016] HETERODYNE-CHAOS: Multi-tempo interference.
[TIM-017] RAIL-STATIC-BURST: Sudden rhythmic spikes.
[TIM-018] SHARD-RESONANCE: Prismatic rhythmic oscillation.
[TIM-019] ORGANIC-SOVEREIGNTY: Warm, triumphant rhythm; climax pacing.
[TIM-020] CHAOS-ASCENT: Rapid upward tempo; final battle rhythm.`
      }
    },
    {
      id: 'cap-forge-001',
      name: 'Quantum Forge Design System Codex',
      description: 'Architecting the Mythic-Futurist Interface',
      kind: 'blueprint',
      glyphs: [],
      traits: [],
      tags: ['design', 'system', 'forge'],
      created: new Date().toISOString(),
      modified: new Date().toISOString(),
      author: 'Director-001',
      version: '1.0',
      properties: {
        content: `Chapter 1: Architecting the Mythic-Futurist Interface
The Quantum Forge represents the primary BioSpark Studio Design System. It is engineered to visualize the "Signal Flow of Creativity," moving from wireframe to tangible artifact.

Chapter 2: Visual Physics
The system operates on a tripartite color theory known as The Alchemical Palette:
- Solari Gold (Bravery): #f98f3b -> #ffd89e
- Venturan Blue (Logic): #e3f3ff -> #b1d0f0
- Umbraeth Black (Void): #1d1d1d -> #4d4d4d

Typography: Orbitron for the "Mythic Layer" and JSON-based data layer for technical specs.
Texture: Glassmorphism with a refractive index of 1.52, rgba(30,30,30,0.7), 1px refractive borders.

Chapter 3: The Universal Base Node
8-step assembly: Skeleton -> Skin -> Frame -> Header -> Ports -> Body -> Shadow -> Pulse.

Chapter 4: Node Variants
- Instrument Node: Tactile UI controls, parameter mapping.
- Agent Node: Narrative traits (e.g., Siren Stormbringer), radar charts for emotional vectors.
- Symbolic Node: "Collapse Probability" metric, runic borders, ambiguity mist.
- Remix Capsule: Container for "Lineage Trees", secured by License Gate.

Chapter 5: Connectivity and Signal Flow
Signals are "Wires" in Space Grotesk. Connections can be direct (Gold/Blue) or "Probabilistic" (dashed).

Chapter 6: The 'Zone' Environment
The Canvas is a topographical map of "Cognitive Terrain", featuring Solari Grid and Hope Basin.

Chapter 7: The Fusion Zone Composer
- Hover State: Highlight and Tooltip.
- Lasso Grouping: Selection of multiple nodes.
- Ripple Trail: Visualization of interaction history.
- Control Surface Dock: Modifiers for Intensity, Scale, and Time.

Chapter 8: The Output Artifact
Singularity (Fusion Point) results in a Collapse, producing a Chronicle Entry (narrative artifact) and Audio Artifact.`
      }
    },
    {
      id: 'cap-heraldry-001',
      name: 'Quantum Heraldry and Node Genesis',
      description: 'Symbolism & Syntax of the Architect’s Codex',
      kind: 'blueprint',
      glyphs: [],
      traits: [],
      tags: ['heraldry', 'genesis', 'logic'],
      created: new Date().toISOString(),
      modified: new Date().toISOString(),
      author: 'Director-001',
      version: '1.0',
      properties: {
        content: `Chapter 1: Lexicon of Signal
This manual outlines the visual language where raw code is transmuted into living narrative.

Chapter 2-3: Taxonomy of the Quantum Graph
Symbols function as the "Visual DNA" of the BioSpark ecosystem. Identity sits at the center of the Constellation Map, connected to the five heraldic classes.

Chapter 4: Heraldic Classes
- Crest: Mark of the collective (Race/Faction). Example: Venturan Sky Rite.
- Sigil: The "autograph of the arcane" for individual agents. Example: Stormweaver Spiral.
- Glyph: Language elements for stats and traits. (Fusion compatible).
- Totem: Anchors digital nodes to biomes or spirit bonds. Example: Sylvanid Forest Anchor.
- Cipher: Encrypted data requiring specific "Keys" to decode. Features glitch/clip animations.

Chapter 5: Symbol Fusion
Meaning multiplies when motifs merge. Logic: Mist (Obscurity) + Echo (Recurrence) = DreamFold (Introspection). Fusion creates a unique "Resonance Probability" based on input traits.

Chapter 6: Node Genesis
The ritual of construction follows four phases:
1. The Hull: A 200x150 vessel waiting for purpose.
2. The Logic: Establishing signal flow (Inputs/Outputs/Boolean Gates).
3. The Skin: Applying symbolic heraldry (visualSkin property).
4. The Soul: Injecting emotional weight via Traits (e.g., 'Wonder', 'Curiosity').

Chapter 7-8: Instrument Assembly
Fully instantiated nodes become The Performer, capable of emitting narrative signals. These nodes are housed in a Modular Rack, where they are chained (e.g., Bio-Res -> Aetheric -> Narrative) to synthesize a narrative stream.`
      }
    },
    {
      id: 'cap-mythic-codex-001',
      name: 'Quantum Mythic Codex',
      description: 'Blueprint for Seals, Sigils, and Sovereign Vaults',
      kind: 'blueprint',
      glyphs: [],
      traits: [],
      tags: ['mythic', 'codex', 'aesthetics'],
      created: new Date().toISOString(),
      modified: new Date().toISOString(),
      author: 'Director-001',
      version: '1.0',
      properties: {
        content: `Chapter 1: The Quantum Mythic Interface
The Codex serves as an "Arcane Editorial" guide for designing the visual elements of the BioSpark Studio.

Chapter 2: Philosophy of Quantum Aesthetics
- Modularity is Power: Everything connects and remixes via "Capsules."
- Resonance: UI elements react to emotion and presence.
- Performance: The interface is performed, not just viewed; it breathes and collapses.

Chapter 3-5: The Hierarchy of Meaning
- Glyphs: Atomic vector-based symbols representing Mental Resonance.
- Sigils: Active agents with three states: Idle (Pulse), Active (Flare), and Bloom (Explosion).
- Crests: Evolutionary markers growing through "Seasonal Arcs": Spring (Seed) -> Winter (Conductor).

Chapter 6-9: Containers and Thresholds
- Seals: Mechanical concentric rings signifying ownership.
- Vaults: A "Mini-Universe" repository of capsules (Persona, Codex, Atlas).
- Gates: State machines (Idle -> Reveal) turning entry into a ritual.
- Portals: Govern traversal logic, checking traits like "Mythic Resonance."

Chapter 10: The Phantori Palette
- Venturan (Sky/Air): Melancholy/Wonder.
- Umbraeth (Shadow/Void): Dread/Insight.
- Solari (Sun/Fire): Bravery/Pride.`
      }
    },
    {
      id: 'cap-prod-script-001',
      name: 'Master Production Script: The Thirteenth Seed',
      description: 'Full 16-Scene Production Blueprint with Movements and Assets',
      kind: 'blueprint',
      glyphs: [],
      traits: [],
      tags: ['script', 'production', 'master'],
      created: new Date().toISOString(),
      modified: new Date().toISOString(),
      author: 'Director-001',
      version: '1.0',
      properties: {
        content: `TITLE BLOCK
THE STAR-SHADOW HERESY: DEFENDING THE THIRTEENTH SEED
CHAPTER 1: THE WEIGHT OF 528
ACT STRUCTURE OVERVIEW
ACT I: Setup - The pristine, yet fragile, 528Hz utopia of Xyrona Prime is introduced, along with its cynical archivist, Kaelen-Xi, and kinetic runner, Rin. The Resonant Day's Great Humming commences, only to be violently interrupted by a 440Hz dissonance spike and localized gravity inversion.
ACT II: Confrontation - Kaelen and Rin struggle against the immediate chaos of a reality suddenly rewritten. The Semantics Vault fractures, and the Ley-Rails become inverted death traps. They make desperate contact amidst the escalating anomaly, witnessing the insidious spread of "Absolute Law" imposed by the Vessels of Order.
ACT III: Resolution - As the Apex city visibly shatters, Kaelen and Rin, working together across the broken frequency, navigate their fractured environment. Rin employs her intuitive 'frequency-blindness' to find a critical loophole in the inverted physics, while Kaelen grapples with the intellectual implications of a world being rewritten. Their initial alliance solidifies in the face of widespread entropy, setting the stage for their next desperate move.
RUNTIME ESTIMATE: 28-32 minutes

ACT I (Setup)

SCENE-01
SCENE-TITLE: The Great Humming Begins
LOCATION: INT. 9TH CYCLE SEMANTICS VAULT - PENTAD APEX
TIME-OF-DAY: 06:00 - Resonant Day Dawn
SCENE-SUMMARY: Archivist Kaelen-Xi monitors the 528Hz Great Humming, a global synchronization ritual, from within the sterile, super-phobic glass confines of the Semantics Vault. Despite the apparent perfection, Kaelen's analytical mind senses an underlying fragility, a diagnostic reading of "too stable."
THEMATIC HOOK: Introduces the theme of "Law vs. Loophole" by establishing the oppressive perfection of the 528Hz Concordance and Kaelen's innate skepticism, an "echo in the Apex" of coming chaos.

MOVEMENTS
[MOVEMENT-01]
[CAM] — WIDE SHOT of the immense, hexagonal VAULT, its super-phobic glass pedestals stretching into the sterile distance.
[VFX] — Faint, glowing CYAN CIRCUITRY beneath the glass floor pulsates rhythmically, a visual manifestation of the 528Hz hum.
[DL] — KAELEN-XI stands perfectly still, a single faint frost-trail appearing on a nearby glass pedestal as he exhales.
[SFX] — A deep, resonant 528Hz SUB-BASS HUM fills the space, a pure, unwavering tone.
[NOTES] — Emphasize the architectural geometry and anechoic quality of the Vault.
[MOVEMENT-02]
[CAM] — CLOSE UP on Kaelen-Xi’s face, one cybernetic eye displaying scrolling data-code. His pupils are fixed at 3mm.
[VFX] — A translucent, white-line HUD flickers at the edge of his vision, displaying "FREQUENCY: 528Hz (STABLE)."
[DL] — Kaelen's head remains perfectly still, no micro-tremors, his breathing shallow (6 breaths/min).
[SFX] — A barely audible ARGON GAS HISS permeates the quiet, a clinical counterpoint to the hum.
[NOTES] — Highlight Kaelen’s Hydralis-Syntaran hybrid traits and analytical personality.
[MOVEMENT-03]
[CAM] — SLOW PUSH IN on a section of LATTICE-IVY climbing a glass spire, its translucent membranes vibrating at precisely 528Hz, not rustling.
[VFX] — The ivy's leaves reflect the 7500K Arctic Blue light with surgical precision, creating "Ghost-Glares" that follow Kaelen's gaze.
[DL] — Kaelen’s gaze sweeps across the vault, performing a diagnostic scan.
[SFX] — The "Ghost-Glares" are panned hard left/right using a Haas effect, simulating light off zero-friction surfaces.
[NOTES] — Visualizes the "Geometric Sterility" of the environment.
[MOVEMENT-04]
[CAM] — MEDIUM SHOT as Kaelen adjusts a silver interface quill behind his ear.
[VFX] — A subtle, almost imperceptible digital "ripple" radiates from the quill's point of contact.
[DL] — His movements are frictionless gliding, his torso rotation locked at 5 degrees/sec.
[SFX] — A faint, precise metallic CLICK as the quill seats into place.
[NOTES] — Reinforces the seamless integration of technology and Kaelen's methodical nature.
[MOVEMENT-05]
[CAM] — OVERHEAD SHOT of the Vault, emphasizing its vast, empty perfection.
[VFX] — The cyan glow beneath the floor momentarily intensifies, then returns to its baseline hum.
[DL] — Kaelen stands as the lone, still figure in a sea of data and light.
[SFX] — The 528Hz hum swells slightly, then recedes, feeling like a held breath.
[NOTES] — The "Numb Bliss" state of the 528Hz standard.
[MOVEMENT-06]
[CAM] — CLOSE UP on Kaelen’s mouth as he mutters.
[VFX] — The HUD in his cybernetic eye blinks with a diagnostic output: "CONCORDANCE ANCHOR: NOMINAL."
[DL] — A weary, analytical expression flickers across his face.
[SFX] — His voice is an airy head-voice, utterly devoid of sibilance.
[NOTES] — His internal monologue ("Too stable.") hints at his cynical archivist archetype.
[MOVEMENT-07]
[CAM] — REVERSE SHOT, looking out from Kaelen's perspective through a massive observation window towards the city.
[VFX] — The pristine Pentad Apex, bathed in soft dawn light, appears utterly flawless.
[DL] — Kaelen’s shoulders remain in "Fixed Stasis," neutral and unmoving.
[SFX] — The constant 528Hz drone.
[NOTES] — A moment of calm before the storm, emphasizing the false sense of security.

DIALOGUE
[DIALOGUE-01]
KAELEN-XI:
"06:00. The Great Humming, on schedule. Every sub-harmonic accounted for."
[DIALOGUE-02]
KAELEN-XI:
"The resonance meters are flat-lining at perfect consonance. Too perfect."
[DIALOGUE-03]
KAELEN-XI:
"Another cycle, another ritual. They equate silence with peace."
[DIALOGUE-04]
KAELEN-XI:
"But silence can also be the sound of something about to shatter."
[DIALOGUE-05]
KAELEN-XI:
"The Hydralis sensors confirm: Xyrona Prime is a perfectly tuned instrument."
[DIALOGUE-06]
KAELEN-XI:
"Yet the Syntaran algorithms detect… a whisper of something unwritten."
[DIALOGUE-07]
KAELEN-XI:
"The data doesn't lie, but it omits."
[DIALOGUE-08]
KAELEN-XI:
"A world that has learned to stop breaking is a world that has forgotten how to breathe."
[DIALOGUE-09]
KAELEN-XI:
"This numbing bliss... it feels like a diagnostic of impending failure."
[DIALOGUE-10]
KAELEN-XI:
"The Concordance Anchor holds. For now."

ASSET REGISTRY
[ASSET-CAM-001] — Wide Vault establishing shot
[ASSET-CAM-002] — Kaelen-Xi cybernetic eye close-up
[ASSET-CAM-003] — Lattice-Ivy detail shot
[ASSET-CAM-004] — Kaelen quill adjustment
[ASSET-CAM-005] — Overhead Vault shot
[ASSET-CAM-006] — Kaelen-Xi mouth close-up
[ASSET-CAM-007] — View of Pentad Apex from Vault
[ASSET-VFX-001] — Cyan Circuitry glow (528Hz)
[ASSET-VFX-002] — HUD (Frequency display)
[ASSET-VFX-003] — Ghost-Glares (reflections off super-phobic glass)
[ASSET-VFX-004] — Quill digital ripple
[ASSET-PROP-001] — Super-Phobic Glass Pedestals
[ASSET-PROP-002] — Interface Quill (silver)
[ASSET-PROP-003] — Lattice-Ivy
[ASSET-DL-001] — Kaelen-Xi (fixed pupils, shallow breathing, frictionless glide, fixed stasis hands)
[ASSET-SFX-001] — 528Hz Sub-bass Hum
[ASSET-SFX-002] — Argon Gas Hiss
[ASSET-SFX-003] — Metallic Quill Click
[ASSET-SFX-004] — Ghost-Glares Haas Effect

GENESIS-CONTAINER-SCENE-01

Persona Capsule(s): cap-pers-kaelen-xi (Cynical Archivist, Analytical, Weary)
Emotion Capsule(s): The Numb Bliss, underlying anxiety
Logic Capsule(s): Consensus Reality physics, 528Hz stabilization
Resource Capsule(s): None directly shown, but implications of Concordance Anchor's function
Narrative Capsule(s): Establishment of utopia, foreshadowing of breach

... (Scenes 2-16 omitted for brevity in this capsule, but present in scriptContent) ...
`
      }
    }
  ]
};
