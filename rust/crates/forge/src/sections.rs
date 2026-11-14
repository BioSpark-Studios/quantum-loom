//! Unfolded device body: a 12-column grid of horizontal sections. Each section
//! is a header (power LED, title, blurb, crest) and a left-to-right body. They
//! share the same addresses as the folded macros, so both views stay in sync.

use crate::app::ForgeApp;
use crate::util::{self, centered, chc};
use egui::{pos2, vec2, Align2, Color32, CursorIcon, Id, Rect, Sense, Stroke, Ui, Vec2};
use instrument_kit::icons;
use instrument_kit::paint::*;
use instrument_kit::theme::*;
use instrument_kit::viz::{self, FieldParams, RimScreen};
use instrument_kit::widgets::{self, Face, Knob, KnobVariant};
use rack_core::bus::{Bus, Value};
use rack_core::catalog::{self, ModuleDef};
use rack_core::engine::{Engine, DEFAULT_ENV};
use rack_core::model::{self, Cell, Device, Instrument, Param};

const PAD: f32 = 8.0;
const GAP: f32 = 6.0;
const HEAD_H: f32 = 23.0;

pub const VIEW_MODES: [(&str, &str); 10] = [
    ("scope", "SCOPE"),
    ("spec", "SPECTRUM"),
    ("sgram", "SPECTROGRAM"),
    ("flow", "FLOW"),
    ("wave", "HISTORY"),
    ("macro", "MACROS"),
    ("phase", "PHASE"),
    ("circuit", "CIRCUIT"),
    ("net", "NETWORK"),
    ("roll", "ROLL"),
];

/// Body height each module type needs.
fn body_h(kind: &str) -> f32 {
    match kind {
        "dmx" => 138.0,
        "aftimeline" => crate::timeline_ui::HEIGHT,
        "automata" | "fractal" => 126.0,
        "fader3" => 104.0,
        "sigils" => 104.0,
        k if catalog::module(k).is_some_and(|m| m.span == 12) => 176.0,
        _ => 100.0,
    }
}

fn row_h(row: &[Cell]) -> f32 {
    row.iter()
        .map(|c| {
            if let Cell::Module { kind, .. } = c {
                body_h(kind)
            } else {
                100.0
            }
        })
        .fold(100.0, f32::max)
        + HEAD_H
}

pub fn device_height(inst: &Instrument, _width: f32) -> f32 {
    let rows = model::hx_rows(&inst.modules);
    if rows.is_empty() {
        return 60.0;
    }
    PAD * 2.0 + rows.iter().map(|r| row_h(r)).sum::<f32>() + GAP * (rows.len() as f32 - 1.0)
}

fn hash(s: &str) -> u32 {
    s.chars()
        .fold(7u32, |h, c| h.wrapping_mul(31).wrapping_add(c as u32))
}

pub fn device_body(
    app: &mut ForgeApp,
    ui: &mut Ui,
    area: Rect,
    dev: &Device,
    inst: &Instrument,
    dev_on: bool,
) {
    let p = ui.painter().clone();
    let light = inst.skin == "ivory";
    if light {
        grad_v(&p, area, 0.0, rgb(0xcfcbbf), rgb(0xc3bfb3));
    } else {
        grad_v(&p, area, 0.0, rgb(0x0d1015), rgb(0x0a0c10));
    }
    inner_shade(&p, area, 0.0, if light { 0.2 } else { 0.6 });
    let rows = model::hx_rows(&inst.modules);
    if rows.is_empty() {
        text(
            &p,
            area.center(),
            Align2::CENTER_CENTER,
            "NO MODULES — ADD SOME IN FORGE · MODULES",
            mono(9.0),
            INK_DIM,
            0.14,
        );
        return;
    }
    let macros = inst.macro_params(&dev.id);
    let dept_ch = catalog::dept(inst.dept).ch;
    let col_w = (area.width() - PAD * 2.0 - GAP * 11.0) / 12.0;
    let mut y = area.top() + PAD;
    for row in &rows {
        let h = row_h(row);
        let mut colx = 0u32;
        for cell in row {
            let span = cell.span();
            let x = area.left() + PAD + colx as f32 * (col_w + GAP);
            let r = Rect::from_min_size(
                pos2(x, y),
                vec2(span as f32 * col_w + (span as f32 - 1.0) * GAP, h),
            );
            colx += span;
            if !ui.is_rect_visible(r) {
                continue;
            }
            match cell {
                Cell::Module { uid, kind, .. } => {
                    if let Some(def) = catalog::module(kind) {
                        section(app, ui, r, dev, def, uid, &inst.skin, dev_on);
                    }
                }
                Cell::Fill { slot, .. } => fill(app, ui, r, dev, *slot, &macros, dept_ch, dev_on),
            }
        }
        y += h + GAP;
    }
}

