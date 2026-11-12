//! MAIN RACK — the 2D equipment rack instruments live in.
//!
//! Left rail: instrument browser by department, plus effects. Centre: the rack
//! of devices. Right rail: inspector for the selected device.

use crate::app::{Action, DragItem, ForgeApp};
use crate::sections;
use crate::shell::{self, SideItem};
use crate::util::{self, chc, col, row};
use egui::{
    pos2, vec2, Align2, Color32, CursorIcon, Id, Rect, ScrollArea, Sense, Stroke, Ui, Vec2,
};
use instrument_kit::icons;
use instrument_kit::paint::{self, *};
use instrument_kit::theme::*;
use instrument_kit::widgets::{self, Knob, KnobVariant};
use rack_core::bus::Bus;
use rack_core::catalog::{self, DEPTS, FX};
use rack_core::engine::Engine;
use rack_core::model::{Device, Instrument, RackState, MAX_FX};

const EAR_W: f32 = 22.0;
const STRIP_H: f32 = 34.0;
const MIX_H: f32 = 27.0;
const MACROS_H: f32 = 66.0;
const FX_H: f32 = 40.0;

/// Skin → (top, bottom, light?) of the brushed unit face.
fn skin(name: &str) -> (Color32, Color32, bool) {
    match name {
        "ivory" => (rgb(0xeeebe2), rgb(0xd4d0c4), true),
        "olive" => (rgb(0x6d6c4c), rgb(0x4b4a33), false),
        "teal" => (rgb(0x2a959d), rgb(0x1b666d), false),
        "ember" => (rgb(0x5a3222), rgb(0x371c13), false),
        "indigo" => (rgb(0x2e3470), rgb(0x1c1f4a), false),
        _ => (rgb(0x262b35), rgb(0x14171d), false),
    }
}

pub fn body(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    ui.painter().rect_filled(area, 0.0, SHELL);

    // ---- left rail: browser ----
    let lib_n = app.state.lib.len();
    let mut items = vec![
        SideItem::new("all", "capsule", "ALL INSTRUMENTS", &lib_n.to_string()),
        SideItem::new("fx", "fx", "EFFECTS", "drag onto a device").color(SIGNAL_ROSE),
        SideItem::sep(),
    ];
    for (i, d) in DEPTS.iter().enumerate() {
        let n = app.state.lib.iter().filter(|x| x.dept == i).count();
        items.push(
            SideItem::new(
                &format!("d{i}"),
                d.crest,
                &format!("{} · {}", d.num, d.name),
                &format!("{n} · {}", d.law),
            )
            .color(chc(d.ch))
            .badge(n),
        );
    }
    let left = shell::side(
        &mut app.prefs,
        ui,
        area,
        "rack.L",
        false,
        &items,
        250.0,
        true,
    );
    if let Some((r, tab)) = &left.body {
        match tab.as_str() {
            "fx" => fx_browser(ui, *r),
            "all" => rack_list(app, ui, *r, None),
            t => rack_list(app, ui, *r, t[1..].parse::<usize>().ok()),
        }
    }

    // ---- right rail: inspector ----
    let sel_name = app
        .sel
        .as_ref()
        .and_then(|id| app.state.device(id))
        .and_then(|d| app.state.inst(&d.inst))
        .map(|i| i.name.clone())
        .unwrap_or_else(|| "none".into());
    let right = shell::side(
        &mut app.prefs,
        ui,
        left.rest,
        "rack.R",
        true,
        &[SideItem::new("dev", "device", "DEVICE", &sel_name)],
        270.0,
        false,
    );
    if let Some((r, _)) = &right.body {
        inspector(app, ui, *r);
    }

    // ---- centre: the rack ----
    let center = right.rest;
    ui.painter().rect_filled(center, 0.0, rgb(0x0b0d12));
    col(ui, center, "rack-center", |ui| {
        ScrollArea::both().auto_shrink(false).show(ui, |ui| {
            let w = (ui.available_width() - 28.0).max(660.0 - 28.0);
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.add_space(10.0);
            let x0 = ui.cursor().left() + 14.0;
            let devs: Vec<Device> = app.state.rack.clone();
            drop_zone(app, ui, x0, w, 0, devs.is_empty(), false);
            for (i, dev) in devs.iter().enumerate() {
                let Some(inst) = app.state.inst(&dev.inst).cloned() else {
                    continue;
                };
                device(app, ui, x0, w, dev, &inst);
                drop_zone(app, ui, x0, w, i + 1, false, i == devs.len() - 1);
            }
            ui.add_space(80.0);
        });
    });
    drag_ghost(ui);
}

