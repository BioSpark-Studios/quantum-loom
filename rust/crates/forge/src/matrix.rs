//! ROUTING MATRIX — the flipped rack. No cables: a source → destination table
//! (enable, live meter, bipolar amount) plus a clickable grid overview.

use crate::app::ForgeApp;
use crate::util::{chc, col};
use egui::{pos2, vec2, Align2, CursorIcon, Id, Rect, ScrollArea, Sense, Stroke, Ui, Vec2};
use instrument_kit::icons;
use instrument_kit::paint::*;
use instrument_kit::theme::*;
use instrument_kit::widgets;
use rack_core::model::{self, Port};

fn port_label(p: &Port) -> String {
    format!("{} · {}", p.label, p.dev)
}

/// Grouped drop-down over ports. Returns the newly picked port id.
fn select(ui: &mut Ui, id: Id, current: &str, ports: &[Port], width: f32) -> Option<String> {
    let label = ports
        .iter()
        .find(|p| p.id == current)
        .map(port_label)
        .unwrap_or_else(|| {
            if let Some(cc) = current.strip_prefix("midi:") {
                format!("CC {cc} · MIDI")
            } else {
                "— missing —".into()
            }
        });
    let mut picked = None;
    ui.scope(|ui| {
        let v = ui.visuals_mut();
        v.widgets.inactive.weak_bg_fill = rgb(0x0b0e13);
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, white(0.08));
        v.widgets.hovered.weak_bg_fill = rgb(0x121722);
        v.widgets.inactive.corner_radius = egui::CornerRadius::same(3);
        let name = label.clone();
        let out = egui::ComboBox::from_id_salt(id)
            .width(width)
            .selected_text(egui::RichText::new(label).font(mono(9.0)).color(INK))
            .show_ui(ui, |ui| {
                let mut group = "";
                for p in ports {
                    if p.dev != group {
                        group = &p.dev;
                        ui.label(egui::RichText::new(group).font(mono(8.0)).color(INK_DIM));
                    }
                    if ui
                        .selectable_label(
                            p.id == current,
                            egui::RichText::new(port_label(p)).font(mono(9.0)),
                        )
                        .clicked()
                    {
                        picked = Some(p.id.clone());
                    }
                }
            });
        out.response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::ComboBox, true, &name));
    });
    picked
}

