//! MIDI input logic, independent of any device backend. A backend (midir in a
//! later pass) feeds raw bytes to [`Midi::handle`]; control changes land on the
//! bus as `midi.cc.<n>` (0..1) and can be assigned as a route source with LEARN.

use crate::bus::Bus;
use crate::model::RackState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Msg {
    NoteOn { ch: u8, note: u8, vel: u8 },
    NoteOff { ch: u8, note: u8 },
    Cc { ch: u8, cc: u8, value: u8 },
    PitchBend { ch: u8, value: i16 },
}

/// Decode one channel-voice message. Running status and system messages are
/// not handled (backends deliver whole messages).
pub fn parse(data: &[u8]) -> Option<Msg> {
    let (&st, rest) = data.split_first()?;
    let (kind, ch) = (st & 0xf0, st & 0x0f);
    let d = |i: usize| rest.get(i).copied().map(|b| b & 0x7f);
    match kind {
        0x90 => {
            let (note, vel) = (d(0)?, d(1)?);
            // A note-on with velocity 0 is a note-off.
            Some(if vel == 0 {
                Msg::NoteOff { ch, note }
            } else {
                Msg::NoteOn { ch, note, vel }
            })
        }
        0x80 => Some(Msg::NoteOff { ch, note: d(0)? }),
        0xb0 => Some(Msg::Cc {
            ch,
            cc: d(0)?,
            value: d(1)?,
        }),
        0xe0 => Some(Msg::PitchBend {
            ch,
            value: ((d(1)? as i16) << 7 | d(0)? as i16) - 8192,
        }),
        _ => None,
    }
}

#[derive(Default, Debug)]
pub struct Midi {
    /// Route waiting for a controller move.
    pub learning: Option<String>,
    /// Control numbers seen so far, sorted.
    pub seen: Vec<u8>,
    /// Last CC: `(cc, value 0..127)`.
    pub last: Option<(u8, u8)>,
}

impl Midi {
    pub fn start_learn(&mut self, route_id: &str) {
        self.learning = Some(route_id.to_owned());
    }

    pub fn cancel_learn(&mut self) {
        self.learning = None;
    }

    /// Returns true if the message changed anything.
    pub fn handle(&mut self, state: &mut RackState, bus: &mut Bus, data: &[u8]) -> bool {
        let Some(Msg::Cc { cc, value, .. }) = parse(data) else {
            return false;
        };
        if let Err(i) = self.seen.binary_search(&cc) {
            self.seen.insert(i, cc);
        }
        bus.set_num(&format!("midi.cc.{cc}"), value as f32 / 127.0);
        self.last = Some((cc, value));
        if let Some(id) = self.learning.take() {
            if let Some(r) = state.routes.iter_mut().find(|r| r.id == id) {
                r.from = format!("midi:{cc}");
                r.amt = 1.0;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_channel_voice_messages() {
        assert_eq!(
            parse(&[0xb0, 7, 100]),
            Some(Msg::Cc {
                ch: 0,
                cc: 7,
                value: 100
            })
        );
        assert_eq!(
            parse(&[0x93, 60, 90]),
            Some(Msg::NoteOn {
                ch: 3,
                note: 60,
                vel: 90
            })
        );
        assert_eq!(
            parse(&[0x93, 60, 0]),
            Some(Msg::NoteOff { ch: 3, note: 60 })
        );
        assert_eq!(
            parse(&[0x80, 60, 64]),
            Some(Msg::NoteOff { ch: 0, note: 60 })
        );
        assert_eq!(
            parse(&[0xe0, 0, 64]),
            Some(Msg::PitchBend { ch: 0, value: 0 })
        );
        assert_eq!(parse(&[0xb0, 7]), None);
        assert_eq!(parse(&[]), None);
        assert_eq!(parse(&[0xf8]), None);
    }

    #[test]
    fn cc_lands_on_the_bus_and_learn_assigns_the_source() {
        let mut state = RackState::factory();
        let mut bus = Bus::new();
        let mut m = Midi::default();
        let rid = state.routes[0].id.clone();
        m.start_learn(&rid);
        assert!(m.handle(&mut state, &mut bus, &[0xb0, 21, 127]));
        assert_eq!(bus.num("midi.cc.21", 0.0), 1.0);
        assert_eq!(state.routes[0].from, "midi:21");
        assert_eq!(state.routes[0].amt, 1.0);
        assert!(m.learning.is_none());
        // a second move no longer reassigns
        m.handle(&mut state, &mut bus, &[0xb0, 22, 10]);
        assert_eq!(state.routes[0].from, "midi:21");
        assert_eq!(m.seen, vec![21, 22]);
        // notes change nothing
        assert!(!m.handle(&mut state, &mut bus, &[0x90, 60, 100]));
    }

    #[test]
    fn a_learned_route_drives_its_destination() {
        let mut state = RackState::factory();
        state.routes.clear();
        let rid = state.add_route("d1:out:2", "d0:in:3", 0.5);
        let mut bus = Bus::new();
        let mut m = Midi::default();
        m.start_learn(&rid);
        m.handle(&mut state, &mut bus, &[0xb0, 1, 127]);
        crate::model::run_routes(&state, &mut bus, 0.0);
        // full-amount MIDI routes jump straight to the controller value
        assert_eq!(bus.num("quill.tension", 0.0), 1.0);
    }
}
