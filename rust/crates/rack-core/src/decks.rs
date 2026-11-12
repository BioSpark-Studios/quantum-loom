//! Decks · model. A mounted Genesis is the disc crate: one deck plays one
//! Mythos, its Containers are the track list, capsules are the beats.
//! Unit: beat. 16 beats = 1 scene; a capsule's length is given in scenes.

use crate::bus::Bus;
use crate::genesis::{Capsule, Container, Genesis, Mythos};
use serde::{Deserialize, Serialize};

pub const LETTERS: [char; 16] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
];
pub const BANKS: [&str; 4] = ["I", "II", "III", "IV"];
/// `(label, glyph, channel)` — the six moods a capsule can carry.
pub const MOODS: [(&str, &str, &str); 6] = [
    ("LOVE", "love", "rose"),
    ("DESIRE", "forge", "warm"),
    ("FEAR", "contagion", "myth"),
    ("HOPE", "prism", "cool"),
    ("WRATH", "vortex", "hot"),
    ("GRIEF", "wave", "life"),
];
/// Loop lengths in beats, with their panel labels.
pub const LOOP_SIZES: [(f64, &str); 11] = [
    (0.0625, "1/16"),
    (0.125, "1/8"),
    (0.25, "1/4"),
    (0.5, "1/2"),
    (1.0, "1"),
    (2.0, "2"),
    (4.0, "4"),
    (8.0, "8"),
    (16.0, "16"),
    (32.0, "32"),
    (64.0, "64"),
];
/// Quantum-event pads: `(name, glyph, channel)`.
pub const QEVENTS: [(&str, &str, &str); 4] = [
    ("DEUS EX", "prism", "amber"),
    ("DEATH", "contagion", "hot"),
    ("SHIFT", "vortex", "cool"),
    ("REVEAL", "lsystem", "myth"),
];

pub fn capsule_beats(c: &Capsule) -> f64 {
    c.payload.scenes.max(1) as f64 * 16.0
}

pub fn track_beats(c: &Container) -> f64 {
    c.capsules.iter().map(capsule_beats).sum::<f64>().max(16.0)
}

/// Capsule index, the capsule and progress (0..1) through it at `beat`.
pub fn capsule_at(c: &Container, beat: f64) -> Option<(usize, &Capsule, f64)> {
    let mut b = 0.0;
    for (i, cap) in c.capsules.iter().enumerate() {
        let len = capsule_beats(cap);
        if beat < b + len {
            return Some((i, cap, (beat - b) / len));
        }
        b += len;
    }
    c.capsules
        .last()
        .map(|cap| (c.capsules.len() - 1, cap, 1.0))
}

/// Deterministic intensity contour of a track at `beat`, 0.03..1.
pub fn intensity(c: &Container, beat: f64) -> f32 {
    let Some((i, cap, local)) = capsule_at(c, beat) else {
        return 0.03;
    };
    let n = (beat * 0.37 + i as f64 * 2.1).sin() * 0.08 + (beat * 1.3 + i as f64).sin() * 0.05;
    let arc = 0.55 + 0.45 * (std::f64::consts::PI * (local * 1.05).min(1.0)).sin();
    (cap.payload.intensity.max(0.05) as f64 * arc + n).clamp(0.03, 1.0) as f32
}

pub fn mood_index(mood: &str) -> Option<usize> {
    ["love", "desire", "fear", "hope", "wrath", "grief"]
        .iter()
        .position(|m| *m == mood)
}

