//! PERFORMANCE · DJ — the classic layout: two jog decks, a four-channel mixer
//! with master, the timeline crossfader, quantum-event pads and the FX rack.
//! Narrative-physics labelling: LOW = Thematic Gravity · MID = Causal
//! Connectivity · HIGH = Exposition · FILTER = Cognitive Dissonance ·
//! FADER = Archetype Mass · XF = Timeline Entropy · TEMPO = Dilation.

use crate::app::ForgeApp;
use crate::util::{self, chc, col, row};
use egui::{
    pos2, vec2, Align2, Color32, CursorIcon, Id, Rect, ScrollArea, Sense, Stroke, Ui, Vec2,
};
use instrument_kit::icons;
use instrument_kit::paint::{self, *};
use instrument_kit::theme::*;
use instrument_kit::widgets::{self, Knob, KnobVariant};
use rack_core::catalog::{DEPTS, FX};
use rack_core::decks::{self, BANKS, LOOP_SIZES, QEVENTS};
use rack_core::engine::Engine;
use rack_core::genesis::Genesis;

fn deck_ch(g: &Genesis, mythos: Option<usize>) -> &'static str {
    mythos
        .and_then(|m| g.mythos().get(m))
        .and_then(|m| DEPTS.iter().find(|d| d.crest == m.crest))
        .map(|d| d.ch)
        .unwrap_or("cool")
}

fn panel(p: &egui::Painter, r: Rect) {
    grad_v(p, r, 10.0, rgb(0x0c111a), rgb(0x080b11));
    outline(p, r, 10.0, alpha(SIGNAL_COOL, 0.16));
}

/// Flat deck button with a coloured outline (PLAY / CUE / LOOP …).
fn deck_btn(
    ui: &mut Ui,
    r: Rect,
    salt: impl std::hash::Hash + std::fmt::Debug,
    label: &str,
    color: Color32,
    on: bool,
    size: f32,
) -> bool {
    let resp = ui
        .interact(r, Id::unique(salt), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand);
    let p = ui.painter();
    if on {
        glow_rect(p, r, 4.0, 10.0, alpha(color, 0.45));
        p.rect_filled(r, 4.0, color);
    } else {
        p.rect_filled(
            r,
            4.0,
            if resp.hovered() {
                rgb(0x101722)
            } else {
                rgb(0x0a0e15)
            },
        );
        outline(
            p,
            r,
            4.0,
            alpha(color, if resp.hovered() { 0.9 } else { 0.55 }),
        );
    }
    text(
        p,
        r.center(),
        Align2::CENTER_CENTER,
        label,
        mono(size),
        if on { rgb(0x04121a) } else { color },
        0.14,
    );
    resp.clicked()
}

pub fn body(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    ui.painter().rect_filled(area, 0.0, SHELL);
    let Some(g) = app.genesis.clone() else {
        crate::shell::stub(
            ui,
            area,
            "disc",
            "NO GENESIS MOUNTED",
            "Decks play the Mythos of a mounted Genesis Container.",
            "USE LOAD IN THE HEADER",
        );
        return;
    };
    col(ui, area, "decks", |ui| {
        ScrollArea::both().auto_shrink(false).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let w = (ui.available_width() - 20.0).max(980.0);
            let x0 = ui.cursor().left() + 10.0;
            ui.add_space(10.0);

            // layout + bank tabs
            let (tr, _) = ui.allocate_exact_size(vec2(w + 20.0, 28.0), Sense::hover());
            row(
                ui,
                Rect::from_min_size(pos2(x0, tr.top()), vec2(w, 28.0)),
                "deck-tabs",
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    tab(ui, "CLASSIC", true, true);
                    tab(ui, "EXTENDED", false, false).on_disabled_hover_text(
                        "The extended eight-deck layout arrives in a later pass",
                    );
                    ui.add_space(10.0);
                    for (i, b) in BANKS.iter().enumerate() {
                        let dept = g
                            .mythos()
                            .get(i * 4)
                            .map(|m| m.department.clone())
                            .unwrap_or_default();
                        if tab(ui, &format!("BANK {b} · {dept}"), app.decks.bank == i, true)
                            .clicked()
                        {
                            app.decks.set_bank(i);
                        }
                    }
                },
            );
            ui.add_space(12.0);

            // controller bar
            let (cr, _) = ui.allocate_exact_size(vec2(w + 20.0, 36.0), Sense::hover());
            let cr = Rect::from_min_size(pos2(x0, cr.top()), vec2(w, 36.0));
            ui.painter().rect_filled(cr, 6.0, rgb(0x0b0e14));
            engrave(
                ui.painter(),
                pos2(cr.left() + 10.0, cr.center().y),
                Align2::LEFT_CENTER,
                "CONTROLLER",
                9.0,
                INK_MID,
            );
            let sel =
                Rect::from_min_size(pos2(cr.left() + 112.0, cr.top() + 5.0), vec2(300.0, 26.0));
            ui.painter().rect_filled(sel, 3.0, rgb(0x06080c));
            outline(ui.painter(), sel, 3.0, white(0.08));
            text(
                ui.painter(),
                pos2(sel.left() + 10.0, sel.center().y),
                Align2::LEFT_CENTER,
                "No controller",
                mono(9.0),
                INK,
                0.08,
            );
            widgets::led_at(
                ui.painter(),
                pos2(sel.right() + 16.0, cr.center().y),
                6.0,
                false,
                SIGNAL_AMBER,
            );
            text(
                ui.painter(),
                pos2(sel.right() + 26.0, cr.center().y),
                Align2::LEFT_CENTER,
                "MIDI · OFF",
                mono(9.0),
                INK_MID,
                0.1,
            );
            ui.interact(cr, Id::unique("ctrl-bar"), Sense::hover())
                .on_hover_text("Controller profiles need MIDI input, which is not built yet");
            ui.add_space(14.0);

            // decks + mixer
            let main_h = 604.0;
            let (mr, _) = ui.allocate_exact_size(vec2(w + 20.0, main_h), Sense::hover());
            let main = Rect::from_min_size(pos2(x0, mr.top()), vec2(w, main_h));
            panel(ui.painter(), main);
            let inner = main.shrink(10.0);
            let dw = ((inner.width() - 20.0) * 0.33).floor();
            let left = Rect::from_min_size(inner.min, vec2(dw, inner.height()));
            let right = Rect::from_min_max(pos2(inner.right() - dw, inner.top()), inner.max);
            let mid = Rect::from_min_max(
                pos2(left.right() + 10.0, inner.top()),
                pos2(right.left() - 10.0, inner.bottom()),
            );
            deck(app, ui, &g, left, false);
            deck(app, ui, &g, right, true);
            mixer(app, ui, &g, mid);
            ui.add_space(14.0);

            // FX rack
            let (fr, _) = ui.allocate_exact_size(vec2(w + 20.0, 150.0), Sense::hover());
            fx_rack(
                app,
                ui,
                Rect::from_min_size(pos2(x0, fr.top()), vec2(w, 150.0)),
            );
            ui.add_space(70.0);
        });
    });
}

