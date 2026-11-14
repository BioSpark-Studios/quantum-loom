//! MAIN TIMELINE — the AetherForge master clock. Glyph tracks hold clips; a
//! track's value at time `t` is its curve over whichever clip covers `t`,
//! scaled by intensity. Quill / Loom / Chronicle run as concurrent alternate
//! timelines derived from the same clock (rate + offset).

use crate::bus::Bus;
use serde::{Deserialize, Serialize};

pub const LEN: f32 = 24.0;
pub const FPS: f32 = 24.0;
pub const RATES: [f32; 4] = [1.0, 2.0, 0.5, -1.0];
pub const CURVES: [&str; 4] = ["ease", "linear", "pulse", "hold"];

pub fn curve(name: &str, p: f32) -> f32 {
    let p = p.clamp(0.0, 1.0);
    match name {
        "linear" => {
            if p < 0.5 {
                p * 2.0
            } else {
                (1.0 - p) * 2.0
            }
        }
        "pulse" => {
            ((p * std::f32::consts::PI * 10.0).sin() * 0.5 + 0.5) * (std::f32::consts::PI * p).sin()
        }
        "hold" => {
            if p < 0.06 {
                p / 0.06
            } else if p > 0.94 {
                (1.0 - p) / 0.06
            } else {
                1.0
            }
        }
        _ => (std::f32::consts::PI * p).sin(),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub id: u32,
    pub start: f32,
    pub len: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub ch: String,
    pub hue: f32,
    pub inten: f32,
    pub spin: f32,
    pub curve: String,
    pub mute: bool,
    pub clips: Vec<Clip>,
}

impl Track {
    pub fn value(&self, t: f32) -> f32 {
        if self.mute {
            return 0.0;
        }
        self.clips
            .iter()
            .filter(|c| t >= c.start && t < c.start + c.len)
            .map(|c| curve(&self.curve, (t - c.start) / c.len))
            .fold(0.0, f32::max)
            * self.inten
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Alt {
    pub id: String,
    pub name: String,
    pub rate: f32,
    pub ofs: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    pub tracks: Vec<Track>,
    pub alts: Vec<Alt>,
    pub t: f32,
    pub playing: bool,
    pub looping: bool,
    /// Follow the rack transport instead of the local play state.
    pub link: bool,
    pub next: u32,
}

impl Default for Timeline {
    fn default() -> Self {
        let tr = |id: &str,
                  name: &str,
                  glyph: &str,
                  ch: &str,
                  inten: f32,
                  spin: f32,
                  curve: &str,
                  clips: &[(f32, f32)]| Track {
            id: id.into(),
            name: name.into(),
            glyph: glyph.into(),
            ch: ch.into(),
            hue: 0.0,
            inten,
            spin,
            curve: curve.into(),
            mute: false,
            clips: clips
                .iter()
                .enumerate()
                .map(|(i, (s, l))| Clip {
                    id: i as u32 + 1,
                    start: *s,
                    len: *l,
                })
                .collect(),
        };
        let alt = |id: &str, name: &str, rate: f32, ofs: f32| Alt {
            id: id.into(),
            name: name.into(),
            rate,
            ofs,
        };
        Self {
            tracks: vec![
                tr(
                    "t-rad",
                    "RADIANT SIGIL",
                    "prism",
                    "cool",
                    0.9,
                    0.25,
                    "ease",
                    &[(0.5, 9.0), (15.0, 7.0)],
                ),
                tr(
                    "t-eng",
                    "ENGINE SEAL",
                    "forge",
                    "amber",
                    0.8,
                    -0.15,
                    "hold",
                    &[(6.0, 11.0)],
                ),
                tr(
                    "t-vor",
                    "VORTEX",
                    "vortex",
                    "myth",
                    0.7,
                    0.6,
                    "pulse",
                    &[(3.0, 5.0), (12.0, 6.0)],
                ),
                tr(
                    "t-mist",
                    "ETHEREAL MIST",
                    "wave",
                    "life",
                    0.55,
                    0.0,
                    "linear",
                    &[(8.0, 12.0)],
                ),
            ],
            alts: vec![
                alt("quill", "QUILL · NARRATIVE", 1.0, 0.0),
                alt("loom", "LOOM · AXIOMS", 0.5, 4.0),
                alt("chronicle", "CHRONICLE · STEPS", 2.0, 0.0),
            ],
            t: 8.6,
            playing: false,
            looping: true,
            link: false,
            next: 100,
        }
    }
}

pub fn wrap(x: f32) -> f32 {
    x.rem_euclid(LEN)
}

/// `hh:mm:ss:ff` at 24 fps.
pub fn timecode(t: f32) -> String {
    let s = t.max(0.0);
    let f = ((s % 1.0) * FPS) as u32;
    format!(
        "{:02}:{:02}:{:02}:{:02}",
        (s / 3600.0) as u32,
        (s / 60.0) as u32 % 60,
        s as u32 % 60,
        f
    )
}

impl Timeline {
    pub fn next_id(&mut self) -> u32 {
        self.next += 1;
        self.next
    }

    /// Advance by `dt` seconds. `transport` is the rack transport state, used when linked.
    pub fn update(&mut self, dt: f32, transport: bool) {
        let running = if self.link { transport } else { self.playing };
        if !running {
            return;
        }
        let mut n = self.t + dt;
        if n >= LEN {
            if self.looping {
                n -= LEN;
            } else {
                n = LEN;
                self.playing = false;
            }
        }
        self.t = n;
    }

    /// Publish the clock, every track value and the alternate timelines.
    pub fn publish(&self, bus: &mut Bus, prefix: &str) {
        bus.set_num("aft.time", self.t / LEN);
        for (i, tr) in self.tracks.iter().enumerate() {
            bus.set_num(&format!("{prefix}.t{i}"), tr.value(self.t));
        }
        for a in &self.alts {
            bus.set_num(
                &format!("aft.alt.{}", a.id),
                wrap(self.t * a.rate + a.ofs) / LEN,
            );
        }
    }

    pub fn add_track(&mut self) -> String {
        let n = self.tracks.len();
        let glyphs = [
            "flare",
            "contagion",
            "lsystem",
            "branch",
            "emotion",
            "solver",
        ];
        let chs = ["cool", "myth", "life", "warm", "amber", "rose"];
        let id = format!("t{}", self.next_id());
        let g = glyphs[n % glyphs.len()];
        let start = self.t.floor().min(LEN - 4.0);
        let cid = self.next_id();
        self.tracks.push(Track {
            id: id.clone(),
            name: g.to_uppercase(),
            glyph: g.into(),
            ch: chs[n % chs.len()].into(),
            hue: 0.0,
            inten: 0.7,
            spin: 0.2,
            curve: "ease".into(),
            mute: false,
            clips: vec![Clip {
                id: cid,
                start,
                len: 4.0,
            }],
        });
        id
    }

    pub fn add_clip(&mut self, track: &str, at: f32) {
        let id = self.next_id();
        let start = ((at * 2.0).floor() / 2.0).clamp(0.0, LEN - 3.0);
        if let Some(t) = self.tracks.iter_mut().find(|t| t.id == track) {
            t.clips.push(Clip {
                id,
                start,
                len: 3.0,
            });
        }
    }

    pub fn move_clip(&mut self, track: &str, clip: u32, start: f32) {
        if let Some(c) = self.clip_mut(track, clip) {
            c.start = ((start * 4.0).round() / 4.0).clamp(0.0, LEN - c.len);
        }
    }

    pub fn resize_clip(&mut self, track: &str, clip: u32, len: f32) {
        if let Some(c) = self.clip_mut(track, clip) {
            c.len = ((len * 4.0).round() / 4.0).clamp(0.5, LEN - c.start);
        }
    }

    pub fn remove_clip(&mut self, track: &str, clip: u32) {
        if let Some(t) = self.tracks.iter_mut().find(|t| t.id == track) {
            t.clips.retain(|c| c.id != clip);
        }
    }

    fn clip_mut(&mut self, track: &str, clip: u32) -> Option<&mut Clip> {
        self.tracks
            .iter_mut()
            .find(|t| t.id == track)?
            .clips
            .iter_mut()
            .find(|c| c.id == clip)
    }

    pub fn cycle_rate(&mut self, alt: &str) {
        if let Some(a) = self.alts.iter_mut().find(|a| a.id == alt) {
            let i = RATES.iter().position(|r| *r == a.rate).unwrap_or(0);
            a.rate = RATES[(i + 1) % RATES.len()];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curves_are_zero_at_the_ends_and_peak_inside() {
        for c in CURVES {
            assert!(curve(c, 0.0).abs() < 1e-5, "{c}");
            assert!(curve(c, 1.0).abs() < 1e-4, "{c}");
            assert!((0.0..=1.0001).contains(&curve(c, 0.5)), "{c}");
        }
        assert!((curve("ease", 0.5) - 1.0).abs() < 1e-5);
        assert_eq!(curve("hold", 0.5), 1.0);
    }

    #[test]
    fn track_value_follows_its_clips() {
        let tl = Timeline::default();
        let rad = &tl.tracks[0];
        assert_eq!(rad.value(0.1), 0.0);
        assert!(rad.value(5.0) > 0.8, "{}", rad.value(5.0));
        assert_eq!(rad.value(12.0), 0.0);
        let mut m = rad.clone();
        m.mute = true;
        assert_eq!(m.value(5.0), 0.0);
    }

    #[test]
    fn clock_loops_or_stops() {
        let mut tl = Timeline {
            t: 23.5,
            playing: true,
            ..Default::default()
        };
        tl.update(1.0, false);
        assert!((tl.t - 0.5).abs() < 1e-4);
        tl.looping = false;
        tl.t = 23.5;
        tl.update(1.0, false);
        assert_eq!(tl.t, LEN);
        assert!(!tl.playing);
        // linked: follows the rack transport, not its own play state
        let mut l = Timeline {
            link: true,
            t: 1.0,
            playing: true,
            ..Default::default()
        };
        l.update(1.0, false);
        assert_eq!(l.t, 1.0);
        l.update(1.0, true);
        assert_eq!(l.t, 2.0);
    }

    #[test]
    fn publishes_clock_tracks_and_alts() {
        let tl = Timeline {
            t: 12.0,
            ..Default::default()
        };
        let mut bus = Bus::new();
        tl.publish(&mut bus, "dmain.m0");
        assert!((bus.num("aft.time", 0.0) - 0.5).abs() < 1e-5);
        assert!(bus.num("dmain.m0.t1", 0.0) > 0.7); // engine seal holds
                                                    // loom runs at half rate, offset 4 s: (12*0.5+4)/24
        assert!((bus.num("aft.alt.loom", 0.0) - 10.0 / 24.0).abs() < 1e-5);
        assert!((bus.num("aft.alt.chronicle", 0.0) - 0.0).abs() < 1e-5); // 24 wraps to 0
    }

    #[test]
    fn clip_editing_snaps_and_clamps() {
        let mut tl = Timeline::default();
        tl.move_clip("t-rad", 1, 100.0);
        assert_eq!(tl.tracks[0].clips[0].start, LEN - 9.0);
        tl.move_clip("t-rad", 1, 1.13);
        assert_eq!(tl.tracks[0].clips[0].start, 1.25);
        tl.resize_clip("t-rad", 1, 0.0);
        assert_eq!(tl.tracks[0].clips[0].len, 0.5);
        tl.add_clip("t-rad", 20.7);
        assert_eq!(tl.tracks[0].clips.last().unwrap().start, 20.5);
        tl.remove_clip("t-rad", 1);
        assert_eq!(tl.tracks[0].clips.len(), 2);
        let id = tl.add_track();
        assert!(tl.tracks.iter().any(|t| t.id == id));
        tl.cycle_rate("quill");
        assert_eq!(tl.alts[0].rate, 2.0);
        assert_eq!(timecode(61.5), "00:01:01:12");
    }
}
