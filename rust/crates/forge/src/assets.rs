//! The asset library: every kit control and screen as a live, addressable
//! component. The Library stage shows them, the Panel stage places them on a
//! faceplate, and the rack draws a saved faceplate with the same code.

use crate::util;
use egui::{pos2, vec2, Align2, Color32, Id, Rect, Sense, Ui, Vec2};
use instrument_kit::paint::*;
use instrument_kit::theme::*;
use instrument_kit::viz::{self, FieldParams, RimScreen};
use instrument_kit::widgets::{self, Face, Knob, KnobVariant};
use rack_core::bus::{Bus, Value};
use rack_core::catalog::DEPTS;
use rack_core::engine::{Engine, DEFAULT_ENV};
use rack_core::panel::{Faceplate, Pane};

pub struct Asset {
    pub key: &'static str,
    pub label: &'static str,
    pub family: &'static str,
    pub ch: &'static str,
    /// Footprint on a faceplate, in points.
    pub size: [f32; 2],
}

pub const FAMILIES: [&str; 7] = [
    "ROTARIES",
    "FADERS & STRIPS",
    "BUTTONS & PADS",
    "METERS & READOUTS",
    "SCREENS · SIGNAL",
    "SCREENS · ALGORITHMIC",
    "GLYPHS & CRESTS",
];

macro_rules! a {
    ($k:literal, $l:literal, $f:literal, $c:literal, $w:literal, $h:literal) => {
        Asset {
            key: $k,
            label: $l,
            family: $f,
            ch: $c,
            size: [$w, $h],
        }
    };
}

pub const ASSETS: &[Asset] = &[
    a!(
        "knob_classic",
        "KNOB · CLASSIC",
        "ROTARIES",
        "cool",
        76.0,
        96.0
    ),
    a!("knob", "KNOB · ARC", "ROTARIES", "cool", 76.0, 96.0),
    a!(
        "knob_dotted",
        "KNOB · DOTTED",
        "ROTARIES",
        "cool",
        76.0,
        96.0
    ),
    a!("knob_forge", "KNOB · FORGE", "ROTARIES", "cool", 76.0, 96.0),
    a!(
        "knob_ringed",
        "KNOB · RINGED",
        "ROTARIES",
        "cool",
        76.0,
        96.0
    ),
    a!("knob_pip", "KNOB · PIP", "ROTARIES", "cool", 76.0, 96.0),
    a!("mini_knob", "MINI KNOB", "ROTARIES", "amber", 44.0, 50.0),
    a!(
        "rim_life",
        "RIM KNOB · AUTOMATA",
        "ROTARIES",
        "life",
        96.0,
        112.0
    ),
    a!(
        "rim_mandel",
        "RIM KNOB · FRACTAL",
        "ROTARIES",
        "cool",
        96.0,
        112.0
    ),
    a!("fader", "FADER", "FADERS & STRIPS", "myth", 48.0, 124.0),
    a!(
        "hslider",
        "H-SLIDER",
        "FADERS & STRIPS",
        "cool",
        130.0,
        16.0
    ),
    a!(
        "amount",
        "BIPOLAR AMOUNT",
        "FADERS & STRIPS",
        "amber",
        130.0,
        20.0
    ),
    a!("pad", "KEY PAD", "BUTTONS & PADS", "rose", 40.0, 40.0),
    a!(
        "gate",
        "PUSH BTN · GATE",
        "BUTTONS & PADS",
        "life",
        78.0,
        26.0
    ),
    a!(
        "toggle",
        "TOGGLE · LATCH",
        "BUTTONS & PADS",
        "amber",
        50.0,
        16.0
    ),
    a!(
        "switch3",
        "LEVER SWITCH",
        "BUTTONS & PADS",
        "forge",
        62.0,
        56.0
    ),
    a!("steps", "STEP ROW", "BUTTONS & PADS", "amber", 205.0, 22.0),
    a!("xy", "XY PAD", "BUTTONS & PADS", "myth", 96.0, 96.0),
    a!("led", "LED", "METERS & READOUTS", "life", 12.0, 12.0),
    a!("vu", "LED METER", "METERS & READOUTS", "life", 150.0, 10.0),
    a!(
        "meter16",
        "LEVEL METER",
        "METERS & READOUTS",
        "cool",
        150.0,
        8.0
    ),
    a!(
        "lcd",
        "LCD READOUT",
        "METERS & READOUTS",
        "cool",
        80.0,
        36.0
    ),
    a!("label", "LABEL", "METERS & READOUTS", "bone", 90.0, 14.0),
    a!("scope", "SCOPE", "SCREENS · SIGNAL", "life", 200.0, 84.0),
    a!(
        "spectrum",
        "SPECTRUM",
        "SCREENS · SIGNAL",
        "warm",
        200.0,
        84.0
    ),
    a!(
        "sgram",
        "SPECTROGRAM",
        "SCREENS · SIGNAL",
        "bone",
        200.0,
        84.0
    ),
    a!("wave", "HISTORY", "SCREENS · SIGNAL", "rose", 200.0, 60.0),
    a!(
        "phase",
        "PHASE SCOPE",
        "SCREENS · SIGNAL",
        "rose",
        110.0,
        110.0
    ),
    a!(
        "roll",
        "PIANO ROLL",
        "SCREENS · SIGNAL",
        "myth",
        200.0,
        84.0
    ),
    a!(
        "flow",
        "PARTICLE FIELD",
        "SCREENS · SIGNAL",
        "cool",
        200.0,
        84.0
    ),
    a!("polar", "RADAR", "SCREENS · SIGNAL", "life", 110.0, 110.0),
    a!(
        "envelope",
        "ENVELOPE",
        "SCREENS · SIGNAL",
        "amber",
        200.0,
        90.0
    ),
    a!(
        "circuit",
        "CIRCUIT",
        "SCREENS · ALGORITHMIC",
        "cool",
        200.0,
        84.0
    ),
    a!(
        "network",
        "NETWORK",
        "SCREENS · ALGORITHMIC",
        "myth",
        200.0,
        84.0
    ),
    a!(
        "crest",
        "CREST TILE",
        "GLYPHS & CRESTS",
        "forge",
        56.0,
        56.0
    ),
    a!("glyph", "GLYPH TILE", "GLYPHS & CRESTS", "cool", 44.0, 44.0),
];

