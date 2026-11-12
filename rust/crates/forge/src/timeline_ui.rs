//! MAIN TIMELINE face: preview, track inspector, transport bar, clip lanes and
//! the concurrent alternate timelines.

use crate::app::ForgeApp;
use crate::util::{self, chc};
use egui::{pos2, vec2, Align2, CursorIcon, Id, Rect, Sense, Stroke, Ui, Vec2};
use instrument_kit::paint::*;
use instrument_kit::theme::*;
use instrument_kit::widgets::{self, Knob, KnobVariant};
use rack_core::timeline::{self, curve, Timeline, CURVES, LEN};

pub const HEIGHT: f32 = 490.0;

fn tc_color(ch: &str) -> egui::Color32 {
    chc(ch)
}

/// Star field + the glyph layers of every active track.
fn preview(ui: &mut Ui, r: Rect, tl: &Timeline, time: f32) {
    let g = mat_screen(ui.painter(), r, 6.0);
    let mut seed = 0x2545_F491u32;
    for _ in 0..70 {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        let (x, y) = (
            (seed & 0xffff) as f32 / 65535.0,
            ((seed >> 16) & 0xffff) as f32 / 65535.0,
        );
        g.circle_filled(
            pos2(r.left() + x * r.width(), r.top() + y * r.height()),
            0.7,
            white(0.12 + (seed % 7) as f32 * 0.03),
        );
    }
    for tr in &tl.tracks {
        let v = tr.value(tl.t);
        if v <= 0.001 {
            continue;
        }
        let c = chc(&tr.ch);
        let size = r.height().min(r.width()) * (0.28 + v * 0.3);
        let rect = Rect::from_center_size(r.center(), Vec2::splat(size));
        let spin = time * tr.spin * 40.0;
        // halo
        radial(
            &g,
            r.center(),
            size * 0.8,
            alpha(c, 0.22 * v),
            alpha(c, 0.0),
            Vec2::ZERO,
        );
        widgets::glyph_tile_at(
            &g,
            rect.translate(vec2((spin.to_radians().sin()) * size * 0.05, 0.0)),
            &tr.glyph,
            c,
            true,
            false,
        );
    }
    text(
        &g,
        r.left_top() + vec2(10.0, 12.0),
        Align2::LEFT_CENTER,
        "PROJECT ENGINE",
        mono(7.0),
        INK_DIM,
        0.16,
    );
    text(
        &g,
        r.left_top() + vec2(10.0, 26.0),
        Align2::LEFT_CENTER,
        "MAIN TIMELINE",
        display(11.0),
        INK,
        0.1,
    );
    text(
        &g,
        r.right_top() + vec2(-10.0, 12.0),
        Align2::RIGHT_CENTER,
        &timeline::timecode(tl.t),
        mono(10.0),
        SIGNAL_COOL,
        0.1,
    );
    let k = alpha(SIGNAL_AMBER, 0.55);
    for (c0, dx, dy) in [
        (r.left_bottom() + vec2(5.0, -5.0), 1.0, -1.0),
        (r.right_bottom() + vec2(-5.0, -5.0), -1.0, -1.0),
    ] {
        g.line_segment([c0, c0 + vec2(9.0 * dx, 0.0)], Stroke::new(1.0, k));
        g.line_segment([c0, c0 + vec2(0.0, 9.0 * dy)], Stroke::new(1.0, k));
    }
}

