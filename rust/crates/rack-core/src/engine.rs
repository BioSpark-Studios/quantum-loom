//! Transport clock and the signal model the screens read.
//!
//! There is no audio backend here: synthesis belongs to the Composer / DSP
//! crate and plugs in behind [`AudioBackend`]. Until then the engine runs the
//! same 16-step clock, picks the same notes and exposes a synthetic model of
//! the current note / filter / drive so scopes and meters move.

use crate::bus::{Bus, Value};
use std::collections::VecDeque;

pub const SCALES: [&[i32]; 5] = [
    &[0, 2, 4, 7, 9],
    &[0, 2, 3, 5, 7, 8, 10],
    &[0, 2, 4, 5, 7, 9, 11],
    &[0, 3, 5, 6, 7, 10],
    &[0, 1, 5, 7, 8],
];
pub const SCALE_NAMES: [&str; 5] = ["PENTA", "AEOLIAN", "IONIAN", "BLUES", "HIRAJ"];
pub const DEFAULT_ENV: [[f32; 2]; 5] = [
    [0.0, 1.0],
    [0.18, 0.4],
    [0.5, 0.65],
    [0.82, 0.2],
    [1.0, 0.0],
];
pub const CHAPTERS: usize = 12;
pub const SCENES_PER_CHAPTER: usize = 4;
pub const ROMAN: [&str; 12] = [
    "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII",
];

/// Seam for a real audio engine. The default implementation is silence.
pub trait AudioBackend: Send {
    fn note_on(&mut self, midi: i32, accent: bool);
    fn set_param(&mut self, key: &str, value: f32);
    /// Latest output block for scopes, if the backend produces audio.
    fn scope(&self) -> Option<&[f32]> {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Note {
    pub abs: u64,
    pub midi: i32,
    pub accent: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Rms {
    pub l: f32,
    pub r: f32,
    pub drive: f32,
}

pub struct Engine {
    pub playing: bool,
    pub armed: bool,
    pub step: i32,
    pub abs: u64,
    pub level: f32,
    pub rms: Rms,
    pub last_midi: i32,
    /// Engine time (seconds) of the last triggered note.
    pub last_trig: f64,
    pub notes: VecDeque<Note>,
    pub hist: [f32; 160],
    pub hist_i: usize,
    /// Seconds since the engine was created.
    pub time: f64,
    until_step: f64,
    frame_acc: f64,
    rng: u32,
    backend: Option<Box<dyn AudioBackend>>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            playing: false,
            armed: false,
            step: -1,
            abs: 0,
            level: 0.0,
            rms: Rms::default(),
            last_midi: 48,
            last_trig: -10.0,
            notes: VecDeque::with_capacity(64),
            hist: [0.0; 160],
            hist_i: 0,
            time: 0.0,
            until_step: 0.0,
            frame_acc: 0.0,
            rng: 0x9E37_79B9,
            backend: None,
        }
    }

    pub fn set_backend(&mut self, backend: Box<dyn AudioBackend>) {
        self.backend = Some(backend);
    }

    /// Seed the patterns and defaults the rack expects before any control is touched.
    pub fn seed(bus: &mut Bus) {
        let a = [1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0].map(|x| x == 1);
        let b = [0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0].map(|x| x == 1);
        bus.init("chr.a", Value::Steps(a.to_vec()));
        bus.init("chr.b", Value::Steps(b.to_vec()));
        bus.init("inst.env", Value::Curve(DEFAULT_ENV.to_vec()));
        bus.init("chr.bpm", Value::Num(0.53));
        bus.init("chr.swing", Value::Num(0.12));
        bus.init("forge.main", Value::Num(0.78));
        bus.init("rack.follow", Value::Bool(true));
        bus.init("rack.chapter", Value::Num(0.0));
        bus.init("rack.scene", Value::Num(0.0));
        bus.init("transport.step", Value::Num(0.0));
        bus.init("transport.playing", Value::Bool(false));
    }

    pub fn rand(&mut self) -> f32 {
        // xorshift32: deterministic noise for meters, no external RNG needed.
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x >> 8) as f32 / (1u32 << 24) as f32
    }

    pub fn bpm(bus: &Bus) -> f32 {
        (60.0 + bus.num("chr.bpm", 0.53) * 120.0).round()
    }

    pub fn bpm_exact(bus: &Bus) -> f32 {
        60.0 + bus.num("chr.bpm", 0.53) * 120.0
    }

    pub fn step_dur(bus: &Bus) -> f64 {
        60.0 / Self::bpm(bus) as f64 / 4.0
    }

    pub fn scale_idx(bus: &Bus) -> usize {
        if bus.flag("mod.QUILL.on", true) {
            ((bus.num("quill.act", 0.6) * 4.0).round() as usize).min(4)
        } else {
            0
        }
    }

    pub fn play(&mut self, bus: &mut Bus) {
        if self.playing {
            return;
        }
        self.playing = true;
        self.until_step = 0.0;
        bus.set_flag("transport.playing", true);
    }