/// Label that follows the pointer while something is being dragged.
fn drag_ghost(ui: &Ui) {
    let Some(item) = egui::DragAndDrop::payload::<DragItem>(ui.ctx()) else {
        return;
    };
    let Some(pos) = ui.ctx().pointer_interact_pos() else {
        return;
    };
    let label = match &*item {
        DragItem::Inst(_) => "INSTRUMENT",
        DragItem::Fx(_) => "EFFECT",
        DragItem::Device(_) => "MOVE DEVICE",
        DragItem::Module(_) => "MODULE",
    };
    let p = ui.ctx().layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        Id::unique("drag-ghost"),
    ));
    let w = measure(&p, label, mono(8.0), 0.14).x + 16.0;
    let r = Rect::from_min_size(pos + vec2(12.0, 10.0), vec2(w, 20.0));
    p.rect_filled(r, 4.0, rgb(0x0d1118));
    outline(&p, r, 4.0, alpha(SIGNAL_COOL, 0.6));
    text(
        &p,
        r.center(),
        Align2::CENTER_CENTER,
        label,
        mono(8.0),
        SIGNAL_COOL,
        0.14,
    );
}

fn rack_list(app: &mut ForgeApp, ui: &mut Ui, area: Rect, dept: Option<usize>) {
    let list: Vec<Instrument> = app
        .state
        .lib
        .iter()
        .filter(|i| dept.is_none_or(|d| i.dept == d))
        .cloned()
        .collect();
    col(ui, area, "rack-list", |ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.add_space(10.0);
            let w = ui.available_width() - 20.0;
            for inst in &list {
                let d = catalog::dept(inst.dept);
                let (cell, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::hover());
                let r = Rect::from_min_size(pos2(cell.left() + 10.0, cell.top()), vec2(w, 40.0));
                let resp = ui.interact(r, Id::unique(("lib", &inst.id)), Sense::click_and_drag());
                if resp.drag_started() {
                    resp.dnd_set_drag_payload(DragItem::Inst(inst.id.clone()));
                }
                let p = ui.painter();
                let (a, b) = if resp.hovered() {
                    (rgb(0x222733), rgb(0x171a21))
                } else {
                    (rgb(0x1a1e26), rgb(0x14171d))
                };
                grad_v(p, r, 6.0, a, b);
                bevel(p, r, 6.0, 0.05, 0.0);
                outline_outside(p, r, 6.0, black(0.6));
                widgets::glyph_tile_at(
                    p,
                    Rect::from_min_size(pos2(r.left() + 6.0, r.top() + 5.0), Vec2::splat(30.0)),
                    d.crest,
                    chc(d.ch),
                    true,
                    false,
                );
                let tw = w - 44.0 - 24.0;
                text_clipped(
                    p,
                    pos2(r.left() + 44.0, r.top() + 14.0),
                    Align2::LEFT_CENTER,
                    &inst.name,
                    mono(10.0),
                    INK,
                    0.08,
                    tw,
                );
                text_clipped(
                    p,
                    pos2(r.left() + 44.0, r.top() + 27.0),
                    Align2::LEFT_CENTER,
                    &format!("{} · {}", d.name, inst.module_names()),
                    instrument_kit::theme::body(10.0),
                    INK_DIM,
                    0.0,
                    tw,
                );
                let er =
                    Rect::from_center_size(pos2(r.right() - 14.0, r.center().y), Vec2::splat(20.0));
                let eresp = ui
                    .interact(er, Id::unique(("lib-edit", &inst.id)), Sense::click())
                    .on_hover_text("Open in Forge · Modules")
                    .on_hover_cursor(CursorIcon::PointingHand);
                icons::paint(
                    ui.painter(),
                    "edit",
                    er.center(),
                    11.0,
                    if eresp.hovered() {
                        SIGNAL_COOL
                    } else {
                        INK_DIM
                    },
                );
                if eresp.clicked() {
                    app.actions.push(Action::EditInstrument(inst.id.clone()));
                } else if resp.double_clicked() {
                    app.actions.push(Action::AddDevice {
                        inst: inst.id.clone(),
                        at: None,
                    });
                }
                resp.on_hover_text("Drag onto the rack · double-click to append")
                    .on_hover_cursor(CursorIcon::Grab);
            }
            if list.is_empty() {
                let (hr, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
                text(
                    ui.painter(),
                    pos2(hr.left() + 10.0, hr.center().y),
                    Align2::LEFT_CENTER,
                    "No instruments in this department yet.",
                    instrument_kit::theme::body(11.0),
                    INK_DIM,
                    0.0,
                );
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                if widgets::cv_btn_wide(ui, "+ DESIGN NEW INSTRUMENT", w, true, false).clicked() {
                    app.actions.push(Action::NewInstrument(dept.unwrap_or(0)));
                }
            });
            ui.add_space(70.0);
        });
    });
}

