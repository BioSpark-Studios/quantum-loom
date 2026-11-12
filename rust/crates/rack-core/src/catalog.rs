//! Departments, the module catalog, narrative effects, preset instruments and
//! panel skins. Everything here is data: add a module by adding a row.

/// One of the sixteen crests (plus AetherForge). `ch` is the signal-colour key
/// the UI resolves; the core never interprets it.
#[derive(Clone, Copy, Debug)]
pub struct Dept {
    pub num: &'static str,
    pub name: &'static str,
    pub crest: &'static str,
    pub ch: &'static str,
    pub law: &'static str,
}

pub const DEPTS: [Dept; 17] = [
    Dept {
        num: "01",
        name: "ATLAS",
        crest: "atlas",
        ch: "cool",
        law: "Space",
    },
    Dept {
        num: "02",
        name: "MYTHOS",
        crest: "mythos",
        ch: "myth",
        law: "World",
    },
    Dept {
        num: "03",
        name: "ARCHITECT",
        crest: "architect",
        ch: "cool",
        law: "Structure",
    },
    Dept {
        num: "04",
        name: "PRISM",
        crest: "prism",
        ch: "bone",
        law: "Light",
    },
    Dept {
        num: "05",
        name: "ANIMUS",
        crest: "animus",
        ch: "amber",
        law: "Character",
    },
    Dept {
        num: "06",
        name: "LOOM",
        crest: "loom",
        ch: "rose",
        law: "Connection",
    },
    Dept {
        num: "07",
        name: "INSTINCT",
        crest: "instinct",
        ch: "myth",
        law: "Reason",
    },
    Dept {
        num: "08",
        name: "ORDER",
        crest: "order",
        ch: "amber",
        law: "Authority",
    },
    Dept {
        num: "09",
        name: "CHRONICLE",
        crest: "chronicle",
        ch: "amber",
        law: "Time",
    },
    Dept {
        num: "10",
        name: "QUILL",
        crest: "quill",
        ch: "myth",
        law: "Narrative",
    },
    Dept {
        num: "11",
        name: "CODEX",
        crest: "codex",
        ch: "life",
        law: "Knowledge",
    },
    Dept {
        num: "12",
        name: "COMPOSER",
        crest: "composer",
        ch: "warm",
        law: "Audio",
    },
    Dept {
        num: "13",
        name: "AXIOM",
        crest: "axiom",
        ch: "cool",
        law: "Rules",
    },
    Dept {
        num: "14",
        name: "CONTINUUM",
        crest: "continuum",
        ch: "life",
        law: "Simulation",
    },
    Dept {
        num: "15",
        name: "FORGE",
        crest: "forge",
        ch: "forge",
        law: "Creation",
    },
    Dept {
        num: "16",
        name: "NEXUS",
        crest: "nexus",
        ch: "bone",
        law: "Bridge",
    },
    Dept {
        num: "17",
        name: "AETHERFORGE",
        crest: "aetherforge",
        ch: "amber",
        law: "Spectacle",
    },
];

pub fn dept(i: usize) -> &'static Dept {
    DEPTS.get(i).unwrap_or(&DEPTS[0])
}

/// A parameter a module exposes: `key` is the short name, `label` what the
/// panel engraves.
pub type ParamDef = (&'static str, &'static str);

#[derive(Clone, Copy, Debug)]
pub struct ModuleDef {
    pub key: &'static str,
    pub name: &'static str,
    /// Eurorack width in HP (1 HP = 16 px in the designer case).
    pub hp: u32,
    pub ch: &'static str,
    pub crest: &'static str,
    /// Shared modules drive the one sound engine and use global addresses;
    /// the rest are per-instance and prefix their keys with `device.module`.
    pub shared: bool,
    pub blurb: &'static str,
    pub params: &'static [ParamDef],
    /// Columns (of 12) the horizontal section occupies in an unfolded device.
    pub span: u32,
}

macro_rules! module {
    ($key:literal, $name:literal, $hp:literal, $ch:literal, $crest:literal, $shared:literal, $span:literal, $blurb:literal, [$(($k:literal, $l:literal)),* $(,)?]) => {
        ModuleDef { key: $key, name: $name, hp: $hp, ch: $ch, crest: $crest, shared: $shared, blurb: $blurb, params: &[$(($k, $l)),*], span: $span }
    };
}