/// Crossfader gain. `assign`: 0 = A side, 1 = THRU, 2 = B side.
pub fn xf_gain(assign: u8, xf: f32) -> f32 {
    if assign == 1 {
        return 1.0;
    }
    let t = if assign == 0 { 1.0 - xf } else { xf };
    (t * 2.0).min(1.0)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Deck {
    pub id: char,
    /// Index of the Mythos this deck plays, if loaded.
    pub mythos: Option<usize>,
    pub track: usize,
    /// Position in beats within the current track.
    pub pos: f64,
    pub playing: bool,
    pub pitch: f32,
    pub sync: bool,
    /// Loop length in beats; 0 = not looping.
    pub loop_len: f64,
    pub loop_start: f64,
    /// 0.5 = half time, 1 = normal, 2 = double time.
    pub rate: f32,
    pub cue: f64,
}

impl Deck {
    fn new(i: usize) -> Self {
        Self {
            id: LETTERS[i],
            mythos: None,
            track: 0,
            pos: 0.0,
            playing: false,
            pitch: 0.5,
            sync: true,
            loop_len: 0.0,
            loop_start: 0.0,
            rate: 1.0,
            cue: 0.0,
        }
    }

    pub fn container<'a>(&self, g: &'a Genesis) -> Option<&'a Container> {
        self.mythos_of(g)?.containers.get(self.track)
    }

    pub fn mythos_of<'a>(&self, g: &'a Genesis) -> Option<&'a Mythos> {
        g.mythos().get(self.mythos?)
    }

    /// Deck tempo: master when synced, else the track's pace bent by the pitch fader.
    pub fn bpm(&self, g: &Genesis, master: f32) -> f32 {
        let own = self
            .container(g)
            .map(|c| c.metadata.pace_bpm)
            .filter(|b| *b > 0.0)
            .unwrap_or(100.0);
        (if self.sync {
            master
        } else {
            own * (0.84 + self.pitch * 0.32)
        }) * self.rate
    }

    pub fn toggle_loop(&mut self, size: f64) {
        if self.loop_len > 0.0 {
            self.loop_len = 0.0;
        } else {
            self.loop_len = size;
            self.loop_start = (self.pos / size).floor() * size;
        }
    }

    pub fn resize_loop(&mut self, size: f64) {
        if self.loop_len > 0.0 {
            self.loop_len = size;
            self.loop_start = (self.loop_start / size).floor() * size;
        }
    }

    pub fn to_zero(&mut self) {
        self.pos = 0.0;
        self.loop_len = 0.0;
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Decks {
    pub decks: Vec<Deck>,
    pub bank: usize,
    /// Deck index shown on the left and right jog.
    pub left: usize,
    pub right: usize,
    /// Effects parked in the four FX-rack slots.
    pub fx: [Option<String>; 4],
    /// Last quantum event fired: `(deck, name)`.
    #[serde(skip)]
    pub last_event: Option<(char, &'static str, f64)>,
    /// Smoothed output level per deck, for the channel meters.
    #[serde(skip)]
    pub levels: [f32; 16],
}

impl Default for Decks {
    fn default() -> Self {
        Self {
            decks: (0..16).map(Deck::new).collect(),
            bank: 0,
            left: 0,
            right: 1,
            fx: [None, None, None, None],
            last_event: None,
            levels: [0.0; 16],
        }
    }
}

impl Decks {
    /// Seed the decks with the Genesis' Mythos in order (deck A = first Mythos).
    pub fn seed(&mut self, g: &Genesis) {
        let n = g.mythos().len();
        for (i, d) in self.decks.iter_mut().enumerate() {
            *d = Deck::new(i);
            d.mythos = (i < n).then_some(i);
        }
    }

    /// Decks of the active bank.
    pub fn bank_range(&self) -> std::ops::Range<usize> {
        self.bank * 4..self.bank * 4 + 4
    }

    pub fn set_bank(&mut self, bank: usize) {
        self.bank = bank.min(3);
        self.left = self.bank * 4;
        self.right = self.bank * 4 + 1;
    }

    /// Left jog switches between the bank's 1st / 3rd deck, right between 2nd / 4th.
    pub fn side_decks(&self, right: bool) -> [usize; 2] {
        let b = self.bank * 4 + right as usize;
        [b, b + 2]
    }

    /// Load the next Mythos from the crate onto a deck.
    pub fn load_next(&mut self, deck: usize, g: &Genesis) {
        let n = g.mythos().len();
        if n == 0 {
            return;
        }
        let d = &mut self.decks[deck];
        d.mythos = Some(d.mythos.map(|m| (m + 1) % n).unwrap_or(0));
        d.track = 0;
        d.to_zero();
    }

    /// Advance every playing deck and publish its clock and level on the bus.
    pub fn update(&mut self, g: &Genesis, bus: &mut Bus, dt: f64, master_bpm: f32) {
        let xf = bus.num("deck.xf", 0.5);
        let (mut mix, mut weight) = (0.0f32, 0.0f32);
        for (i, d) in self.decks.iter_mut().enumerate() {
            let Some(tracks) = d.mythos_of(g).map(|m| m.containers.len()) else {
                self.levels[i] = 0.0;
                continue;
            };
            if d.playing {
                let bpm = d.bpm(g, master_bpm) as f64;
                d.pos += dt * bpm / 60.0;
                if d.loop_len > 0.0 && d.pos >= d.loop_start + d.loop_len {
                    d.pos = d.loop_start + (d.pos - d.loop_start) % d.loop_len;
                }
                let len = d.container(g).map(track_beats).unwrap_or(16.0);
                if d.pos >= len {
                    // Track list auto-advances at the end of a Container.
                    d.pos -= len;
                    d.track = (d.track + 1) % tracks.max(1);
                }
            }
            let k = format!("deck.{}", d.id);
            let it = d.container(g).map(|c| intensity(c, d.pos)).unwrap_or(0.0);
            let vol = bus.num(&format!("{k}.vol"), 0.8)
                * (0.4 + bus.num(&format!("{k}.gain"), 0.7) * 0.86);
            // Odd decks sit on the A side of the crossfader, even ones on B.
            let out = if d.playing {
                it * vol * xf_gain(if i % 2 == 0 { 0 } else { 2 }, xf)
            } else {
                0.0
            };
            self.levels[i] += (out - self.levels[i]) * 0.25;
            if (bus.num(&format!("{k}.out"), 0.0) - out).abs() > 0.004 {
                bus.set_num(&format!("{k}.out"), out);
            }
            if d.playing {
                bus.set_num(&format!("{k}.clock"), d.pos as f32);
                mix += it * out;
                weight += out;
            }
        }
        let v = if weight > 0.0 { mix / weight } else { 0.0 };
        if (bus.num("mix.intensity", 0.0) - v).abs() > 0.004 {
            bus.set_num("mix.intensity", v);
        }
    }
}

/// `mm:ss.cc`
pub fn timecode(sec: f64) -> String {
    let sec = sec.max(0.0);
    format!(
        "{:02}:{:02}.{:02}",
        (sec / 60.0) as u32,
        (sec % 60.0) as u32,
        ((sec % 1.0) * 100.0) as u32
    )
}

/// `SC 02 · 5 &` — scene and beat with sixteenth subdivision, counted from 0.
pub fn step_str(beat: f64) -> String {
    let sub = ["", " e", " &", " a"][((beat % 1.0) * 4.0) as usize % 4];
    format!(
        "SC {:02} · {}{sub}",
        (beat / 16.0) as u32 + 1,
        (beat % 16.0) as u32
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decks_seed_from_genesis_and_advance() {
        let g = Genesis::oros_v7();
        let mut d = Decks::default();
        d.seed(&g);
        assert_eq!(d.decks[15].mythos, Some(15));
        let mut bus = Bus::new();
        d.decks[0].playing = true;
        for _ in 0..600 {
            d.update(&g, &mut bus, 1.0 / 60.0, 120.0);
        }
        // 10 s at 120 bpm = 20 beats.
        assert!((d.decks[0].pos - 20.0).abs() < 0.1, "{}", d.decks[0].pos);
        assert!(bus.num("deck.A.out", 0.0) > 0.0);
        assert!(bus.num("mix.intensity", 0.0) > 0.0);
        assert_eq!(d.decks[1].pos, 0.0);
    }

    #[test]
    fn loops_wrap_and_tracks_advance() {
        let g = Genesis::oros_v7();
        let mut d = Decks::default();
        d.seed(&g);
        let mut bus = Bus::new();
        d.decks[0].playing = true;
        d.decks[0].pos = 5.2;
        d.decks[0].toggle_loop(4.0);
        assert_eq!(d.decks[0].loop_start, 4.0);
        for _ in 0..1200 {
            d.update(&g, &mut bus, 1.0 / 60.0, 120.0);
        }
        assert!((4.0..8.0).contains(&d.decks[0].pos));
        d.decks[0].toggle_loop(4.0);
        let len = track_beats(d.decks[0].container(&g).unwrap());
        d.decks[0].pos = len - 0.01;
        d.update(&g, &mut bus, 0.1, 120.0);
        assert_eq!(d.decks[0].track, 1);
    }

    #[test]
    fn crossfader_law_and_formatting() {
        assert_eq!(xf_gain(1, 0.0), 1.0);
        assert_eq!(xf_gain(0, 1.0), 0.0);
        assert_eq!(xf_gain(0, 0.5), 1.0);
        assert_eq!(xf_gain(2, 0.25), 0.5);
        assert_eq!(timecode(75.5), "01:15.50");
        assert_eq!(step_str(21.5), "SC 02 · 5 &");
    }

    #[test]
    fn side_decks_follow_the_bank() {
        let mut d = Decks::default();
        d.set_bank(2);
        assert_eq!(d.side_decks(false), [8, 10]);
        assert_eq!(d.side_decks(true), [9, 11]);
        assert_eq!(d.bank_range(), 8..12);
    }
}
