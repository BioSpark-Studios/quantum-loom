//! Bottom drawer. This pass carries the CONSOLE tab — the headless core's
//! output stream; the performance keyboard, stage and browser tabs follow.

use crate::app::ForgeApp;
use crate::util::{self, chc, col, row};
use egui::{pos2, vec2, Align2, CursorIcon, Id, Rect, ScrollArea, Sense, Stroke, Ui};
use instrument_kit::icons;
use instrument_kit::paint::*;
use instrument_kit::theme::*;
use instrument_kit::widgets;
use rack_core::catalog::TARGETS;

const OPEN_H: f32 = 250.0;

pub fn height(app: &ForgeApp) -> f32 {
    if app.prefs.drawer_open {
        OPEN_H
    } else {
        0.0
    }
}

pub fn show(app: &mut ForgeApp, ui: &mut Ui, dr: Rect, body: Rect) {
    // handle pill, centred on the seam
    let hw = 112.0;
    let handle = Rect::from_min_size(
        pos2(body.center().x - hw / 2.0, body.bottom() - 30.0),
        vec2(hw, 30.0),
    );
    let resp = ui
        .interact(handle, Id::unique("drawer-handle"), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text("Drawer · console");
    let p = ui.painter().clone();
    shadow(&p, handle, 8.0, 0.0, 14.0, black(0.7));
    grad(
        &p,
        handle,
        egui::CornerRadius {
            nw: 8,
            ne: 8,
            sw: 0,
            se: 0,
        },
        rgb(0x1a1f28),
        rgb(0x10141a),
        true,
    );
    widgets::glyph_tile_at(
        &p,
        Rect::from_center_size(
            pos2(handle.left() + 18.0, handle.center().y),
            egui::Vec2::splat(16.0),
        ),
        "console",
        SIGNAL_MYTH,
        true,
        false,
    );
    text(
        &p,
        pos2(handle.left() + 34.0, handle.center().y),
        Align2::LEFT_CENTER,
        "DRAWER",
        mono(9.0),
        if resp.hovered() { INK } else { INK_MID },
        0.16,
    );
    icons::paint(
        &p,
        "down",
        pos2(handle.right() - 14.0, handle.center().y),
        10.0,
        INK_MID,
    );
    if resp.clicked() {
        app.prefs.drawer_open = !app.prefs.drawer_open;
    }
    if !app.prefs.drawer_open {
        return;
    }
    grad_v(&p, dr, 0.0, rgb(0x14181f), rgb(0x0b0d12));
    p.line_segment(
        [dr.left_top(), dr.right_top()],
        Stroke::new(1.0, black(1.0)),
    );
    p.line_segment(
        [
            dr.left_top() + vec2(0.0, 1.0),
            dr.right_top() + vec2(0.0, 1.0),
        ],
        Stroke::new(1.0, white(0.07)),
    );
    let tabs = Rect::from_min_size(dr.min, vec2(dr.width(), 32.0));
    row(ui, tabs.shrink2(vec2(12.0, 0.0)), "drawer-tabs", |ui| {
        widgets::tool_btn(ui, "console", Some("CONSOLE"), true, Some(SIGNAL_COOL));
        ui.add_enabled_ui(false, |ui| {
            for (i, l) in [
                ("keys", "PERFORM"),
                ("map", "STAGE"),
                ("browser", "BROWSER"),
                ("gear", "SETTINGS"),
            ] {
                widgets::tool_btn(ui, i, Some(l), false, None)
                    .on_disabled_hover_text("Arrives in a later pass");
            }
        });
    });
    console(
        app,
        ui,
        Rect::from_min_max(
            pos2(dr.left() + 12.0, tabs.bottom()),
            pos2(dr.right() - 12.0, dr.bottom() - 8.0),
        ),
    );
}

/// HEADLESS CORE · OUTPUT — the frame every adapter sees.
fn console(app: &mut ForgeApp, ui: &mut Ui, r: Rect) {
    let left_w = (r.width() * 0.34).clamp(240.0, 380.0);
    let left = Rect::from_min_size(r.min, vec2(left_w, r.height()));
    let right = Rect::from_min_max(pos2(left.right() + 12.0, r.top()), r.max);
    col(ui, left, "core-left", |ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        let (tr, _) = ui.allocate_exact_size(vec2(left_w, 14.0), Sense::hover());
        engrave(
            ui.painter(),
            tr.left_center(),
            Align2::LEFT_CENTER,
            "HEADLESS CORE · OUTPUT",
            9.0,
            INK_MID,
        );
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
            for (k, l, c) in TARGETS {
                let on = app.prefs.targets.get(*k).copied().unwrap_or(false);
                if widgets::chip(ui, l, on, util::ch(c)).clicked() {
                    app.prefs.targets.insert((*k).to_owned(), !on);
                }
            }
        });
        if app.prefs.targets.get("dmx").copied().unwrap_or(false) {
            let (dr, _) = ui.allocate_exact_size(vec2(left_w, 56.0), Sense::hover());
            let cw = (left_w - 28.0) / 8.0;
            for i in 0..8 {
                let v = app.core.dmx.get(i).copied().unwrap_or(0);
                let c = Rect::from_min_size(
                    pos2(dr.left() + i as f32 * (cw + 4.0), dr.top()),
                    vec2(cw, 56.0),
                );
                let p = ui.painter();
                p.rect_filled(c, 3.0, rgb(0x07090d));
                let h = 56.0 * v as f32 / 255.0;
                if h > 1.0 {
                    grad_v(
                        p,
                        Rect::from_min_max(pos2(c.left(), c.bottom() - h), c.max),
                        3.0,
                        alpha(rgb(0xffd88a), 0.85),
                        alpha(rgb(0xffffff), 0.85),
                    );
                }
                text(
                    p,
                    pos2(c.center().x, c.bottom() - 2.0),
                    Align2::CENTER_BOTTOM,
                    &v.to_string(),
                    mono(7.0),
                    if h > 12.0 {
                        egui::Color32::BLACK
                    } else {
                        INK_DIM
                    },
                    0.0,
                );
            }
        }
        for (i, l) in app.core_log.iter().enumerate() {
            let (lr, _) = ui.allocate_exact_size(vec2(left_w, 11.0), Sense::hover());
            text_clipped(
                ui.painter(),
                lr.left_center(),
                Align2::LEFT_CENTER,
                l,
                mono(9.0),
                alpha(SIGNAL_LIFE, if i == 0 { 1.0 } else { 0.5 }),
                0.0,
                left_w,
            );
        }
    });
    let g = mat_screen(ui.painter(), right, 5.0);
    let _ = g;
    col(ui, right.shrink2(vec2(8.0, 6.0)), "core-stream", |ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 3.0;
            if app.core.rows.is_empty() {
                ui.label(egui::RichText::new("Core idle — add or activate an instrument.").font(instrument_kit::theme::body(11.0)).color(INK_DIM));
            }
            let cols = ((ui.available_width() / 300.0).floor() as usize).max(1);
            let cw = (ui.available_width() - 16.0 * (cols as f32 - 1.0)) / cols as f32;
            for chunk in app.core.rows.chunks(cols) {
                let (lr, _) = ui.allocate_exact_size(vec2(ui.available_width(), 12.0), Sense::hover());
                for (i, rw) in chunk.iter().enumerate() {
                    let c = Rect::from_min_size(pos2(lr.left() + i as f32 * (cw + 16.0), lr.top()), vec2(cw, 12.0));
                    let p = ui.painter();
                    text_clipped(p, c.left_center(), Align2::LEFT_CENTER, &rw.addr, mono(9.0), INK_MID, 0.0, cw - 104.0);
                    let bar = Rect::from_min_size(pos2(c.right() - 96.0, c.center().y - 1.5), vec2(52.0, 3.0));
                    p.rect_filled(bar, 2.0, white(0.06));
                    p.rect_filled(Rect::from_min_size(bar.min, vec2(52.0 * rw.v, 3.0)), 2.0, chc(rw.ch));
                    text(p, c.right_center(), Align2::RIGHT_CENTER, &format!("{:.3}", rw.v), mono(9.0), SIGNAL_COOL, 0.0);
                }
            }
            ui.add_space(6.0);
            ui.label(egui::RichText::new("The core sees only addresses and numbers. Adapters turn the same frame into prose beats, lighting cues or Unreal parameters.").font(instrument_kit::theme::body(11.0)).color(INK_DIM));
        });
    });
}