fn tab(ui: &mut Ui, label: &str, on: bool, enabled: bool) -> egui::Response {
    let w = measure(ui.painter(), label, mono(8.5), 0.14).x + 18.0;
    let (r, resp) = ui.allocate_exact_size(
        vec2(w.ceil(), 24.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let p = ui.painter();
    if on {
        p.rect_filled(r, 3.0, alpha(SIGNAL_COOL, 0.12));
        outline(p, r, 3.0, alpha(SIGNAL_COOL, 0.6));
    }
    let ink = if on {
        SIGNAL_COOL
    } else if !enabled {
        INK_DIM
    } else if resp.hovered() {
        INK
    } else {
        INK_MID
    };
    text(
        p,
        r.center(),
        Align2::CENTER_CENTER,
        label,
        mono(8.5),
        ink,
        0.14,
    );
    if enabled {
        resp.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        resp
    }
}

/// Scrolling narrative waveform: intensity around the playhead.
fn narr_wave(
    p: &egui::Painter,
    r: Rect,
    c: Option<&rack_core::genesis::Container>,
    pos: f64,
    color: Color32,
) {
    p.rect_filled(r, 3.0, rgb(0x05070b));
    let Some(c) = c else {
        text(
            p,
            r.center(),
            Align2::CENTER_CENTER,
            "EMPTY",
            mono(8.0),
            INK_DIM,
            0.14,
        );
        return;
    };
    let len = decks::track_beats(c);
    let n = (r.width() / 3.0) as usize;
    let span = 96.0; // beats visible
    let g = p.with_clip_rect(r);
    let mut mesh = egui::epaint::Mesh::default();
    for i in 0..n {
        let b = pos + (i as f64 / n as f64 - 0.5) * span;
        if b < 0.0 || b > len {
            continue;
        }
        let v = decks::intensity(c, b) * (0.72 + 0.28 * ((b * 7.3).sin() as f32).abs());
        let h = v * r.height() * 0.9;
        let x = r.left() + i as f32 * 3.0;
        mesh.add_colored_rect(
            Rect::from_center_size(pos2(x + 1.0, r.center().y), vec2(2.0, h.max(1.0))),
            alpha(color, if b < pos { 0.4 } else { 0.9 }),
        );
    }
    g.add(egui::Shape::mesh(mesh));
    // capsule boundaries
    let mut b = 0.0;
    for cap in &c.capsules {
        let x = r.center().x + ((b - pos) / span) as f32 * r.width();
        if x > r.left() && x < r.right() {
            g.line_segment(
                [pos2(x, r.top()), pos2(x, r.bottom())],
                Stroke::new(1.0, white(0.25)),
            );
        }
        b += decks::capsule_beats(cap);
    }
    g.line_segment(
        [r.center_top(), r.center_bottom()],
        Stroke::new(1.5, Color32::WHITE),
    );
}

fn deck(app: &mut ForgeApp, ui: &mut Ui, g: &Genesis, r: Rect, right: bool) {
    let idx = if right {
        app.decks.right
    } else {
        app.decks.left
    };
    let master = Engine::bpm_exact(&app.bus);
    let side = app.decks.side_decks(right);
    let d = app.decks.decks[idx].clone();
    let ch = deck_ch(g, d.mythos);
    let dc = chc(ch);
    let cont = d.container(g);
    let p = ui.painter().clone();

    narr_wave(
        &p,
        Rect::from_min_size(r.min, vec2(r.width(), 46.0)),
        cont,
        d.pos,
        dc,
    );
    let mut y = r.top() + 56.0;

    // label row
    let lab = Rect::from_min_size(pos2(r.left(), y), vec2(r.width(), 40.0));
    for (k, di) in side.iter().enumerate() {
        let b = Rect::from_min_size(
            pos2(lab.left() + k as f32 * 30.0, lab.top() + 6.0),
            vec2(28.0, 28.0),
        );
        let on = *di == idx;
        let resp = ui
            .interact(b, Id::unique(("deck-layer", right, k)), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        p.rect_filled(
            b,
            3.0,
            if on {
                alpha(SIGNAL_COOL, 0.14)
            } else {
                rgb(0x0a0d12)
            },
        );
        outline(
            &p,
            b,
            3.0,
            if on {
                alpha(SIGNAL_COOL, 0.7)
            } else {
                white(0.08)
            },
        );
        text(
            &p,
            b.center(),
            Align2::CENTER_CENTER,
            &decks::LETTERS[*di].to_string(),
            mono(9.0),
            if on { SIGNAL_COOL } else { INK_MID },
            0.0,
        );
        if resp.clicked() {
            if right {
                app.decks.right = *di;
            } else {
                app.decks.left = *di;
            }
        }
    }
    let bpm = d.bpm(g, master);
    let name_w = (r.width() - 70.0 - 56.0 - 74.0 - 40.0 - 24.0).max(40.0);
    text(
        &p,
        pos2(lab.left() + 70.0, lab.top() + 13.0),
        Align2::LEFT_CENTER,
        &format!("DECK {}", d.id),
        display(13.0),
        INK,
        0.1,
    );
    let sub = format!(
        "{} · {}",
        d.mythos_of(g).map(|m| m.name.as_str()).unwrap_or("EMPTY"),
        cont.map(|c| c.name.as_str()).unwrap_or("—")
    );
    text_clipped(
        &p,
        pos2(lab.left() + 70.0, lab.top() + 28.0),
        Align2::LEFT_CENTER,
        &sub,
        mono(8.0),
        dc,
        0.1,
        name_w,
    );
    util::row_rtl(ui, lab, ("deck-ro", right), |ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let (pr, presp) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
        ui.painter()
            .circle_stroke(pr.center(), 12.5, Stroke::new(1.0, alpha(SIGNAL_COOL, 0.6)));
        text(
            ui.painter(),
            pr.center(),
            Align2::CENTER_CENTER,
            "+",
            mono(12.0),
            SIGNAL_COOL,
            0.0,
        );
        if presp
            .on_hover_text("Load next Mythos from the crate")
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            app.decks.load_next(idx, g);
        }
        widgets::readout_box(
            ui,
            Some("TIMECODE"),
            &decks::timecode(d.pos * 60.0 / bpm.max(1.0) as f64),
            Channel::Cool,
            74.0,
        );
        widgets::readout_box(
            ui,
            Some("BPM"),
            &format!("{bpm:.1}"),
            if d.sync {
                Channel::Life
            } else {
                Channel::Amber
            },
            56.0,
        );
    });
    y += 48.0;

    // jog + tempo
    let body_h = 226.0;
    let tempo_w = 104.0;
    let jog_c = pos2(r.left() + (r.width() - tempo_w) / 2.0, y + body_h / 2.0);
    let jr = ((r.width() - tempo_w) / 2.0 - 16.0).min(104.0);
    p.circle_stroke(jog_c, jr, Stroke::new(2.0, white(0.08)));
    p.circle_stroke(jog_c, jr - 8.0, Stroke::new(9.0, alpha(dc, 0.18)));
    p.circle_stroke(jog_c, jr - 8.0, Stroke::new(5.0, dc));
    for i in 0..60 {
        let a = i as f32 * 6.0;
        p.line_segment(
            [
                polar(jog_c, jr - 20.0, a),
                polar(jog_c, jr - if i % 5 == 0 { 28.0 } else { 24.0 }, a),
            ],
            Stroke::new(1.0, white(0.18)),
        );
    }
    let needle = (d.pos * 33.3 % 360.0) as f32;
    p.line_segment(
        [polar(jog_c, jr, needle), polar(jog_c, jr - 16.0, needle)],
        Stroke::new(4.0, Color32::WHITE),
    );
    // plasma orb platter — drag to scrub
    let or = jr * 0.54;
    let orb = Rect::from_center_size(jog_c, Vec2::splat(or * 2.0));
    let oresp = ui
        .interact(orb, Id::unique(("orb", right)), Sense::drag())
        .on_hover_text("Drag to scratch / scrub")
        .on_hover_cursor(CursorIcon::Grab);
    if oresp.dragged() {
        let dd = oresp.drag_delta();
        let dk = &mut app.decks.decks[idx];
        dk.pos = (dk.pos + (dd.x - dd.y) as f64 * 0.05).max(0.0);
    }
    p.circle_filled(jog_c, or + 6.0, rgb(0x0b0e14));
    p.circle_stroke(jog_c, or + 6.0, Stroke::new(2.0, rgb(0x2a2f3a)));
    radial(
        &p,
        jog_c,
        or,
        rgb(0x1a2030),
        rgb(0x05070c),
        vec2(-0.25, -0.35),
    );
    let level = app.decks.levels[idx];
    let t = app.engine.time as f32;
    for k in 0..5 {
        let amp = or * (0.12 + 0.22 * level) * (1.0 - k as f32 * 0.12);
        let pts: Vec<egui::Pos2> = (0..=40)
            .map(|i| {
                let u = i as f32 / 40.0;
                let env = (u * std::f32::consts::PI).sin();
                pos2(
                    jog_c.x + (u - 0.5) * or * 1.5,
                    jog_c.y
                        + (u * 9.0 + t * (1.0 + k as f32 * 0.3) + d.pos as f32 * 0.4 + k as f32)
                            .sin()
                            * amp
                            * env,
                )
            })
            .collect();
        polyline(&p, pts, 1.0, alpha(dc, 0.25 + 0.12 * k as f32));
    }
    p.circle_filled(jog_c + vec2(or * 0.62, 0.0), 2.0, Color32::WHITE);

    // tempo box
    let tb = Rect::from_min_size(pos2(r.right() - tempo_w, y), vec2(tempo_w, body_h));
    outline(&p, tb, 4.0, white(0.07));
    engrave(
        &p,
        pos2(tb.center().x, tb.top() + 12.0),
        Align2::CENTER_CENTER,
        "TEMPO · DILATION",
        7.0,
        INK_DIM,
    );
    util::centered(
        ui,
        Rect::from_min_size(pos2(tb.left(), tb.top() + 20.0), vec2(tempo_w, 172.0)),
        ("tempo", right),
        0.0,
        |ui| {
            let mut pitch = d.pitch;
            if widgets::fader(
                ui,
                &mut pitch,
                168.0,
                24.0,
                if d.sync {
                    Channel::Life
                } else {
                    Channel::Amber
                },
                None,
                true,
            )
            .changed()
            {
                let dk = &mut app.decks.decks[idx];
                dk.pitch = pitch;
                dk.sync = false;
            }
        },
    );
    for (k, (l, dv)) in [("−", -0.5), ("+", 0.5)].into_iter().enumerate() {
        let b = Rect::from_min_size(
            pos2(tb.center().x - 25.0 + k as f32 * 27.0, tb.bottom() - 28.0),
            vec2(23.0, 20.0),
        );
        if deck_btn(ui, b, ("nudge", right, k), l, SIGNAL_COOL, false, 10.0) {
            let dk = &mut app.decks.decks[idx];
            dk.pos = (dk.pos + dv).max(0.0);
        }
    }
    y += body_h + 8.0;

    // position strip
    let ps = Rect::from_min_size(pos2(r.left(), y), vec2(r.width(), 26.0));
    p.rect_filled(ps, 3.0, rgb(0x06080c));
    outline(&p, ps, 3.0, white(0.06));
    let tr = text(
        &p,
        pos2(ps.left() + 8.0, ps.center().y),
        Align2::LEFT_CENTER,
        &decks::step_str(d.pos),
        mono(13.0),
        dc,
        0.08,
    );
    let extra = if d.rate == 2.0 {
        " · DOUBLE TIME"
    } else if d.rate == 0.5 {
        " · HALF TIME"
    } else {
        ""
    };
    text(
        &p,
        pos2(tr.right() + 10.0, ps.center().y + 1.0),
        Align2::LEFT_CENTER,
        &format!("BEAT {} FROM 0{extra}", d.pos as u32),
        mono(7.0),
        INK_DIM,
        0.14,
    );
    y += 34.0;

    // loop section
    let ls = Rect::from_min_size(pos2(r.left(), y), vec2(r.width(), 108.0));
    outline(&p, ls, 4.0, white(0.07));
    let k_loop = format!("deck.{}.loopsz", d.id);
    let li = ((app.bus.num(&k_loop, 0.4) * 10.0).round() as usize).min(10);
    let (size, label) = LOOP_SIZES[li];
    let looping = d.loop_len > 0.0;
    let lc = if looping {
        Channel::Life
    } else {
        Channel::Cool
    };
    util::row(
        ui,
        Rect::from_min_size(pos2(ls.left() + 6.0, ls.top() + 4.0), vec2(116.0, 78.0)),
        ("loopk", right),
        |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let before = li;
            util::knob(
                ui,
                &mut app.bus,
                &k_loop,
                Knob::new(44.0, lc)
                    .variant(KnobVariant::Dotted)
                    .label("LOOP LEN")
                    .label_size(7.0)
                    .default_value(0.4)
                    .show_value(false),
            );
            let after = ((app.bus.num(&k_loop, 0.4) * 10.0).round() as usize).min(10);
            if after != before {
                app.decks.decks[idx].resize_loop(LOOP_SIZES[after].0);
            }
            widgets::readout_box(ui, Some("BEATS"), label, lc, 46.0);
        },
    );
    let bx = ls.left() + 126.0;
    let bw = ls.right() - 8.0 - bx;
    if deck_btn(
        ui,
        Rect::from_min_size(pos2(bx, ls.top() + 6.0), vec2(bw, 28.0)),
        ("loop", right),
        if looping { "EXIT" } else { "LOOP" },
        SIGNAL_LIFE,
        looping,
        9.0,
    ) {
        app.decks.decks[idx].toggle_loop(size);
    }
    for (k, (l, dv)) in [("½", -1i32), ("×2", 1)].into_iter().enumerate() {
        let b = Rect::from_min_size(
            pos2(bx + k as f32 * (bw / 2.0 + 1.0), ls.top() + 38.0),
            vec2(bw / 2.0 - 1.0, 20.0),
        );
        if deck_btn(ui, b, ("lsz", right, k), l, SIGNAL_COOL, false, 8.0) {
            let ni = (li as i32 + dv).clamp(0, 10) as usize;
            app.bus.set_num(&k_loop, ni as f32 / 10.0);
            app.decks.decks[idx].resize_loop(LOOP_SIZES[ni].0);
        }
    }
    for (k, (rate, l)) in [(0.5, "HALF"), (1.0, "1×"), (2.0, "DOUBLE")]
        .into_iter()
        .enumerate()
    {
        let b = Rect::from_min_size(
            pos2(bx + k as f32 * (bw / 3.0 + 1.0), ls.top() + 62.0),
            vec2(bw / 3.0 - 2.0, 20.0),
        );
        if deck_btn(
            ui,
            b,
            ("rate", right, k),
            l,
            SIGNAL_COOL,
            d.rate == rate,
            7.5,
        ) {
            app.decks.decks[idx].rate = rate;
        }
    }
    let secs = size * 60.0 / bpm.max(1.0) as f64;
    let info = if looping {
        format!(
            "LOOP {label} · {} · @{}",
            decks::timecode(secs),
            decks::timecode(d.loop_start * 60.0 / bpm.max(1.0) as f64)
        )
    } else {
        format!("{label} beat = {:.0} ms", secs * 1000.0)
    };
    text(
        &p,
        pos2(ls.left() + 8.0, ls.bottom() - 10.0),
        Align2::LEFT_CENTER,
        &info,
        mono(8.0),
        INK_MID,
        0.06,
    );
    y += 116.0;

    // transport
    let bw = (r.width() - 12.0) / 4.0;
    let tb = |k: usize, y: f32, h: f32| {
        Rect::from_min_size(pos2(r.left() + k as f32 * (bw + 4.0), y), vec2(bw, h))
    };
    if deck_btn(
        ui,
        tb(0, y, 38.0),
        ("play", right),
        if d.playing { "PAUSE" } else { "PLAY" },
        SIGNAL_LIFE,
        d.playing,
        10.0,
    ) {
        app.decks.decks[idx].playing = !d.playing;
    }
    if deck_btn(
        ui,
        tb(1, y, 38.0),
        ("zero", right),
        "|< 0",
        INK_MID,
        false,
        10.0,
    ) {
        app.decks.decks[idx].to_zero();
    }
    if deck_btn(
        ui,
        tb(2, y, 38.0),
        ("cue", right),
        "CUE",
        SIGNAL_AMBER,
        false,
        10.0,
    ) {
        let dk = &mut app.decks.decks[idx];
        if dk.playing {
            dk.pos = dk.cue;
            dk.playing = false;
        } else {
            dk.cue = dk.pos;
        }
    }
    if deck_btn(
        ui,
        tb(3, y, 38.0),
        ("sync", right),
        "SYNC",
        rgb(0x5b8cff),
        d.sync,
        10.0,
    ) {
        let dk = &mut app.decks.decks[idx];
        dk.sync = !dk.sync;
        if dk.sync {
            dk.pos = dk.pos.floor() + (app.engine.abs as f64 / 4.0).fract();
        }
    }
    y += 46.0;
    // quantum event pads
    for (k, (name, glyph, qch)) in QEVENTS.iter().enumerate() {
        let b = tb(k, y, r.bottom() - y);
        let qc = chc(qch);
        let resp = ui
            .interact(b, Id::unique(("qev", right, k)), Sense::click())
            .on_hover_text(format!("Quantum event · {name}"))
            .on_hover_cursor(CursorIcon::PointingHand);
        let fired = app
            .decks
            .last_event
            .is_some_and(|(dk, n, at)| dk == d.id && n == *name && app.engine.time - at < 0.35);
        if fired {
            glow_rect(&p, b, 4.0, 12.0, alpha(qc, 0.5));
        }
        p.rect_filled(
            b,
            4.0,
            if fired {
                mix(qc, rgb(0x0a0e15), 0.3)
            } else {
                rgb(0x0a0e15)
            },
        );
        outline(
            &p,
            b,
            4.0,
            alpha(qc, if resp.hovered() { 0.9 } else { 0.45 }),
        );
        widgets::glyph_tile_at(
            &p,
            Rect::from_center_size(pos2(b.center().x, b.center().y - 6.0), Vec2::splat(28.0)),
            glyph,
            qc,
            false,
            false,
        );
        text(
            &p,
            pos2(b.center().x, b.bottom() - 9.0),
            Align2::CENTER_CENTER,
            name,
            mono(6.5),
            qc,
            0.14,
        );
        if resp.clicked() {
            app.decks.last_event = Some((d.id, name, app.engine.time));
            app.core_log
                .push_front(format!("qevent  deck {} · {name}", d.id));
            app.core_log.truncate(6);
        }
    }
    let _ = icons::paint;
}