fn section_bg(p: &egui::Painter, r: Rect, skin: &str) {
    let (a, b, lite) = match skin {
        "ivory" => (rgb(0xf6f4ee), rgb(0xe3dfd4), true),
        "olive" => (rgb(0x5d5c40), rgb(0x46452f), false),
        "teal" => (rgb(0x1f7a82), rgb(0x165a60), false),
        "ember" => (rgb(0x4a2a1d), rgb(0x321a12), false),
        "indigo" => (rgb(0x262b5e), rgb(0x1a1d44), false),
        _ => (rgb(0x232832), rgb(0x181b22), false),
    };
    grad_v(p, r, 5.0, a, b);
    if !lite {
        brushed(p, r, 0.02);
    }
    bevel(p, r, 5.0, if lite { 0.9 } else { 0.08 }, 0.0);
    outline_outside(p, r, 5.0, black(if lite { 0.25 } else { 0.8 }));
}

#[allow(clippy::too_many_arguments)]
fn section(
    app: &mut ForgeApp,
    ui: &mut Ui,
    r: Rect,
    dev: &Device,
    def: &'static ModuleDef,
    uid: &str,
    skin: &str,
    dev_on: bool,
) {
    let prefix = model::mod_prefix(&dev.id, uid);
    let pkey = if def.shared {
        format!("mod.{}.on", def.name)
    } else {
        format!("{prefix}.on")
    };
    let on = app.bus.flag(&pkey, true);
    let sc = chc(def.ch);
    let light = skin == "ivory";
    let p = ui.painter().clone();
    section_bg(&p, r, skin);
    let head = Rect::from_min_size(r.min, vec2(r.width(), HEAD_H));
    p.line_segment(
        [head.left_bottom(), head.right_bottom()],
        Stroke::new(1.0, black(0.6)),
    );
    let led = Rect::from_center_size(pos2(head.left() + 13.0, head.center().y), Vec2::splat(10.0));
    let lresp = ui
        .interact(
            led.expand(4.0),
            Id::unique(("sec-led", &dev.id, uid)),
            Sense::click(),
        )
        .on_hover_text(if on {
            "Bypass section"
        } else {
            "Enable section"
        })
        .on_hover_cursor(CursorIcon::PointingHand);
    if on {
        glow_rect(&p, led, 2.0, 6.0, alpha(sc, 0.6));
        p.rect_filled(led, 2.0, sc);
    } else {
        p.rect_filled(led, 2.0, rgb(0x3a2620));
    }
    if lresp.clicked() {
        app.bus.set_flag(&pkey, !on);
    }
    let title_ink = if light { rgb(0x2a2e36) } else { rgb(0xe5e9f0) };
    let t = text_clipped(
        &p,
        pos2(head.left() + 25.0, head.center().y),
        Align2::LEFT_CENTER,
        def.name,
        display(11.0),
        title_ink,
        0.12,
        r.width() - 60.0,
    );
    let sub_w = head.right() - 30.0 - t.right() - 8.0;
    if sub_w > 30.0 {
        text_clipped(
            &p,
            pos2(t.right() + 7.0, head.center().y + 0.5),
            Align2::LEFT_CENTER,
            def.blurb,
            mono(9.0),
            if light { rgb(0x4a505a) } else { INK_DIM },
            0.0,
            sub_w,
        );
    }
    widgets::glyph_tile_at(
        &p,
        Rect::from_center_size(
            pos2(head.right() - 17.0, head.center().y),
            Vec2::splat(18.0),
        ),
        def.crest,
        sc,
        true,
        !on,
    );

    let body = Rect::from_min_max(
        pos2(r.left() + 10.0, head.bottom() + 6.0),
        pos2(r.right() - 10.0, r.bottom() - 6.0),
    );
    let active = on && dev_on;
    util::col(ui, r, ("sec-body", &dev.id, uid), |ui| {
        if !active {
            ui.set_opacity(0.32);
        }
        if !on {
            ui.disable();
        }
        module_body(app, ui, body, def, &prefix, dev);
    });
}