pub fn asset(key: &str) -> Option<&'static Asset> {
    ASSETS.iter().find(|a| a.key == key)
}

pub fn hash(s: &str) -> u32 {
    s.chars()
        .fold(7u32, |h, c| h.wrapping_mul(31).wrapping_add(c as u32))
}

/// Draw asset `key` centred in `rect`, bound to bus address `addr`.
/// `live` = false paints it without taking input (edit mode).
pub fn draw(
    ui: &mut Ui,
    bus: &mut Bus,
    engine: &mut Engine,
    key: &str,
    addr: &str,
    rect: Rect,
    label: &str,
    ch: Channel,
    live: bool,
) {
    let Some(a) = asset(key) else { return };
    let size = vec2(a.size[0], a.size[1]);
    let r = Rect::from_center_size(rect.center(), size);
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(Id::unique(("asset", addr)))
            .max_rect(r)
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );
    if !live {
        child.disable();
        child.set_opacity(1.0);
    }
    let ui = &mut child;
    let id = Id::unique(("assetv", addr));
    let lab = if label.is_empty() { None } else { Some(label) };
    let knob = |ui: &mut Ui, bus: &mut Bus, v: KnobVariant, s: f32, value: bool| {
        util::knob(
            ui,
            bus,
            addr,
            Knob {
                size: s,
                ch,
                variant: v,
                label: lab,
                label_size: 8.0,
                show_value: value,
                ..Default::default()
            },
        );
    };
    match key {
        "knob_classic" => knob(ui, bus, KnobVariant::Classic, 52.0, true),
        "knob" => knob(ui, bus, KnobVariant::Arc, 52.0, true),
        "knob_dotted" => knob(ui, bus, KnobVariant::Dotted, 52.0, true),
        "knob_forge" => knob(ui, bus, KnobVariant::Forge, 52.0, true),
        "knob_ringed" => knob(ui, bus, KnobVariant::Ringed, 52.0, true),
        "knob_pip" => knob(ui, bus, KnobVariant::Pip, 52.0, true),
        "mini_knob" => knob(ui, bus, KnobVariant::Arc, 30.0, false),
        "rim_life" | "rim_mandel" => {
            let mut v = bus.num(addr, 0.5);
            let (hot, kind) = if key == "rim_life" {
                (true, RimScreen::Life)
            } else {
                (false, RimScreen::Mandel)
            };
            if viz::rim_knob(
                ui,
                id,
                &mut v,
                96.0,
                hot,
                if label.is_empty() { "RECURSION" } else { label },
                kind,
                0.5,
            )
            .changed()
            {
                bus.set_num(addr, v);
            }
        }
        "fader" => {
            util::fader(ui, bus, addr, 0.6, 108.0, 20.0, ch, lab, true);
        }
        "hslider" => {
            let mut v = bus.num(addr, 0.6);
            if widgets::hfader(ui, &mut v, 130.0, ch.color(), 0.6).changed() {
                bus.set_num(addr, v);
            }
        }
        "amount" => {
            let mut v = bus.num(addr, 0.5) * 2.0 - 1.0;
            if widgets::amount(ui, &mut v, 130.0, ch.color()).changed() {
                bus.set_num(addr, (v + 1.0) / 2.0);
            }
        }
        "pad" => {
            let lit = bus.flag(addr, false);
            if widgets::pad(
                ui,
                Face::Text(if label.is_empty() { "PAD" } else { label }),
                ch,
                lit,
                40.0,
            )
            .clicked()
            {
                bus.set_flag(addr, !lit);
            }
        }
        "gate" => {
            util::gate(
                ui,
                bus,
                addr,
                if label.is_empty() { "GATE" } else { label },
                ch,
                false,
            );
        }
        "toggle" => {
            let on = bus.flag(addr, false);
            if widgets::latch(
                ui,
                if label.is_empty() { "ON" } else { label },
                on,
                ch.color(),
            )
            .clicked()
            {
                bus.set_flag(addr, !on);
            }
        }
        "switch3" => {
            let mut v = (bus.num(addr, 1.0) * 2.0).round() as usize;
            if widgets::slide_switch(ui, &mut v, &["A", "B", "C"], ch).changed() {
                bus.set_num(addr, v as f32 / 2.0);
            }
        }
        "steps" => {
            let mut pat: Vec<bool> = bus
                .steps(addr)
                .map(|s| s.to_vec())
                .unwrap_or_else(|| (0..8).map(|i| (hash(addr) >> i) & 1 == 1).collect());
            pat.resize(8, false);
            let cur = (bus.num("transport.step", 0.0) as usize) / 2;
            if widgets::step_row(ui, &mut pat, Some(cur), ch, 22.0) {
                bus.set(addr, Value::Steps(pat));
            }
        }
        "xy" => {
            util::xy(ui, bus, addr, 96.0, ch, None);
        }
        "led" => {
            widgets::led(
                ui,
                (bus.num("transport.step", 0.0) as usize).is_multiple_of(4),
                ch,
                8.0,
            );
        }
        "vu" => {
            widgets::vu(ui, engine.rms.r, 150.0, 10.0, None);
        }
        "meter16" => {
            let (mr, _) = ui.allocate_exact_size(vec2(150.0, 8.0), Sense::hover());
            widgets::meter16(ui.painter(), mr, (engine.level * 16.0) as usize, ch.color());
        }
        "lcd" => {
            widgets::readout_box(
                ui,
                Some(if label.is_empty() { "BPM" } else { label }),
                &format!("{:.1}", Engine::bpm_exact(bus)),
                ch,
                80.0,
            );
        }
        "label" => {
            let (lr, _) = ui.allocate_exact_size(size, Sense::hover());
            engrave(
                ui.painter(),
                lr.center(),
                Align2::CENTER_CENTER,
                if label.is_empty() { "LABEL" } else { label },
                10.0,
                INK,
            );
        }
        "scope" => {
            let t = engine.time as f32;
            viz::scope(ui, size, ch, label, None, |u| engine.synth_wave(bus, u, t));
        }
        "spectrum" => {
            let b: Vec<f32> = (0..24)
                .map(|i| engine.synth_band(bus, i as f32 / 24.0))
                .collect();
            viz::spectrum(ui, id, size, ch, label, &b);
        }
        "sgram" => {
            let b: Vec<f32> = (0..32)
                .map(|i| engine.synth_band(bus, i as f32 / 32.0))
                .collect();
            viz::spectrogram(ui, id, size, label, &b);
        }
        "wave" => {
            viz::waveform(ui, size, ch, label, &engine.hist, engine.hist_i);
        }
        "phase" => {
            viz::phase_scope(
                ui,
                110.0,
                ch,
                label,
                1.0 + Engine::scale_idx(bus) as f32,
                3.0,
                engine.amp(bus),
                1.0,
                engine.time as f32,
            );
        }
        "roll" => {
            let notes: Vec<(u64, i32, bool)> = engine
                .notes
                .iter()
                .map(|n| (n.abs, n.midi, n.accent))
                .collect();
            viz::piano_roll(ui, size, ch, label, &notes, engine.abs);
        }
        "flow" => {
            viz::particle_field(
                ui,
                id,
                size,
                ch,
                label,
                50,
                FieldParams {
                    level: engine.level,
                    ..Default::default()
                },
            );
        }
        "polar" => {
            viz::polar_screen(ui, id, 110.0, ch, label, 0.6, 0.5, 0.4, true, engine.level);
        }
        "envelope" => {
            let mut pts: Vec<[f32; 2]> = bus
                .curve(addr)
                .map(|c| c.to_vec())
                .unwrap_or_else(|| DEFAULT_ENV.to_vec());
            if viz::curve_editor(ui, id, &mut pts, size, ch, label, None) {
                bus.set(addr, Value::Curve(pts));
            }
        }
        "circuit" => {
            viz::circuit_art(ui, size, hash(addr), ch.color());
        }
        "network" => {
            viz::network_art(ui, size, hash(addr), ch.color(), engine.time as f32);
        }
        "crest" | "glyph" => {
            let d = &DEPTS[hash(addr) as usize % 16];
            let (g, tone) = if key == "crest" {
                (d.crest, util::chc(d.ch))
            } else {
                (
                    ["wave", "gate", "clock", "loop", "vortex", "logic"][hash(addr) as usize % 6],
                    ch.color(),
                )
            };
            widgets::glyph_tile(ui, g, tone, a.size[0], false, false);
        }
        _ => {}
    }
}

