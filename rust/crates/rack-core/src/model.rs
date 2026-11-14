//! The rack: instruments (ordered groups of modules + macros), devices
//! (instances of an instrument with their own parameter prefix), the routing
//! matrix and the output frame the adapters consume.
//!
//! Hierarchy: node → module → instrument → rack device. A device parameter
//! lives at `deviceId.moduleUid.key`; shared engine modules use global keys.

use crate::bus::Bus;
use crate::catalog::{self, ModuleDef};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModuleRef {
    pub uid: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MacroRef {
    #[serde(rename = "mod")]
    pub module: String,
    pub key: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Instrument {
    pub id: String,
    pub name: String,
    pub dept: usize,
    pub modules: Vec<ModuleRef>,
    pub macros: Vec<MacroRef>,
    #[serde(default = "graphite")]
    pub skin: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faceplate: Option<String>,
}

fn graphite() -> String {
    "graphite".into()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FxSlot {
    #[serde(rename = "type")]
    pub kind: String,
    pub on: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub inst: String,
    pub open: bool,
    pub fx: Vec<FxSlot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Route {
    pub id: String,
    pub from: String,
    pub to: String,
    /// Bipolar amount, −1..1. Negative inverts the source.
    pub amt: f32,
    pub on: bool,
}

pub const MAX_FX: usize = 4;
pub const MAX_MACROS: usize = 8;

/// A resolved parameter of a device.
#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub key: &'static str,
    pub label: &'static str,
    /// Full bus address.
    pub p: String,
    pub module: String,
    pub kind: String,
}

pub fn mod_prefix(dev_id: &str, uid: &str) -> String {
    format!("{dev_id}.{uid}")
}

pub fn module_params(def: &ModuleDef, prefix: &str) -> Vec<(&'static str, &'static str, String)> {
    def.params
        .iter()
        .map(|(k, l)| {
            (
                *k,
                *l,
                if def.shared {
                    (*k).to_owned()
                } else {
                    format!("{prefix}.{k}")
                },
            )
        })
        .collect()
}

impl Instrument {
    pub fn blank(dept: usize) -> Self {
        Self {
            id: String::new(),
            name: "UNTITLED INSTRUMENT".into(),
            dept,
            modules: vec![],
            macros: vec![],
            skin: graphite(),
            faceplate: None,
        }
    }

    pub fn params(&self, dev_id: &str) -> Vec<Param> {
        self.modules
            .iter()
            .filter_map(|m| catalog::module(&m.kind).map(|d| (m, d)))
            .flat_map(|(m, d)| {
                module_params(d, &mod_prefix(dev_id, &m.uid))
                    .into_iter()
                    .map(|(key, label, p)| Param {
                        key,
                        label,
                        p,
                        module: m.uid.clone(),
                        kind: m.kind.clone(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn macro_params(&self, dev_id: &str) -> Vec<Param> {
        let all = self.params(dev_id);
        self.macros
            .iter()
            .filter_map(|mc| {
                all.iter()
                    .find(|p| p.module == mc.module && p.key == mc.key)
                    .cloned()
            })
            .collect()
    }

    pub fn hp(&self) -> u32 {
        self.modules
            .iter()
            .filter_map(|m| catalog::module(&m.kind))
            .map(|d| d.hp)
            .sum()
    }

    pub fn module_names(&self) -> String {
        self.modules
            .iter()
            .map(|m| catalog::module(&m.kind).map(|d| d.name).unwrap_or("?"))
            .collect::<Vec<_>>()
            .join(" + ")
    }

    pub fn slug(&self) -> String {
        let mut s = String::new();
        let mut gap = false;
        for c in self.name.to_lowercase().chars() {
            if c.is_ascii_alphanumeric() {
                if gap && !s.is_empty() {
                    s.push('_');
                }
                gap = false;
                s.push(c);
            } else {
                gap = true;
            }
        }
        s
    }
}

/// Monotonic id source (`d1a`, `m7`, …); the counter is persisted with the rack.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ids(pub u64);

impl Ids {
    pub fn next(&mut self, prefix: &str) -> String {
        self.0 += 1;
        format!("{prefix}{}", radix36(self.0))
    }
}

fn radix36(mut n: u64) -> String {
    const D: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = vec![];
    loop {
        out.push(D[(n % 36) as usize]);
        n /= 36;
        if n == 0 {
            break;
        }
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RackState {
    pub lib: Vec<Instrument>,
    pub rack: Vec<Device>,
    pub routes: Vec<Route>,
    pub ids: Ids,
}

impl Default for RackState {
    fn default() -> Self {
        Self::factory()
    }
}

impl RackState {
    /// The rack a new vault opens with.
    pub fn factory() -> Self {
        let ids = Ids(1000);
        let lib = presets();
        let dev = |id: &str, inst: &str, open: bool, fx: &[(&str, bool)]| Device {
            id: id.into(),
            inst: inst.into(),
            open,
            fx: fx
                .iter()
                .map(|(k, on)| FxSlot {
                    kind: (*k).into(),
                    on: *on,
                })
                .collect(),
        };
        let rack = vec![
            dev("dmain", "i-main", true, &[]),
            dev("d0", "i-story", false, &[("echo", true), ("comp", true)]),
            dev("deido", "i-eidolon", true, &[]),
            dev("dscript", "i-script", true, &[]),
            dev(
                "d1",
                "i-tornado",
                true,
                &[("plague", true), ("drift", true), ("quant", false)],
            ),
            dev("d2", "i-lights", false, &[("drift", true)]),
            dev("d3", "i-master", false, &[]),
            dev("dherald", "i-herald", true, &[]),
            dev("dhearth", "i-hearth", true, &[]),
            dev("dnexus", "i-nexus", true, &[]),
        ];
        let route = |id: &str, from: &str, to: &str, amt: f32| Route {
            id: id.into(),
            from: from.into(),
            to: to.into(),
            amt,
            on: true,
        };
        let routes = vec![
            route("r0", "d1:out:lfo", "d2:in:0", 0.6),
            route("r1", "d0:out:gate", "d2:in:2", 0.8),
            route("r2", "d1:out:2", "d0:in:3", -0.3),
        ];
        Self {
            lib,
            rack,
            routes,
            ids,
        }
    }

    /// Add any factory preset missing from a loaded library (new presets ship
    /// with new builds; user instruments are never touched).
    pub fn merge_presets(&mut self) {
        for p in presets() {
            if !self.lib.iter().any(|i| i.id == p.id) {
                self.lib.push(p);
            }
        }
    }

    pub fn inst(&self, id: &str) -> Option<&Instrument> {
        self.lib.iter().find(|i| i.id == id)
    }

    pub fn device(&self, id: &str) -> Option<&Device> {
        self.rack.iter().find(|d| d.id == id)
    }

    pub fn device_mut(&mut self, id: &str) -> Option<&mut Device> {
        self.rack.iter_mut().find(|d| d.id == id)
    }

    /// Insert a new device of `inst_id` at `at`; returns its id.
    pub fn add_device(&mut self, inst_id: &str, at: usize) -> Option<String> {
        self.inst(inst_id)?;
        let id = self.ids.next("d");
        let at = at.min(self.rack.len());
        self.rack.insert(
            at,
            Device {
                id: id.clone(),
                inst: inst_id.into(),
                open: false,
                fx: vec![],
            },
        );
        Some(id)
    }

    /// Move a device so it lands at gap index `at` (0 = before the first device).
    pub fn move_device(&mut self, dev_id: &str, at: usize) {
        let Some(from) = self.rack.iter().position(|d| d.id == dev_id) else {
            return;
        };
        let d = self.rack.remove(from);
        let to = if from < at { at - 1 } else { at };
        self.rack.insert(to.min(self.rack.len()), d);
    }

    pub fn remove_device(&mut self, dev_id: &str) {
        self.rack.retain(|d| d.id != dev_id);
        let pre = format!("{dev_id}:");
        self.routes
            .retain(|r| !r.from.starts_with(&pre) && !r.to.starts_with(&pre));
    }

    pub fn add_fx(&mut self, dev_id: &str, fx: &str) -> bool {
        if catalog::fx(fx).is_none() {
            return false;
        }
        match self.device_mut(dev_id) {
            Some(d) if d.fx.len() < MAX_FX => {
                d.fx.push(FxSlot {
                    kind: fx.into(),
                    on: true,
                });
                true
            }
            _ => false,
        }
    }

    /// Save a draft into the library (new id if it has none); returns the id.
    pub fn save_instrument(&mut self, mut inst: Instrument) -> String {
        if inst.id.is_empty() {
            inst.id = self.ids.next("i");
        }
        let id = inst.id.clone();
        match self.lib.iter_mut().find(|i| i.id == id) {
            Some(slot) => *slot = inst,
            None => self.lib.push(inst),
        }
        id
    }

    pub fn delete_instrument(&mut self, inst_id: &str) {
        let gone: Vec<String> = self
            .rack
            .iter()
            .filter(|d| d.inst == inst_id)
            .map(|d| d.id.clone())
            .collect();
        for d in gone {
            self.remove_device(&d);
        }
        self.lib.retain(|i| i.id != inst_id);
    }

    pub fn add_route(&mut self, from: &str, to: &str, amt: f32) -> String {
        let id = self.ids.next("r");
        self.routes.push(Route {
            id: id.clone(),
            from: from.into(),
            to: to.into(),
            amt,
            on: true,
        });
        id
    }

    /// Connect or disconnect a matrix cell.
    pub fn toggle_route(&mut self, from: &str, to: &str) {
        match self
            .routes
            .iter()
            .position(|r| r.from == from && r.to == to)
        {
            Some(i) => {
                self.routes.remove(i);
            }
            None => {
                self.add_route(from, to, 0.5);
            }
        }
    }
}

fn presets() -> Vec<Instrument> {
    catalog::PRESETS
        .iter()
        .map(|(id, name, dept, types, picks)| {
            // Preset module uids are derived from the preset id so they are
            // stable across launches and saved routes keep pointing at them.
            let modules: Vec<ModuleRef> = types
                .iter()
                .enumerate()
                .map(|(i, t)| ModuleRef {
                    uid: format!("{}m{i}", &id[2..]),
                    kind: (*t).into(),
                })
                .collect();
            let macros = picks
                .iter()
                .map(|(i, key)| MacroRef {
                    module: modules[*i].uid.clone(),
                    key: (*key).into(),
                })
                .collect();
            Instrument {
                id: (*id).into(),
                name: (*name).into(),
                dept: *dept,
                modules,
                macros,
                skin: catalog::default_skin(id).into(),
                faceplate: None,
            }
        })
        .collect()
}

// ---- routing ---------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum PortSource {
    /// A bus address.
    Param(String),
    /// The chronicle gate pattern at the current step.
    Gate,
    /// The shared LFO.
    Lfo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    pub id: String,
    pub label: String,
    pub source: PortSource,
    /// Owning device / group name, for menus.
    pub dev: String,
    pub ch: &'static str,
    /// Deck controls are routable but not shown in the grid overview.
    pub deck: bool,
}

pub const DECK_LETTERS: &str = "ABCDEFGHIJKLMNOP";
pub const DECK_DESTS: &[(&str, &str)] = &[
    ("vol", "MASS (FADER)"),
    ("gain", "GAIN"),
    ("eqm", "GRAVITY (LOW)"),
    ("eqw", "CAUSALITY (MID)"),
    ("eqs", "EXPOSITION (HIGH)"),
    ("filt", "DISSONANCE (FILTER)"),
    ("fx0", "FX 1"),
    ("fx1", "FX 2"),
    ("fx2", "FX 3"),
];

pub struct Ports {
    pub src: Vec<Port>,
    pub dst: Vec<Port>,
}

impl Ports {
    pub fn src(&self, id: &str) -> Option<&Port> {
        self.src.iter().find(|p| p.id == id)
    }
    pub fn dst(&self, id: &str) -> Option<&Port> {
        self.dst.iter().find(|p| p.id == id)
    }
}

pub fn all_ports(state: &RackState) -> Ports {
    let (mut src, mut dst) = (vec![], vec![]);
    for d in &state.rack {
        let Some(inst) = state.inst(&d.inst) else {
            continue;
        };
        let ch = catalog::dept(inst.dept).ch;
        let macros = inst.macro_params(&d.id);
        for (i, m) in macros.iter().enumerate() {
            dst.push(Port {
                id: format!("{}:in:{i}", d.id),
                label: m.label.into(),
                source: PortSource::Param(m.p.clone()),
                dev: inst.name.clone(),
                ch,
                deck: false,
            });
            src.push(Port {
                id: format!("{}:out:{i}", d.id),
                label: m.label.into(),
                source: PortSource::Param(m.p.clone()),
                dev: inst.name.clone(),
                ch,
                deck: false,
            });
        }
        if inst
            .modules
            .iter()
            .any(|m| m.kind == "chronicle" || m.kind == "seq")
        {
            src.push(Port {
                id: format!("{}:out:gate", d.id),
                label: "GATE".into(),
                source: PortSource::Gate,
                dev: inst.name.clone(),
                ch,
                deck: false,
            });
        }
        src.push(Port {
            id: format!("{}:out:lfo", d.id),
            label: "LFO".into(),
            source: PortSource::Lfo,
            dev: inst.name.clone(),
            ch,
            deck: false,
        });
    }
    src.push(Port {
        id: "decks:out:mix".into(),
        label: "MIX INTENSITY".into(),
        source: PortSource::Param("mix.intensity".into()),
        dev: "DECKS".into(),
        ch: "myth",
        deck: true,
    });
    for (i, l) in DECK_LETTERS.chars().enumerate() {
        let dev = format!("DECK {l}");
        let ch = ["cool", "myth", "amber", "bone"][i / 4];
        src.push(Port {
            id: format!("deck{l}:out:lvl"),
            label: "LEVEL OUT".into(),
            source: PortSource::Param(format!("deck.{l}.out")),
            dev: dev.clone(),
            ch,
            deck: true,
        });
        for (k, label) in DECK_DESTS {
            dst.push(Port {
                id: format!("deck{l}:in:{k}"),
                label: (*label).into(),
                source: PortSource::Param(format!("deck.{l}.{k}")),
                dev: dev.clone(),
                ch,
                deck: true,
            });
        }
    }
    dst.push(Port {
        id: "decks:in:xf".into(),
        label: "CROSSFADER".into(),
        source: PortSource::Param("deck.xf".into()),
        dev: "DECKS".into(),
        ch: "myth",
        deck: true,
    });
    dst.push(Port {
        id: "decks:in:phys".into(),
        label: "PHYSICS OF MYTH".into(),
        source: PortSource::Param("myth.physics".into()),
        dev: "DECKS".into(),
        ch: "myth",
        deck: true,
    });
    Ports { src, dst }
}

/// Current value of a route source. `t_ms` is engine time in milliseconds.
pub fn port_value(bus: &Bus, from: &str, port: Option<&Port>, t_ms: f64) -> f32 {
    if let Some(cc) = from.strip_prefix("midi:") {
        return bus.num(&format!("midi.cc.{cc}"), 0.0);
    }
    match port.map(|p| &p.source) {
        None => 0.0,
        Some(PortSource::Gate) => {
            let s = bus.num("transport.step", 0.0) as usize;
            match bus.steps("chr.a") {
                Some(a) if a.get(s).copied().unwrap_or(false) => 1.0,
                _ => 0.0,
            }
        }
        Some(PortSource::Lfo) => {
            0.5 + 0.5
                * (t_ms * 0.001 * (0.4 + bus.num("inst.rate", 0.34) as f64 * 6.0)).sin() as f32
        }
        Some(PortSource::Param(p)) => bus.num(p, 0.5),
    }
}

/// Run every enabled route once. Called at 20 Hz regardless of which view is
/// showing; a route pulls its destination toward the source by `|amt|`.
pub fn run_routes(state: &RackState, bus: &mut Bus, t_ms: f64) {
    if state.routes.is_empty() {
        return;
    }
    let ports = all_ports(state);
    for r in &state.routes {
        if !r.on || r.amt == 0.0 {
            continue;
        }
        let Some(d) = ports.dst(&r.to) else { continue };
        let PortSource::Param(dest) = &d.source else {
            continue;
        };
        let midi = r.from.starts_with("midi:");
        let s = ports.src(&r.from);
        if !midi && s.is_none() {
            continue;
        }
        if !midi && !r.from.starts_with("deck") {
            let s_dev = r.from.split(':').next().unwrap_or("");
            if !bus.flag(&format!("dev.{s_dev}.on"), true)
                || bus.flag(&format!("dev.{s_dev}.mute"), false)
            {
                continue;
            }
        }
        let cur = match bus.get(dest) {
            None => 0.5,
            Some(v) => match v.as_num() {
                Some(n) => n,
                None => continue,
            },
        };
        let mut v = port_value(bus, &r.from, s, t_ms);
        if r.amt < 0.0 {
            v = 1.0 - v;
        }
        let k = r.amt.abs();
        let nv = if midi && k >= 0.99 {
            v
        } else {
            cur + (v - cur) * k * 0.35
        };
        if (nv - cur).abs() > 0.002 {
            bus.set_num(dest, nv.clamp(0.0, 1.0));
        }
    }
}

// ---- headless core output --------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct FrameRow {
    /// OSC-style address, e.g. `/tornado_synth/vort`.
    pub addr: String,
    pub v: f32,
    pub ch: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CoreFrame {
    pub rows: Vec<FrameRow>,
    /// First eight rows as DMX levels.
    pub dmx: Vec<u8>,
}

/// Build the frame the output adapters see: one row per macro of every active,
/// audible device, after its insert effects and level.
pub fn core_frame(state: &RackState, bus: &Bus, t_ms: f64) -> CoreFrame {
    let any_solo = state
        .rack
        .iter()
        .any(|d| bus.flag(&format!("dev.{}.solo", d.id), false));
    let mut rows = vec![];
    for dev in &state.rack {
        let Some(inst) = state.inst(&dev.inst) else {
            continue;
        };
        if !bus.flag(&format!("dev.{}.on", dev.id), true) {
            continue;
        }
        if bus.flag(&format!("dev.{}.mute", dev.id), false)
            || (any_solo && !bus.flag(&format!("dev.{}.solo", dev.id), false))
        {
            continue;
        }
        let lvl = bus.num(&format!("dev.{}.level", dev.id), 0.8) / 0.8;
        let slug = inst.slug();
        for m in inst.macro_params(&dev.id) {
            let mut v = bus.num(&m.p, 0.5);
            for (i, f) in dev.fx.iter().enumerate() {
                if f.on && catalog::fx(&f.kind).is_some() {
                    let amt = bus.num(&format!("{}.fx{i}.amt", dev.id), 1.0);
                    v += (catalog::fx_apply(&f.kind, v, t_ms) - v) * amt;
                }
            }
            rows.push(FrameRow {
                addr: format!("/{slug}/{}", m.label.to_lowercase()),
                v: (v * lvl).clamp(0.0, 1.0),
                ch: catalog::dept(inst.dept).ch,
            });
        }
    }
    let dmx = rows
        .iter()
        .take(8)
        .map(|r| (r.v * 255.0).round() as u8)
        .collect();
    CoreFrame { rows, dmx }
}

// ---- unfolded device layout --------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    Module {
        uid: String,
        kind: String,
        span: u32,
    },
    /// Leftover row space becomes a VIEW section.
    Fill { span: u32, slot: usize },
}

impl Cell {
    pub fn span(&self) -> u32 {
        match self {
            Cell::Module { span, .. } | Cell::Fill { span, .. } => *span,
        }
    }
}

/// Pack sections into 12-column rows. Leftover space of three or more columns
/// becomes a VIEW section; one or two columns stretch the last section.
pub fn hx_rows(mods: &[ModuleRef]) -> Vec<Vec<Cell>> {
    let mut out: Vec<Vec<Cell>> = vec![];
    let mut row: Vec<Cell> = vec![];
    let mut used = 0;
    let mut slot = 0;
    let close =
        |row: &mut Vec<Cell>, used: &mut u32, out: &mut Vec<Vec<Cell>>, slot: &mut usize| {
            let left = 12 - *used;
            if !row.is_empty() && left > 0 {
                if left >= 3 {
                    row.push(Cell::Fill {
                        span: left,
                        slot: *slot + row.len(),
                    });
                } else if let Some(Cell::Module { span, .. }) = row.last_mut() {
                    *span += left;
                }
            }
            *slot += row.len();
            out.push(std::mem::take(row));
            *used = 0;
        };
    for m in mods {
        let Some(def) = catalog::module(&m.kind) else {
            continue;
        };
        let sp = def.span.min(12);
        if used + sp > 12 {
            close(&mut row, &mut used, &mut out, &mut slot);
        }
        row.push(Cell::Module {
            uid: m.uid.clone(),
            kind: m.kind.clone(),
            span: sp,
        });
        used += sp;
    }
    if !row.is_empty() {
        close(&mut row, &mut used, &mut out, &mut slot);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_rack_is_consistent() {
        let s = RackState::factory();
        assert_eq!(s.lib.len(), catalog::PRESETS.len());
        for d in &s.rack {
            assert!(s.inst(&d.inst).is_some(), "{} → {}", d.id, d.inst);
        }
        let ports = all_ports(&s);
        for r in &s.routes {
            assert!(ports.src(&r.from).is_some(), "missing source {}", r.from);
            assert!(ports.dst(&r.to).is_some(), "missing destination {}", r.to);
        }
    }

    #[test]
    fn shared_modules_use_global_addresses() {
        let s = RackState::factory();
        let story = s.inst("i-story").unwrap();
        let m = story.macro_params("d0");
        assert_eq!(m[0].p, "chr.bpm");
        let lights = s.inst("i-lights").unwrap();
        let m = lights.macro_params("d2");
        assert_eq!(m[0].p, "d2.lightsm0.c1");
        assert_eq!(m[4].p, "d2.lightsm1.xy.x");
    }

    #[test]
    fn rows_pack_to_twelve_columns() {
        let s = RackState::factory();
        for inst in &s.lib {
            for row in hx_rows(&inst.modules) {
                assert_eq!(row.iter().map(Cell::span).sum::<u32>(), 12, "{}", inst.name);
            }
        }
        // chronicle (8) + quill (6) wraps: 8 + fill 4, then 6 + fill 6.
        let rows = hx_rows(&s.inst("i-story").unwrap().modules);
        assert_eq!(rows.len(), 2);
        assert!(matches!(rows[0][1], Cell::Fill { span: 4, .. }));
    }

    #[test]
    fn move_and_remove_devices() {
        let mut s = RackState::factory();
        let first = s.rack[0].id.clone();
        s.move_device(&first, 3);
        assert_eq!(s.rack[2].id, first);
        s.remove_device("d1");
        assert!(s.device("d1").is_none());
        assert!(s
            .routes
            .iter()
            .all(|r| !r.from.starts_with("d1:") && !r.to.starts_with("d1:")));
    }

    #[test]
    fn fx_slots_cap_at_four() {
        let mut s = RackState::factory();
        for _ in 0..6 {
            s.add_fx("d3", "echo");
        }
        assert_eq!(s.device("d3").unwrap().fx.len(), MAX_FX);
        assert!(!s.add_fx("d3", "nope"));
    }

    #[test]
    fn routes_pull_destination_toward_source() {
        let mut s = RackState::factory();
        s.routes.clear();
        // tornado macro 2 (cont.vort) → story macro 3 (quill.tension)
        s.add_route("d1:out:2", "d0:in:3", 1.0);
        let mut bus = Bus::new();
        bus.set_num("cont.vort", 1.0);
        bus.set_num("quill.tension", 0.0);
        for _ in 0..60 {
            run_routes(&s, &mut bus, 0.0);
        }
        assert!(bus.num("quill.tension", 0.0) > 0.95);
        // muting the source device stops the route
        bus.set_num("quill.tension", 0.0);
        bus.set_flag("dev.d1.mute", true);
        run_routes(&s, &mut bus, 0.0);
        assert_eq!(bus.num("quill.tension", 1.0), 0.0);
    }

    #[test]
    fn negative_amount_inverts() {
        let mut s = RackState::factory();
        s.routes.clear();
        s.add_route("d1:out:2", "d0:in:3", -1.0);
        let mut bus = Bus::new();
        bus.set_num("cont.vort", 1.0);
        bus.set_num("quill.tension", 1.0);
        for _ in 0..80 {
            run_routes(&s, &mut bus, 0.0);
        }
        assert!(bus.num("quill.tension", 1.0) < 0.05);
    }

    #[test]
    fn core_frame_respects_mute_solo_and_fx() {
        let s = RackState::factory();
        let mut bus = Bus::new();
        let all = core_frame(&s, &bus, 0.0);
        assert!(all.rows.iter().any(|r| r.addr == "/story_clock/tempo"));
        assert_eq!(all.dmx.len(), 8);

        bus.set_flag("dev.d0.solo", true);
        let solo = core_frame(&s, &bus, 0.0);
        assert!(solo
            .rows
            .iter()
            .all(|r| r.addr.starts_with("/story_clock/")));
        // d0 has a compressor: 0.5 stays 0.35 + 0 = 0.35 at full amount.
        assert!((solo.rows[0].v - 0.35).abs() < 1e-4, "{}", solo.rows[0].v);

        bus.set_flag("dev.d0.mute", true);
        assert!(core_frame(&s, &bus, 0.0).rows.is_empty());
    }

    #[test]
    fn state_round_trips_through_json() {
        let s = RackState::factory();
        let json = serde_json::to_string(&s).unwrap();
        let back: RackState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn ids_are_base36_and_unique() {
        let mut ids = Ids(34);
        assert_eq!(ids.next("d"), "dz");
        assert_eq!(ids.next("d"), "d10");
    }
}