fn knobs(ui: &mut Ui, bus: &mut Bus, ch: &str, size: f32, list: &[(&str, &str, &str, f32, bool)]) {
    ui.spacing_mut().item_spacing.x = 2.0;
    for (key, label, variant, default, bipolar) in list {
        util::knob(
            ui,
            bus,
            key,
            Knob {
                size,
                ch: util::ch(ch),
                variant: KnobVariant::from_key(variant),
                label: Some(label),
                label_size: 8.0,
                bipolar: *bipolar,
                default: *default,
                show_value: false,
                color: None,
            },
        );
    }
    ui.spacing_mut().item_spacing.x = 12.0;
}

fn sim_scope(ui: &mut Ui, engine: &Engine, bus: &Bus, size: Vec2, ch: &str, label: &str) {
    let t = engine.time as f32;
    let seed = (engine.time * 60.0) as u32;
    viz::scope(
        ui,
        size,
        util::ch(ch),
        label,
        Some(engine.armed && false),
        |u| {
            let n =
                (((u * 977.0) as u32 ^ seed).wrapping_mul(2654435761) >> 20) as f32 / 4096.0 - 0.5;
            engine.synth_wave(bus, u, t) + n * 0.03
        },
    );
}

fn bands(engine: &mut Engine, bus: &Bus, n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| engine.synth_band(bus, i as f32 / n as f32))
        .collect()
}