fn fx_browser(ui: &mut Ui, area: Rect) {
    col(ui, area, "fx-browser", |ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.add_space(10.0);
            let w = ui.available_width() - 20.0;
            let (tr, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
            paint::paragraph(
                ui.painter(),
                pos2(tr.left() + 10.0, tr.top()),
                "QUARTER-RACK EFFECTS · DRAG UNDER AN INSTRUMENT",
                display(8.0),
                INK_MID,
                w,
            );
            for f in FX {
                let (cell, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::hover());
                let r = Rect::from_min_size(pos2(cell.left() + 10.0, cell.top()), vec2(w, 34.0));
                let resp = ui.interact(r, Id::unique(("fxb", f.key)), Sense::drag());
                if resp.drag_started() {
                    resp.dnd_set_drag_payload(DragItem::Fx(f.key.to_owned()));
                }
                let p = ui.painter();
                let (a, b) = if resp.hovered() {
                    (rgb(0x222733), rgb(0x171a21))
                } else {
                    (rgb(0x1a1e26), rgb(0x14171d))
                };
                grad_v(p, r, 6.0, a, b);
                bevel(p, r, 6.0, 0.05, 0.0);
                outline_outside(p, r, 6.0, black(0.6));
                widgets::glyph_tile_at(
                    p,
                    Rect::from_min_size(pos2(r.left() + 6.0, r.top() + 4.0), Vec2::splat(26.0)),
                    f.glyph,
                    chc(f.ch),
                    true,
                    false,
                );
                text_clipped(
                    p,
                    pos2(r.left() + 40.0, r.center().y),
                    Align2::LEFT_CENTER,
                    f.name,
                    mono(10.0),
                    INK,
                    0.08,
                    w - 48.0,
                );
                resp.on_hover_cursor(CursorIcon::Grab);
            }
            ui.add_space(70.0);
        });
    });
}

/// Gap between devices: accepts a library instrument or a device being moved.
fn drop_zone(app: &mut ForgeApp, ui: &mut Ui, x0: f32, w: f32, at: usize, empty: bool, last: bool) {
    let dragging = egui::DragAndDrop::payload::<DragItem>(ui.ctx())
        .is_some_and(|d| matches!(&*d, DragItem::Inst(_) | DragItem::Device(_)));
    let base = if empty {
        120.0
    } else if last {
        46.0
    } else {
        6.0
    };
    // A fixed hit height while dragging keeps the layout from shifting under the pointer.
    let (cell, _) = ui.allocate_exact_size(vec2(w + 28.0, base), Sense::hover());
    let r = Rect::from_min_size(pos2(x0, cell.top()), vec2(w, base));
    let hit = if dragging && !empty && !last {
        r.expand2(vec2(0.0, 11.0))
    } else {
        r
    };
    let resp = ui.interact(hit, Id::unique(("drop", at)), Sense::hover());
    let over = dragging && resp.contains_pointer();
    let p = ui.painter();
    if over {
        let hl = if empty || last { r } else { hit };
        p.rect_filled(hl, 3.0, alpha(SIGNAL_COOL, 0.12));
        outline(p, hl, 3.0, alpha(SIGNAL_COOL, 0.6));
    }
    if empty {
        dashed(p, r, white(0.12));
        text(
            p,
            r.center(),
            Align2::CENTER_CENTER,
            "DRAG AN INSTRUMENT HERE FROM THE LEFT · OR DOUBLE-CLICK ONE",
            mono(10.0),
            INK_DIM,
            0.14,
        );
    } else if last {
        let lr = Rect::from_min_max(pos2(r.left(), r.top() + 6.0), r.max);
        dashed(p, lr, white(0.08));
        text(
            p,
            lr.center(),
            Align2::CENTER_CENTER,
            "ADD DEVICE · DRAG AN INSTRUMENT HERE",
            mono(9.0),
            if over { SIGNAL_COOL } else { INK_DIM },
            0.16,
        );
    }
    if let Some(payload) = resp.dnd_release_payload::<DragItem>() {
        match &*payload {
            DragItem::Inst(id) => app.actions.push(Action::AddDevice {
                inst: id.clone(),
                at: Some(at),
            }),
            DragItem::Device(id) => app.actions.push(Action::MoveDevice {
                dev: id.clone(),
                at,
            }),
            _ => {}
        }
    }
}

fn ear(ui: &mut Ui, r: Rect, left: bool, dev_id: Option<&str>) {
    let p = ui.painter();
    let cr = if left {
        egui::CornerRadius {
            nw: 4,
            sw: 4,
            ne: 0,
            se: 0,
        }
    } else {
        egui::CornerRadius {
            nw: 0,
            sw: 0,
            ne: 4,
            se: 4,
        }
    };
    grad(p, r, cr, rgb(0x3a404c), rgb(0x20242c), true);
    p.line_segment(
        [
            pos2(r.left() + 2.0, r.top() + 0.5),
            pos2(r.right() - 2.0, r.top() + 0.5),
        ],
        Stroke::new(1.0, white(0.15)),
    );
    let edge = if left { r.right() } else { r.left() };
    p.line_segment(
        [pos2(edge, r.top()), pos2(edge, r.bottom())],
        Stroke::new(1.0, rgb(0x0a0c10)),
    );
    rack_hole(p, pos2(r.center().x, r.top() + 11.0));
    rack_hole(p, pos2(r.center().x, r.bottom() - 11.0));
    if let Some(id) = dev_id {
        for dy in [-4.0, 0.0, 4.0] {
            for dx in [-2.0, 2.0] {
                p.circle_filled(r.center() + vec2(dx, dy), 0.9, rgb(0x6a7282));
            }
        }
        let resp = ui
            .interact(r, Id::unique(("ear", id)), Sense::drag())
            .on_hover_text("Drag to reorder")
            .on_hover_cursor(CursorIcon::Grab);
        if resp.drag_started() {
            resp.dnd_set_drag_payload(DragItem::Device(id.to_owned()));
        }
    }
}