fn mixer(app: &mut ForgeApp, ui: &mut Ui, g: &Genesis, r: Rect) {
    let p = ui.painter().clone();
    grad_v(&p, r, 8.0, rgb(0x0a0e15), rgb(0x070a10));
    outline(&p, r, 8.0, alpha(SIGNAL_COOL, 0.12));
    let range = app.decks.bank_range();
    let master_w = (r.width() * 0.3).clamp(110.0, 190.0);
    let cw = (r.width() - master_w - 12.0) / 4.0;
    let strip_h = r.height() - 150.0;
    for (n, di) in range.enumerate() {
        let d = app.decks.decks[di].clone();
        let ch = deck_ch(g, d.mythos);
        let (c, dc) = (util::ch(ch), chc(ch));
        let sr = Rect::from_min_size(
            pos2(r.left() + 6.0 + n as f32 * cw, r.top() + 8.0),
            vec2(cw, strip_h),
        );
        if n > 0 {
            p.line_segment(
                [sr.left_top(), sr.left_bottom()],
                Stroke::new(1.0, white(0.05)),
            );
        }
        text(
            &p,
            pos2(sr.center().x, sr.top() + 10.0),
            Align2::CENTER_CENTER,
            &format!("CHANNEL {}", n + 1),
            mono(7.0),
            INK_MID,
            0.14,
        );
        text(
            &p,
            pos2(sr.center().x, sr.top() + 23.0),
            Align2::CENTER_CENTER,
            &d.id.to_string(),
            display(11.0),
            dc,
            0.0,
        );
        let key = |k: &str| format!("deck.{}.{k}", d.id);
        let mut yy = sr.top() + 32.0;
        let k = |ui: &mut Ui,
                 bus: &mut rack_core::Bus,
                 name: &str,
                 label: &str,
                 variant: KnobVariant,
                 bip: bool,
                 def: f32,
                 yy: &mut f32| {
            util::centered(
                ui,
                Rect::from_min_size(pos2(sr.left(), *yy), vec2(cw, 40.0)),
                ("mixk", di, name),
                0.0,
                |ui| {
                    util::knob(
                        ui,
                        bus,
                        &key(name),
                        Knob {
                            size: 26.0,
                            ch: c,
                            variant,
                            label: Some(label),
                            label_size: 6.5,
                            bipolar: bip,
                            default: def,
                            show_value: false,
                            color: None,
                        },
                    );
                },
            );
            *yy += 41.0;
        };
        k(
            ui,
            &mut app.bus,
            "gain",
            "GAIN",
            KnobVariant::Arc,
            false,
            0.7,
            &mut yy,
        );
        yy += 14.0;
        let eq_top = yy;
        k(
            ui,
            &mut app.bus,
            "eqs",
            "HIGH EXPO",
            KnobVariant::Arc,
            true,
            0.5,
            &mut yy,
        );
        k(
            ui,
            &mut app.bus,
            "eqw",
            "MID CAUSAL",
            KnobVariant::Arc,
            true,
            0.5,
            &mut yy,
        );
        k(
            ui,
            &mut app.bus,
            "eqm",
            "LOW GRAVITY",
            KnobVariant::Arc,
            true,
            0.5,
            &mut yy,
        );
        // channel meter beside the EQ
        let lit = (app.decks.levels[di] * 16.0).round() as usize;
        for i in 0..16 {
            let seg = Rect::from_min_size(
                pos2(sr.left() + 6.0, eq_top + 116.0 - i as f32 * 7.4),
                vec2(4.0, 6.0),
            );
            p.rect_filled(seg, 1.0, if i < lit { dc } else { white(0.06) });
        }
        yy += 6.0;
        k(
            ui,
            &mut app.bus,
            "filt",
            "FILTER",
            KnobVariant::Ringed,
            true,
            0.5,
            &mut yy,
        );
        let nb = Rect::from_center_size(pos2(sr.center().x, yy + 12.0), vec2(22.0, 16.0));
        outline(&p, nb, 2.0, SIGNAL_AMBER);
        text(
            &p,
            nb.center(),
            Align2::CENTER_CENTER,
            &(n + 1).to_string(),
            mono(8.0),
            SIGNAL_AMBER,
            0.0,
        );
        yy += 24.0;
        let fh = (sr.bottom() - yy - 18.0).max(50.0);
        util::centered(
            ui,
            Rect::from_min_size(pos2(sr.left(), yy), vec2(cw, fh + 16.0)),
            ("mixf", di),
            0.0,
            |ui| {
                util::fader(
                    ui,
                    &mut app.bus,
                    &key("vol"),
                    0.8,
                    fh,
                    22.0,
                    c,
                    Some("MASS"),
                    false,
                );
            },
        );
    }
    // master
    let mr = Rect::from_min_max(
        pos2(r.right() - master_w - 6.0, r.top() + 8.0),
        pos2(r.right() - 6.0, r.top() + 8.0 + strip_h),
    );
    p.line_segment(
        [mr.left_top(), mr.left_bottom()],
        Stroke::new(1.0, white(0.05)),
    );
    text(
        &p,
        pos2(mr.center().x, mr.top() + 10.0),
        Align2::CENTER_CENTER,
        "MASTER",
        mono(7.0),
        INK_MID,
        0.14,
    );
    let mut yy = mr.top() + 22.0;
    for (key, label, c, variant, def) in [
        ("forge.main", "GAIN", Channel::Bone, KnobVariant::Arc, 0.78),
        (
            "myth.physics",
            "PHYSICS · MYTH",
            Channel::Myth,
            KnobVariant::Ringed,
            0.5,
        ),
        ("deck.booth", "BOOTH", Channel::Amber, KnobVariant::Arc, 0.6),
    ] {
        util::centered(
            ui,
            Rect::from_min_size(pos2(mr.left(), yy), vec2(master_w, 48.0)),
            ("mk", key),
            0.0,
            |ui| {
                util::knob(
                    ui,
                    &mut app.bus,
                    key,
                    Knob::new(30.0, c)
                        .variant(variant)
                        .label(label)
                        .label_size(6.5)
                        .default_value(def)
                        .show_value(false),
                );
            },
        );
        if key == "forge.main" {
            for (i, (l, v)) in [("L", app.engine.rms.l), ("R", app.engine.rms.r)]
                .into_iter()
                .enumerate()
            {
                let vr = Rect::from_min_size(
                    pos2(mr.left() + 8.0, yy + 50.0 + i as f32 * 9.0),
                    vec2(master_w - 16.0, 6.0),
                );
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt(Id::unique(("mvu", i)))
                        .max_rect(vr),
                );
                widgets::vu(&mut child, v, vr.width(), 6.0, Some(l));
            }
            yy += 22.0;
        }
        yy += 52.0;
    }
    let mi = app.bus.num("mix.intensity", 0.0);
    text(
        &p,
        pos2(mr.center().x, mr.bottom() - 34.0),
        Align2::CENTER_CENTER,
        "NARRATIVE OUT",
        mono(7.0),
        INK_DIM,
        0.14,
    );
    let bar = Rect::from_center_size(
        pos2(mr.center().x, mr.bottom() - 18.0),
        vec2(master_w - 24.0, 6.0),
    );
    p.rect_filled(bar, 3.0, white(0.06));
    p.rect_filled(
        Rect::from_min_size(bar.min, vec2(bar.width() * mi, 6.0)),
        3.0,
        SIGNAL_MYTH,
    );

    // crossfader · timeline entropy
    let xr = Rect::from_min_max(
        pos2(r.left() + 40.0, r.bottom() - 132.0),
        pos2(r.right() - master_w - 20.0, r.bottom() - 60.0),
    );
    text(
        &p,
        pos2(xr.left() + 40.0, xr.top() + 8.0),
        Align2::CENTER_CENTER,
        "TIMELINE A",
        mono(8.0),
        SIGNAL_MYTH,
        0.14,
    );
    text(
        &p,
        pos2(xr.center().x, xr.top() + 8.0),
        Align2::CENTER_CENTER,
        "ENTROPY",
        mono(8.0),
        INK_MID,
        0.14,
    );
    text(
        &p,
        pos2(xr.right() - 40.0, xr.top() + 8.0),
        Align2::CENTER_CENTER,
        "TIMELINE B",
        mono(8.0),
        SIGNAL_MYTH,
        0.14,
    );
    let track = Rect::from_center_size(
        pos2(xr.center().x, xr.top() + 36.0),
        vec2(xr.width() - 40.0, 4.0),
    );
    let resp = ui
        .interact(
            track.expand2(vec2(10.0, 16.0)),
            Id::unique("xfader"),
            Sense::click_and_drag(),
        )
        .on_hover_cursor(CursorIcon::ResizeHorizontal)
        .on_hover_text("Crossfader · double-click to centre");
    let mut xf = app.bus.num("deck.xf", 0.5);
    if resp.is_pointer_button_down_on() {
        if let Some(pos) = resp.interact_pointer_pos() {
            xf = ((pos.x - track.left()) / track.width()).clamp(0.0, 1.0);
            app.bus.set_num("deck.xf", xf);
        }
    }
    if resp.double_clicked() {
        xf = 0.5;
        app.bus.set_num("deck.xf", xf);
    }
    p.rect_filled(track, 2.0, rgb(0x04060a));
    let cap = Rect::from_center_size(
        pos2(track.left() + track.width() * xf, track.center().y),
        vec2(24.0, 30.0),
    );
    paint::shadow(&p, cap, 3.0, 2.0, 6.0, black(0.7));
    grad_v(&p, cap, 3.0, rgb(0x2a2f3a), rgb(0x0e1117));
    outline(&p, cap, 3.0, black(0.7));
    p.line_segment(
        [
            pos2(cap.center().x, cap.top() + 5.0),
            pos2(cap.center().x, cap.bottom() - 5.0),
        ],
        Stroke::new(2.0, SIGNAL_MYTH),
    );
    for (side, gain) in [(0u8, decks::xf_gain(0, xf)), (2, decks::xf_gain(2, xf))] {
        let half = track.width() / 2.0 - 6.0;
        let x0 = if side == 0 {
            track.center().x - 3.0 - half * gain
        } else {
            track.center().x + 3.0
        };
        p.rect_filled(
            Rect::from_min_size(pos2(x0, track.bottom() + 22.0), vec2(half * gain, 3.0)),
            1.5,
            SIGNAL_MYTH,
        );
    }
}

