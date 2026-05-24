import { Project, NarrativeScope, ContainerType, Scene, DialogueLine, MediaType } from '../types';
import { BLUEPRINT_CONTAINER } from './blueprints';

const generateScenes = (): Scene[] => {
  const scenes: Scene[] = [];
  
  const sceneData = [
    {
      id: 'scene-01',
      number: 1,
      heading: 'INT. 9TH CYCLE SEMANTICS VAULT - PENTAD APEX - DAWN',
      description: 'Archivist Kaelen-Xi monitors the 528Hz Great Humming from within the sterile Semantics Vault.',
      characters: ['KAELEN-XI'],
      dialogue: [
        "06:00. The Great Humming, on schedule. Every sub-harmonic accounted for.",
        "The resonance meters are flat-lining at perfect consonance. Too perfect.",
        "Another cycle, another ritual. They equate silence with peace.",
        "But silence can also be the sound of something about to shatter.",
        "The Hydralis sensors confirm: Xyrona Prime is a perfectly tuned instrument.",
        "Yet the Syntaran algorithms detect… a whisper of something unwritten.",
        "The data doesn't lie, but it omits.",
        "A world that has learned to stop breaking is a world that has forgotten how to breathe.",
        "This numbing bliss... it feels like a diagnostic of impending failure.",
        "The Concordance Anchor holds. For now."
      ]
    },
    {
      id: 'scene-02',
      number: 2,
      heading: 'EXT. LEY-RAIL - SKY-ROOT BRIDGE - DAWN',
      description: 'Rin navigates the energy conduits of Xyrona Prime with hyper-kinetic joy.',
      characters: ['RIN'],
      dialogue: [
        "Woo! This is the good stuff, Kaelen! Pure kinetic flow!",
        "Don't need a frequency, just need the speed!",
        "The rail is humming in my marrow today. Feels alive.",
        "They call it 'Great Humming,' I call it 'the morning rush.'",
        "Ky'rennei energy, Zha'thik engineering. Perfect combination.",
        "Just me, the rail, and a thousand places to be.",
        "Hey, Arch-Boy, you still counting shadows in your marble maze?",
        "I see a flicker up ahead... but nothing a little speed can't fix.",
        "This is freedom, Kaelen. The real kind.",
        "Forget the numbers, just feel the pulse."
      ]
    },
    {
      id: 'scene-03',
      number: 3,
      heading: 'INT. 9TH CYCLE SEMANTICS VAULT / EXT. LEY-RAIL - DAWN',
      description: 'The frequency snaps, replaced by a jarring 440Hz dissonance spike.',
      characters: ['KAELEN-XI', 'RIN'],
      dialogue: [
        "KAELEN-XI: Stable. Too stable.",
        "RIN: Feel that, Kaelen? The whole world singing!",
        "KAELEN-XI: No... my resonance meter... it's spiking!",
        "RIN: What was that? The rail just... hiccuped!",
        "KAELEN-XI: 440Hz interference! It's a dissonance spike!",
        "RIN: The energy is sour, Kaelen! Beyond sour, it's... twisting!",
        "KAELEN-XI: The data-spires are bleeding! This isn't just interference, Rin. This is an attack!",
        "RIN: Attack?! What kind of attack breaks the hum?",
        "KAELEN-XI: One that aims to rewrite the very laws of reality!"
      ]
    },
    {
      id: 'scene-04',
      number: 4,
      heading: 'INT. 9TH CYCLE SEMANTICS VAULT - MOMENTS LATER',
      description: 'Localized gravity inversion within the Vault.',
      characters: ['KAELEN-XI'],
      dialogue: [
        "No... this isn't right! The Anchor... it's compromised!",
        "Gravity is... inverting! I'm falling... upwards!",
        "My data-streams are corrupted... processing impossible physics!",
        "The Vault... it's shedding its memories!",
        "Archival tome 7-Sigma-9... dissolving into silt!",
        "This isn't just chaos, this is... a deliberate re-coding of reality!",
        "Who has this power? To weaponize consensus reality itself?",
        "My lungs feel... a cryogenic chill. A familiar numbness... but screaming now.",
        "I am an archivist of ghosts... and today, the ghosts are screaming in 440Hz.",
        "Must... reach... comm-link. Rin."
      ]
    },
    {
      id: 'scene-05',
      number: 5,
      heading: 'INT. 9TH CYCLE SEMANTICS VAULT (VIEWING PORT) - MOMENTS LATER',
      description: 'Kaelen witnesses the sky bruising under crystalline white light.',
      characters: ['KAELEN-XI'],
      dialogue: [
        "The sky... it's bruising. They're not just breaching, they're... rebranding.",
        "That light... that cold, crystalline white. It's the Vessels of Order.",
        "They want to erase individual dissonance. Reconstruct their Empyrean Anchor.",
        "They're not just inverting gravity, Rin. They're rewriting the world.",
        "That sigil... I've seen it in the restricted archives. 'The Architect's Lock.'",
        "It's the color of absolute law. The color of the end of the world.",
        "This pressure... the air itself is trying to settle into a perfect, unmoving crystalline state.",
        "They're imposing 963Hz. The frequency of... divine rule.",
        "I need to reach Rin. She's kinetic, she might... find a loophole.",
        "Rin! Are you there? The Apex... it's falling apart!"
      ]
    },
    {
      id: 'scene-06',
      number: 6,
      heading: 'EXT. INVERTED LEY-RAIL - SKY-ROOT BRIDGE - MOMENTS LATER',
      description: 'Rin struggles with gravity inversion on the rails.',
      characters: ['RIN'],
      dialogue: [
        "Kaelen! The rail... it just turned into a fall!",
        "I'm running on air! No, I'm... clinging to the ceiling!",
        "My stomach's doing a full loop-de-loop! What did they do?!",
        "The energy is going crazy! It's pulling me up when I want to go down!",
        "My whip! It won't stick! This isn't how Ley-Rails work!",
        "Don't need the math, Kaelen, just need the speed. But now... I need grip!",
        "The city... it's falling apart, piece by piece, into the sky!",
        "This is a bad glitch. A really, really bad glitch.",
        "Hang on, Rin. Don't think about it. Be the rail. Even if the rail is wrong.",
        "I'm the kinetic spark... I just need to find the spark in this mess."
      ]
    },
    {
      id: 'scene-07',
      number: 7,
      heading: 'INT. 9TH CYCLE SEMANTICS VAULT - CONTINUOUS',
      description: 'Archival storm within the Vault.',
      characters: ['KAELEN-XI', 'RIN'],
      dialogue: [
        "KAELEN-XI: Damn it! Every protocol is failing!",
        "KAELEN-XI: The data... it's dissolving! The 9th Cycle Semantics Vault... becoming a ghost!",
        "KAELEN-XI: Cannot lose this knowledge! It's all we have left!",
        "KAELEN-XI: My lungs... Aethyr-Silt! Already?",
        "KAELEN-XI: Rin! You there? The comm-link is barely holding!",
        "KAELEN-XI: This is more than a breach. It's semantic erasure on a planetary scale!",
        "KAELEN-XI: I manage the cycles, the eleventh, the ninth... but now, they're unmaking them!",
        "KAELEN-XI: If the archive dies, we lose everything... our names, our history!",
        "KAELEN-XI: Hold on, Rin. I'm trying to stabilize the signal.",
        "RIN: Kaelen...? I hear you... barely... The rail... it's breaking."
      ]
    },
    {
      id: 'scene-08',
      number: 8,
      heading: 'INT. 9TH CYCLE SEMANTICS VAULT / EXT. INVERTED LEY-RAIL - CONTINUOUS',
      description: 'Fragmented comm-link established between Kaelen and Rin.',
      characters: ['KAELEN-XI', 'RIN'],
      dialogue: [
        "RIN: Kaelen! The rail! It's inverted! I'm... I'm upside down!",
        "KAELEN-XI: Don't let go, Rin! The Vessels... they've breached the Apex! They're rewriting gravity!",
        "RIN: Rewriting?! It feels like... like the world's just a broken... a broken algorithm!",
        "KAELEN-XI: They're using a 440Hz spike! It's a dissonance frequency! It's shattering the marble-code!",
        "RIN: Marble-code?! My body's marble-code right now! I'm going to flat-line!",
        "KAELEN-XI: Focus! Your frequency-blindness! Can you... can you feel a current?",
        "RIN: I feel... a jitter! A weird, sideways pull! It's not the hum anymore!",
        "KAELEN-XI: The probability of our survival is a rounding error, Rin. Fortunately, I’ve always been fond of the decimal places.",
        "RIN: You talk too much for someone about to hit a wall at Mach one! Shut up and jump!",
        "KAELEN-XI: They want absolute order. We need... a glitch. Find the glitch, Rin!",
        "RIN: A glitch... yeah... I think I'm living it!"
      ]
    },
    {
      id: 'scene-09',
      number: 9,
      heading: 'INT. 9TH CYCLE SEMANTICS VAULT (VIEWING PORT) - CONTINUOUS',
      description: 'The Apex consumed by crystalline white light.',
      characters: ['KAELEN-XI', 'RIN', 'MALCOR'],
      dialogue: [
        "KAELEN-XI: Rin! The Apex... it's being consumed! The sky is losing its color!",
        "RIN: I see it! It's like... a disease! A white, cold disease climbing the towers!",
        "KAELEN-XI: They're not just breaching, they're... reconstructing! Re-coding the Marble-Code!",
        "RIN: The ground below me... it's like glass exploding upwards! I can't get a grip!",
        "KAELEN-XI: The Concordance Anchor... it's screaming. That's a 963Hz signature!",
        "MALCOR: Your 'freedom' is a stutter in a perfect sentence, Archivist. I am simply the editor.",
        "KAELEN-XI: Malcor! You bastard! You're destroying everything!",
        "MALCOR: I am restoring order. The chaos of your 'glitch' ends now.",
        "RIN: Kaelen! I'm slipping! The air's too heavy! I can't breathe up here!",
        "KAELEN-XI: Hang on, Rin! Don't let his frequency consume you! Fight the logic!",
        "MALCOR: Absolute order is a choice. You simply lack the vision."
      ]
    },
    {
      id: 'scene-10',
      number: 10,
      heading: 'INT. 9TH CYCLE SEMANTICS VAULT - CONTINUOUS',
      description: 'Kaelen searches for a glitch in the data.',
      characters: ['KAELEN-XI', 'RIN'],
      dialogue: [
        "KAELEN-XI: Malcor... 'editor.' He wants to clean the code. Eliminate the variables.",
        "KAELEN-XI: But where there is law, there must be a loophole. A glitch in the system.",
        "KAELEN-XI: Rin, are you detecting any... any aberrations in the energy flow? Something illogical?",
        "RIN: Illogical is my middle name right now, Kaelen! Everything's going sideways!",
        "KAELEN-XI: The text on this tome... it's trying to rewrite itself. 'Where the order is absolute, the shadow is the only truth.'",
        "KAELEN-XI: The Riddle of the Star-Shadow... it's real. And it's manifesting here.",
        "KAELEN-XI: My archive is collapsing. This isn't random. It's targeted semantic erasure.",
        "KAELEN-XI: I need a counter-frequency. Something organic. Something that breaks their perfect sentence.",
        "KAELEN-XI: The air... it's getting thicker. Malcor's 'logic' is filling the Apex.",
        "KAELEN-XI: This rebreather... I'll need it. The Aethyr-Silt is becoming a physical entity.",
        "RIN: Kaelen! I'm losing altitude! I'm falling... up!"
      ]
    },
    {
      id: 'scene-11',
      number: 11,
      heading: 'EXT. INVERTED LEY-RAIL - SKY-ROOT BRIDGE - CONTINUOUS',
      description: 'Rin discovers the jitter-flow.',
      characters: ['RIN', 'KAELEN-XI'],
      dialogue: [
        "RIN: Kaelen? Are you still there? The rail's gone wild!",
        "KAELEN-XI: Find a glitch, Rin! A broken circuit! Anything that isn't their perfect law!",
        "RIN: Perfect law feels like... static trying to eat my face! And that whistling... what is that?!",
        "RIN: I don't see anything, but I feel... a jitter-flow. Like a tremor in the logic.",
        "RIN: The city's shedding its skin below me! I'm seeing ghosts of buildings!",
        "RIN: The air... it tastes like rusted iron and screams!",
        "RIN: Okay, jitter-flow... be the jitter-flow. Where's the imperfection?",
        "RIN: There! A flicker! A weak spot! Come on, whip, don't fail me now!",
        "RIN: I'm on it, Kaelen! I'm riding the glitch!",
        "KAELEN-XI: Hold to the rail, Rin! Don't let go! The dissonance is high!"
      ]
    },
    {
      id: 'scene-12',
      number: 12,
      heading: 'INT. 9TH CYCLE SEMANTICS VAULT (VIEWING PORT) - CONTINUOUS',
      description: 'The horizon rewritten as code.',
      characters: ['KAELEN-XI'],
      dialogue: [
        "It's not just breaking... they're unmaking. The horizon... it's just raw code.",
        "This is Malcor's 'Empyrean Anchor.' Pure, distilled 963Hz. A mathematical god.",
        "He wants to erase every glitch. Every flaw. Every... organic breath.",
        "The Unwritten Wastes Logic... 'If SilentStatus equals true, InitiateDissolution of ActivePersona.'",
        "My comm-link... the word 'Unwritten.' They're trying to erase my very name.",
        "This isn't law. This is... an algorithmic tyranny.",
        "If I stay silent... I dissolve. If I speak... I am erased.",
        "The Riddle of the Star-Shadow: 'Where the order is absolute, the shadow is the only truth.'",
        "The shadow... the loophole. Rin. She's the shadow. The glitch.",
        "They want a perfect sentence. But a perfect sentence has no life. No choice.",
        "I will not be unwritten."
      ]
    },
    {
      id: 'scene-13',
      number: 13,
      heading: 'EXT. INVERTED LEY-RAIL - SKY-ROOT BRIDGE - CONTINUOUS',
      description: 'Rin rides the glitch current.',
      characters: ['RIN', 'KAELEN-XI'],
      dialogue: [
        "RIN: Kaelen! I feel it! A... a broken circuit! A jitter that's going somewhere!",
        "KAELEN-XI: The glitch! You found the loophole! Ride it, Rin! Ride the dissonance!",
        "RIN: It's not safe! It's wild! But it's moving! I'm going in!",
        "RIN: Zha’thik engineering, Ky’rennei light! We’re the only loophole in the dead of the night!",
        "RIN: Who needs absolute order when you have pure, chaotic speed?!",
        "RIN: This is my purpose, Kaelen! Beyond the rails, beyond the hum!",
        "KAELEN-XI: She doesn't hear the hum, she just feels the slide... through the neon tunnels where the glitches hide!",
        "RIN: I don't need the math, I just need the speed! I'm the kinetic spark that the Concordance needs!",
        "RIN: The Apex is breaking, the peace is a lie! But the glitch... the glitch is true!",
        "RIN: Look out below, Malcor! The rule-breaker is coming!"
      ]
    },
    {
      id: 'scene-14',
      number: 14,
      heading: 'INT. 9TH CYCLE SEMANTICS VAULT (EXIT HATCH) - CONTINUOUS',
      description: 'Kaelen reaches the exit hatch.',
      characters: ['KAELEN-XI', 'RIN'],
      dialogue: [
        "KAELEN-XI: Rin's found a current. The glitch. She's moving.",
        "KAELEN-XI: I need to reach the convergence point. This exit hatch is our only way out.",
        "RIN: I'm coming, Kaelen! Fast as the chaos will carry me!",
        "KAELEN-XI: Good. Because out here... the Anchor-weight is crushing everything.",
        "KAELEN-XI: Your 'freedom' might be chaotic, Rin. But it's alive. And that's what we need.",
        "KAELEN-XI: The probability of hitting this convergence point... it's still a rounding error. But I like our chances.",
        "RIN: You always did like the decimal places, Arch-Boy!",
        "KAELEN-XI: The Apex is turning to petrified marble-code. We can't stay here.",
        "KAELEN-XI: Your glitch is our only truth now.",
        "KAELEN-XI: Just keep moving, Rin. I'll meet you on the outside."
      ]
    },
    {
      id: 'scene-15',
      number: 15,
      heading: 'EXT. PENTAD APEX - SKYLINE - CONTINUOUS',
      description: 'Kaelen witnesses the full scale of the collapse.',
      characters: ['KAELEN-XI', 'RIN'],
      dialogue: [
        "KAELEN-XI: The Apex is ours... no, it's theirs. Or it's nothing.",
        "RIN: I see you, Kaelen! I'm coming! Just follow the bad code!",
        "KAELEN-XI: The very fabric of Xyrona Prime... it's tearing. The 528Hz peace is a lie.",
        "RIN: The Ley-Rail is screaming, the energy flows! Where the runner goes, nobody knows!",
        "KAELEN-XI: She doesn't hear the hum... she just feels the slide. That's our only truth now.",
        "KAELEN-XI: The ground... Null-Code. My footprints vanish. Semantic erasure is physical.",
        "RIN: Zha’thik engineering, Ky’rennei light! We’re the only loophole in the dead of the night!",
        "KAELEN-XI: We are the loophole. The glitch. The anomaly.",
        "KAELEN-XI: The Arch-Boy and the Runner. Unlikely alliance, indeed.",
        "KAELEN-XI: The dissonance high and the pressure below... but we're still moving. We're still breathing."
      ]
    },
    {
      id: 'scene-16',
      number: 16,
      heading: 'EXT. PENTAD APEX - CONVERGENCE POINT - CONTINUOUS',
      description: 'Kaelen and Rin meet amidst the ruins.',
      characters: ['KAELEN-XI', 'RIN'],
      dialogue: [
        "RIN: Archivist. You made it. Thought you were going to flat-line in that marble cave.",
        "KAELEN-XI: Runner. You're... a chaotic constant. Never thought I'd be glad for an error.",
        "RIN: The sky still looks angry. But it's not tearing anymore.",
        "KAELEN-XI: Broken, but breathing. A new frequency. An imperfect one.",
        "RIN: So... where to, Arch-Boy? The rest of the glitch?",
        "KAELEN-XI: The Apex is lost. But the 'shadow is the only truth,' Rin.",
        "KAELEN-XI: Malcor wanted to erase individual dissonance. He failed. We are the dissonance.",
        "RIN: And the glitch. We choose the glitch, Kaelen. We choose the choice.",
        "KAELEN-XI: The 13th Seed... it woke up. With us.",
        "RIN: Next stop: wherever the echoes lead, right?",
        "KAELEN-XI: To the ruins of the First Pulse. To track the origins of his weapons.",
        "RIN: Sounds like a plan. A messy, chaotic, beautiful plan."
      ]
    }
  ];

  return sceneData.map(data => ({
    ...data,
    dialogue: data.dialogue.map((line, idx) => {
      const parts = line.split(': ');
      const character = parts.length > 1 ? parts[0] : data.characters[0];
      const text = parts.length > 1 ? parts[1] : parts[0];
      return {
        id: `${data.id}-d${idx}`,
        character,
        text
      };
    })
  }));
};