// ---- materials ---------------------------------------------------------------

/// `(key, label, family, top, bottom)`
pub const MATERIALS: [(&str, &str, &str, u32, u32); 12] = [
    ("alloy", "BIO ALLOY", "METAL", 0x2c2f5c, 0x1b3948),
    ("graphite", "GRAPHITE", "DARK", 0x262b35, 0x14171d),
    ("void", "VOID BLACK", "DARK", 0x0b0d12, 0x05060a),
    ("brushed", "BRUSHED ALU", "METAL", 0xb9bec7, 0x8a8f99),
    ("steel", "SPUN STEEL", "METAL", 0x6a707c, 0x3e434d),
    ("ivory", "IVORY", "OTHER", 0xeeebe2, 0xd4d0c4),
    ("brass", "AGED BRASS", "METAL", 0x7a6332, 0x4a3b1c),
    ("ember", "EMBER", "ORGANIC", 0x5a3222, 0x371c13),
    ("teal", "DEEP TEAL", "ORGANIC", 0x2a959d, 0x1b666d),
    ("indigo", "INDIGO", "DARK", 0x2e3470, 0x1c1f4a),
    ("glass", "SMOKED GLASS", "GLASS", 0x1a2230, 0x0c1018),
    ("screen", "CRT SCREEN", "SCREEN", 0x06120c, 0x030806),
];