    pub fn stop(&mut self, bus: &mut Bus) {
        self.playing = false;
        bus.set_flag("transport.playing", false);
    }

    pub fn toggle_play(&mut self, bus: &mut Bus) {
        if self.playing {
            self.stop(bus)
        } else {
            self.play(bus)
        }
    }

    pub fn toggle_arm(&mut self, bus: &mut Bus) {
        self.armed = !self.armed;
        bus.set_flag("engine.on", self.armed);
        if self.armed {
            self.play(bus);
        } else {
            bus.set_flag("engine.rec", false);
        }
    }

    pub fn toggle_rec(&mut self, bus: &mut Bus) {
        let rec = !bus.flag("engine.rec", false);
        if rec && !self.armed {
            self.toggle_arm(bus);
        }
        bus.set_flag("engine.rec", rec);
    }

    /// Advance the engine by `dt` seconds of wall time.
    pub fn update(&mut self, bus: &mut Bus, dt: f64) {
        let dt = dt.clamp(0.0, 0.25);
        self.time += dt;
        if self.playing {
            self.until_step -= dt;
            let mut guard = 0;
            while self.until_step <= 0.0 && guard < 8 {
                self.tick(bus);
                guard += 1;
            }
        }
        // Meter / envelope decay at a fixed 60 Hz so it is frame-rate independent.
        self.frame_acc += dt;
        while self.frame_acc >= 1.0 / 60.0 {
            self.frame_acc -= 1.0 / 60.0;
            self.level *= 0.94;
            let (n1, n2) = (self.rand(), self.rand());
            self.rms.l = (self.level * 0.9 + n1 * 0.03).clamp(0.0, 1.0);
            self.rms.r = (self.level * 0.85 + n2 * 0.03).clamp(0.0, 1.0);
            self.rms.drive = (self.level * bus.num("comp.drive", 0.6)).clamp(0.0, 1.0);
            let n = self.hist.len();
            self.hist[self.hist_i % n] = self.rms.r;
            self.hist_i += 1;
        }
    }

    fn tick(&mut self, bus: &mut Bus) {
        let s = (self.step + 1).rem_euclid(16);
        self.step = s;
        self.abs += 1;
        bus.set_num("transport.step", s as f32);

        if s == 0 && self.abs > 1 && bus.flag("rack.follow", true) {
            let scene = bus.num("rack.scene", 0.0) as usize + 1;
            if scene >= SCENES_PER_CHAPTER {
                bus.set_num("rack.scene", 0.0);
                let ch = (bus.num("rack.chapter", 0.0) as usize + 1) % CHAPTERS;
                bus.set_num("rack.chapter", ch as f32);
            } else {
                bus.set_num("rack.scene", scene as f32);
            }
        }

        let hit = bus
            .steps("chr.a")
            .map(|a| a.get(s as usize).copied().unwrap_or(false))
            .unwrap_or(false);
        let accent = bus
            .steps("chr.b")
            .map(|b| b.get(s as usize).copied().unwrap_or(false))
            .unwrap_or(false);
        if hit && bus.flag("mod.CHRONICLE.on", true) {
            self.trigger(bus, s, accent);
        }
        let sw = bus.num("chr.swing", 0.12) as f64 * 0.5;
        let dur = Self::step_dur(bus) * if s % 2 == 0 { 1.0 + sw } else { 1.0 - sw };
        self.until_step += dur;
    }

    fn trigger(&mut self, bus: &mut Bus, step: i32, accent: bool) {
        let scale = SCALES[Self::scale_idx(bus)];
        let atl = bus.flag("mod.ATLAS.on", true);
        let g = |k: &str, d: f32| if atl { bus.num(k, d) } else { 0.0 };
        let (lat, lon, alt) = (
            g("atlas.lat", 0.42),
            g("atlas.lon", 0.61),
            g("atlas.alt", 0.28),
        );
        let spread = 1 + (alt * 2.0).round() as i32;
        let len = scale.len() as i32;
        let deg =
            (step * (1 + (lon * 5.0).round() as i32) + (lat * 7.0).round() as i32) % (len * spread);
        let midi = 40
            + ((bus.num("comp.pitch", 0.5) - 0.5) * 24.0).round() as i32
            + scale[(deg % len) as usize]
            + 12 * (deg / len)
            + if accent { 12 } else { 0 };
        self.voice(bus, midi, accent);
    }

    pub fn voice(&mut self, bus: &mut Bus, midi: i32, accent: bool) {
        if self.notes.len() >= 64 {
            self.notes.pop_front();
        }
        self.notes.push_back(Note {
            abs: self.abs,
            midi,
            accent,
        });
        self.level = if accent { 1.0 } else { 0.75 };
        self.last_midi = midi;
        self.last_trig = self.time;
        bus.set_num("transport.note", midi as f32);
        if self.armed {
            if let Some(b) = self.backend.as_mut() {
                b.note_on(midi, accent);
            }
        }
    }