/// One rack device: ears, header strip, macros or unfolded sections, mix strip
/// and its row of insert effects.
fn device(app: &mut ForgeApp, ui: &mut Ui, x0: f32, w: f32, dev: &Device, inst: &Instrument) {
    let dept = catalog::dept(inst.dept);
    let dc = chc(dept.ch);
    let on = app.bus.flag(&format!("dev.{}.on", dev.id), true);
    let selected = app.sel.as_deref() == Some(dev.id.as_str());
    let (top, bottom, light) = skin(&inst.skin);
    let body_w = w - EAR_W * 2.0;

    let mid_h = if dev.open {
        sections::device_height(inst, body_w)
    } else {
        MACROS_H
    };
    let unit_h = STRIP_H + mid_h + MIX_H;
    let (cell, _) = ui.allocate_exact_size(vec2(w + 28.0, unit_h), Sense::hover());
    let unit = Rect::from_min_size(pos2(x0, cell.top()), vec2(w, unit_h));
    if !ui.is_rect_visible(unit.expand(20.0)) {
        fx_row(app, ui, x0, w, dev);
        return;
    }
    // Clicking anywhere on the unit that is not a control selects the device.
    let sel_resp = ui.interact(unit, Id::unique(("dev-sel", &dev.id)), Sense::click());
    if sel_resp.clicked() {
        app.actions.push(Action::Select(dev.id.clone()));
    }

    let p = ui.painter().clone();
    paint::shadow(&p, unit, 4.0, 4.0, 10.0, black(0.5));
    if selected {
        glow_rect(&p, unit, 4.0, 14.0, alpha(dc, 0.3));
    }
    grad_stops(
        &p,
        unit,
        4.0,
        &[(0.0, top), (0.5, mix(top, bottom, 0.45)), (1.0, bottom)],
        true,
    );
    brushed(&p, unit, if light { 0.0 } else { 0.022 });
    outline_outside(&p, unit, 4.0, if selected { dc } else { black(0.85) });

    ear(
        ui,
        Rect::from_min_size(unit.min, vec2(EAR_W, unit_h)),
        true,
        Some(&dev.id),
    );
    ear(
        ui,
        Rect::from_min_size(pos2(unit.right() - EAR_W, unit.top()), vec2(EAR_W, unit_h)),
        false,
        None,
    );

    let bx = unit.left() + EAR_W;
    // ---- header strip ----
    let strip = Rect::from_min_size(pos2(bx, unit.top()), vec2(body_w, STRIP_H));
    let base = rgb(0x14171d);
    grad_stops(
        &p,
        strip,
        0.0,
        &[
            (0.0, mix(dc, base, 0.38)),
            (0.6, mix(dc, base, 0.14)),
            (1.0, base),
        ],
        false,
    );
    p.line_segment(
        [
            strip.left_top() + vec2(0.0, 0.5),
            strip.right_top() + vec2(0.0, 0.5),
        ],
        Stroke::new(1.0, white(0.1)),
    );
    p.line_segment(
        [strip.left_bottom(), strip.right_bottom()],
        Stroke::new(1.0, black(0.7)),
    );
    row(
        ui,
        strip.shrink2(vec2(10.0, 0.0)),
        ("strip", &dev.id),
        |ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            // department power emblem = bypass
            let (tr, tresp) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
            if on {
                glow_rect(ui.painter(), tr, 13.0, 10.0, alpha(dc, 0.6));
            }
            widgets::glyph_tile_at(ui.painter(), tr, dept.crest, dc, true, !on);
            if on {
                ui.painter()
                    .circle_stroke(tr.center(), 13.0, Stroke::new(1.0, dc));
            }
            if tresp
                .on_hover_text(if on {
                    "Bypass instrument"
                } else {
                    "Activate instrument"
                })
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                app.bus.set_flag(&format!("dev.{}.on", dev.id), !on);
            }
            // name LCD
            let lw = (body_w - 420.0).clamp(120.0, 300.0);
            let (lr, _) = ui.allocate_exact_size(vec2(lw, 24.0), Sense::hover());
            let p = ui.painter();
            p.rect_filled(lr, 3.0, rgb(0x06080c));
            paint::inner_shade(p, lr, 3.0, 0.6);
            let tag = format!("{}·{}", dept.num, dept.law.to_uppercase());
            let t = text(
                p,
                pos2(lr.left() + 10.0, lr.center().y + 1.0),
                Align2::LEFT_CENTER,
                &tag,
                mono(8.0),
                INK_DIM,
                0.12,
            );
            text_clipped(
                p,
                pos2(t.right() + 8.0, lr.center().y),
                Align2::LEFT_CENTER,
                &inst.name,
                mono(12.0),
                alpha(dc, if on { 1.0 } else { 0.35 }),
                0.08,
                (lr.right() - t.right() - 14.0).max(1.0),
            );
            ui.spacing_mut().item_spacing.x = 4.0;
            if widgets::cv_btn(ui, "EDIT", false, false)
                .on_hover_text("Edit in Forge")
                .clicked()
            {
                app.actions.push(Action::EditInstrument(inst.id.clone()));
            }
            let (xr, xresp) = ui.allocate_exact_size(vec2(26.0, 24.0), Sense::click());
            let p = ui.painter();
            grad_v(p, xr, 3.0, rgb(0x252a34), rgb(0x171b22));
            icons::paint(
                p,
                "close",
                xr.center(),
                10.0,
                if xresp.hovered() { SIGNAL_HOT } else { INK_MID },
            );
            if xresp
                .on_hover_text("Remove from rack")
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                app.actions.push(Action::RemoveDevice(dev.id.clone()));
            }
        },
    );
    if body_w > 620.0 {
        let _ = light;
        let ink = rgb(0xe9edf4);
        text(
            &p,
            pos2(strip.right() - 10.0, strip.center().y),
            Align2::RIGHT_CENTER,
            dept.name,
            display(15.0),
            ink,
            0.22,
        );
    }

    // ---- middle: macros or unfolded sections ----
    let mid = Rect::from_min_size(pos2(bx, strip.bottom()), vec2(body_w, mid_h));
    if dev.open {
        sections::device_body(app, ui, mid, dev, inst, on);
    } else {
        let macros = inst.macro_params(&dev.id);
        row(
            ui,
            mid.shrink2(vec2(10.0, 0.0)),
            ("macros", &dev.id),
            |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if !on {
                    ui.set_opacity(0.35);
                }
                for m in &macros {
                    util::knob(
                        ui,
                        &mut app.bus,
                        &m.p,
                        Knob::new(34.0, util::ch(dept.ch))
                            .variant(KnobVariant::Arc)
                            .label(m.label)
                            .label_size(8.0)
                            .show_value(false),
                    );
                }
                if macros.is_empty() {
                    let (hr, _) = ui.allocate_exact_size(vec2(300.0, 20.0), Sense::hover());
                    text(
                        ui.painter(),
                        hr.left_center(),
                        Align2::LEFT_CENTER,
                        "No macros — choose some in the designer",
                        instrument_kit::theme::body(11.0),
                        INK_DIM,
                        0.0,
                    );
                }
            },
        );
    }

    // ---- mix strip ----
    let mix_r = Rect::from_min_size(pos2(bx, mid.bottom()), vec2(body_w, MIX_H));
    if inst.skin == "graphite" {
        grad_v(&p, mix_r, 0.0, rgb(0x101318), rgb(0x0b0d11));
    } else {
        p.rect_filled(mix_r, 0.0, black(0.45));
    }
    p.line_segment(
        [mix_r.left_top(), mix_r.right_top()],
        Stroke::new(1.0, black(0.8)),
    );
    mix_strip(app, ui, mix_r, dev, inst, dc, on);

    fx_row(app, ui, x0, w, dev);
}