pub fn material(key: &str) -> (Color32, Color32) {
    let m = MATERIALS
        .iter()
        .find(|m| m.0 == key)
        .unwrap_or(&MATERIALS[0]);
    (rgb(m.3), rgb(m.4))
}

/// Paint a pane's plate: material, bevelled frame, accent bar, circuit traces.
pub fn pane_bg(
    p: &egui::Painter,
    r: Rect,
    pane: &Pane,
    device_mat: &str,
    selected: bool,
    plate: Option<egui::TextureId>,
) {
    let key = if pane.material == "inherit" {
        device_mat
    } else {
        pane.material.as_str()
    };
    let (a, b) = material(key);
    let accent = util::chc(&pane.accent);
    let r = r.shrink(2.0);
    let light = a.r() as u32 + a.g() as u32 + a.b() as u32 > 420;
    match plate {
        // Composited on the GPU: gradient, grain and accent glow in one texture.
        Some(tex) => {
            let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            p.add(egui::epaint::RectShape::filled(r, 8.0, Color32::WHITE).with_texture(tex, uv));
        }
        None => {
            grad_v(p, r, 8.0, a, b);
            if key != "screen" && !light {
                brushed(p, r, 0.02);
            }
        }
    }
    if key == "screen" {
        let g = p.with_clip_rect(r);
        let mut x = r.left();
        while x < r.right() {
            g.line_segment(
                [pos2(x, r.top()), pos2(x, r.bottom())],
                egui::Stroke::new(1.0, alpha(SIGNAL_LIFE, 0.08)),
            );
            x += 16.0;
        }
        let mut y = r.top();
        while y < r.bottom() {
            g.line_segment(
                [pos2(r.left(), y), pos2(r.right(), y)],
                egui::Stroke::new(1.0, alpha(SIGNAL_LIFE, 0.08)),
            );
            y += 16.0;
        }
    } else if !light {
        // etched circuit traces in two corners
        let g = p.with_clip_rect(r);
        for (i, (c0, sx)) in [
            (r.left_top() + vec2(10.0, 34.0), 1.0f32),
            (r.right_bottom() + vec2(-10.0, -34.0), -1.0),
        ]
        .into_iter()
        .enumerate()
        {
            for k in 0..3 {
                let y = c0.y + k as f32 * 9.0 * sx;
                let len = 26.0 + ((hash(&pane.id) >> (k + i * 3)) % 5) as f32 * 7.0;
                polyline(
                    &g,
                    vec![
                        pos2(c0.x, y),
                        pos2(c0.x + len * sx, y),
                        pos2(c0.x + (len + 8.0) * sx, y - 8.0 * sx),
                        pos2(c0.x + (len + 30.0) * sx, y - 8.0 * sx),
                    ],
                    1.0,
                    alpha(accent, 0.45),
                );
            }
        }
    }
    bevel(p, r, 8.0, if light { 0.6 } else { 0.1 }, 0.5);
    outline(p, r, 8.0, if selected { GOLD } else { black(0.7) });
    if selected {
        outline(p, r.shrink(1.0), 7.0, alpha(GOLD, 0.5));
    }
    // accent glow bars top and bottom
    for y in [r.top() + 4.0, r.bottom() - 4.0] {
        let bar = Rect::from_center_size(pos2(r.center().x, y), vec2(r.width() * 0.22, 2.0));
        glow_rect(p, bar, 1.0, 8.0, alpha(accent, 0.5));
        p.rect_filled(bar, 1.0, accent);
    }
    if r.width() > 70.0 && r.height() > 70.0 {
        for c in [
            r.left_top() + vec2(12.0, 12.0),
            r.right_top() + vec2(-12.0, 12.0),
            r.left_bottom() + vec2(12.0, -12.0),
            r.right_bottom() + vec2(-12.0, -12.0),
        ] {
            screw(p, c, 7.0);
        }
    }
    if pane.show_header && !pane.title.is_empty() {
        let ink = if light { rgb(0x20242c) } else { rgb(0xe9edf4) };
        let sq = Rect::from_min_size(r.left_top() + vec2(24.0, 13.0), Vec2::splat(8.0));
        p.rect_filled(sq, 2.0, accent);
        text_clipped(
            p,
            pos2(sq.right() + 7.0, sq.center().y),
            Align2::LEFT_CENTER,
            &pane.title,
            display(11.0),
            ink,
            0.14,
            r.width() - 60.0,
        );
    }
}