pub fn body(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    ui.painter().rect_filled(area, 0.0, SHELL);
    let ports = model::all_ports(&app.state);
    let t_ms = app.engine.time * 1000.0;
    col(ui, area, "matrix", |ui| {
        ScrollArea::both().auto_shrink(false).show(ui, |ui| {
            ui.add_space(14.0);
            let w = (ui.available_width() - 28.0).max(760.0);
            let x0 = ui.cursor().left() + 14.0;
            let frame_top = ui.cursor().top();
            let bg = ui.painter().add(egui::Shape::Noop);
            let bg2 = ui.painter().add(egui::Shape::Noop);
            let inner_w = w - 20.0;
            let xi = x0 + 10.0;
            ui.add_space(10.0);

            // bar
            let (bar, _) = ui.allocate_exact_size(vec2(w, 26.0), Sense::hover());
            let bar = Rect::from_min_size(pos2(xi, bar.top()), vec2(inner_w, 26.0));
            engrave(ui.painter(), bar.left_center(), Align2::LEFT_CENTER, &format!("ROUTING MATRIX · {} ROUTES", app.state.routes.len()), 9.0, INK_MID);
            crate::util::row_rtl(ui, bar, "mx-bar", |ui| {
                let (mr, _) = ui.allocate_exact_size(vec2(96.0, 20.0), Sense::hover());
                ui.painter().rect_filled(mr, 3.0, rgb(0x0a0c10));
                widgets::led_at(ui.painter(), pos2(mr.left() + 11.0, mr.center().y), 6.0, false, SIGNAL_AMBER);
                text(ui.painter(), pos2(mr.left() + 20.0, mr.center().y), Align2::LEFT_CENTER, "MIDI · OFF", mono(9.0), INK_MID, 0.1);
                mr_tip(ui, mr);
                if widgets::cv_btn(ui, "CLEAR", false, true).clicked() {
                    app.state.routes.clear();
                }
                let can = !ports.src.is_empty() && !ports.dst.is_empty();
                if ui.add_enabled_ui(can, |ui| widgets::cv_btn(ui, "+ ADD ROUTE", true, false)).inner.clicked() {
                    app.state.add_route(&ports.src[0].id, &ports.dst[0].id, 0.5);
                }
            });
            ui.add_space(4.0);

            // table
            let table_top = ui.cursor().top();
            let cols = |r: Rect| {
                let flex = r.width() - 16.0 - 14.0 - 60.0 - 70.0 - 18.0 - 6.0 * 8.0;
                let a = flex * 1.3 / 3.5;
                let m = flex * 0.9 / 3.5;
                let mut x = r.left() + 8.0;
                let mut out = [Rect::NOTHING; 7];
                for (i, cw) in [14.0, a, 60.0, m, a, 70.0, 18.0].into_iter().enumerate() {
                    out[i] = Rect::from_min_size(pos2(x, r.top()), vec2(cw, r.height()));
                    x += cw + 8.0;
                }
                out
            };
            let (hr, _) = ui.allocate_exact_size(vec2(w, 22.0), Sense::hover());
            let hr = Rect::from_min_size(pos2(xi, hr.top()), vec2(inner_w, 22.0));
            ui.painter().rect_filled(hr, 0.0, rgb(0x0d1014));
            let hc = cols(hr);
            for (i, l) in [(1, "SOURCE"), (2, "LIVE"), (3, "AMOUNT"), (4, "DESTINATION")] {
                text(ui.painter(), hc[i].left_center(), Align2::LEFT_CENTER, l, mono(8.0), INK_DIM, 0.16);
            }
            ui.spacing_mut().item_spacing.y = 0.0;
            let mut remove: Option<usize> = None;
            let routes = app.state.routes.clone();
            for (i, r) in routes.iter().enumerate() {
                let (rr, _) = ui.allocate_exact_size(vec2(w, 30.0), Sense::hover());
                let rr = Rect::from_min_size(pos2(xi, rr.top()), vec2(inner_w, 30.0));
                let p = ui.painter().clone();
                p.rect_filled(rr, 0.0, rgb(0x07090c));
                p.line_segment([rr.left_bottom(), rr.right_bottom()], Stroke::new(1.0, white(0.04)));
                let c = cols(rr);
                let s = ports.src(&r.from);
                let ch = s.map(|s| chc(s.ch)).unwrap_or(SIGNAL_AMBER);
                let k = if r.on { 1.0 } else { 0.45 };
                let on_r = Rect::from_center_size(c[0].center(), Vec2::splat(10.0));
                let on_resp = ui.interact(on_r.expand(4.0), Id::unique(("mx-on", &r.id)), Sense::click()).on_hover_text(if r.on { "Disable" } else { "Enable" }).on_hover_cursor(CursorIcon::PointingHand);
                if r.on {
                    glow_rect(&p, on_r, 2.0, 5.0, alpha(SIGNAL_LIFE, 0.6));
                }
                p.rect_filled(on_r, 2.0, if r.on { SIGNAL_LIFE } else { rgb(0x2a2f3a) });
                if on_resp.clicked() {
                    app.state.routes[i].on = !r.on;
                }
                crate::util::col(ui, rr, ("mx-row", &r.id), |ui| {
                    ui.set_opacity(k);
                    crate::util::row(ui, c[1], ("mx-src", &r.id), |ui| {
                        if let Some(n) = select(ui, Id::unique(("mx-s", &r.id)), &r.from, &ports.src, c[1].width() - 8.0) {
                            app.state.routes[i].from = n;
                        }
                    });
                    let v = model::port_value(&app.bus, &r.from, s, t_ms);
                    let m = Rect::from_center_size(c[2].center(), vec2(60.0, 4.0));
                    p.rect_filled(m, 2.0, white(0.06));
                    p.rect_filled(Rect::from_min_size(m.min, vec2(60.0 * v.clamp(0.0, 1.0), 4.0)), 2.0, alpha(SIGNAL_COOL, k));
                    crate::util::row(ui, c[3], ("mx-amt", &r.id), |ui| {
                        let mut a = r.amt;
                        if widgets::amount(ui, &mut a, c[3].width(), ch).on_hover_text("Drag · double-click = 0").changed() {
                            app.state.routes[i].amt = a;
                        }
                    });
                    crate::util::row(ui, c[4], ("mx-dst", &r.id), |ui| {
                        if let Some(n) = select(ui, Id::unique(("mx-d", &r.id)), &r.to, &ports.dst, c[4].width() - 8.0) {
                            app.state.routes[i].to = n;
                        }
                    });
                });
                crate::util::row(ui, c[5], ("mx-learn", &r.id), |ui| {
                    ui.add_enabled_ui(false, |ui| widgets::cv_btn(ui, "LEARN", false, false)).response.on_disabled_hover_text("MIDI learn arrives with the MIDI input crate");
                });
                let xr = ui.interact(c[6], Id::unique(("mx-x", &r.id)), Sense::click()).on_hover_text("Remove route").on_hover_cursor(CursorIcon::PointingHand);
                icons::paint(&p, "close", c[6].center(), 10.0, if xr.hovered() { SIGNAL_HOT } else { INK_DIM });
                if xr.clicked() {
                    remove = Some(i);
                }
            }
            if let Some(i) = remove {
                app.state.routes.remove(i);
            }
            if routes.is_empty() {
                let (er, _) = ui.allocate_exact_size(vec2(w, 34.0), Sense::hover());
                ui.painter().rect_filled(Rect::from_min_size(pos2(xi, er.top()), vec2(inner_w, 34.0)), 0.0, rgb(0x07090c));
                text(ui.painter(), pos2(xi + 10.0, er.center().y), Align2::LEFT_CENTER, "No routes. Add one, or click a cell in the grid below.", instrument_kit::theme::body(11.0), INK_DIM, 0.0);
            }
            let table = Rect::from_min_max(pos2(xi, table_top), pos2(xi + inner_w, ui.cursor().top()));
            outline(ui.painter(), table, 4.0, black(0.6));
            ui.add_space(10.0);

            // grid overview
            let gsrc: Vec<&Port> = ports.src.iter().filter(|s| !s.id.starts_with("deck")).collect();
            let gdst: Vec<&Port> = ports.dst.iter().filter(|d| !d.deck).collect();
            let grid_h = 6.0 + 18.0 + 70.0 + gsrc.len() as f32 * 22.0 + 6.0;
            let grid_w = (166.0 + gdst.len() as f32 * 22.0 + 12.0).max(inner_w);
            let (gr, _) = ui.allocate_exact_size(vec2(grid_w + 20.0, grid_h), Sense::hover());
            let gr = Rect::from_min_size(pos2(xi, gr.top()), vec2(grid_w, grid_h));
            let p = ui.painter().clone();
            p.rect_filled(gr, 4.0, rgb(0x0a0c10));
            engrave(&p, pos2(gr.left() + 6.0, gr.top() + 8.0), Align2::LEFT_TOP, "OVERVIEW · RACK INSTRUMENTS · CLICK A CELL TO CONNECT / DISCONNECT · DECK ROUTES LIVE IN THE TABLE ABOVE", 9.0, INK_MID);
            let gx = gr.left() + 6.0 + 162.0;
            let gy = gr.top() + 6.0 + 18.0 + 70.0;
            for (j, d) in gdst.iter().enumerate() {
                // vertical column label, reading upward
                let c = pos2(gx + j as f32 * 22.0 + 10.0, gy - 4.0);
                let galley = p.ctx().fonts_mut(|f| f.layout_job(job(&d.label, mono(8.0), chc(d.ch), 0.08)));
                let mut ts = egui::epaint::TextShape::new(pos2(c.x - 5.0, c.y), galley, chc(d.ch));
                ts.angle = -std::f32::consts::FRAC_PI_2;
                p.add(ts);
            }
            let mut toggle: Option<(String, String)> = None;
            for (i, s) in gsrc.iter().enumerate() {
                let y = gy + i as f32 * 22.0;
                p.circle_filled(pos2(gr.left() + 10.0, y + 10.0), 3.0, chc(s.ch));
                text_clipped(&p, pos2(gr.left() + 18.0, y + 10.0), Align2::LEFT_CENTER, &format!("{} · {}", s.dev, s.label), mono(9.0), INK_MID, 0.0, 146.0);
                for (j, d) in gdst.iter().enumerate() {
                    let cell = Rect::from_min_size(pos2(gx + j as f32 * 22.0, y), Vec2::splat(20.0));
                    if !ui.is_rect_visible(cell) {
                        continue;
                    }
                    let resp = ui.interact(cell, Id::unique(("mx-cell", &s.id, &d.id)), Sense::click()).on_hover_text(format!("{} · {} → {} · {}", s.dev, s.label, d.dev, d.label)).on_hover_cursor(CursorIcon::PointingHand);
                    let c = chc(s.ch);
                    match app.state.routes.iter().find(|r| r.from == s.id && r.to == d.id) {
                        Some(r) if r.on => {
                            glow_rect(&p, cell, 3.0, 6.0, alpha(c, 0.6));
                            p.rect_filled(cell, 3.0, c);
                        }
                        Some(_) => {
                            p.rect_filled(cell, 3.0, mix(c, rgb(0x151820), 0.35));
                        }
                        None => {
                            p.rect_filled(cell, 3.0, if resp.hovered() { rgb(0x232835) } else { rgb(0x151820) });
                        }
                    }
                    if resp.clicked() {
                        toggle = Some((s.id.clone(), d.id.clone()));
                    }
                }
            }
            if let Some((s, d)) = toggle {
                app.state.toggle_route(&s, &d);
            }
            ui.add_space(8.0);
            let (fr, _) = ui.allocate_exact_size(vec2(w, 20.0), Sense::hover());
            text(ui.painter(), pos2(x0 + w / 2.0, fr.center().y), Align2::CENTER_CENTER, "ROUTES KEEP RUNNING ON BOTH SIDES · NEGATIVE AMOUNT INVERTS", mono(8.0), INK_DIM, 0.14);
            ui.add_space(6.0);
            let frame = Rect::from_min_max(pos2(x0, frame_top), pos2(x0 + w, ui.cursor().top()));
            ui.painter().set(bg, egui::Shape::rect_filled(frame, 6.0, rgb(0x13161b)));
            ui.painter().set(bg2, egui::Shape::rect_stroke(frame, 6.0, Stroke::new(1.0, black(0.85)), egui::StrokeKind::Outside));
            ui.add_space(80.0);
        });
    });
}

fn mr_tip(ui: &mut Ui, r: Rect) {
    ui.interact(r, Id::unique("mx-midi"), Sense::hover())
        .on_hover_text(
            "MIDI input is not built into this pass; routes from rack sources work now.",
        );
}