fn mix_strip(
    app: &mut ForgeApp,
    ui: &mut Ui,
    r: Rect,
    dev: &Device,
    inst: &Instrument,
    dc: Color32,
    on: bool,
) {
    let (state, bus): (&RackState, &mut Bus) = (&app.state, &mut app.bus);
    let _ = state;
    let (k_mute, k_solo, k_level, k_pan) = (
        format!("dev.{}.mute", dev.id),
        format!("dev.{}.solo", dev.id),
        format!("dev.{}.level", dev.id),
        format!("dev.{}.pan", dev.id),
    );
    let (mute, solo) = (bus.flag(&k_mute, false), bus.flag(&k_solo, false));
    let mut toggle_open = false;
    row(ui, r.shrink2(vec2(10.0, 0.0)), ("mix", &dev.id), |ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let (fr, fresp) = ui.allocate_exact_size(vec2(22.0, 20.0), Sense::click());
        let p = ui.painter();
        p.rect_filled(fr, 3.0, rgb(0x1d222a));
        // ▾ unfolded, ▸ folded
        let c = fr.center();
        let tri = if dev.open {
            vec![
                c + vec2(-4.0, -2.0),
                c + vec2(4.0, -2.0),
                c + vec2(0.0, 3.5),
            ]
        } else {
            vec![
                c + vec2(-2.0, -4.0),
                c + vec2(3.5, 0.0),
                c + vec2(-2.0, 4.0),
            ]
        };
        p.add(egui::Shape::convex_polygon(tri, dc, Stroke::NONE));
        toggle_open = fresp
            .on_hover_text(if dev.open {
                "Fold to front panel"
            } else {
                "Unfold sections"
            })
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked();

        let nw = (measure(ui.painter(), &inst.name, mono(10.0), 0.08).x + 16.0).max(90.0);
        let (nr, _) = ui.allocate_exact_size(vec2(nw, 18.0), Sense::hover());
        ui.painter()
            .rect_filled(nr, 2.0, mix(dc, rgb(0x0a0c10), 0.14));
        text(
            ui.painter(),
            pos2(nr.left() + 8.0, nr.center().y),
            Align2::LEFT_CENTER,
            &inst.name,
            mono(10.0),
            INK,
            0.08,
        );

        if widgets::latch(ui, "MUTE", mute, SIGNAL_AMBER).clicked() {
            bus.set_flag(&k_mute, !mute);
        }
        if widgets::latch(ui, "SOLO", solo, SIGNAL_LIFE).clicked() {
            bus.set_flag(&k_solo, !solo);
        }
        let (lr, _) = ui.allocate_exact_size(vec2(36.0, 12.0), Sense::hover());
        engrave(
            ui.painter(),
            lr.left_center(),
            Align2::LEFT_CENTER,
            "LEVEL",
            7.0,
            INK_DIM,
        );
        let mut level = bus.num(&k_level, 0.8);
        if widgets::hfader(ui, &mut level, 130.0, dc, 0.8)
            .on_hover_text(format!("Level {}%", (level * 100.0).round()))
            .changed()
        {
            bus.set_num(&k_level, level);
        }
        util::knob(
            ui,
            bus,
            &k_pan,
            Knob {
                size: 24.0,
                color: Some(dc),
                variant: KnobVariant::Arc,
                bipolar: true,
                show_value: false,
                ..Default::default()
            },
        );

        // level meter, driven by the transport like the hardware it mimics
        let step = bus.num("transport.step", 0.0) as i32;
        let lit = if on && !mute {
            (level * 16.0 * (0.55 + 0.45 * (((step * 7) % 5) as f32 / 4.0))).round() as usize
        } else {
            0
        };
        let tag = format!("{} SEC · {} FX", inst.modules.len(), dev.fx.len());
        let tag_w = measure(ui.painter(), &tag, mono(8.0), 0.1).x;
        let mw = (ui.available_width() - tag_w - 12.0).clamp(0.0, 200.0);
        if mw >= 60.0 {
            let (mr, _) = ui.allocate_exact_size(vec2(mw, 8.0), Sense::hover());
            widgets::meter16(ui.painter(), mr, lit, dc);
        }
        if ui.available_width() >= tag_w {
            let (tr, _) = ui.allocate_exact_size(vec2(tag_w, 12.0), Sense::hover());
            text(
                ui.painter(),
                tr.left_center(),
                Align2::LEFT_CENTER,
                &tag,
                mono(8.0),
                INK_DIM,
                0.1,
            );
        }
    });
    if toggle_open {
        app.actions.push(Action::ToggleOpen(dev.id.clone()));
    }
}

