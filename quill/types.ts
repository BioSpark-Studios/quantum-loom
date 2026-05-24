// ============================================================================
// QUANTUM QUILL GENESIS - CORE DATA TYPES (EXTENSIBLE VERSION V2.6)
// ============================================================================

export type UUID = string;
export type Timestamp = string;
export type HexColor = string;
export type Emoji = string;

// --- Enums ---

export enum CapsuleEventType { 
  ACTION = "action", 
  DIALOGUE = "dialogue", 
  OBSERVATION = "observation", 
  STATE_CHANGE = "state_change", 
  EMOTION_SHIFT = "emotion_shift", 
  ENVIRONMENTAL = "environmental" 
}

export enum VaultType { 
  SCREENPLAY = "screenplay", 
  NOVEL = "novel", 
  STAGE_PLAY = "stage_play", 
  LIBRARY = "library", 
  PLAYGROUND = "playground", 
  COLLABORATION = "collaboration", 
  EXPERIMENT = "experiment" 
}

export enum VaultStatus { 
  ACTIVE = "active", 
  ARCHIVED = "archived", 
  TEMPLATE = "template" 
}

export enum PortalAccessType { 
  READ_ONLY = "read-only", 
  READ_WRITE = "read-write", 
  FULL_ACCESS = "full-access" 
}

export enum NarrativeScope { 
  SINGLE_SCENE = "single_scene", 
  MULTI_SCENE = "multi_scene", 
  ACT = "act", 
  FULL_STORY = "full_story", 
  EPIC = "epic" 
}

export enum ContainerType { 
  PERSONA = "persona", 
  EMOTION = "emotion", 
  NARRATIVE = "narrative", 
  CODEX = "codex", 
  FAUNA = "fauna", 
  FLORA = "flora", 
  URBAN = "urban", 
  CULTURE = "culture", 
  MUSIC = "music", 
  LOGIC = "logic", 
  RESOURCE = "resource", 
  TRIGGER = "trigger",
  BLUEPRINT = "blueprint"
}

// --- Core Structures ---

export interface Glyph {
  id: UUID;
  name: string;
  symbol: Emoji;
  description: string;
  category: string;
  metadata: Record<string, any>;
}

export interface Trait {
  id: UUID;
  name: string;
  description: string;
  category: string;
  value: number | string | boolean;
  modifiers?: Record<string, number>;
  tags: string[];
}

export interface Seal {
  id: UUID;
  name: string;
  glyph: Glyph;
  waxColor: HexColor;
  isLocked: boolean;
  lineageHash: string;
  significance: number;
  ritualRequired: string;
  sealedAt: Timestamp;
}

export interface BaseCapsule {
  id: UUID;
  name: string;
  description: string;
  kind: string; // Open-ended: "persona", "urban", "narrative", "rule", etc.
  glyphs: Glyph[];
  traits: Trait[];
  tags: string[];
  created: Timestamp;
  modified: Timestamp;
  author: string;
  version: string;
  properties: Record<string, any>; // Flexible storage
  seal?: Seal;
}

export interface MythosContainer {
  id: UUID;
  name: string;
  description: string;
  mythosType: string;
  color?: HexColor;
  icon?: Emoji;
  capsules: BaseCapsule[];
  maxCapsules: number;
  properties: Record<string, any>;
  created: Timestamp;
  modified: Timestamp;
  tags: string[];
}

export interface GenesisContainer {
  id: UUID;
  name: string;
  description: string;
  seal?: Seal; // Seal appears FIRST
  genre: string[];
  themes: string[];
  narrativeScope: NarrativeScope;
  setting: {
    location: string;
    timePeriod: string;
    atmosphere: string;
    culturalContext?: string;
  };
  mythosContainers: MythosContainer[];
  maxMythosContainers: number;
  globalSettings: Record<string, any>;
  version: string;
  created: Timestamp;
  modified: Timestamp;
}

// ============================================================================
// EXISTING APP TYPES (BRIDGED)
// ============================================================================

export interface ChapterMarker {
  id: string;
  time: number;
  label: string;
  color?: string;
}

export interface Project {
  id: string;
  name: string;
  lastModified: number;
  scriptContent: string;
  productionScript?: string;
  scenes: Scene[];
  timeline: TimelineItem[];
  characterVoices: Record<string, string>;
  genesis: GenesisContainer;
  chapterMarkers: ChapterMarker[];
}