fn module_body(
    app: &mut ForgeApp,
    ui: &mut Ui,
    r: Rect,
    def: &'static ModuleDef,
    prefix: &str,
    dev: &Device,
) {
    let ch = def.ch;
    let c = util::ch(ch);
    let x = |k: &str| format!("{prefix}.{k}");
    let salt = (dev.id.as_str(), prefix, def.key);
    let (bus, engine) = (&mut app.bus, &mut app.engine);
    let step = bus.num("transport.step", 0.0) as usize;
    match def.key {
        "atlas" => centered(ui, r, salt, 12.0, |ui| {
            let fp = FieldParams {
                scale: bus.num("atlas.lat", 0.42),
                drift: bus.num("atlas.lon", 0.61),
                speed: bus.num("atlas.alt", 0.28),
                level: engine.level,
                ..Default::default()
            };
            viz::particle_field(
                ui,
                Id::unique(("atlas", prefix)),
                vec2(150.0, 72.0),
                c,
                "TERRAIN",
                36,
                fp,
            );
            knobs(
                ui,
                bus,
                ch,
                38.0,
                &[
                    ("atlas.lat", "LAT", "arc", 0.42, false),
                    ("atlas.lon", "LON", "dotted", 0.61, false),
                    ("atlas.alt", "ALT", "ringed", 0.28, false),
                ],
            );
        }),
        "continuum" => centered(ui, r, salt, 12.0, |ui| {
            let run = bus.flag("cont.run", true);
            viz::polar_screen(
                ui,
                Id::unique(("cont", prefix)),
                78.0,
                Channel::Life,
                "",
                bus.num("cont.vort", 0.74),
                bus.num("cont.steps", 0.5),
                bus.num("cont.visc", 0.18),
                run,
                engine.level,
            );
            knobs(
                ui,
                bus,
                ch,
                34.0,
                &[
                    ("cont.visc", "VISC", "arc", 0.18, false),
                    ("cont.vort", "VORT", "forge", 0.74, false),
                    ("cont.steps", "STEPS", "dotted", 0.5, false),
                    ("cont.res", "RES", "pip", 0.62, false),
                ],
            );
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 5.0;
                ui.add_space(((ui.available_height() - 57.0) / 2.0).max(0.0));
                util::gate(ui, bus, "cont.run", "RUN", Channel::Life, true);
                if widgets::gate_btn(ui, "RESET", Channel::Hot, false).clicked() {
                    bus.set_num("cont.reset", engine.time as f32);
                }
            });
        }),
        "instinct" => centered(ui, r, salt, 12.0, |ui| {
            let mut pts: Vec<[f32; 2]> = bus
                .curve("inst.env")
                .map(|c| c.to_vec())
                .unwrap_or_else(|| DEFAULT_ENV.to_vec());
            let ph = ((engine.time - engine.last_trig) / (Engine::step_dur(bus) * 2.4)) as f32;
            if viz::curve_editor(
                ui,
                Id::unique(("env", prefix)),
                &mut pts,
                vec2(150.0, 72.0),
                Channel::Myth,
                "ENV",
                Some(ph),
            ) {
                bus.set("inst.env", Value::Curve(pts));
            }
            knobs(
                ui,
                bus,
                ch,
                34.0,
                &[
                    ("inst.rate", "RATE", "arc", 0.34, false),
                    ("inst.depth", "DEPTH", "ringed", 0.66, false),
                    ("inst.shape", "SHAPE", "dotted", 0.5, false),
                ],
            );
        }),
        "chronicle" => centered(ui, r, salt, 12.0, |ui| {
            let cell = (((r.width() - 260.0) / 16.0) - 3.0).clamp(11.0, 22.0);
            let playing = engine.playing;
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 5.0;
                ui.add_space(((ui.available_height() - cell * 2.0 - 5.0) / 2.0).max(0.0));
                util::steps(ui, bus, "chr.a", Some(step), Channel::Amber, cell);
                util::steps(ui, bus, "chr.b", Some(step), Channel::Warm, cell);
            });
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 5.0;
                ui.add_space(((ui.available_height() - 67.0) / 2.0).max(0.0));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    widgets::readout_box(
                        ui,
                        Some("BPM"),
                        &format!("{:.1}", Engine::bpm_exact(bus)),
                        Channel::Amber,
                        60.0,
                    );
                    widgets::readout_box(
                        ui,
                        Some("STEP"),
                        &format!("{:02}", step + 1),
                        Channel::Amber,
                        44.0,
                    );
                });
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    if widgets::gate_btn(
                        ui,
                        if playing { "STOP" } else { "PLAY" },
                        Channel::Life,
                        playing,
                    )
                    .clicked()
                    {
                        engine.toggle_play(bus);
                    }
                    if widgets::gate_btn(ui, "RAND", Channel::Myth, false).clicked() {
                        let a: Vec<bool> = (0..16).map(|_| engine.rand() > 0.55).collect();
                        let b: Vec<bool> = (0..16).map(|_| engine.rand() > 0.85).collect();
                        bus.set("chr.a", Value::Steps(a));
                        bus.set("chr.b", Value::Steps(b));
                    }
                });
            });
            knobs(
                ui,
                bus,
                "amber",
                38.0,
                &[
                    ("chr.bpm", "TEMPO", "ringed", 0.53, false),
                    ("chr.swing", "SWING", "arc", 0.12, false),
                ],
            );
        }),
        "composer" => centered(ui, r, salt, 12.0, |ui| {
            let b = bands(engine, bus, 24);
            viz::spectrum(
                ui,
                Id::unique(("fft", prefix)),
                vec2(170.0, 72.0),
                Channel::Warm,
                "FFT",
                &b,
            );
            util::fader(
                ui,
                bus,
                "comp.gain",
                0.6,
                60.0,
                20.0,
                Channel::Warm,
                Some("GAIN"),
                true,
            );
            knobs(
                ui,
                bus,
                ch,
                38.0,
                &[
                    ("comp.pitch", "PITCH", "arc", 0.5, true),
                    ("comp.drive", "DRIVE", "forge", 0.6, false),
                    ("comp.formant", "CUTOFF", "dotted", 0.4, false),
                ],
            );
        }),
        "quill" => centered(ui, r, salt, 12.0, |ui| {
            let notes: Vec<(u64, i32, bool)> = engine
                .notes
                .iter()
                .map(|n| (n.abs, n.midi, n.accent))
                .collect();
            viz::piano_roll(
                ui,
                vec2(170.0, 72.0),
                Channel::Myth,
                "BEATS",
                &notes,
                engine.abs,
            );
            knobs(
                ui,
                bus,
                ch,
                38.0,
                &[
                    ("quill.act", "ACT", "dotted", 0.6, false),
                    ("quill.tension", "TENSION", "arc", 0.78, false),
                ],
            );
            util::xy(ui, bus, "quill.xy", 76.0, Channel::Myth, None);
        }),
        "forge" => centered(ui, r, salt, 12.0, |ui| {
            util::knob(
                ui,
                bus,
                "forge.main",
                Knob::new(52.0, Channel::Forge)
                    .variant(KnobVariant::Forge)
                    .label("MAIN")
                    .default_value(0.78)
                    .show_value(false),
            );
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 5.0;
                ui.add_space(((ui.available_height() - 34.0) / 2.0).max(0.0));
                widgets::vu(ui, engine.rms.l, 110.0, 8.0, Some("L"));
                widgets::vu(ui, engine.rms.r, 110.0, 8.0, Some("R"));
                widgets::vu(ui, engine.rms.drive, 110.0, 8.0, Some("D"));
            });
            let mut mode = bus.num("forge.mode", 2.0) as usize;
            if widgets::slide_switch(ui, &mut mode, &["MUTE", "CUE", "ON"], Channel::Forge)
                .changed()
            {
                bus.set_num("forge.mode", mode as f32);
            }
            ui.spacing_mut().item_spacing.x = 6.0;
            if widgets::pad(ui, Face::Text("ARM"), Channel::Hot, engine.armed, 28.0).clicked() {
                engine.toggle_arm(bus);
            }
            if widgets::pad(
                ui,
                Face::Text("REC"),
                Channel::Hot,
                bus.flag("engine.rec", false),
                28.0,
            )
            .clicked()
            {
                engine.toggle_rec(bus);
            }
        }),
        "macro4" => centered(ui, r, salt, 12.0, |ui| {
            knobs(
                ui,
                bus,
                ch,
                38.0,
                &[
                    (&x("a"), "A", "arc", 0.5, false),
                    (&x("b"), "B", "dotted", 0.5, false),
                    (&x("c"), "C", "ringed", 0.5, false),
                    (&x("d"), "D", "forge", 0.5, false),
                ],
            );
        }),
        "fader3" => centered(ui, r, salt, 4.0, |ui| {
            for n in 1..=3 {
                util::fader(
                    ui,
                    bus,
                    &x(&format!("f{n}")),
                    0.3 + n as f32 * 0.15,
                    64.0,
                    18.0,
                    c,
                    Some(&format!("F{n}")),
                    true,
                );
            }
        }),
        "gates4" => centered(ui, r, salt, 5.0, |ui| {
            for pair in [[1, 2], [3, 4]] {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 5.0;
                    ui.add_space(((ui.available_height() - 57.0) / 2.0).max(0.0));
                    for n in pair {
                        util::gate(ui, bus, &x(&format!("g{n}")), &format!("G{n}"), c, false);
                    }
                });
            }
        }),
        "scope" => centered(ui, r, salt, 12.0, |ui| {
            sim_scope(
                ui,
                engine,
                bus,
                vec2((r.width() - 110.0).clamp(90.0, 160.0), 72.0),
                ch,
                "OUT",
            );
            knobs(
                ui,
                bus,
                ch,
                34.0,
                &[
                    (&x("gain"), "GAIN", "arc", 0.5, false),
                    (&x("trim"), "TRIM", "dotted", 0.5, false),
                ],
            );
        }),
        "xy" => centered(ui, r, salt, 12.0, |ui| {
            util::xy(ui, bus, &x("xy"), 86.0, c, None);
        }),
        "seq" => centered(ui, r, salt, 12.0, |ui| {
            let cell = (((r.width() - 150.0) / 16.0) - 3.0).clamp(10.0, 22.0);
            util::steps(ui, bus, &x("seq"), Some(step), c, cell);
            knobs(
                ui,
                bus,
                ch,
                32.0,
                &[
                    (&x("len"), "LEN", "dotted", 1.0, false),
                    (&x("prob"), "PROB", "arc", 0.5, false),
                    (&x("rot"), "ROT", "ringed", 0.0, false),
                ],
            );
        }),
        "automata" | "fractal" => centered(ui, r, salt, 12.0, |ui| {
            let (key, def_v, hot, label, kind) = if def.key == "automata" {
                (x("dens"), 0.45, true, "DENSITY", RimScreen::Life)
            } else {
                (x("depth"), 0.5, false, "RECURSION", RimScreen::Mandel)
            };
            let mut v = bus.num(&key, def_v);
            if viz::rim_knob(
                ui,
                Id::unique(("rim", prefix)),
                &mut v,
                96.0,
                hot,
                label,
                kind,
                def_v,
            )
            .changed()
            {
                bus.set_num(&key, v);
            }
        }),
        "dmx" => centered(ui, r, salt, 10.0, |ui| {
            for n in 1..=6 {
                let key = x(&format!("c{n}"));
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 3.0;
                    util::fader(
                        ui,
                        bus,
                        &key,
                        0.5,
                        96.0,
                        16.0,
                        c,
                        Some(&format!("CH{n}")),
                        false,
                    );
                    let (rr, _) = ui.allocate_exact_size(vec2(44.0, 11.0), Sense::hover());
                    readout(
                        ui.painter(),
                        rr.center(),
                        Align2::CENTER_CENTER,
                        &format!("{}", (bus.num(&key, 0.5) * 255.0).round()),
                        9.0,
                        SIGNAL_COOL,
                    );
                });
            }
        }),
        "sigils" => centered(ui, r, salt, 5.0, |ui| {
            for half in 0..2 {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    for i in 0..4 {
                        let (k, l) = def.params[half * 4 + i];
                        let key = x(k);
                        let lit = bus.flag(&key, false);
                        if widgets::pad(ui, Face::Text(&l[..l.len().min(3)]), c, lit, 22.0)
                            .on_hover_text(l)
                            .clicked()
                        {
                            bus.set_flag(&key, !lit);
                        }
                    }
                });
            }
        }),
        "aftimeline" => crate::timeline_ui::face(app, ui, r, prefix),
        "stem" => centered(ui, r, salt, 10.0, |ui| {
            let (sr, _) = ui.allocate_exact_size(vec2(120.0, 72.0), Sense::hover());
            let g = mat_screen(ui.painter(), sr, 6.0);
            icons::paint(
                &g,
                "audio",
                pos2(sr.center().x, sr.center().y - 8.0),
                26.0,
                alpha(chc(ch), 0.7),
            );
            text(
                &g,
                pos2(sr.center().x, sr.bottom() - 12.0),
                Align2::CENTER_CENTER,
                "NO MEDIA · MASTER CLOCK",
                mono(7.0),
                INK_DIM,
                0.1,
            );
            knobs(
                ui,
                bus,
                ch,
                34.0,
                &[
                    (&x("gain"), "GAIN", "arc", 0.7, false),
                    (&x("ofs"), "OFFSET", "dotted", 0.5, true),
                ],
            );
        }),
        _ => generic_wide(ui, bus, engine, r, def, prefix),
    }
}