/// Quarter-width effect units under a device, four per row.
fn fx_row(app: &mut ForgeApp, ui: &mut Ui, x0: f32, w: f32, dev: &Device) {
    let slots = (((dev.fx.len() + 1) as f32 / 4.0).ceil() as usize * 4).clamp(4, 8);
    let per_row = if w < 760.0 { 2 } else { 4 };
    let rows = slots.div_ceil(per_row);
    let h = rows as f32 * FX_H + (rows as f32 - 1.0) * 2.0;
    ui.add_space(2.0);
    let (cell, _) = ui.allocate_exact_size(vec2(w + 28.0, h), Sense::hover());
    let area = Rect::from_min_size(pos2(x0, cell.top()), vec2(w, h));
    if !ui.is_rect_visible(area) {
        return;
    }
    let dragging_fx = egui::DragAndDrop::payload::<DragItem>(ui.ctx())
        .is_some_and(|d| matches!(&*d, DragItem::Fx(_)));
    let zone = ui.interact(area, Id::unique(("fxrow", &dev.id)), Sense::hover());
    let over = dragging_fx && zone.contains_pointer();
    if over {
        glow_rect(ui.painter(), area, 3.0, 14.0, alpha(SIGNAL_MYTH, 0.35));
        outline_outside(ui.painter(), area, 3.0, SIGNAL_MYTH);
    }
    if let Some(payload) = zone.dnd_release_payload::<DragItem>() {
        if let DragItem::Fx(k) = &*payload {
            app.actions.push(Action::AddFx {
                dev: dev.id.clone(),
                fx: k.clone(),
            });
        }
    }
    let uw = (w - 2.0 * (per_row as f32 - 1.0)) / per_row as f32;
    for i in 0..slots {
        let r = Rect::from_min_size(
            pos2(
                area.left() + (i % per_row) as f32 * (uw + 2.0),
                area.top() + (i / per_row) as f32 * (FX_H + 2.0),
            ),
            vec2(uw, FX_H),
        );
        match dev
            .fx
            .get(i)
            .and_then(|f| catalog::fx(&f.kind).map(|d| (f, d)))
        {
            Some((f, d)) => fx_unit(app, ui, r, dev, i, f.on, d),
            None => {
                let p = ui.painter();
                p.rect_filled(r, 3.0, rgb(0x0b0d11));
                paint::inner_shade(p, r, 3.0, 0.5);
                if i == dev.fx.len() && dev.fx.len() < MAX_FX {
                    text(
                        p,
                        r.center(),
                        Align2::CENTER_CENTER,
                        "+ DROP EFFECT",
                        mono(8.0),
                        INK_DIM,
                        0.12,
                    );
                }
            }
        }
    }
}