fn fx_rack(app: &mut ForgeApp, ui: &mut Ui, r: Rect) {
    let p = ui.painter().clone();
    panel(&p, r);
    let head = Rect::from_min_size(
        pos2(r.left() + 10.0, r.top() + 8.0),
        vec2(r.width() - 20.0, 24.0),
    );
    row(ui, head, "fxr-head", |ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        let (lr, _) = ui.allocate_exact_size(vec2(296.0, 20.0), Sense::hover());
        engrave(
            ui.painter(),
            lr.left_center(),
            Align2::LEFT_CENTER,
            "FX RACK · CLICK AN EFFECT TO LOAD A SLOT",
            8.0,
            INK_MID,
        );
        for f in FX {
            let (_, meaning) = f.parts();
            let fc = chc(f.ch);
            let w = measure(ui.painter(), meaning, mono(8.0), 0.04).x + 30.0;
            let (cr, resp) = ui.allocate_exact_size(vec2(w.ceil(), 20.0), Sense::click());
            let pp = ui.painter();
            pp.rect_filled(cr, 10.0, rgb(0x0a0e15));
            outline(
                pp,
                cr,
                10.0,
                alpha(fc, if resp.hovered() { 0.9 } else { 0.5 }),
            );
            icons::paint(pp, f.glyph, pos2(cr.left() + 11.0, cr.center().y), 10.0, fc);
            text(
                pp,
                pos2(cr.left() + 21.0, cr.center().y),
                Align2::LEFT_CENTER,
                meaning,
                mono(8.0),
                INK,
                0.04,
            );
            if resp
                .on_hover_text(f.name)
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                if let Some(slot) = app.decks.fx.iter_mut().find(|s| s.is_none()) {
                    *slot = Some(f.key.to_owned());
                }
            }
        }
    });
    let sw = (r.width() - 20.0 - 24.0) / 4.0;
    for i in 0..4 {
        let sr = Rect::from_min_size(
            pos2(r.left() + 10.0 + i as f32 * (sw + 8.0), r.top() + 40.0),
            vec2(sw, r.height() - 50.0),
        );
        p.rect_filled(sr, 6.0, rgb(0x080b11));
        outline(&p, sr, 6.0, white(0.06));
        match app.decks.fx[i]
            .clone()
            .and_then(|k| rack_core::catalog::fx(&k))
        {
            None => {
                text(
                    &p,
                    sr.center(),
                    Align2::CENTER_CENTER,
                    "EMPTY MODULE",
                    mono(7.5),
                    INK_DIM,
                    0.16,
                );
            }
            Some(f) => {
                let fc = chc(f.ch);
                let (technique, meaning) = f.parts();
                p.line_segment(
                    [
                        pos2(sr.left() + 6.0, sr.top() + 1.0),
                        pos2(sr.right() - 6.0, sr.top() + 1.0),
                    ],
                    Stroke::new(2.0, alpha(fc, 0.6)),
                );
                widgets::glyph_tile_at(
                    &p,
                    Rect::from_min_size(
                        pos2(sr.left() + 12.0, sr.center().y - 18.0),
                        Vec2::splat(36.0),
                    ),
                    f.glyph,
                    fc,
                    false,
                    false,
                );
                text_clipped(
                    &p,
                    pos2(sr.left() + 58.0, sr.center().y - 8.0),
                    Align2::LEFT_CENTER,
                    meaning,
                    display(11.0),
                    INK,
                    0.1,
                    sw - 150.0,
                );
                text_clipped(
                    &p,
                    pos2(sr.left() + 58.0, sr.center().y + 8.0),
                    Align2::LEFT_CENTER,
                    technique,
                    mono(8.0),
                    fc,
                    0.06,
                    sw - 150.0,
                );
                util::row_rtl(ui, sr.shrink2(vec2(8.0, 0.0)), ("fxslot", i), |ui| {
                    if widgets::tool_btn(ui, "close", None, false, None)
                        .on_hover_text("Unload")
                        .clicked()
                    {
                        app.decks.fx[i] = None;
                    }
                    util::knob(
                        ui,
                        &mut app.bus,
                        &format!("deck.fxrack.{i}"),
                        Knob::new(34.0, util::ch(f.ch))
                            .label("WET")
                            .label_size(7.0)
                            .default_value(0.5)
                            .show_value(false),
                    );
                });
            }
        }
    }
}