pub const MODULES: &[ModuleDef] = &[
    module!("atlas", "ATLAS", 14, "cool", "atlas", true, 4, "Terrain source · note root/walk", [("atlas.lat", "LAT"), ("atlas.lon", "LON"), ("atlas.alt", "ALT")]),
    module!("continuum", "CONTINUUM", 16, "life", "continuum", true, 4, "Vortex sim · glide/Q", [("cont.visc", "VISC"), ("cont.vort", "VORT"), ("cont.steps", "STEPS"), ("cont.res", "RES")]),
    module!("instinct", "INSTINCT", 13, "myth", "instinct", true, 4, "Envelope + LFO", [("inst.rate", "RATE"), ("inst.depth", "DEPTH"), ("inst.shape", "SHAPE")]),
    module!("chronicle", "CHRONICLE", 20, "amber", "chronicle", true, 8, "16-step sequencer · clock", [("chr.bpm", "TEMPO"), ("chr.swing", "SWING")]),
    module!("composer", "COMPOSER", 14, "warm", "composer", true, 6, "Voice · gain/pitch/drive", [("comp.gain", "GAIN"), ("comp.pitch", "PITCH"), ("comp.drive", "DRIVE"), ("comp.formant", "CUTOFF")]),
    module!("quill", "QUILL", 12, "myth", "quill", true, 6, "Arc · mode/tension/XY", [("quill.act", "ACT"), ("quill.tension", "TENSION"), ("quill.xy.x", "BRIGHT"), ("quill.xy.y", "WOBBLE")]),
    module!("forge", "FORGE", 10, "forge", "forge", true, 4, "Master out · arm/rec", [("forge.main", "MAIN")]),
    module!("macro4", "MACRO", 8, "cool", "knob", false, 3, "4 free macro knobs", [("a", "A"), ("b", "B"), ("c", "C"), ("d", "D")]),
    module!("fader3", "FADERS", 8, "life", "fader", false, 3, "3 faders", [("f1", "F1"), ("f2", "F2"), ("f3", "F3")]),
    module!("gates4", "GATES", 7, "life", "gate", false, 3, "4 latching gates", [("g1", "G1"), ("g2", "G2"), ("g3", "G3"), ("g4", "G4")]),
    module!("scope", "SCOPE", 12, "cool", "wave", false, 4, "Live scope + trims", [("gain", "GAIN"), ("trim", "TRIM")]),
    module!("xy", "XY", 10, "rose", "node", false, 3, "2D modulation pad", [("xy.x", "X"), ("xy.y", "Y")]),
    module!("seq", "STEPS", 20, "amber", "clock", false, 6, "16-step gate row", [("len", "LEN"), ("prob", "PROB"), ("rot", "ROT")]),
    module!("automata", "AUTOMATA", 10, "life", "contagion", false, 3, "Game of Life density", [("dens", "DENSITY")]),
    module!("fractal", "FRACTAL", 10, "cool", "lsystem", false, 3, "Mandelbrot recursion", [("depth", "DEPTH")]),
    module!("aloom", "AXIOMATIC LOOM", 40, "cool", "loom", false, 12, "Ludic engine · crest convergence · axioms · memory crystals", [("heat", "HEAT"), ("grav", "GRAVITY"), ("res", "RESONANCE"), ("rec", "RECURSION"), ("resOut", "RES OUT"), ("loadOut", "LOAD OUT"), ("gate", "GATE")]),
    module!("atomgraph", "ATOM GRAPH", 66, "cool", "node", false, 12, "Node graph · prompt chains through the LLM router", [("out", "OUT")]),
    module!("stem", "STEM", 14, "warm", "audio", false, 4, "Audio/video player locked to a clock", [("gain", "GAIN"), ("ofs", "OFFSET")]),
    module!("herald", "HERALD", 60, "amber", "nexus", false, 12, "Terminal egress node · runic checksum · vortex protocol · anchor pinning · tidal sync", [("ping", "PING"), ("loss", "LOSS"), ("anchorw", "ANCHOR W"), ("tidal", "TIDAL"), ("vortex", "VORTEX"), ("energy", "ENERGY"), ("tide", "TIDE"), ("ack", "ACK")]),
    module!("hearth", "HEARTHSTONE", 60, "warm", "order", false, 12, "Sanctum OS · rack health orb · the Weft of device threads · the Pulse · prompt oracle", [("health", "HEALTH"), ("warmth", "WARMTH")]),
    module!("scriptorium", "CHRONOSCRIPTORIUM", 66, "amber", "quill", false, 12, "Narrative performance engine · rune pads · living scribe · temporal scroll · oracle prose", [("last", "LAST RUNE"), ("f1", "EMBER"), ("f2", "VEIL"), ("f3", "TIDE"), ("f4", "ECHO"), ("k1", "FLICKER"), ("k2", "HUE"), ("pad0", "PAD 1"), ("pad1", "PAD 2"), ("pad2", "PAD 3"), ("pad3", "PAD 4")]),
    module!("nexus", "ORCHESTRATION NEXUS", 66, "bone", "nexus", false, 12, "Chronospatial canvas · the live rack as orbiting bodies · woven causal threads · Chorus coherence", [("orbit", "ORBIT"), ("warp", "WARP"), ("coherence", "CHORUS"), ("th0", "THREAD 1"), ("th1", "THREAD 2"), ("th2", "THREAD 3")]),
    module!("eidolon", "EIDOLON STAGE", 66, "rose", "animus", false, 12, "Character performance · psyche constellation · emotion river on the Main Timeline", [("anx", "ANXIETY"), ("joy", "JOY"), ("res", "RESOLVE"), ("cur", "CURIOSITY"), ("grf", "GRIEF"), ("tru", "TRUST"), ("post", "POSTURE"), ("gaze", "GAZE"), ("proj", "PROJECT"), ("xy.x", "OPEN"), ("xy.y", "GESTURE"), ("valence", "VALENCE"), ("arousal", "AROUSAL")]),
    module!("aftimeline", "MAIN TIMELINE", 66, "amber", "clock", false, 12, "AetherForge master clock · glyph tracks · concurrent alt timelines", [("t0", "TRK 1"), ("t1", "TRK 2"), ("t2", "TRK 3"), ("t3", "TRK 4"), ("t4", "TRK 5"), ("t5", "TRK 6")]),
    module!("atmos", "ATMOSPHERE", 14, "cool", "atlas", false, 5, "World aether · pressure/fog/gravity · ambient engine", [("press", "PRESSURE"), ("fog", "FOG"), ("grav", "GRAVITY"), ("turb", "TURB"), ("amb", "AMBIENT")]),
    module!("sigils", "SIGILS", 10, "amber", "gate", false, 3, "8 latching elemental sigils", [("s0", "FLARE"), ("s1", "FROST"), ("s2", "VORTEX"), ("s3", "STORM"), ("s4", "BOLT"), ("s5", "TIDE"), ("s6", "SHARD"), ("s7", "WARD")]),
    module!("dmx", "LIGHT DESK", 16, "bone", "prism", false, 5, "6 DMX channels · 0–255", [("c1", "CH1"), ("c2", "CH2"), ("c3", "CH3"), ("c4", "CH4"), ("c5", "CH5"), ("c6", "CH6")]),
];