fn fx_unit(
    app: &mut ForgeApp,
    ui: &mut Ui,
    r: Rect,
    dev: &Device,
    slot: usize,
    on: bool,
    d: &catalog::FxDef,
) {
    let fc = chc(d.ch);
    let p = ui.painter().clone();
    grad_v(&p, r, 3.0, rgb(0x22262f), rgb(0x15181e));
    brushed(&p, r, 0.02);
    outline_outside(&p, r, 3.0, black(0.85));
    p.line_segment(
        [
            pos2(r.left() + 2.0, r.top() + 1.0),
            pos2(r.right() - 2.0, r.top() + 1.0),
        ],
        Stroke::new(2.0, alpha(fc, 0.5)),
    );
    rack_hole(&p, pos2(r.left() + 10.0, r.center().y));
    rack_hole(&p, pos2(r.right() - 10.0, r.center().y));
    let (technique, meaning) = d.parts();
    row(
        ui,
        Rect::from_min_max(
            pos2(r.left() + 20.0, r.top()),
            pos2(r.right() - 20.0, r.bottom()),
        ),
        ("fxu", &dev.id, slot),
        |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let (pr, presp) = ui.allocate_exact_size(Vec2::splat(13.0), Sense::click());
            widgets::led_at(ui.painter(), pr.center(), 9.0, on, SIGNAL_LIFE);
            if presp
                .on_hover_text(if on { "Bypass" } else { "Enable" })
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                app.actions.push(Action::ToggleFx {
                    dev: dev.id.clone(),
                    slot,
                });
            }
            let (tr, _) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::hover());
            widgets::glyph_tile_at(ui.painter(), tr, d.glyph, fc, true, !on);
            let name_w = (ui.available_width() - 26.0 - 38.0 - 22.0).max(0.0);
            let (nr, _) = ui.allocate_exact_size(vec2(name_w, 28.0), Sense::hover());
            if name_w > 20.0 {
                let k = if on { 1.0 } else { 0.45 };
                text_clipped(
                    ui.painter(),
                    pos2(nr.left(), nr.top() + 8.0),
                    Align2::LEFT_CENTER,
                    meaning,
                    display(9.0),
                    alpha(rgb(0xe5e9f0), k),
                    0.08,
                    name_w,
                );
                text_clipped(
                    ui.painter(),
                    pos2(nr.left(), nr.top() + 20.0),
                    Align2::LEFT_CENTER,
                    technique,
                    mono(8.0),
                    alpha(fc, k),
                    0.0,
                    name_w,
                );
            }
            util::knob(
                ui,
                &mut app.bus,
                &format!("{}.fx{slot}.amt", dev.id),
                Knob::new(26.0, util::ch(d.ch))
                    .variant(KnobVariant::Arc)
                    .default_value(1.0)
                    .show_value(false),
            )
            .on_hover_text("Amount");
            let (xr, xresp) = ui.allocate_exact_size(vec2(14.0, 20.0), Sense::click());
            icons::paint(
                ui.painter(),
                "close",
                xr.center(),
                9.0,
                if xresp.hovered() { SIGNAL_HOT } else { INK_DIM },
            );
            if xresp
                .on_hover_text("Remove")
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                app.actions.push(Action::RemoveFx {
                    dev: dev.id.clone(),
                    slot,
                });
            }
        },
    );
}