pub fn face(app: &mut ForgeApp, ui: &mut Ui, r: Rect, prefix: &str) {
    let mut tl = app.timelines.remove(prefix).unwrap_or_default();
    let p = ui.painter().clone();
    let time = app.engine.time as f32;
    let sel_id = app
        .fui
        .track
        .clone()
        .filter(|id| tl.tracks.iter().any(|t| &t.id == id))
        .or_else(|| tl.tracks.first().map(|t| t.id.clone()));
    app.fui.track = sel_id.clone();

    // ---- top: preview + inspector ----
    let top_h = 184.0;
    let insp_w = 214.0;
    let pv = Rect::from_min_size(r.min, vec2(r.width() - insp_w - 8.0, top_h));
    preview(ui, pv, &tl, time);
    let ins = Rect::from_min_size(pos2(r.right() - insp_w, r.top()), vec2(insp_w, top_h));
    if let Some(ti) = sel_id
        .as_ref()
        .and_then(|id| tl.tracks.iter().position(|t| &t.id == id))
    {
        let mut remove = false;
        {
            let tr = &mut tl.tracks[ti];
            text(
                &p,
                ins.left_top() + vec2(2.0, 8.0),
                Align2::LEFT_CENTER,
                &format!("TRAITS · TRACK {}", ti + 1),
                mono(7.5),
                INK_DIM,
                0.14,
            );
            let nr = Rect::from_min_size(ins.left_top() + vec2(0.0, 18.0), vec2(insp_w, 24.0));
            util::row(ui, nr, ("aft-name", &tr.id), |ui| {
                widgets::text_field(ui, &mut tr.name, "Track name", insp_w, 24.0, GOLD);
            });
            let c = util::ch(&tr.ch);
            util::centered(
                ui,
                Rect::from_min_size(ins.left_top() + vec2(0.0, 46.0), vec2(insp_w, 74.0)),
                ("aft-knobs", &tr.id),
                0.0,
                |ui| {
                    let mut v = tr.inten;
                    if Knob::new(40.0, c)
                        .variant(KnobVariant::Arc)
                        .label("INTENSITY")
                        .label_size(7.0)
                        .default_value(0.7)
                        .show(ui, &mut v)
                        .changed()
                    {
                        tr.inten = v;
                    }
                    let mut v = tr.hue;
                    if Knob::new(40.0, c)
                        .variant(KnobVariant::Dotted)
                        .label("HUE")
                        .label_size(7.0)
                        .default_value(0.0)
                        .show(ui, &mut v)
                        .changed()
                    {
                        tr.hue = v;
                    }
                    let mut v = (tr.spin + 1.0) / 2.0;
                    if Knob::new(40.0, c)
                        .variant(KnobVariant::Ringed)
                        .label("SPIN")
                        .label_size(7.0)
                        .default_value(0.5)
                        .show(ui, &mut v)
                        .changed()
                    {
                        tr.spin = v * 2.0 - 1.0;
                    }
                },
            );
            // curve selector
            let sw = insp_w / CURVES.len() as f32;
            for (k, name) in CURVES.iter().enumerate() {
                let b = Rect::from_min_size(
                    ins.left_top() + vec2(k as f32 * sw, 126.0),
                    vec2(sw - 2.0, 18.0),
                );
                let on = tr.curve == *name;
                let resp = ui
                    .interact(b, Id::unique(("aft-curve", &tr.id, k)), Sense::click())
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
                        alpha(SIGNAL_COOL, 0.6)
                    } else {
                        white(0.07)
                    },
                );
                text(
                    &p,
                    b.center(),
                    Align2::CENTER_CENTER,
                    &name.to_uppercase(),
                    mono(7.0),
                    if on { SIGNAL_COOL } else { INK_DIM },
                    0.1,
                );
                if resp.clicked() {
                    tr.curve = (*name).into();
                }
            }
            let cr =
                Rect::from_min_size(ins.left_top() + vec2(0.0, 148.0), vec2(insp_w - 70.0, 30.0));
            let pts: Vec<egui::Pos2> = (0..=40)
                .map(|i| {
                    pos2(
                        cr.left() + cr.width() * i as f32 / 40.0,
                        cr.bottom() - 2.0 - curve(&tr.curve, i as f32 / 40.0) * (cr.height() - 4.0),
                    )
                })
                .collect();
            polyline_glow(&p, pts, 1.4, c.color());
            let rm = Rect::from_min_size(
                pos2(ins.right() - 66.0, ins.top() + 152.0),
                vec2(66.0, 24.0),
            );
            if util::row(ui, rm, ("aft-rm", &tr.id), |ui| {
                widgets::cv_btn(ui, "REMOVE", false, true).clicked()
            }) {
                remove = true;
            }
        }
        if remove {
            let id = tl.tracks.remove(ti).id;
            app.fui.track = None;
            let _ = id;
        }
    }

    // ---- transport bar ----
    let mut y = r.top() + top_h + 8.0;
    let bar = Rect::from_min_size(pos2(r.left(), y), vec2(r.width(), 30.0));
    let running = if tl.link {
        app.engine.playing
    } else {
        tl.playing
    };
    util::row(ui, bar, ("aft-bar", prefix), |ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        if widgets::pad(
            ui,
            widgets::Face::Icon("loop"),
            instrument_kit::Channel::Cool,
            false,
            26.0,
        )
        .on_hover_text("To start")
        .clicked()
        {
            tl.t = 0.0;
        }
        if widgets::pad(
            ui,
            widgets::Face::Icon(if running { "pause" } else { "play" }),
            instrument_kit::Channel::Life,
            running,
            26.0,
        )
        .on_hover_text("Play / pause")
        .clicked()
        {
            if tl.link {
                app.engine.toggle_play(&mut app.bus);
            } else {
                tl.playing = !tl.playing;
            }
        }
        if widgets::pad(
            ui,
            widgets::Face::Icon("sync"),
            instrument_kit::Channel::Cool,
            false,
            26.0,
        )
        .on_hover_text("Next frame")
        .clicked()
        {
            tl.t = (tl.t + 1.0 / timeline::FPS).min(LEN);
        }
        let (tr, _) = ui.allocate_exact_size(vec2(220.0, 24.0), Sense::hover());
        let t = readout(
            ui.painter(),
            pos2(tr.left(), tr.center().y),
            Align2::LEFT_CENTER,
            &timeline::timecode(tl.t),
            17.0,
            SIGNAL_COOL,
        );
        text(
            ui.painter(),
            pos2(t.right() + 10.0, tr.center().y + 2.0),
            Align2::LEFT_CENTER,
            &format!("/ {}", timeline::timecode(LEN)),
            mono(11.0),
            INK_DIM,
            0.04,
        );
    });
    util::row_rtl(ui, bar, ("aft-bar-r", prefix), |ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        if widgets::cv_btn(ui, "+ TRACK", false, false).clicked() {
            app.fui.track = Some(tl.add_track());
        }
        if widgets::gate_btn(ui, "LINK TRANSPORT", instrument_kit::Channel::Life, tl.link)
            .on_hover_text("Follow the rack transport")
            .clicked()
        {
            tl.link = !tl.link;
        }
        if widgets::gate_btn(ui, "LOOP", instrument_kit::Channel::Life, tl.looping).clicked() {
            tl.looping = !tl.looping;
        }
    });
    y += 38.0;

    // ---- tracks ----
    let head_w = 150.0;
    let lane_x0 = r.left() + head_w + 6.0;
    let lane_w = r.width() - head_w - 6.0;
    let sec_x = |s: f32| lane_x0 + s / LEN * lane_w;
    let ruler = Rect::from_min_size(pos2(r.left(), y), vec2(r.width(), 18.0));
    p.rect_filled(ruler, 2.0, black(0.25));
    engrave(
        &p,
        pos2(r.left() + 6.0, ruler.center().y),
        Align2::LEFT_CENTER,
        "MASTER TRACKS",
        8.0,
        INK_DIM,
    );
    for i in 0..=(LEN as i32 / 2) {
        text(
            &p,
            pos2(sec_x(i as f32 * 2.0) + 3.0, ruler.center().y),
            Align2::LEFT_CENTER,
            &format!("{}s", i * 2),
            mono(7.0),
            white(0.3),
            0.0,
        );
    }
    let scrub = ui
        .interact(
            Rect::from_min_max(
                pos2(lane_x0, ruler.top()),
                pos2(lane_x0 + lane_w, ruler.bottom()),
            ),
            Id::unique(("aft-scrub", prefix)),
            Sense::click_and_drag(),
        )
        .on_hover_cursor(CursorIcon::ResizeHorizontal);
    if scrub.is_pointer_button_down_on() {
        if let Some(pos) = scrub.interact_pointer_pos() {
            tl.t = ((pos.x - lane_x0) / lane_w * LEN).clamp(0.0, LEN);
        }
    }
    y += 20.0;
    let tracks_top = y;
    let ids: Vec<String> = tl.tracks.iter().map(|t| t.id.clone()).collect();
    let selected = app.fui.track.clone();
    for id in &ids {
        let row = Rect::from_min_size(pos2(r.left(), y), vec2(r.width(), 32.0));
        if row.bottom() > r.bottom() - 86.0 {
            break;
        }
        let Some(ti) = tl.tracks.iter().position(|t| &t.id == id) else {
            continue;
        };
        let tc = tc_color(&tl.tracks[ti].ch);
        let on = selected.as_deref() == Some(id.as_str());
        p.rect_filled(row, 3.0, if on { alpha(tc, 0.07) } else { black(0.18) });
        if on {
            p.line_segment([row.left_top(), row.left_bottom()], Stroke::new(2.0, tc));
        }
        let head = Rect::from_min_size(row.min, vec2(head_w, 32.0));
        let hr = ui
            .interact(head, Id::unique(("aft-head", id)), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        if hr.clicked() {
            app.fui.track = Some(id.clone());
        }
        let v = tl.tracks[ti].value(tl.t);
        glow_dot(
            &p,
            pos2(head.left() + 10.0, head.center().y),
            3.0,
            alpha(tc, 0.25 + v * 0.75),
        );
        text_clipped(
            &p,
            pos2(head.left() + 22.0, head.center().y - 6.0),
            Align2::LEFT_CENTER,
            &tl.tracks[ti].name,
            mono(9.0),
            INK,
            0.08,
            head_w - 52.0,
        );
        text(
            &p,
            pos2(head.left() + 22.0, head.center().y + 7.0),
            Align2::LEFT_CENTER,
            &format!("GLYPH · {}", tl.tracks[ti].curve.to_uppercase()),
            mono(7.0),
            INK_DIM,
            0.1,
        );
        let mb =
            Rect::from_center_size(pos2(head.right() - 12.0, head.center().y), vec2(18.0, 16.0));
        let mr = ui
            .interact(mb, Id::unique(("aft-mute", id)), Sense::click())
            .on_hover_text("Mute")
            .on_hover_cursor(CursorIcon::PointingHand);
        let muted = tl.tracks[ti].mute;
        p.rect_filled(mb, 2.0, if muted { SIGNAL_AMBER } else { rgb(0x1a1e25) });
        text(
            &p,
            mb.center(),
            Align2::CENTER_CENTER,
            "M",
            mono(8.0),
            if muted { egui::Color32::BLACK } else { INK_DIM },
            0.0,
        );
        if mr.clicked() {
            tl.tracks[ti].mute = !muted;
        }
        // lane
        let lane = Rect::from_min_size(pos2(lane_x0, y), vec2(lane_w, 32.0));
        p.rect_filled(lane, 2.0, black(0.2));
        let lr = ui.interact(lane, Id::unique(("aft-lane", id)), Sense::click());
        if lr.double_clicked() {
            if let Some(pos) = lr.interact_pointer_pos() {
                tl.add_clip(id, (pos.x - lane_x0) / lane_w * LEN);
                app.fui.track = Some(id.clone());
            }
        }
        let clips = tl.tracks[ti].clips.clone();
        let cname = tl.tracks[ti].curve.clone();
        for c in clips {
            let cr = Rect::from_min_size(
                pos2(sec_x(c.start), y + 3.0),
                vec2(c.len / LEN * lane_w, 26.0),
            );
            let body = ui
                .interact(
                    cr,
                    Id::unique(("aft-clip", id, c.id)),
                    Sense::click_and_drag(),
                )
                .on_hover_text("Drag to move · right edge to resize · double-click to delete")
                .on_hover_cursor(CursorIcon::Grab);
            let edge = Rect::from_min_max(pos2(cr.right() - 7.0, cr.top()), cr.max);
            let er = ui
                .interact(edge, Id::unique(("aft-edge", id, c.id)), Sense::drag())
                .on_hover_cursor(CursorIcon::ResizeHorizontal);
            if er.dragged() {
                if let Some(pos) = er.interact_pointer_pos() {
                    tl.resize_clip(id, c.id, (pos.x - lane_x0) / lane_w * LEN - c.start);
                }
            } else if body.dragged() {
                tl.move_clip(id, c.id, c.start + body.drag_delta().x / lane_w * LEN);
            }
            if body.double_clicked() {
                tl.remove_clip(id, c.id);
                continue;
            }
            if body.clicked() || body.drag_started() {
                app.fui.track = Some(id.clone());
            }
            p.rect_filled(cr, 3.0, alpha(tc, 0.18));
            outline(&p, cr, 3.0, tc);
            let g = p.with_clip_rect(cr);
            let pts: Vec<egui::Pos2> = (0..=24)
                .map(|i| {
                    pos2(
                        cr.left() + cr.width() * i as f32 / 24.0,
                        cr.bottom() - 2.0 - curve(&cname, i as f32 / 24.0) * (cr.height() - 5.0),
                    )
                })
                .collect();
            polyline(&g, pts, 1.2, tc);
        }
        y += 36.0;
    }
    // playhead
    let ph = sec_x(tl.t);
    p.line_segment(
        [pos2(ph, ruler.top()), pos2(ph, y)],
        Stroke::new(2.0, SIGNAL_AMBER),
    );
    p.add(egui::Shape::convex_polygon(
        vec![
            pos2(ph - 5.0, ruler.top()),
            pos2(ph + 5.0, ruler.top()),
            pos2(ph, ruler.top() + 8.0),
        ],
        SIGNAL_AMBER,
        Stroke::NONE,
    ));
    let _ = tracks_top;

    // ---- concurrent alternate timelines ----
    y += 4.0;
    engrave(
        &p,
        pos2(r.left() + 6.0, y + 6.0),
        Align2::LEFT_CENTER,
        "CONCURRENT TIMELINES",
        8.0,
        INK_DIM,
    );
    y += 16.0;
    let alts = tl.alts.clone();
    for a in alts {
        if y + 20.0 > r.bottom() {
            break;
        }
        let row = Rect::from_min_size(pos2(r.left(), y), vec2(r.width(), 20.0));
        p.rect_filled(row, 2.0, black(0.15));
        text_clipped(
            &p,
            pos2(row.left() + 6.0, row.center().y),
            Align2::LEFT_CENTER,
            &a.name,
            mono(8.0),
            INK,
            0.08,
            head_w - 40.0,
        );
        let rb = Rect::from_min_size(
            pos2(row.left() + head_w - 34.0, row.top() + 2.0),
            vec2(32.0, 16.0),
        );
        let rr = ui
            .interact(rb, Id::unique(("aft-rate", &a.id)), Sense::click())
            .on_hover_text("Cycle the rate")
            .on_hover_cursor(CursorIcon::PointingHand);
        outline(&p, rb, 2.0, white(0.12));
        text(
            &p,
            rb.center(),
            Align2::CENTER_CENTER,
            &format!("×{}", a.rate),
            mono(8.0),
            SIGNAL_AMBER,
            0.0,
        );
        if rr.clicked() {
            tl.cycle_rate(&a.id);
        }
        let lane = Rect::from_min_size(pos2(lane_x0, y + 8.0), vec2(lane_w, 4.0));
        p.rect_filled(lane, 2.0, alpha(SIGNAL_MYTH, 0.15));
        let pos = timeline::wrap(tl.t * a.rate + a.ofs) / LEN;
        p.rect_filled(
            Rect::from_min_size(lane.min, vec2(lane_w * pos, 4.0)),
            2.0,
            alpha(SIGNAL_MYTH, 0.6),
        );
        let x = lane.left() + lane_w * pos;
        p.line_segment(
            [pos2(x, y + 2.0), pos2(x, y + 18.0)],
            Stroke::new(2.0, SIGNAL_MYTH),
        );
        y += 22.0;
    }

    app.timelines.insert(prefix.to_owned(), tl);
}