pub fn module(key: &str) -> Option<&'static ModuleDef> {
    MODULES.iter().find(|m| m.key == key)
}

pub const MODULE_GROUPS: &[(&str, &[&str])] = &[
    (
        "ENGINE · SHARED",
        &[
            "atlas",
            "continuum",
            "instinct",
            "chronicle",
            "composer",
            "quill",
            "forge",
        ],
    ),
    (
        "UTILITY · PER INSTANCE",
        &["macro4", "fader3", "gates4", "xy", "seq", "scope"],
    ),
    ("ALGORITHMIC", &["automata", "fractal"]),
    ("AETHERFORGE · WORLD", &["aftimeline", "atmos", "sigils"]),
    ("MEDIA", &["stem"]),
    ("DIGITAL ALCHEMY", &["aloom"]),
    (
        "ATOMS · AGENTIC",
        &["atomgraph", "eidolon", "nexus", "scriptorium", "hearth"],
    ),
    ("OUTPUT", &["dmx", "herald"]),
];

/// Narrative / data effects. They operate on a device's output values in the
/// core frame, never on audio.
#[derive(Clone, Copy, Debug)]
pub struct FxDef {
    pub key: &'static str,
    /// "Technique · Meaning", e.g. "Echo · Foreshadow".
    pub name: &'static str,
    pub glyph: &'static str,
    pub ch: &'static str,
}

pub const FX: &[FxDef] = &[
    FxDef {
        key: "echo",
        name: "Echo · Foreshadow",
        glyph: "loop",
        ch: "myth",
    },
    FxDef {
        key: "verb",
        name: "Reverb · Linger",
        glyph: "wave",
        ch: "cool",
    },
    FxDef {
        key: "comp",
        name: "Compressor · Pace",
        glyph: "fader",
        ch: "amber",
    },
    FxDef {
        key: "invert",
        name: "Invert · Reversal",
        glyph: "split",
        ch: "rose",
    },
    FxDef {
        key: "quant",
        name: "Quantize · Beats",
        glyph: "gate",
        ch: "life",
    },
    FxDef {
        key: "drift",
        name: "Drift · Entropy",
        glyph: "vortex",
        ch: "warm",
    },
    FxDef {
        key: "plague",
        name: "Contagion · Spread",
        glyph: "contagion",
        ch: "life",
    },
    FxDef {
        key: "gate",
        name: "Logic · Threshold",
        glyph: "logic",
        ch: "cool",
    },
];