fn inspector(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    let dev = app
        .sel
        .as_ref()
        .and_then(|id| app.state.device(id))
        .cloned();
    let inst = dev.as_ref().and_then(|d| app.state.inst(&d.inst)).cloned();
    col(ui, area.shrink2(vec2(12.0, 4.0)), "inspector", |ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            let (Some(dev), Some(inst)) = (dev, inst) else {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("Select a device in the rack.")
                        .font(instrument_kit::theme::body(11.0))
                        .color(INK_DIM),
                );
                return;
            };
            let d = catalog::dept(inst.dept);
            let w = ui.available_width();
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.add_space(8.0);
            let (hr, _) = ui.allocate_exact_size(vec2(w, 34.0), Sense::hover());
            let p = ui.painter();
            widgets::glyph_tile_at(
                p,
                Rect::from_min_size(hr.min, Vec2::splat(34.0)),
                d.crest,
                chc(d.ch),
                true,
                false,
            );
            text_clipped(
                p,
                pos2(hr.left() + 42.0, hr.top() + 11.0),
                Align2::LEFT_CENTER,
                &inst.name,
                mono(11.0),
                INK,
                0.08,
                w - 44.0,
            );
            text_clipped(
                p,
                pos2(hr.left() + 42.0, hr.top() + 25.0),
                Align2::LEFT_CENTER,
                &format!("{} · {} · {}", d.num, d.name, d.law),
                mono(8.0),
                INK_DIM,
                0.12,
                w - 44.0,
            );

            sub_head(ui, &format!("MODULES · {}", inst.modules.len()));
            ui.spacing_mut().item_spacing.y = 3.0;
            for m in &inst.modules {
                let Some(def) = catalog::module(&m.kind) else {
                    continue;
                };
                let (lr, _) = ui.allocate_exact_size(vec2(w, 22.0), Sense::hover());
                widgets::glyph_tile_at(
                    ui.painter(),
                    Rect::from_min_size(pos2(lr.left(), lr.top() + 1.0), Vec2::splat(20.0)),
                    def.crest,
                    chc(def.ch),
                    false,
                    false,
                );
                text(
                    ui.painter(),
                    pos2(lr.left() + 26.0, lr.center().y),
                    Align2::LEFT_CENTER,
                    def.name,
                    mono(9.0),
                    INK_MID,
                    0.0,
                );
            }
            ui.add_space(5.0);
            sub_head(ui, &format!("EFFECTS · {}/{}", dev.fx.len(), MAX_FX));
            for (k, f) in dev.fx.iter().enumerate() {
                let Some(def) = catalog::fx(&f.kind) else {
                    continue;
                };
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let (tr, _) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
                    widgets::glyph_tile_at(ui.painter(), tr, def.glyph, chc(def.ch), true, !f.on);
                    let nw = (ui.available_width() - 64.0).max(10.0);
                    let (nr, _) = ui.allocate_exact_size(vec2(nw, 20.0), Sense::hover());
                    text_clipped(
                        ui.painter(),
                        nr.left_center(),
                        Align2::LEFT_CENTER,
                        def.name,
                        mono(9.0),
                        if f.on { INK } else { INK_DIM },
                        0.0,
                        nw,
                    );
                    if widgets::tool_btn(ui, "playm", None, f.on, Some(SIGNAL_LIFE))
                        .on_hover_text(if f.on { "Bypass" } else { "Enable" })
                        .clicked()
                    {
                        app.actions.push(Action::ToggleFx {
                            dev: dev.id.clone(),
                            slot: k,
                        });
                    }
                    if widgets::tool_btn(ui, "close", None, false, None)
                        .on_hover_text("Remove effect")
                        .clicked()
                    {
                        app.actions.push(Action::RemoveFx {
                            dev: dev.id.clone(),
                            slot: k,
                        });
                    }
                });
            }
            if dev.fx.is_empty() {
                ui.label(
                    egui::RichText::new("Drag effects from the left rail onto the device.")
                        .font(instrument_kit::theme::body(11.0))
                        .color(INK_DIM),
                );
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if widgets::cv_btn(ui, "EDIT IN FORGE", false, false).clicked() {
                    app.actions.push(Action::EditInstrument(inst.id.clone()));
                }
                if widgets::cv_btn(ui, "REMOVE", false, true).clicked() {
                    app.actions.push(Action::RemoveDevice(dev.id.clone()));
                }
            });
            ui.add_space(70.0);
        });
    });
}

pub fn sub_head(ui: &mut Ui, s: &str) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
    text(
        ui.painter(),
        pos2(r.left(), r.bottom() - 3.0),
        Align2::LEFT_BOTTOM,
        s,
        mono(8.0),
        INK_DIM,
        0.16,
    );
    ui.painter().line_segment(
        [r.left_bottom(), r.right_bottom()],
        Stroke::new(1.0, white(0.05)),
    );
}

/// Signal taps the visualizers read, so screens share one model of the sound.
pub struct Signal<'a> {
    pub engine: &'a mut Engine,
    pub bus: &'a Bus,
}

impl Signal<'_> {
    pub fn bands(&mut self, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| self.engine.synth_band(self.bus, i as f32 / n as f32))
            .collect()
    }
}