    // ---- read side for visualizers -------------------------------------

    pub fn amp(&self, bus: &Bus) -> f32 {
        0.12 + 0.88 * self.level * bus.num("comp.gain", 0.6) * 1.4
    }

    pub fn cycles(&self) -> f32 {
        1.5 + (self.last_midi - 28) as f32 / 12.0
    }

    pub fn harmonics(bus: &Bus) -> i32 {
        1 + (bus.num("comp.formant", 0.4) * 9.0).floor() as i32
    }

    /// Band-limited saw at the current note; `u` is 0..1 across the screen.
    pub fn synth_wave(&self, bus: &Bus, u: f32, t: f32) -> f32 {
        let h_max = Self::harmonics(bus);
        let c = self.cycles();
        let mut y = 0.0;
        for h in 1..=h_max {
            let h = h as f32;
            y += (std::f32::consts::TAU * h * c * u + t * 3.0 * h).sin() / h;
        }
        let d = bus.num("comp.drive", 0.6) * 3.0;
        (y * 0.6 * (1.0 + d)).tanh() / (1.0 + d).tanh() * self.amp(bus)
    }

    /// Synthetic spectrum magnitude for a normalised frequency `f` (0..1).
    pub fn synth_band(&mut self, bus: &Bus, f: f32) -> f32 {
        let base = 0.02 + (self.last_midi - 28) as f32 / 600.0;
        let h_max = Self::harmonics(bus) + 2;
        let cut = bus.num("comp.formant", 0.4);
        let mut v = 0.0;
        for h in 1..=h_max {
            let d = f - h as f32 * base;
            v += (-(d * d) / 0.0005).exp() / (h as f32).sqrt();
        }
        let lp = 1.0 / (1.0 + (f / (cut * 0.9 + 0.05)).powi(4));
        (v * lp * self.amp(bus) * 1.3 + self.rand() * 0.04).clamp(0.0, 1.0)
    }
}

/// Sample a piecewise-linear envelope at `n` evenly spaced points.
pub fn sample_env(pts: &[[f32; 2]], n: usize) -> Vec<f32> {
    let mut s: Vec<[f32; 2]> = pts.to_vec();
    s.sort_by(|a, b| a[0].total_cmp(&b[0]));
    if s.len() < 2 || n < 2 {
        return vec![s.first().map(|p| p[1]).unwrap_or(0.0); n.max(1)];
    }
    (0..n)
        .map(|i| {
            let x = i as f32 / (n - 1) as f32;
            let mut j = 0;
            while j < s.len() - 2 && s[j + 1][0] < x {
                j += 1;
            }
            let (a, b) = (s[j], s[j + 1]);
            let u = if b[0] == a[0] {
                0.0
            } else {
                (x - a[0]) / (b[0] - a[0])
            };
            a[1] + (b[1] - a[1]) * u.clamp(0.0, 1.0)
        })
        .collect()
}

pub fn midi_name(m: i32) -> String {
    const N: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    format!("{}{}", N[m.rem_euclid(12) as usize], m.div_euclid(12) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(e: &mut Engine, bus: &mut Bus, secs: f64) {
        let mut t = 0.0;
        while t < secs {
            e.update(bus, 1.0 / 120.0);
            t += 1.0 / 120.0;
        }
    }

    #[test]
    fn clock_steps_at_tempo() {
        let mut bus = Bus::new();
        Engine::seed(&mut bus);
        bus.set_num("chr.swing", 0.0);
        let mut e = Engine::new();
        e.play(&mut bus);
        // 124 bpm → 16th = 0.121 s → about 16.5 steps in 2 s.
        run(&mut e, &mut bus, 2.0);
        assert!((16..=18).contains(&e.abs), "abs = {}", e.abs);
        assert!(!e.notes.is_empty());
    }

    #[test]
    fn follow_advances_scene_then_chapter() {
        let mut bus = Bus::new();
        Engine::seed(&mut bus);
        let mut e = Engine::new();
        e.play(&mut bus);
        for _ in 0..(16 * 4 + 1) {
            e.tick(&mut bus);
        }
        assert_eq!(bus.num("rack.chapter", 0.0), 1.0);
        assert_eq!(bus.num("rack.scene", 9.0), 0.0);
    }

    #[test]
    fn stopped_engine_does_not_advance() {
        let mut bus = Bus::new();
        Engine::seed(&mut bus);
        let mut e = Engine::new();
        run(&mut e, &mut bus, 1.0);
        assert_eq!(e.abs, 0);
    }

    #[test]
    fn envelope_sampling_hits_endpoints() {
        let v = sample_env(&DEFAULT_ENV, 11);
        assert_eq!(v.len(), 11);
        assert!((v[0] - 1.0).abs() < 1e-6);
        assert!(v[10].abs() < 1e-6);
    }

    #[test]
    fn note_names() {
        assert_eq!(midi_name(60), "C4");
        assert_eq!(midi_name(69), "A4");
    }
}