pub fn fx(key: &str) -> Option<&'static FxDef> {
    FX.iter().find(|f| f.key == key)
}

impl FxDef {
    /// The technique ("Echo") and its narrative meaning ("Foreshadow").
    pub fn parts(&self) -> (&'static str, &'static str) {
        self.name
            .split_once(" · ")
            .unwrap_or((self.name, self.name))
    }
}

/// Apply an effect to a normalised value. `t_ms` is engine time in milliseconds.
pub fn fx_apply(key: &str, v: f32, t_ms: f64) -> f32 {
    match key {
        "comp" => 0.35 + (v - 0.5) * 0.4,
        "invert" => 1.0 - v,
        "quant" => (v * 4.0).round() / 4.0,
        "drift" => (v + (t_ms * 0.0013).sin() as f32 * 0.12).clamp(0.0, 1.0),
        "plague" => (v * (0.6 + 0.4 * ((t_ms * 0.0007).sin() as f32).powi(2))).clamp(0.0, 1.0),
        "gate" => {
            if v > 0.5 {
                1.0
            } else {
                0.0
            }
        }
        // echo / verb shape time, not level: the frame value passes through.
        _ => v,
    }
}

pub const SKINS: &[(&str, &str)] = &[
    ("graphite", "#262b35"),
    ("ivory", "#e4e1d6"),
    ("olive", "#5e5d40"),
    ("teal", "#22858d"),
    ("ember", "#4e2a1d"),
    ("indigo", "#272c62"),
];

pub fn default_skin(inst_id: &str) -> &'static str {
    match inst_id {
        "i-story" => "indigo",
        "i-lights" => "ivory",
        "i-bloom" => "teal",
        "i-master" => "ember",
        _ => "graphite",
    }
}

/// Preset instrument: `(id, name, dept index, module types, macro picks)`,
/// where a macro pick is `(module index, param key)`.
pub type PresetDef = (
    &'static str,
    &'static str,
    usize,
    &'static [&'static str],
    &'static [(usize, &'static str)],
);

pub const PRESETS: &[PresetDef] = &[
    (
        "i-main",
        "MAIN TIMELINE",
        5,
        &["aftimeline"],
        &[(0, "t0"), (0, "t1"), (0, "t2"), (0, "t3")],
    ),
    (
        "i-story",
        "STORY CLOCK",
        8,
        &["chronicle", "quill"],
        &[
            (0, "chr.bpm"),
            (0, "chr.swing"),
            (1, "quill.act"),
            (1, "quill.tension"),
            (1, "quill.xy.x"),
        ],
    ),
    (
        "i-tornado",
        "TORNADO SYNTH",
        13,
        &["atlas", "continuum", "instinct", "composer"],
        &[
            (0, "atlas.lat"),
            (0, "atlas.lon"),
            (1, "cont.vort"),
            (1, "cont.visc"),
            (2, "inst.rate"),
            (3, "comp.drive"),
            (3, "comp.formant"),
            (3, "comp.pitch"),
        ],
    ),
    (
        "i-lights",
        "LIGHT DESK",
        3,
        &["dmx", "xy"],
        &[
            (0, "c1"),
            (0, "c2"),
            (0, "c3"),
            (0, "c4"),
            (1, "xy.x"),
            (1, "xy.y"),
        ],
    ),
    (
        "i-bloom",
        "AUTOMATA BLOOM",
        10,
        &["automata", "fractal", "seq", "macro4"],
        &[(0, "dens"), (1, "depth"), (2, "prob"), (3, "a"), (3, "b")],
    ),
    (
        "i-loom",
        "AXIOMATIC LOOM",
        5,
        &["aloom"],
        &[(0, "heat"), (0, "grav"), (0, "res"), (0, "rec")],
    ),
    ("i-graph", "ATOM GRAPH", 15, &["atomgraph"], &[(0, "out")]),
    (
        "i-stems",
        "STEM PAIR",
        11,
        &["stem", "stem", "macro4"],
        &[(0, "gain"), (1, "gain"), (0, "ofs"), (1, "ofs"), (2, "a")],
    ),
    (
        "i-atmos",
        "ATMOSPHERE",
        16,
        &["atmos", "sigils", "xy"],
        &[
            (0, "press"),
            (0, "fog"),
            (0, "grav"),
            (0, "amb"),
            (2, "xy.x"),
            (2, "xy.y"),
        ],
    ),
    (
        "i-perform",
        "AETHER PERFORMANCE",
        16,
        &["xy", "sigils", "scope", "fader3"],
        &[
            (0, "xy.x"),
            (0, "xy.y"),
            (2, "gain"),
            (3, "f1"),
            (3, "f2"),
            (3, "f3"),
        ],
    ),
    (
        "i-eidolon",
        "EIDOLON STAGE",
        4,
        &["eidolon"],
        &[
            (0, "anx"),
            (0, "joy"),
            (0, "res"),
            (0, "grf"),
            (0, "post"),
            (0, "xy.x"),
            (0, "xy.y"),
            (0, "proj"),
        ],
    ),
    (
        "i-nexus",
        "ORCHESTRATION NEXUS",
        15,
        &["nexus"],
        &[
            (0, "orbit"),
            (0, "warp"),
            (0, "coherence"),
            (0, "th0"),
            (0, "th1"),
        ],
    ),
    (
        "i-script",
        "CHRONOSCRIPTORIUM",
        9,
        &["scriptorium"],
        &[
            (0, "f1"),
            (0, "f2"),
            (0, "f3"),
            (0, "f4"),
            (0, "k1"),
            (0, "k2"),
            (0, "last"),
        ],
    ),
    (
        "i-hearth",
        "HEARTHSTONE",
        7,
        &["hearth"],
        &[(0, "health"), (0, "warmth")],
    ),
    (
        "i-herald",
        "HERALD",
        15,
        &["herald"],
        &[
            (0, "ping"),
            (0, "loss"),
            (0, "anchorw"),
            (0, "tidal"),
            (0, "energy"),
        ],
    ),
    (
        "i-master",
        "MASTER FORGE",
        14,
        &["forge", "scope"],
        &[(0, "forge.main"), (1, "gain")],
    ),
];