export const STAR_SHADOW_PROJECT: Project = {
  id: 'star-shadow-heresy-001',
  name: 'The Star-Shadow Heresy',
  lastModified: Date.now(),
  scriptContent: `TITLE BLOCK
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

... (Scenes 2-15 inferred and present in scenes array) ...

SCENE-16
SCENE-TITLE: The Sovereign Error
LOCATION: EXT. PENTAD APEX - CONVERGENCE POINT
TIME-OF-DAY: Post-Collapse Dusk
SCENE-SUMMARY: Kaelen and Rin meet amidst the ruins of the Apex. The 528Hz frequency is gone, replaced by a raw, imperfect resonance. They embrace their status as "dissonance" and prepare to protect the 13th Seed.
THEMATIC HOOK: The beauty of imperfection and the sovereignty of the glitch.

MOVEMENTS
[MOVEMENT-31]
[CAM] — 65mm Large Format wide shot. Tracking shot at waist height.
[VFX] — The sky is peeling like wallpaper, showing a void of raw stars behind the clouds.
[DL] — Kaelen and Rin walk toward the horizon. Kaelen limps heavily on his dominant leg.
[SFX] — Rain hitting dry earth (Petrichor Foley). Warm analog synth pads.
[NOTES] — The "Limp of Truth" leitmotif.
[MOVEMENT-32]
[CAM] — Slow zoom out into a high-angle "God View."
[VFX] — Digital HUD dissolves into the rain droplets.
[DL] — Rin looks up, letting rain hit her face. Kaelen closes his eyes.
[SFX] — Final resonant sustain of the 13th Seed Pulse. Fade to natural wind.
[NOTES] — Resolution of the arc.

DIALOGUE
[DIALOGUE-16-01]
RIN: "Archivist. You made it. Thought you were going to flat-line in that marble cave."
[DIALOGUE-16-02]
KAELEN-XI: "Runner. You're... a chaotic constant. Never thought I'd be glad for an error."
[DIALOGUE-16-03]
RIN: "The sky still looks angry. But it's not tearing anymore."
[DIALOGUE-16-04]
KAELEN-XI: "Broken, but breathing. A new frequency. An imperfect one."
[DIALOGUE-16-05]
RIN: "So... where to, Arch-Boy? The rest of the glitch?"
[DIALOGUE-16-06]
KAELEN-XI: "The Apex is lost. But the 'shadow is the only truth,' Rin."
[DIALOGUE-16-07]
KAELEN-XI: "Malcor wanted to erase individual dissonance. He failed. We are the dissonance."
[DIALOGUE-16-08]
RIN: "And the glitch. We choose the glitch, Kaelen. We choose the choice."
[DIALOGUE-16-09]
KAELEN-XI: "The 13th Seed... it woke up. With us."
[DIALOGUE-16-10]
RIN: "Next stop: wherever the echoes lead, right?"
[DIALOGUE-16-11]
KAELEN-XI: "To the ruins of the First Pulse. To track the origins of his weapons."
[DIALOGUE-16-12]
RIN: "Sounds like a plan. A messy, chaotic, beautiful plan."
`,
  scenes: generateScenes(),
  timeline: [],
  characterVoices: {
    'KAELEN-XI': 'Zephyr',
    'RIN': 'Kore',
    'MALCOR': 'Charon'
  },
  chapterMarkers: [
    { id: 'ch1', time: 0, label: 'The Apex Breach', color: '#6366f1' },
    { id: 'ch2', time: 45, label: 'Descent into Venturis', color: '#f59e0b' },
    { id: 'ch3', time: 90, label: 'The Star-Shadow Logic', color: '#ef4444' },
    { id: 'ch4', time: 135, label: 'The Thirteenth Seed', color: '#10b981' }
  ],
  genesis: {
    id: 'genesis-1111-1316-book1-0001',
    name: "The Star-Shadow Heresy: The Thirteenth Seed",
    description: "The first volume of the 13th Movement, following Archivist Kaelen-Xi and Runner Rin as they defend the modern Fivefold Concord against the Vessels of Order.",
    genre: ["Epic Science-Fantasy", "Post-Singularity Noir", "Archaeological Technomancy"],
    themes: ["The Sovereignty of the Glitch", "The Burden of Ancestral Memory", "Law vs. Loophole", "Organic Imperfection"],
    narrativeScope: NarrativeScope.EPIC,
    setting: {
      location: "Xyrona Prime (Pentad Apex and Peripheral Ruins)",
      timePeriod: "The 13th Movement (The Era of the Fivefold Concord)",
      atmosphere: "Technologically utopian yet haunted by deep-time echoes; vibrant and high-stakes.",
      culturalContext: "A unified society where Zha'thik engineering and Ky’rennei energy-trade maintain a fragile 528Hz peace."
    },
    mythosContainers: [
      {
        id: "mythos-13-the-apex-breach",
        name: "Movement 13: The Echo in the Apex",
        description: "The intrusion of ancient chaos into modern utopia during the Resonant Day celebration.",
        mythosType: "spatial",
        capsules: [
          {
            id: "cap-pers-kaelen-xi",
            name: "Kaelen-Xi",
            description: "Cynical Archivist",
            kind: "persona",
            glyphs: [],
            traits: [],
            tags: ["Hydralis-Syntaran"],
            created: new Date().toISOString(),
            modified: new Date().toISOString(),
            author: "System",
            version: "1.0",
            properties: {
              archetype: "Cynical Archivist",
              age: 34,
              personality: "Analytical, weary, dry-humored."
            }
          },
          {
            id: "cap-pers-rin-runner",
            name: "Rin",
            description: "Ley-Rail Runner",
            kind: "persona",
            glyphs: [],
            traits: [],
            tags: ["Ky’rennei"],
            created: new Date().toISOString(),
            modified: new Date().toISOString(),
            author: "System",
            version: "1.0",
            properties: {
              archetype: "Ley-Rail Runner",
              age: 26,
              personality: "Impulsive, loyal, hyper-kinetic."
            }
          }
        ],
        maxCapsules: 10,
        properties: {},
        created: new Date().toISOString(),
        modified: new Date().toISOString(),
        tags: ["Movement 13"]
      },
      BLUEPRINT_CONTAINER
    ],
    maxMythosContainers: 12,
    globalSettings: {
      defaultPacing: 120,
      defaultTension: 0.7,
      narrativeStyle: "Polyphonic-Cinematic"
    },
    version: "1.0.0",
    created: new Date().toISOString(),
    modified: new Date().toISOString()
  }
};