/// Control surface for instruments whose bespoke face is not ported yet: a
/// live screen plus one knob per parameter, on the real addresses.
fn generic_wide(
    ui: &mut Ui,
    bus: &mut Bus,
    engine: &mut Engine,
    r: Rect,
    def: &'static ModuleDef,
    prefix: &str,
) {
    let c = util::ch(def.ch);
    let sw = (r.width() * 0.34).clamp(160.0, 420.0);
    let screen = Rect::from_min_size(r.min, vec2(sw, r.height()));
    util::row(ui, screen, ("gw-screen", prefix), |ui| {
        let first = def
            .params
            .first()
            .map(|p| bus.num(&format!("{prefix}.{}", p.0), 0.5))
            .unwrap_or(0.5);
        let second = def
            .params
            .get(1)
            .map(|p| bus.num(&format!("{prefix}.{}", p.0), 0.5))
            .unwrap_or(0.5);
        match hash(def.key) % 3 {
            0 => {
                viz::network_art(
                    ui,
                    vec2(sw, r.height()),
                    hash(prefix),
                    c.color(),
                    engine.time as f32 * (0.4 + first),
                );
            }
            1 => {
                viz::particle_field(
                    ui,
                    Id::unique(("gw", prefix)),
                    vec2(sw, r.height()),
                    c,
                    "",
                    ((sw * r.height()) / 500.0) as usize,
                    FieldParams {
                        scale: first,
                        drift: second,
                        level: engine.level,
                        ..Default::default()
                    },
                );
            }
            _ => {
                let b = bands(engine, bus, 32);
                viz::spectrogram(ui, Id::unique(("gw", prefix)), vec2(sw, r.height()), "", &b);
            }
        };
    });
    text(
        ui.painter(),
        pos2(screen.left() + 8.0, screen.top() + 7.0),
        Align2::LEFT_TOP,
        "CONTROL SURFACE · FULL FACE IN A LATER PASS",
        mono(7.0),
        INK_DIM,
        0.12,
    );
    let rest = Rect::from_min_max(pos2(screen.right() + 12.0, r.top()), r.max);
    let n = def.params.len().max(1);
    let per_row = ((rest.width() / 62.0).floor() as usize).max(1);
    let rows = n.div_ceil(per_row);
    let size = if rows > 2 { 26.0 } else { 36.0 };
    let rh = rest.height() / rows as f32;
    for (ri, chunk) in def.params.chunks(per_row).enumerate() {
        let rr = Rect::from_min_size(
            pos2(rest.left(), rest.top() + ri as f32 * rh),
            vec2(rest.width(), rh),
        );
        centered(ui, rr, ("gw-k", prefix, ri), 4.0, |ui| {
            for (k, l) in chunk {
                util::knob(
                    ui,
                    bus,
                    &format!("{prefix}.{k}"),
                    Knob::new(size, c)
                        .label(l)
                        .label_size(7.0)
                        .show_value(false),
                );
            }
        });
    }
}