export interface Scene {
  id: string;
  number: number;
  heading: string;
  description: string;
  characters: string[];
  dialogue: DialogueLine[];
  transition?: 'cut' | 'fade' | 'dissolve' | 'wipe';
}

export interface DialogueLine {
  id: string;
  character: string;
  text: string;
  parenthetical?: string;
}

export enum MediaType {
  VIDEO = 'VIDEO',
  IMAGE = 'IMAGE',
  AUDIO = 'AUDIO',
  BLANK = 'BLANK',
  CAPSULE = 'CAPSULE' 
}

export interface TimelineItem {
  id: string;
  sceneId: string;
  type: MediaType;
  src: string; 
  duration: number; 
  label: string;
  thumbnail?: string;
  track: 'visual' | 'audio' | 'music';
  startTime: number; 
  capsuleId?: string; 
}

export interface GenerationStatus {
  isGenerating: boolean;
  progress: string;
  error?: string;
}

export const DEMO_SCRIPT = `XYRONA PRIME: THE VERTICALITY'S SHATTERED HARMONY
A DIRECTOR'S VERSION SCREENPLAY

FADE IN:

EXT. COSMIC VOID - EONS AGO (EPOCH 1: THE PRIMORDIAL RESONANCE)

A swirling, ethereal NEBULA of pure AETHER, iridescent golds and deep violets, hangs in the black.

CLOSE ON a central point, a nascent star, beginning to pulse. A deep, SUB-SONIC HUM reverberates, feeling ancient and primal.

VFX: The nebula begins to coalesce rapidly, forming a nascent planet, Xyrona Prime. The pulsing core intensifies, a beacon of deep, internal light.

SOUND: The sub-sonic hum resolves into a singular, immense PLANETARY HEARTBEAT, anchoring the world.

A shimmering, impossibly vast CELESTIAL BEING, ASTRION (CHAR-LUM-AST), descends from the cosmic veil. Its form is composed of pure starlight, trailing cometary dust.

VFX: Astrion’s glowing hands begin to sculpt, not with physical force, but with intricate, resonant light. We see five distinct STRATA emerge on Xyrona Prime, like concentric layers of a colossal, living organism. Each layer pulses with a unique color and frequency.

CAMERA: PULL BACK, revealing Xyrona Prime as a marvel of verticality, a layered world hanging in space.

VFX: A torrent of AETHERIC GOLD, like liquid starlight, rains down onto the uppermost stratum, pooling into jagged, crystalline peaks – THE PRISM PEAKS (LOC-APEX-SAN).

From this blinding light, beings of abstract perfection spontaneously form: THE LUMINARITES (FACT-LUM-ASC). Their skin shimmers, reflecting the Aetheric Gold. They gaze upwards, a silent, almost reverent yearning for dissolution.

VFX: In stark contrast, deep within the abyssal depths of Xyrona, under immense pressure, BIOLUMINESCENT CORAL CITADELS rise from the crushing darkness.

SOUND: Deep, guttural CHANTS echo through the water, overlaid with sonar pings and the immense groaning of the planet's mantle.

THE HYDRALIS (FACT-HYD-DEEP) emerge from these cities. Their forms are adapted to pressure, dark skin with glowing sigils, eyes reflecting the faint bioluminescence. They seem to carry an immense, unseen weight.

VFX: The middle stratum ignites with lush, vibrant green. ENORMOUS, SENTIENT TREES (Sylvanids) burst forth, their root-networks (MAG-BIO-SYL) spreading like a vast, green nervous system across the planet’s surface.

SOUND: A gentle, harmonious WHISPERING OF LEAVES, like a chorus, fills the air.

EXT. ARCHON CITY RUINS - EPOCH 2: ERA OF WHISPERING ASH (~12,000 SOLAR CYCLES AGO)

LIGHTING: Bleak, overcast, the air thick with a pervasive, grey-violet MIST.

ENVIRONMENT: The shattered remnants of a geometrically perfect, but now ruined, ANCIENT CITY. Colossal, broken obelisks point to a perpetually swirling, violet-hued CHASM in the distance – THE MOUTH OF NIHILOS (LOC-VOID-NIH).

VFX: The mist around the chasm writhes, pulsing with faint, agonizing FACES that quickly dissolve into nothingness. This is the "Identity-Loss Terror."

SOUND: A low, guttural MOAN emanates from the chasm, overlaid with faint, discordant whispers.

CAMERA: PUSH IN on the chasm. Swarms of small, insectoid creatures with CHITINOUS EXOSKELETONS (FAUN-MNEM-PARA) pour out, their forms subtly glitching, their movement unnatural, as if seeking something lost. These are the Mnemosyne-Parasites.

EXT. INVERTED NEEDLE - DAWN - EPOCH 2 (CONTINUOUS)

LIGHTING: A stark, rising sun casts long shadows over a truly colossal structure: THE INVERTED NEEDLE (LOC-INV-NEED). It’s a massive Prism-Steel pylon, impossibly thin yet immense, piercing the upper atmosphere, its base anchored deep below. It glows faintly with contained Aether.

CAMERA: WIDE SHOT, showing Luminarites, Hydralis, and Sylvanids working together, their distinct technologies and bioluminescences merging to seal the Needle. This is the "First Quarantine."

VFX: Energy seals pulse around the Needle's base, containing the violet mist of the Mnemosyne.

INT. HYDRALIS ABYSSAL CITY - EPOCH 3: ASCENDANCE & DEEPENING DEBT (~2,000 SOLAR CYCLES AGO)

LIGHTING: Deep, emerald green bioluminescence casts long, shifting shadows. The pressure is palpable.

ENVIRONMENT: Vast, ancient structures of coral and dark rock, carved with intricate glyphs. Fish and glowing fauna drift silently.

SOUND: Deep, resonant HUMS and SONAR PINGS. An occasional, unnerving GROAN from the planet's core.

A HYDRALIS SCIENTIST, cloaked in robes, observes an experimental, crude LEVITATION PACK (ART-HYD-LEV) being tested on a small, contained creature. The creature hovers fitfully, occasionally flickering.

VFX: The creature’s form subtly glitched, a transient blur at the edges of its being.

SCIENTIST (O.S.)
(Deep, resonant, with an echo)
The molecular shift... it holds. For now. But the Resonance Static... the cost.

INT. VENTURAN SKY-CITY - DAZZLING DAY - EPOCH 3 (CONTINUOUS)

LIGHTING: Bright, crystalline, reflecting off gleaming metal and polished surfaces.

ENVIRONMENT: A magnificent SKY-CITY (LOC-SKY-CIT), a marvel of engineering, floats effortlessly amongst the clouds, anchored by shimmering AETHER-ARC TECHNOLOGY (TEC-AET-VEN). Below, the world stretches out, a distant, green blur.

SOUND: A high-frequency, almost musical HUM of Aetheric engines. Light, airy, confident.

VENTURANS, clad in sleek, sophisticated attire, move with an air of buoyant confidence. They are absorbed in their technology, their faces lit by holographic displays.

VFX: We see thin, almost invisible Aetheric tethers extending from the Sky-City, subtly pulsing as they draw energy from below.

EXT. VERDANT BELT - DUSK - EPOCH 3 (CONTINUOUS)

LIGHTING: Soft, diffused light filtering through a dense canopy of ancient, glowing trees.

ENVIRONMENT: Lush, verdant forest. Massive, gnarled CHRONO-OAKS (MAG-BIO-SYL) dominate the landscape, their leaves emitting a soft, temporal glow.

SOUND: The gentle rustling of leaves, deep and rhythmic, like a breathing organism. Distant, mournful WHISPERS.

A group of ROBED FIGURES, VOID PILGRIMS (FACT-VOID-PIL), gather around a complex, multi-faceted ARTIFACT (ART-UNPLAY). It resembles a cross between an ancient instrument and a crystalline sculpture, intricately carved, but utterly silent. This is THE UNPLAYABLE INSTRUMENT.

They touch it with reverence, but it remains inert. Their faces are etched with a quiet desperation.

EXT. XAL'KROTH'S CHASM - NIGHT - EPOCH 3 (CONTINUOUS)

LIGHTING: Deep, oppressive darkness, punctuated by sickly, violet-red glows.

ENVIRONMENT: A jagged, volcanic chasm. Twisted, petrified forms jut out of the rock. At the very center, a pool of black, viscous liquid slowly churns.

SOUND: A low, resonant GROWL, like something ancient and vast waking up. Faint, insidious WHISPERS.

OBRAK (CHAR-OBS-XAL), leader of THE OBSCURIATI (FACT-OBS-SHADOW), a gaunt figure in dark, cultish robes, kneels before the chasm, holding a sacrificial blade. His eyes reflect a terrifying, nihilistic zeal.

OBRAK
Lord of Infinite Hungers, Xal'Kroth… hasten the cleansing. Accelerate the collapse.

VFX: The black liquid churns more vigorously, a shadowy form briefly coalescing within, then dissolving.

INT. INVERTED NEEDLE - CORE CHAMBER - EPOCH 4: THE DEEP CREASE (PRESENT DAY)

LIGHTING: Flickering, strained. Emergency lights cast stark shadows.

ENVIRONMENT: The colossal Prism-Steel core of the Inverted Needle. It hums with immense, contained energy, but now emits a subtle, distressing GROAN. Fine CRACKS spiderweb across its surface.

SOUND: The hum is discordant, punctuated by sharp, metallic STRESS FRACTURES.

A holographic display flickers: "BUOYANCY DEBT: 14.8%... CRITICAL THRESHOLD IMMINENT."

EXT. VENTURAN SKY-CITADEL - DAY - EPOCH 4 (CONTINUOUS)

LIGHTING: Bright, but with an underlying, unsettling DIMNESS.

ENVIRONMENT: The gleaming Venturan Citadel (LOC-SKY-CITAD), once pristine, now shows subtle signs of strain. Scaffolding is visible on some sections.

SOUND: The Aether-Arc hum is now slightly off-key, a faint, high-pitched WHINE occasionally cutting through.

TALIA (CHAR-VEN-TALIA), sharp, intelligent, and driven, paces her command bridge. She points to a holographic projection of Xyrona Prime, showing the Sky-Cities visibly lower by a fraction, the Hydralis abyssal cities outlined in red, indicating crushing pressure.

TALIA
The Apex Fall is no longer a theoretical. If the Buoyancy Debt reaches 15%… we begin to sink. We will not be caught unprepared. Activate phase-four Aetheric extraction protocols. I want a solution, now.

Her ambition is clear, but a flicker of deep-seated fear crosses her eyes. She clutches a data-slate, her knuckles white.

INT. HYDRALIS ABYSSAL LABORATORY - NIGHT - EPOCH 4 (CONTINUOUS)

LIGHTING: Bioluminescent corals pulse erratically, casting jittery light.

ENVIRONMENT: A cramped, advanced laboratory, cluttered with experimental Hydralium tech. Water ripples with unseen pressure.

SOUND: The deep planetary hum is now more pronounced, almost a low GROWL. SONAR PINGS are rapid, frantic.

SILAS (CHAR-HYD-SIL), a Hydralis engineer, struggles with his ART-HYD-LEV pack. He’s taller and leaner than other Hydralis, his movements fluid but his form frequently GLITCHES. His skin shimmers, almost transparent at the edges.

VFX: As he activates the pack, his body blurs, a faint, translucent outline appearing next to his physical form. He lets out a gasp of pain.

SILAS
(Voice layered, echoing, as if two voices speak at once)
The Resonance Static… it’s increasing. Like a thousand forgotten whispers screaming in my core. I'm… unraveling.

He sees faint, shimmering violet MIST (Grief-Static) permeating the very rock around him, vibrating with unseen agony. He reaches out, and his hand phases through a solid console. A look of terror.

SILAS (CONT'D)
How can I save them, if I cannot hold myself together?

EXT. SYLVANID GROVES - DAWN - EPOCH 4 (CONTINUOUS)

LIGHTING: Ethereal, misty, the light struggling to penetrate the ancient canopy.

ENVIRONMENT: The Sylvanid Groves (LOC-SUR-GRO) are beautiful but unsettling. The leaves of the Chrono-Oaks (MAG-BIO-SYL) are pulsating erratically, some moving in agonizing slow motion, others blurring with unnatural speed.

SOUND: A low, rhythmic VIBRATION. The whispering of leaves is replaced by an insistent, irregular STUTTER.

ELOWEN (CHAR-SYL-ELO), the Sylvanid High Sage, stands amongst the trembling trees. Her eyes are wide with a terrifying vision. She clutches a sacred staff woven with vibrant vines (SIG-SYL-OURO).

ELOWEN
(Voice trembling, deep, resonant)
The Stuttering World… I see it. The Needle groans. The Sky-Cities weep. And the Abyss… it chokes on memory. Only a Re-Naming… a Grand Syzygy… can forestall the silence.

VFX: Images flash through her eyes: Sky-Cities falling, abyssal cities crumbling, the planet itself twisting in pain.

EXT. VERDANT BELT - DAY - EPOCH 4 (CONTINUOUS)

LIGHTING: Harsher, more fractured light.

ENVIRONMENT: Once-pristine sections of the Verdant Belt are now blighted. Patches of land are covered in a grotesque, chitinous growth, violet-black and oozing. These are VORTHEX CHITIN HIVES (FAUN-VORTHEX-CHIT).

SOUND: A guttural, scratching NOISE. The air is thick with a wet, sickly sound.

VFX: The chitinous growth actively spreads, consuming the green. Black, viscous liquid drips from the growths.

CAMERA: PULL BACK to reveal vast sections of the Verdant Belt, turning into a "Dark Nebula Zone" – the Chitin-Blight. The Ley-Line Nodes (LEY-NODE-CORE) within the blighted areas flicker and die.

INT. CELESTIAL NAVIGATOR'S OBSERVATORY - NIGHT - EPOCH 4 (CONTINUOUS)

LIGHTING: Deep blues and purples, lit by holographic celestial projections.

ENVIRONMENT: A circular room, its walls a vast, intricate star map.

SOUND: The soft hum of ancient machinery.

A CELESTIAL NAVIGATOR, an ancient Luminarite, points to a holographic projection of Xyrona Prime's moons.

NAVIGATOR
Aethel-Vora, Nyx-Kylos, Hydra-Syr… the alignment is confirmed. 14 cycles. The Grand Syzygy approaches.

VFX: The projection glows red, highlighting the Inverted Needle. Text appears: "HAZARD: TEMPORAL DILATION – 500%."

NAVIGATOR (CONT'D)
Dire predictions. Complete Ley-Line draining. Catastrophic temporal dilation at the Needle.

The Luminarite’s face, usually serene, holds a trace of profound dread.

EPOCH 5: THE GRAND SYZYGY – CONVERGENCE OF FATES (CLIMAX)

EXT. INVERTED NEEDLE - GRAND SYZYGY - DAY

LIGHTING: Chaotic, apocalyptic. The sky is a maelstrom of violent, iridescent Aetheric storms. Lightning flashes violet-gold.

ENVIRONMENT: The Inverted Needle groans, its cracks visibly widening, spitting sparks of pure Aether. Below, the Verdant Belt is a battleground.

SOUND: A deafening roar of Aetheric energy, clashing with the STUTTERING of Chrono-Oaks. The planetary heartbeat is now a frantic, irregular THUMP.

VFX: The entire planet appears to be vibrating, as if struggling to hold its form. The moon, NYX-KYLOS, hangs ominously in the sky, appearing 'hollowed out' and silent, intensifying Void hazards.

EXT. VERDANT BELT - GROUND LEVEL - SYZYGY (CONTINUOUS)

LIGHTING: Flickering, intense. Violet-black mist swirls from Mnemosyne breaches.

ENVIRONMENT: Sylvanid "Still-Pulse" Phalanxes (FACT-PHALANX-MOBIUS), their forms rooted deeply, glow with emerald light, creating localized temporal stasis fields. Venturan units in power suits calibrate shields to a precise 440Hz, battling swarms of Vorthex Chitin that mimic their frequency.

SOUND: The clash of energy shields against chitin, dissonant screeches. The Sylvanid chants provide a counterpoint, a rhythmic, grounding pulse.

VFX: The Litho-Lichen seeding around the Southern Void-Drain grows rapidly, forming a shimmering, iridescent barrier against the "Grief Well."

INT. INVERTED NEEDLE - CORE CHAMBER - SYZYGY (CONTINUOUS)

LIGHTING: Overloaded, blinding. Sparks fly, arcs of Aether dance wildly.

ENVIRONMENT: The Needle's core chamber is in utter chaos. Consoles explode. The temperature is unbearable.

SOUND: A piercing, unbearable WHINE. The air vibrates with "Anxiety Shear" (WEATHER-AETHER-STORM-STATIC).

TALIA (CHAR-VEN-TALIA) stands at a massive control console, her face etched with desperation. She's pushing the system beyond its limits, overriding safety protocols. Her hands are burned from the raw energy.

TALIA
(Shouting over the noise)
Redirect all core Aether! Bypass the dampeners! We need more! More!

VFX: A massive Aether-Arc conduit, glowing incandescently, begins to pull energy directly from the Needle's deep core. The entire structure groans in protest. Talia’s desperation manifests as a subtle, shimmering violet halo around her, almost like the Mnemosyne mist.

TALIA (CONT'D)
(To herself, fiercely)
I will not watch us fall.

VFX: A burst of spatial distortion. The air ripples violently.

SILAS (CHAR-HYD-SIL) PHASES into the chamber, his form wildly unstable, flickering in and out of existence. He looks translucent, barely there, his face a mask of agony.

SILAS
(His voice a tortured, layered echo)
The Debt… the grief… you’re amplifying it!

He perceives Talia’s fear as a cold, sharp echo in his own being. He sees the Mnemosyne's chaotic resonance being channeled by her actions, not just around her.

CAMERA: PUSH IN on Talia’s shocked face. She sees Silas, a phantom made real by her actions.

Silas lunges forward, his flickering hand reaching for the conduit Talia is manipulating.

VFX: As his hand makes contact with the glowing Prism-Steel conduit, a blinding FLASH of multi-spectral light erupts.

SOUND: A deafening, visceral SHOCKWAVE of energy and emotion.

CAMERA: EXTREME CLOSE UP on their hands.

Talia’s eyes widen in terror and shock. She feels an overwhelming, crushing weight of ancient memory, of countless forgotten lives, of profound, existential grief pouring into her being. She almost collapses under the psychic burden.

Silas, conversely, experiences her icy, primal fear of oblivion, her profound loneliness, her desperate yearning for control, amplifying in his fragmented consciousness.

VFX: For a brief, searing moment, Silas's form solidifies, painfully real. Then he shimmers violently, his molecules fighting for cohesion, threads of light tearing from his edges.

SILAS
(A guttural scream, raw and pure)
The frequency… it’s… here.

He realizes his "Resonance Static" is not a flaw, but the missing piece. He consciously channels the chaotic Aether and volatile Mnemosyne resonance through his own unraveling form.

EXT. VERDANT BELT - ELOWEN'S LOCATION - SYZYGY (CONTINUOUS)

LIGHTING: The air is thick with diffuse, slow-motion light.

ENVIRONMENT: Elowen stands at the heart of the Chrono-Oaks, her face serene despite the chaos. Her Sylvanid Phalanxes are rooted around her, glowing.

SOUND: Elowen’s voice begins to rise, a powerful, ancient CHANT (RIT-SYL-ATTUNE) that seems to vibrate the very ground. The chant harmonizes with the Sylvanid Still-Pulse, cutting through the anxiety shear.

VFX: As she chants, waves of green energy pulse outwards from her, flowing along the Sorrow-Channels of the Prism-Steel pylons. The Chrono-Oaks around her enter "Slow-Light" processing, creating vast, localized temporal stasis fields, dampening the Syzygy's effects.

ELOWEN
(Eyes closed, voice echoing with ancestral power)
Weavers of Green, hear me! Re-Name this broken world! Attune the severed cord!

INT. INVERTED NEEDLE - CORE CHAMBER - SYZYGY (CONTINUOUS)

LIGHTING: The blinding light from Silas and Talia’s connection intensifies.

ENVIRONMENT: The Unplayable Instrument (ART-UNPLAY), previously inert, has appeared at the periphery of the chamber, drawn by the immense, clashing energies. It begins to GLOW.

SOUND: The Instrument emits a low, resonant HUM, then a single, pure NOTE, impossibly complex, emerges.

Silas, his body still flickering violently, instinctively reaches out his other hand to Talia's, seeking an anchor in the storm.

VFX: Their desperate, raw touch completes a circuit. The Instrument pulses with blinding, sonic-aetheric light.

SOUND: A full, deafening CRESCENDO erupts from the Instrument, amplified by the planetary heartbeat. It feels like the entire universe is singing through them.

VFX: The Instrument doesn't just play; it begins to rewrite reality. Fissures in the air, previously 'Z-fighting' with chaotic energy, visibly mend, becoming harmonically stable. It morphs into a shimmering, abstract SIG-QUILL, writing patterns in the air.

Talia gasps, her mind reeling. She feels Silas’s profound existential grief, his essence, so deeply that she almost becomes him. He experiences her profound loneliness, her terror of falling, as if it were his own. The intimacy is terrifying, yet utterly consuming.

EXT. VERDANT BELT - MNEMOSYNE ATTACK - SYZYGY (CONTINUOUS)

LIGHTING: The violet mist brightens, turning a sickly green.

ENVIRONMENT: The Vorthex Chitin Hives (FAUN-VORTHEX-CHIT) churn, sending forth endless swarms of Mnemosyne-Parasites.

SOUND: Their screeches become louder, more desperate, as they sense the shift from terror to resolve.

VFX: The Parasites attempt to mimic the Instrument's 440Hz pulse, trying to corrupt the frequency, lure defending units into the thickening Grief-Mist, and consume the Instrument's nascent power. Some of the attacking forces begin to visibly mutate, their forms twisting (HAZ-VOID-BLIGHT).

INT. INVERTED NEEDLE - CORE CHAMBER - SYZYGY (CONTINUOUS)

LIGHTING: The light of the Instrument is now steady, radiant, but still overwhelmingly bright.

ENVIRONMENT: Temporal Dilation (HAZ-TEMP-DIL) hits its peak 500% at the Needle, but Silas's stabilized resonance creates a small, shimmering bubble where time flows normally.

SOUND: The cacophony of the outside world is now muffled by the Needle's stabilized field. The Instrument's new song is vibrant, complex, and filled with a fragile hope.

Talia, shaken to her core, but with newfound clarity, moves with purpose. She directs the redirected Aether, now stabilized by Silas, into a vast "Resonance-Cradle" (THM-RES-CRADLE) forming around the Needle's base.

VFX: The Litho-Lichen around the Void-Drain rapidly expands, calcifying the immediate Void-Chitin into a vast, new, iridescent planetary crust. The raging Mnemosyne swarms, caught in the Resonance-Cradle, are frozen mid-flight, turning into grotesque, but inert, statues of chitin and crystal.

EXT. XYRONA PRIME - WIDE SHOT - SYZYGY (CONTINUOUS)

LIGHTING: The planet shimmers with a new, complex aura.

VFX: All three moons align perfectly. The entire planet seems to exhale. The terrifying "Stuttering World" effect recedes. Xyrona Prime is transformed into a fragile "Resonance-Cradle," its surface now a mosaic of verdant green, shimmering apex, deep blues, and unsettling calcified violet-black.

SOUND: The planetary heartbeat stabilizes, still deep, but now with a continuous, evolving, complex HARMONY – the "New Song."

EPOCH 6: AFTERMATH & THE NEW SONG (IMMEDIATE POST-SYZYGY)

INT. HYDRALIS ABYSSAL CHAMBER - POST-SYZYGY

LIGHTING: Soft, contemplative bioluminescence.

ENVIRONMENT: A chamber deep within the abyssal city, serene and calm.

SOUND: The gentle, rhythmic planetary heartbeat, now clearer, permeates the water.

SILAS (CHAR-HYD-SIL) sits alone, his form now in a constant state of quantum superposition. He is translucent, flickering softly, a ghost-like shimmer at his edges. He can pass a hand through solid rock, yet touch a drop of water.

VFX: His existence is a delicate balance of physical and non-Euclidean. He picks up a small, shimmering piece of solidified Mnemosyne chitin, which passes through his hand, then solidifies within it.

SILAS
(Voice is a soft, melodic echo, no longer pained)
I am the bridge. The missing note. Forever here… and not.

He looks at his hands, a faint, almost mournful smile. A profound vulnerability, now a visible part of his essence, draws one in.

INT. VENTURAN CITADEL - TALIA'S RESEARCH LAB - POST-SYZYGY

LIGHTING: Bright, but no longer harsh. Softened.

ENVIRONMENT: Talia's lab is still high-tech, but now focuses on global energy distribution and environmental integration, not just Venturan supremacy. Data screens show complex models of Xyrona Prime's new resonance.

SOUND: The subtle hum of Aether-Arcs is now balanced, harmonious.

TALIA (CHAR-VEN-TALIA) works diligently, her movements precise. She traces the pathways of the Resonance-Cradle on a holographic map. Her expression is focused, but her eyes hold a deep, lingering sadness, a ghost of the terror and desperation she experienced.

VFX: She unconsciously touches her hand, where Silas's touch solidified her. A faint, phantom shimmer lingers.

TALIA
(To herself, quiet)
The debt… it never truly goes away. Only acknowledged. Managed.

Her cold ambition is gone, replaced by a fierce, protective drive. She looks at a distant point in the sky, a yearning in her gaze, for a connection she can barely achieve.

EXT. ELOWEN'S GROVE - DAY - POST-SYZYGY

LIGHTING: Soft, vibrant, alive.

ENVIRONMENT: Elowen's Grove (LOC-SUR-GRO) flourishes. The Chrono-Oaks pulse with a healthy, steady rhythm. The calcified Void-Chitin (from the Syzygy) is now integrated into the landscape, shimmering like iridescent mountains, an unsettling but stable part of the new world-mesh.

SOUND: A gentle, constant hum. The rustling of leaves is a complex, melodic chorus.

ELOWEN (CHAR-SYL-ELO) walks amongst the trees, her staff now a living conduit of light. She gestures, and new saplings begin to sprout from the calcified chitin, weaving green into the violet-black.

ELOWEN
(Calm, wise, her voice carrying the resonance of the planet)
Even discordance has its place in the new harmony. We must continuously re-tune.

Her prophecies now speak not of a single end, but a continuous, living struggle, a "New Verticality" (RULE-VERT-01 Evolved).

INT. ECHO-CHAMBER - NEWLY DISCOVERED - DEEP ABYSS - POST-SYZYGY

LIGHTING: Eerily beautiful. Deep, cosmic blacks and purples, lit by distant, faint nebulae of pure memory.

ENVIRONMENT: A cavern of impossible scale, far below the Hydralis layers, now stable due to the Instrument's activation. This is the Echo-Chamber (LOC-VOID-ECHO).

SOUND: Utter silence, save for faint, almost imperceptible WHISPERS, like ghosts of thought.

A small group of Void Pilgrims explores this terrifying, beautiful new frontier. They carry specialized resonance detectors.

VFX: The detectors light up, revealing faint, shimmering outlines of entities within the dark. Not parasitic, but mournful, translucent, like shattered stained glass. These are the dormant fragments of ARCHON CONSCIOUSNESS.

PILGRIM 1
(Whispering)
They… they are not hostile. They mourn.

PILGRIM 2
(Voice hushed, reverent)
The forgotten lore… the true nature of the Mnemosyne. A path to redemption, not merely battle.

The passage to this chamber, LOC-ABY-VOR, now functions as a treacherous entry point, promising profound knowledge and ancient threats.

EXT. XYRONA PRIME - WIDE SHOT - DAWN - AFTERMATH

LIGHTING: A beautiful, complex sunrise over the newly balanced, yet scarred, world.

ENVIRONMENT: Xyrona Prime now showcases the "New Verticality." Sky-Cities still float, but with a conscious, managed ebb and flow. The Verdant Belt thrives, interwoven with calcified chitin. Abyssal cities exist in a precarious, newly respected balance.

VFX: Subtle, localized resonance shifts are visible across the strata – brief moments of phasing, shimmering echoes, a living planet in constant, careful re-tuning. The underlying Buoyancy Debt (RULE-EQUIL-02) is still a constant, low-frequency hum, a reminder.

SOUND: The "New Song" of Xyrona Prime plays, a complex, multi-layered symphony of existence.

VFX: But from deep within the newly calcified areas, almost imperceptible, new, more insidious forms of the Mnemosyne, adapted to the "Resonance-Cradle," begin to emerge, subtle shadows weaving into the fragile harmony.

The intimate, impossible dance between the strata has only just begun.

FADE OUT.`;