/// Rect of a component inside its pane.
pub fn comp_rect(pane_rect: Rect, kind: &str, x: f32, y: f32) -> Rect {
    let s = asset(kind)
        .map(|a| vec2(a.size[0], a.size[1]))
        .unwrap_or(vec2(40.0, 40.0));
    // Keep the component inside its pane, clear of the header strip.
    let (hx, hy) = (
        (s.x / 2.0 + 4.0).min(pane_rect.width() / 2.0),
        (s.y / 2.0 + 4.0).min(pane_rect.height() / 2.0),
    );
    let cx = (pane_rect.left() + pane_rect.width() * x)
        .clamp(pane_rect.left() + hx, pane_rect.right() - hx);
    let cy = (pane_rect.top() + pane_rect.height() * y).clamp(
        pane_rect.top() + hy + 14.0_f32.min(pane_rect.height() * 0.1),
        pane_rect.bottom() - hy,
    );
    Rect::from_center_size(pos2(cx, cy), s)
}

/// Draw a whole faceplate, live, into `rect` (used for a device's folded front).
pub fn plate(
    ui: &mut Ui,
    bus: &mut Bus,
    engine: &mut Engine,
    rect: Rect,
    fp: &Faceplate,
    prefix: &str,
) {
    for (pane, nr) in fp.tree.layout([0.0, 0.0, 1.0, 1.0]) {
        let pr = Rect::from_min_size(
            pos2(
                rect.left() + nr[0] * rect.width(),
                rect.top() + nr[1] * rect.height(),
            ),
            vec2(nr[2] * rect.width(), nr[3] * rect.height()),
        );
        pane_bg(ui.painter(), pr, pane, &fp.material, false, None);
        for c in &pane.components {
            let addr = c
                .param
                .clone()
                .unwrap_or_else(|| format!("{prefix}.{}", c.id));
            let cr = comp_rect(pr, &c.kind, c.x, c.y);
            ui.scope_builder(egui::UiBuilder::new().max_rect(pr), |ui| {
                ui.set_clip_rect(pr.intersect(ui.clip_rect()));
                draw(
                    ui,
                    bus,
                    engine,
                    &c.kind,
                    &addr,
                    cr,
                    &c.label,
                    util::ch(asset(&c.kind).map(|a| a.ch).unwrap_or("cool")),
                    true,
                );
            });
        }
    }
}