/// Output adapters that interpret the same core frame.
pub const TARGETS: &[(&str, &str, &str)] = &[
    ("narr", "NARRATIVE", "myth"),
    ("dmx", "DMX · LIGHTS", "bone"),
    ("unreal", "UNREAL · OSC", "cool"),
    ("midi", "MIDI", "amber"),
    ("ws", "WEBSOCKET", "life"),
];

pub const FACTIONS: &[(&str, &str)] = &[
    ("Luminarite", "#EDD878"),
    ("Venturan", "#00BFFF"),
    ("Sylvanid", "#2ECC71"),
    ("Hydralis", "#1A8FD1"),
    ("Syntaran", "#D4890A"),
];

pub fn faction_color(name: &str) -> &'static str {
    FACTIONS
        .iter()
        .find(|f| f.0 == name)
        .unwrap_or(&FACTIONS[1])
        .1
}

/// `(key, label, signal colour)` for the master connection.
pub const CONNECTIONS: &[(&str, &str, &str)] = &[
    ("local", "LOCALHOST", "life"),
    ("p2p", "P2P HOST", "amber"),
    ("remote", "REMOTE VAULT", "cool"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_reference_real_modules_and_params() {
        for (id, _, dept, types, picks) in PRESETS {
            assert!(*dept < DEPTS.len(), "{id}");
            for t in *types {
                assert!(module(t).is_some(), "{id}: unknown module {t}");
            }
            for (i, key) in *picks {
                let m = module(types[*i]).unwrap();
                assert!(
                    m.params.iter().any(|p| p.0 == *key),
                    "{id}: {} has no param {key}",
                    m.key
                );
            }
        }
    }

    #[test]
    fn groups_cover_every_module_once() {
        let mut seen: Vec<&str> = MODULE_GROUPS
            .iter()
            .flat_map(|g| g.1.iter().copied())
            .collect();
        seen.sort();
        let mut all: Vec<&str> = MODULES.iter().map(|m| m.key).collect();
        all.sort();
        assert_eq!(seen, all);
    }

    #[test]
    fn effects_stay_in_range() {
        for f in FX {
            for i in 0..=10 {
                let v = fx_apply(f.key, i as f32 / 10.0, 1234.5);
                assert!((0.0..=1.0).contains(&v), "{} → {v}", f.key);
            }
        }
        assert_eq!(fx_apply("invert", 0.25, 0.0), 0.75);
        assert_eq!(fx_apply("quant", 0.6, 0.0), 0.5);
    }
}