/// VIEW section that fills leftover row space with a live readout.
#[allow(clippy::too_many_arguments)]
fn fill(
    app: &mut ForgeApp,
    ui: &mut Ui,
    r: Rect,
    dev: &Device,
    slot: usize,
    macros: &[Param],
    ch: &str,
    dev_on: bool,
) {
    let sc = chc(ch);
    let key = format!("{}.{slot}", dev.id);
    let seed = hash(&key);
    let mode = app
        .prefs
        .fills
        .get(&key)
        .cloned()
        .unwrap_or_else(|| VIEW_MODES[seed as usize % VIEW_MODES.len()].0.to_owned());
    let idx = VIEW_MODES.iter().position(|m| m.0 == mode).unwrap_or(0);
    let p = ui.painter().clone();
    grad_v(&p, r, 5.0, rgb(0x14171e), rgb(0x0e1015));
    outline_outside(&p, r, 5.0, black(0.8));
    let head = Rect::from_min_size(r.min, vec2(r.width(), HEAD_H));
    p.line_segment(
        [head.left_bottom(), head.right_bottom()],
        Stroke::new(1.0, black(0.6)),
    );
    let d = pos2(head.left() + 13.0, head.center().y);
    p.add(egui::Shape::convex_polygon(
        vec![
            d + vec2(0.0, -5.0),
            d + vec2(5.0, 0.0),
            d + vec2(0.0, 5.0),
            d + vec2(-5.0, 0.0),
        ],
        sc,
        Stroke::NONE,
    ));
    let t = text(
        &p,
        pos2(head.left() + 25.0, head.center().y),
        Align2::LEFT_CENTER,
        "VIEW",
        display(11.0),
        rgb(0xe5e9f0),
        0.12,
    );
    text_clipped(
        &p,
        pos2(t.right() + 7.0, head.center().y + 0.5),
        Align2::LEFT_CENTER,
        VIEW_MODES[idx].1,
        mono(9.0),
        INK_DIM,
        0.0,
        (head.right() - 50.0 - t.right()).max(1.0),
    );
    let mut stepv = 0i32;
    for (i, (icon, d, tip)) in [("chev", -1, "Previous view"), ("chevR", 1, "Next view")]
        .into_iter()
        .enumerate()
    {
        let b = Rect::from_center_size(
            pos2(head.right() - 34.0 + i as f32 * 20.0, head.center().y),
            Vec2::splat(18.0),
        );
        let resp = ui
            .interact(b, Id::unique(("fill-nav", &key, i)), Sense::click())
            .on_hover_text(tip)
            .on_hover_cursor(CursorIcon::PointingHand);
        p.rect_filled(b, 3.0, rgb(0x0b0d11));
        icons::paint(
            &p,
            icon,
            b.center(),
            10.0,
            if resp.hovered() { sc } else { INK_MID },
        );
        if resp.clicked() {
            stepv = d;
        }
    }
    if stepv != 0 {
        let n = VIEW_MODES.len() as i32;
        app.prefs.fills.insert(
            key.clone(),
            VIEW_MODES[(idx as i32 + stepv).rem_euclid(n) as usize]
                .0
                .to_owned(),
        );
    }
    let stage = Rect::from_min_max(
        pos2(r.left() + 6.0, head.bottom() + 6.0),
        pos2(r.right() - 6.0, r.bottom() - 6.0),
    );
    if stage.width() < 20.0 || stage.height() < 20.0 {
        return;
    }
    let (bus, engine) = (&mut app.bus, &mut app.engine);
    let c = util::ch(ch);
    let size = stage.size();
    util::col(ui, stage, ("fill-wrap", &key), |ui| {
        if !dev_on {
            ui.set_opacity(0.35);
        }
        util::row(ui, stage, ("fill", &key), |ui| {
            let id = Id::unique(("fillv", &key, mode.as_str()));
            let mv = |i: usize, d: f32| macros.get(i).map(|m| bus.num(&m.p, d)).unwrap_or(d);
            match mode.as_str() {
                "scope" => sim_scope(ui, engine, bus, size, ch, ""),
                "spec" => {
                    let b = bands(engine, bus, ((size.x / 9.0) as usize).clamp(12, 64));
                    viz::spectrum(ui, id, size, c, "", &b);
                }
                "sgram" => {
                    let b = bands(engine, bus, 32);
                    viz::spectrogram(ui, id, size, "", &b);
                }
                "flow" => {
                    let fp = FieldParams {
                        scale: mv(0, 0.4),
                        drift: mv(1, 0.5),
                        speed: mv(2, 0.3),
                        level: engine.level,
                        ..Default::default()
                    };
                    viz::particle_field(
                        ui,
                        id,
                        size,
                        c,
                        "",
                        ((size.x * size.y / 260.0) as usize).min(140),
                        fp,
                    );
                }
                "wave" => {
                    viz::waveform(ui, size, c, "", &engine.hist, engine.hist_i);
                }
                "phase" => {
                    let s = size.x.min(size.y);
                    ui.add_space((size.x - s) / 2.0);
                    viz::phase_scope(
                        ui,
                        s,
                        c,
                        "",
                        1.0 + Engine::scale_idx(bus) as f32,
                        2.0 + (bus.num("cont.vort", 0.74) * 3.0).round(),
                        engine.amp(bus),
                        bus.num("comp.formant", 0.4) * std::f32::consts::PI,
                        engine.time as f32,
                    );
                }
                "roll" => {
                    let notes: Vec<(u64, i32, bool)> = engine
                        .notes
                        .iter()
                        .map(|n| (n.abs, n.midi, n.accent))
                        .collect();
                    viz::piano_roll(ui, size, c, "", &notes, engine.abs);
                }
                "circuit" => {
                    viz::circuit_art(ui, size, seed, sc);
                }
                "net" => {
                    viz::network_art(ui, size, seed, sc, engine.time as f32);
                }
                _ => {
                    let items: Vec<(&str, f32)> = macros
                        .iter()
                        .map(|m| (m.label, bus.num(&m.p, 0.5)))
                        .collect();
                    viz::macro_bars(ui, size, sc, &items);
                }
            }
        });
    });
    // corner reticles
    let k = alpha(sc, 0.55);
    for (c0, dx, dy) in [
        (stage.left_top() + vec2(3.0, 3.0), 1.0, 1.0),
        (stage.right_top() + vec2(-3.0, 3.0), -1.0, 1.0),
        (stage.left_bottom() + vec2(3.0, -3.0), 1.0, -1.0),
        (stage.right_bottom() + vec2(-3.0, -3.0), -1.0, -1.0),
    ] {
        p.line_segment([c0, c0 + vec2(8.0 * dx, 0.0)], Stroke::new(1.0, k));
        p.line_segment([c0, c0 + vec2(0.0, 8.0 * dy)], Stroke::new(1.0, k));
    }
    let _ = Color32::WHITE;
}
