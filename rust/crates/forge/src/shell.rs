//! FORGE shell: sign-in gate, project vaults, the three-tier header
//! (Vault · Launcher · Workspace tools) and the icon rails with flyouts.

use crate::app::{Action, ForgeApp, Prefs, SideState, APPS, STAGES};
use crate::util::{chc, col, row, row_rtl};
use egui::{
    pos2, vec2, Align2, Color32, CursorIcon, Frame, Margin, Popup, Rect, Response, ScrollArea,
    Sense, Stroke, Ui, Vec2,
};
use instrument_kit::icons;
use instrument_kit::paint::{self, *};
use instrument_kit::theme::*;
use instrument_kit::widgets::{self, Face, Knob, KnobVariant};
use rack_core::catalog::{self, CONNECTIONS, DEPTS, FACTIONS};
use rack_core::engine::{Engine, ROMAN};
use rack_core::genesis::Genesis;
use rack_core::store::{self, Session, Vault};

pub const R1_H: f32 = 47.0;
pub const R2_H: f32 = 47.0;
pub const R3_H: f32 = 38.0;
pub const HEADER_H: f32 = R1_H + R2_H + R3_H + 1.0;
pub const RAIL_W: f32 = 44.0;

fn conn_items() -> [(&'static str, Color32); 3] {
    [
        (CONNECTIONS[0].1, chc(CONNECTIONS[0].2)),
        (CONNECTIONS[1].1, chc(CONNECTIONS[1].2)),
        (CONNECTIONS[2].1, chc(CONNECTIONS[2].2)),
    ]
}

fn conn_index(key: &str) -> usize {
    CONNECTIONS.iter().position(|c| c.0 == key).unwrap_or(0)
}

fn brand(p: &egui::Painter, left: egui::Pos2, icon: f32, size: f32) -> f32 {
    icons::paint(
        p,
        "forge",
        pos2(left.x + icon / 2.0, left.y),
        icon,
        SIGNAL_FORGE,
    );
    let r = text(
        p,
        pos2(left.x + icon + 8.0, left.y),
        Align2::LEFT_CENTER,
        "FORGE",
        serif(size),
        FORGE_INK,
        0.32,
    );
    r.right() - left.x
}

fn menu_frame() -> Frame {
    Frame::new()
        .fill(rgb(0x0d1118))
        .stroke(Stroke::new(1.0, white(0.1)))
        .corner_radius(8)
        .inner_margin(Margin::same(8))
}

fn menu_title(ui: &mut Ui, s: &str) {
    let (r, _) =
        ui.allocate_exact_size(vec2(ui.available_width().max(190.0), 18.0), Sense::hover());
    text(
        ui.painter(),
        pos2(r.left() + 4.0, r.top() + 3.0),
        Align2::LEFT_TOP,
        s,
        mono(8.0),
        INK_DIM,
        0.16,
    );
}

/// Menu row with an optional leading icon or status dot.
fn menu_item(
    ui: &mut Ui,
    icon: Option<&str>,
    dot: Option<Color32>,
    label: &str,
    on: bool,
) -> Response {
    let (r, resp) =
        ui.allocate_exact_size(vec2(ui.available_width().max(190.0), 28.0), Sense::click());
    let p = ui.painter();
    if resp.hovered() {
        p.rect_filled(r, 4.0, white(0.05));
    }
    let ink = if on {
        dot.unwrap_or(INK)
    } else if resp.hovered() {
        INK
    } else {
        INK_MID
    };
    let mut x = r.left() + 8.0;
    if let Some(i) = icon {
        icons::paint(p, i, pos2(x + 7.0, r.center().y), 14.0, ink);
        x += 22.0;
    }
    if let Some(c) = dot {
        p.circle_filled(pos2(x + 3.5, r.center().y), 3.5, c);
        x += 15.0;
    }
    text(
        p,
        pos2(x, r.center().y),
        Align2::LEFT_CENTER,
        label,
        mono(9.0),
        ink,
        0.1,
    );
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

fn user_chip(ui: &mut Ui, user: &str) -> Response {
    let (r, resp) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
    let p = ui.painter();
    p.circle_filled(r.center(), 14.0, rgb(0x1a2130));
    p.circle_stroke(r.center(), 13.5, Stroke::new(1.0, white(0.15)));
    text(
        p,
        r.center(),
        Align2::CENTER_CENTER,
        &store::initials(user),
        mono(10.0),
        INK,
        0.0,
    );
    resp.on_hover_text(user.to_owned())
        .on_hover_cursor(CursorIcon::PointingHand)
}

// ---- gate ------------------------------------------------------------------

fn gate_backdrop(p: &egui::Painter, full: Rect) {
    p.rect_filled(full, 0.0, SHELL);
    // radial-gradient(900px 500px at 50% 0%, rgba(255,122,60,.07), transparent 70%)
    let w = 900.0_f32.min(full.width());
    let r = Rect::from_center_size(pos2(full.center().x, full.top()), vec2(w * 1.4, 700.0));
    let g = p.with_clip_rect(full);
    paint::radial(
        &g,
        r.center(),
        r.width() / 2.0,
        alpha(rgb(0xff7a3c), 0.07),
        alpha(rgb(0xff7a3c), 0.0),
        Vec2::ZERO,
    );
}

/// Mock sign-in: operator name and where to connect.
pub fn gate(app: &mut ForgeApp, ui: &mut Ui, full: Rect) {
    gate_backdrop(ui.painter(), full);
    let remote = app.gate.conn != 0;
    let h = if remote { 462.0 } else { 396.0 };
    let card = Rect::from_center_size(full.center(), vec2(380.0_f32.min(full.width() - 24.0), h));
    let p = ui.painter().clone();
    paint::shadow(&p, card, 12.0, 30.0, 80.0, black(0.7));
    grad_v(&p, card, 12.0, rgb(0x10141b), rgb(0x0a0d12));
    outline(&p, card, 12.0, white(0.07));

    let inner = card.shrink2(vec2(28.0, 30.0));
    let cx = inner.center().x;
    icons::paint(
        &p,
        "forge",
        pos2(cx, inner.top() + 24.0),
        48.0,
        SIGNAL_FORGE,
    );
    text(
        &p,
        pos2(cx, inner.top() + 76.0),
        Align2::CENTER_CENTER,
        "FORGE",
        serif(30.0),
        FORGE_INK,
        0.4,
    );
    text(
        &p,
        pos2(cx, inner.top() + 110.0),
        Align2::CENTER_CENTER,
        "BIOSPARK · QUANTUM ENGINE CLIENT",
        mono(8.0),
        INK_DIM,
        0.2,
    );

    let body = Rect::from_min_max(pos2(inner.left(), inner.top() + 134.0), inner.max);
    let mut go = false;
    col(ui, body, "gate", |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let w = ui.available_width();
        field_label(ui, "OPERATOR");
        let r = widgets::text_field(
            ui,
            &mut app.gate.user,
            "Name or handle",
            w,
            36.0,
            rgb(0xff7a3c),
        );
        if !r.has_focus() && app.gate.user.is_empty() && !remote {
            r.request_focus();
        }
        go |= r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        ui.add_space(16.0);
        field_label(ui, "CONNECT TO");
        if let Some(i) = widgets::segmented(ui, w, &conn_items(), app.gate.conn) {
            app.gate.conn = i;
        }
        if remote {
            ui.add_space(16.0);
            let (label, hint) = if app.gate.conn == 1 {
                ("PEER ADDRESS", "peer id or ip:port")
            } else {
                ("VAULT HOST", "vault.example.net")
            };
            field_label(ui, label);
            let r = widgets::text_field(ui, &mut app.gate.host, hint, w, 36.0, rgb(0xff7a3c));
            go |= r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        }
        ui.add_space(16.0);
        let ready = !app.gate.user.trim().is_empty();
        go |= ui
            .add_enabled_ui(ready, |ui| widgets::go_btn(ui, "ENTER THE FORGE", w, false))
            .inner
            .clicked();
        ui.add_space(16.0);
        let (r, _) = ui.allocate_exact_size(vec2(w, 16.0), Sense::hover());
        text(
            ui.painter(),
            r.center(),
            Align2::CENTER_CENTER,
            "Mock sign-in — no authentication yet.",
            body_font(12.0),
            INK_DIM,
            0.0,
        );
    });
    if go && !app.gate.user.trim().is_empty() {
        let user = app.gate.user.trim().to_owned();
        app.prefs.last_user = user.clone();
        app.session = Some(Session {
            user,
            conn: CONNECTIONS[app.gate.conn.min(2)].0.into(),
            host: app.gate.host.trim().to_owned(),
            vault: None,
        });
        app.persist_session();
    }
}

fn body_font(size: f32) -> egui::FontId {
    instrument_kit::theme::body(size)
}

fn field_label(ui: &mut Ui, s: &str) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 14.0), Sense::hover());
    text(
        ui.painter(),
        r.left_top(),
        Align2::LEFT_TOP,
        s,
        mono(8.0),
        INK_DIM,
        0.16,
    );
}

// ---- project vaults --------------------------------------------------------

pub fn vault_picker(app: &mut ForgeApp, ui: &mut Ui, full: Rect) {
    gate_backdrop(ui.painter(), full);
    let Some(sess) = app.session.clone() else {
        return;
    };
    let bar = Rect::from_min_size(full.min, vec2(full.width(), 54.0));
    let p = ui.painter().clone();
    p.rect_filled(bar, 0.0, rgb(0x0a0d12));
    p.line_segment(
        [bar.left_bottom(), bar.right_bottom()],
        Stroke::new(1.0, white(0.06)),
    );
    let bw = brand(&p, pos2(bar.left() + 16.0, bar.center().y), 22.0, 15.0);
    text(
        &p,
        pos2(bar.left() + 16.0 + bw + 18.0, bar.center().y),
        Align2::LEFT_CENTER,
        "PROJECT VAULTS",
        mono(9.0),
        INK_MID,
        0.2,
    );
    row_rtl(ui, bar.shrink2(vec2(16.0, 0.0)), "vbar", |ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        if widgets::tool_btn(ui, "signout", None, false, None)
            .on_hover_text("Sign out")
            .clicked()
        {
            app.actions.push(Action::SignOut);
        }
        user_chip(ui, &sess.user);
        if let Some(i) = widgets::segmented(ui, 330.0, &conn_items(), conn_index(&sess.conn)) {
            app.actions.push(Action::SetConn(i));
        }
    });

    let area = Rect::from_min_max(pos2(full.left(), bar.bottom()), full.max);
    let now = store::now_ms();
    let mut sorted: Vec<Vault> = app.vaults.clone();
    sorted.sort_by_key(|v| std::cmp::Reverse(v.updated));
    let mut open: Option<(String, Option<u32>)> = None;
    let mut create = false;

    col(ui, area, "vaults", |ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            let avail = ui.available_width();
            let grid_w = avail.min(1200.0) - 48.0;
            let cols = ((grid_w + 14.0) / 244.0).floor().max(1.0);
            let cw = (grid_w - 14.0 * (cols - 1.0)) / cols;
            let x0 = ui.max_rect().left() + (avail - grid_w) / 2.0;
            let n = sorted.len() + 2;
            let rows = (n as f32 / cols).ceil() as usize;
            let hist_id = app.picker.hist.clone();
            let hist_extra = |v: &Vault| {
                if hist_id.as_deref() == Some(v.id.as_str()) {
                    (v.versions.len() as f32 * 24.0).min(140.0) + 8.0
                } else {
                    0.0
                }
            };
            let mut y = ui.cursor().top() + 24.0;
            for rw in 0..rows {
                let mut row_h: f32 = 150.0;
                for c in 0..cols as usize {
                    let i = rw * cols as usize + c;
                    if i >= 2 && i - 2 < sorted.len() {
                        row_h = row_h.max(150.0 + hist_extra(&sorted[i - 2]));
                    }
                }
                for c in 0..cols as usize {
                    let i = rw * cols as usize + c;
                    if i >= n {
                        break;
                    }
                    let r =
                        Rect::from_min_size(pos2(x0 + c as f32 * (cw + 14.0), y), vec2(cw, 150.0));
                    match i {
                        0 => create |= new_vault_card(app, ui, r),
                        1 => open_file_card(app, ui, r),
                        _ => {
                            let v = &sorted[i - 2];
                            let r = Rect::from_min_size(r.min, vec2(cw, 150.0 + hist_extra(v)));
                            if let Some(o) = vault_card(app, ui, r, v, now) {
                                open = Some(o);
                            }
                        }
                    }
                }
                y += row_h + 14.0;
            }
            ui.allocate_rect(
                Rect::from_min_max(pos2(x0, ui.cursor().top()), pos2(x0 + grid_w, y + 24.0)),
                Sense::hover(),
            );
        });
    });

    if create {
        if let Some((name, faction)) = app.picker.new.take() {
            if !name.trim().is_empty() {
                app.vaults.insert(0, Vault::new(&name, &faction));
                app.persist_session();
            }
        }
    }
    if let Some((id, ver)) = open {
        app.open_vault(&id, ver);
    }
}

fn add_card_frame(p: &egui::Painter, r: Rect) {
    outline(p, r, 10.0, white(0.1));
    dashed(p, r.shrink(1.0), white(0.08));
}

fn new_vault_card(app: &mut ForgeApp, ui: &mut Ui, r: Rect) -> bool {
    let p = ui.painter().clone();
    add_card_frame(&p, r);
    let mut create = false;
    if app.picker.new.is_some() {
        col(ui, r.shrink(16.0), "vnew", |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            let w = ui.available_width();
            let Some((name, faction)) = app.picker.new.as_mut() else {
                return;
            };
            let f = widgets::text_field(ui, name, "Vault name", w, 36.0, rgb(0xff7a3c));
            if f.gained_focus() || (!f.has_focus() && name.is_empty()) {
                f.request_focus();
            }
            create |= f.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for (fname, hexc) in FACTIONS {
                    let c = hex(hexc);
                    let on = faction == fname;
                    let (br, resp) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::click());
                    let p = ui.painter();
                    p.rect_filled(br, 6.0, rgb(0x0d1118));
                    if on {
                        glow_rect(p, br, 6.0, 10.0, alpha(c, 0.4));
                        outline(p, br, 6.0, c);
                    }
                    icons::paint(
                        p,
                        "faction",
                        br.center(),
                        18.0,
                        alpha(c, if on { 1.0 } else { 0.45 }),
                    );
                    if resp
                        .on_hover_text(*fname)
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        *faction = (*fname).to_owned();
                    }
                }
            });
            let (lr, _) = ui.allocate_exact_size(vec2(w, 9.0), Sense::hover());
            text(
                ui.painter(),
                lr.left_center(),
                Align2::LEFT_CENTER,
                &faction.to_uppercase(),
                mono(8.0),
                INK_DIM,
                0.12,
            );
            let mut cancel = false;
            let ready = !name.trim().is_empty();
            ui.horizontal(|ui| {
                create |= ui
                    .add_enabled_ui(ready, |ui| widgets::go_btn(ui, "CREATE", 84.0, true))
                    .inner
                    .clicked();
                cancel = widgets::tool_btn(ui, "close", None, false, None)
                    .on_hover_text("Cancel")
                    .clicked();
            });
            if cancel {
                app.picker.new = None;
            }
        });
    } else {
        let resp = ui
            .interact(r, egui::Id::unique("vault-new"), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        let ink = if resp.hovered() { FORGE_INK } else { INK_MID };
        icons::paint(
            &p,
            "newdoc",
            pos2(r.center().x, r.center().y - 24.0),
            28.0,
            ink,
        );
        text(
            &p,
            pos2(r.center().x, r.center().y + 6.0),
            Align2::CENTER_CENTER,
            "NEW VAULT",
            mono(9.0),
            ink,
            0.18,
        );
        text(
            &p,
            pos2(r.center().x, r.center().y + 28.0),
            Align2::CENTER_CENTER,
            "name + faction",
            body_font(12.0),
            INK_DIM,
            0.0,
        );
        if resp.clicked() {
            app.picker.new = Some((String::new(), "Venturan".into()));
        }
    }
    create
}

fn open_file_card(app: &mut ForgeApp, ui: &mut Ui, r: Rect) {
    let p = ui.painter().clone();
    add_card_frame(&p, r);
    let resp = ui.interact(r, egui::Id::unique("vault-open"), Sense::hover());
    let ink = if resp.hovered() { FORGE_INK } else { INK_MID };
    icons::paint(
        &p,
        "disc",
        pos2(r.center().x, r.center().y - 24.0),
        28.0,
        ink,
    );
    text(
        &p,
        pos2(r.center().x, r.center().y + 6.0),
        Align2::CENTER_CENTER,
        "OPEN GENESIS",
        mono(9.0),
        ink,
        0.18,
    );
    let msg = app
        .picker
        .error
        .clone()
        .unwrap_or_else(|| "Drop a Genesis .json onto this window".into());
    let c = if app.picker.error.is_some() {
        SIGNAL_HOT
    } else {
        INK_DIM
    };
    text_clipped(
        &p,
        pos2(r.center().x, r.center().y + 28.0),
        Align2::CENTER_CENTER,
        &msg,
        body_font(12.0),
        c,
        0.0,
        r.width() - 16.0,
    );
}

fn vault_card(
    app: &mut ForgeApp,
    ui: &mut Ui,
    r: Rect,
    v: &Vault,
    now: u64,
) -> Option<(String, Option<u32>)> {
    let fc = hex(catalog::faction_color(&v.faction));
    let p = ui.painter().clone();
    let main = Rect::from_min_size(r.min, vec2(r.width(), 150.0 - 30.0));
    let resp = ui
        .interact(main, egui::Id::unique(("vault", &v.id)), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(format!("Open {}", v.name));
    if resp.hovered() {
        glow_rect(&p, r, 10.0, 24.0, alpha(fc, 0.2));
    }
    grad_v(&p, r, 10.0, rgb(0x11151d), rgb(0x0b0e13));
    outline(&p, r, 10.0, if resp.hovered() { fc } else { white(0.07) });
    icons::paint(
        &p,
        "faction",
        pos2(r.left() + 29.0, r.top() + 29.0),
        26.0,
        fc,
    );
    text_clipped(
        &p,
        pos2(r.left() + 52.0, r.top() + 29.0),
        Align2::LEFT_CENTER,
        &v.name,
        serif(13.0),
        INK,
        0.12,
        r.width() - 64.0,
    );
    let mut sub = format!(
        "{} · {}",
        v.faction.to_uppercase(),
        store::ago(v.updated, now)
    );
    if let Some(s) = &v.source {
        sub.push_str(&format!(" · {s}"));
    }
    text_clipped(
        &p,
        pos2(r.left() + 16.0, r.top() + 56.0),
        Align2::LEFT_CENTER,
        &sub,
        mono(8.0),
        INK_DIM,
        0.12,
        r.width() - 32.0,
    );

    let ft = Rect::from_min_size(pos2(r.left(), r.top() + 120.0), vec2(r.width(), 30.0));
    p.line_segment(
        [ft.left_top(), ft.right_top()],
        Stroke::new(1.0, white(0.05)),
    );
    let label = match v.last_version() {
        Some(n) => format!("v{n} · {} saved", v.versions.len()),
        None => "no versions".into(),
    };
    text(
        &p,
        pos2(ft.left() + 16.0, ft.center().y),
        Align2::LEFT_CENTER,
        &label,
        mono(8.0),
        INK_MID,
        0.1,
    );
    let mut out = resp.clicked().then(|| (v.id.clone(), None));
    let hist_open = app.picker.hist.as_deref() == Some(v.id.as_str());
    if !v.versions.is_empty() {
        row_rtl(
            ui,
            ft.shrink2(vec2(10.0, 0.0)),
            ("vhist-btn", &v.id),
            |ui| {
                if widgets::tool_btn(ui, "history", None, hist_open, None)
                    .on_hover_text("Version history")
                    .clicked()
                {
                    app.picker.hist = if hist_open { None } else { Some(v.id.clone()) };
                }
            },
        );
    }
    if hist_open {
        let list = Rect::from_min_max(
            pos2(r.left() + 16.0, ft.bottom() + 2.0),
            pos2(r.right() - 10.0, r.bottom() - 6.0),
        );
        col(ui, list, ("vhist", &v.id), |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                for ver in v.versions.iter().rev() {
                    let (lr, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::hover());
                    let p = ui.painter();
                    text(
                        p,
                        lr.left_center(),
                        Align2::LEFT_CENTER,
                        &format!("v{}", ver.n),
                        mono(9.0),
                        INK,
                        0.0,
                    );
                    text(
                        p,
                        pos2(lr.left() + 30.0, lr.center().y),
                        Align2::LEFT_CENTER,
                        &store::ago(ver.at, now),
                        mono(9.0),
                        INK_MID,
                        0.0,
                    );
                    let b = Rect::from_min_max(
                        pos2(lr.right() - 44.0, lr.top() + 2.0),
                        pos2(lr.right(), lr.bottom() - 2.0),
                    );
                    let br = ui
                        .interact(b, egui::Id::unique(("vopen", &v.id, ver.n)), Sense::click())
                        .on_hover_cursor(CursorIcon::PointingHand);
                    let c = if br.hovered() { SIGNAL_AMBER } else { INK_MID };
                    outline(
                        ui.painter(),
                        b,
                        3.0,
                        if br.hovered() {
                            SIGNAL_AMBER
                        } else {
                            white(0.12)
                        },
                    );
                    text(
                        ui.painter(),
                        b.center(),
                        Align2::CENTER_CENTER,
                        "OPEN",
                        mono(7.5),
                        c,
                        0.1,
                    );
                    if br.clicked() {
                        out = Some((v.id.clone(), Some(ver.n)));
                    }
                }
            });
        });
    }
    out
}

// ---- header ----------------------------------------------------------------

pub fn header(app: &mut ForgeApp, ui: &mut Ui, hd: Rect) {
    let p = ui.painter().clone();
    let r1 = Rect::from_min_size(hd.min, vec2(hd.width(), R1_H));
    let r2 = Rect::from_min_size(pos2(hd.left(), r1.bottom()), vec2(hd.width(), R2_H));
    let r3 = Rect::from_min_size(pos2(hd.left(), r2.bottom()), vec2(hd.width(), R3_H));
    grad_v(&p, r1, 0.0, rgb(0x151a22), rgb(0x0c0f14));
    brushed(&p, r1, 0.015);
    p.line_segment(
        [r1.left_bottom(), r1.right_bottom()],
        Stroke::new(1.0, white(0.05)),
    );
    p.rect_filled(r2, 0.0, rgb(0x090c11));
    p.line_segment(
        [r2.left_bottom(), r2.right_bottom()],
        Stroke::new(1.0, white(0.04)),
    );
    p.rect_filled(r3, 0.0, rgb(0x07090d));
    p.line_segment(
        [hd.left_bottom(), hd.right_bottom()],
        Stroke::new(1.0, Color32::BLACK),
    );

    // Container-query behaviour: labels drop to icons when the header is narrow.
    let compact = hd.width() < 1280.0;
    let tiny = hd.width() < 980.0;
    vault_row(app, ui, r1.shrink2(vec2(10.0, 0.0)), compact, tiny);
    launcher_row(app, ui, r2.shrink2(vec2(10.0, 0.0)), compact, tiny);
    tools_row(app, ui, r3.shrink2(vec2(10.0, 0.0)), compact, tiny);
}

fn vault_row(app: &mut ForgeApp, ui: &mut Ui, r: Rect, compact: bool, tiny: bool) {
    let Some(sess) = app.session.clone() else {
        return;
    };
    let vault = app.vault().cloned();
    let (vname, vfac) = vault
        .as_ref()
        .map(|v| (v.name.clone(), v.faction.clone()))
        .unwrap_or(("—".into(), "Venturan".into()));
    let fc = hex(catalog::faction_color(&vfac));
    let conn = CONNECTIONS[conn_index(&sess.conn)];
    let now = store::now_ms();

    // right cluster first so the left side knows how much room is left
    let right_w = row_rtl(ui, r, "r1-right", |ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let chip = user_chip(ui, &sess.user);
        Popup::menu(&chip).frame(menu_frame()).show(|ui| {
            menu_title(ui, &sess.user.to_uppercase());
            if menu_item(ui, Some("vault"), None, "PROJECT VAULTS", false).clicked() {
                app.actions.push(Action::ToPicker);
            }
            if menu_item(ui, Some("signout"), None, "SIGN OUT", false).clicked() {
                app.actions.push(Action::SignOut);
            }
        });
        widgets::sep(ui);
        perf(
            ui,
            app.fps,
            app.bus.num("transport.step", 0.0) as i32,
            compact,
        );
        ui.min_rect().width()
    });

    let left = Rect::from_min_max(r.min, pos2(r.right() - right_w - 8.0, r.bottom()));
    row(ui, left, "r1-left", |ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let (br, _) = ui.allocate_exact_size(vec2(112.0, 30.0), Sense::hover());
        brand(ui.painter(), br.left_center(), 22.0, 15.0);

        // project switcher
        let max_name = if compact { 120.0 } else { 180.0 };
        let nw = measure(ui.painter(), &vname, mono(10.0), 0.12)
            .x
            .min(max_name);
        let (pr, presp) = ui.allocate_exact_size(vec2(nw + 58.0, 30.0), Sense::click());
        let p = ui.painter();
        p.rect_filled(pr, 6.0, rgb(0x0a0d12));
        outline(p, pr, 6.0, if presp.hovered() { fc } else { white(0.08) });
        icons::paint(p, "vault", pos2(pr.left() + 18.0, pr.center().y), 16.0, fc);
        text_clipped(
            p,
            pos2(pr.left() + 33.0, pr.center().y),
            Align2::LEFT_CENTER,
            &vname,
            mono(10.0),
            INK,
            0.12,
            max_name + 4.0,
        );
        icons::paint(p, "down", pos2(pr.right() - 14.0, pr.center().y), 12.0, fc);
        if presp
            .on_hover_text("Project vaults · switch project")
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            app.actions.push(Action::ToPicker);
        }

        // master connection
        let cc = chc(conn.2);
        let lw = if compact {
            0.0
        } else {
            measure(ui.painter(), conn.1, mono(8.0), 0.14).x + 6.0
        };
        let (cr, cresp) = ui.allocate_exact_size(vec2(47.0 + lw, 30.0), Sense::click());
        let p = ui.painter();
        if cresp.hovered() {
            p.rect_filled(cr, 6.0, white(0.05));
        }
        icons::paint(
            p,
            "conn",
            pos2(cr.left() + 17.0, cr.center().y),
            16.0,
            if cresp.hovered() { INK } else { INK_MID },
        );
        glow_dot(p, pos2(cr.left() + 34.0, cr.center().y), 3.5, cc);
        if !compact {
            text(
                p,
                pos2(cr.left() + 44.0, cr.center().y),
                Align2::LEFT_CENTER,
                conn.1,
                mono(8.0),
                if cresp.hovered() { INK } else { INK_MID },
                0.14,
            );
        }
        let cresp = cresp
            .on_hover_text(format!("Master connection · {}", conn.1))
            .on_hover_cursor(CursorIcon::PointingHand);
        Popup::menu(&cresp).frame(menu_frame()).show(|ui| {
            menu_title(ui, "MASTER CONNECTION");
            for (i, (k, n, c)) in CONNECTIONS.iter().enumerate() {
                if menu_item(ui, None, Some(chc(c)), n, *k == conn.0).clicked() {
                    app.actions.push(Action::SetConn(i));
                }
            }
        });

        widgets::sep(ui);
        genesis_banner(app, ui, compact, tiny);

        // versions
        let vlabel = vault
            .as_ref()
            .and_then(|v| v.last_version())
            .map(|n| format!("v{n}"));
        let vresp = widgets::tool_btn(ui, "history", vlabel.as_deref(), false, None)
            .on_hover_text("Versions · save / restore");
        Popup::menu(&vresp)
            .frame(menu_frame())
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                menu_title(ui, &format!("{vname} · VERSIONS"));
                if widgets::go_btn(ui, "SAVE VERSION", 210.0, true).clicked() {
                    app.actions.push(Action::SaveVersion);
                }
                let versions = vault
                    .as_ref()
                    .map(|v| v.versions.clone())
                    .unwrap_or_default();
                if versions.is_empty() {
                    let (nr, _) = ui.allocate_exact_size(vec2(210.0, 22.0), Sense::hover());
                    text(
                        ui.painter(),
                        nr.left_center() + vec2(4.0, 0.0),
                        Align2::LEFT_CENTER,
                        "No versions yet.",
                        body_font(11.0),
                        INK_DIM,
                        0.0,
                    );
                }
                for ver in versions.iter().rev() {
                    let (lr, _) = ui.allocate_exact_size(vec2(210.0, 24.0), Sense::hover());
                    let p = ui.painter();
                    text(
                        p,
                        lr.left_center() + vec2(4.0, 0.0),
                        Align2::LEFT_CENTER,
                        &format!("v{}", ver.n),
                        mono(9.0),
                        INK,
                        0.0,
                    );
                    text(
                        p,
                        lr.left_center() + vec2(34.0, 0.0),
                        Align2::LEFT_CENTER,
                        &store::ago(ver.at, now),
                        mono(9.0),
                        INK_MID,
                        0.0,
                    );
                    let b = Rect::from_min_max(
                        pos2(lr.right() - 62.0, lr.top() + 3.0),
                        pos2(lr.right(), lr.bottom() - 3.0),
                    );
                    let br = ui
                        .interact(b, egui::Id::unique(("restore", ver.n)), Sense::click())
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text("Replace the current rack with this version");
                    outline(
                        ui.painter(),
                        b,
                        3.0,
                        if br.hovered() {
                            SIGNAL_AMBER
                        } else {
                            white(0.12)
                        },
                    );
                    text(
                        ui.painter(),
                        b.center(),
                        Align2::CENTER_CENTER,
                        "RESTORE",
                        mono(7.5),
                        if br.hovered() { SIGNAL_AMBER } else { INK_MID },
                        0.1,
                    );
                    if br.clicked() {
                        app.actions.push(Action::RestoreVersion(ver.n));
                        ui.close();
                    }
                }
            });
        if widgets::tool_btn(ui, "books", None, false, None)
            .on_hover_text("Asset / Mythos library")
            .clicked()
        {
            app.actions.push(Action::SetStage("library"));
        }
    });
}

fn perf(ui: &mut Ui, fps: f32, step: i32, compact: bool) {
    let w = if compact { 76.0 } else { 112.0 };
    let (r, resp) = ui.allocate_exact_size(vec2(w, 24.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, 4.0, rgb(0x05070a));
    icons::paint(
        p,
        "pulse",
        pos2(r.left() + 15.0, r.center().y),
        14.0,
        INK_DIM,
    );
    let fc = if fps < 30.0 {
        SIGNAL_HOT
    } else if fps < 50.0 {
        SIGNAL_AMBER
    } else {
        SIGNAL_LIFE
    };
    let mut x = r.left() + 26.0;
    let base = r.center().y + 4.0;
    x = text(
        p,
        pos2(x, base),
        Align2::LEFT_BOTTOM,
        &format!("{}", fps.round() as i32),
        mono(11.0),
        fc,
        0.0,
    )
    .right()
        + 4.0;
    if !compact {
        x = text(
            p,
            pos2(x, base),
            Align2::LEFT_BOTTOM,
            "FPS",
            mono(7.0),
            INK_DIM,
            0.14,
        )
        .right()
            + 7.0;
    }
    x = text(
        p,
        pos2(x, base),
        Align2::LEFT_BOTTOM,
        &format!("{step:02}"),
        mono(11.0),
        SIGNAL_COOL,
        0.0,
    )
    .right()
        + 4.0;
    if !compact {
        text(
            p,
            pos2(x, base),
            Align2::LEFT_BOTTOM,
            "TICK",
            mono(7.0),
            INK_DIM,
            0.14,
        );
    }
    resp.on_hover_text("System performance · render FPS · transport tick");
}

/// The mounted Genesis Container: disc, LCD and LOAD menu.
fn genesis_banner(app: &mut ForgeApp, ui: &mut Ui, compact: bool, tiny: bool) {
    let g = app.genesis.clone();
    let dc = g.as_ref().map(|g| chc(g.channel())).unwrap_or(SIGNAL_COOL);
    let playing = app.engine.playing;
    if !tiny {
        let (dr, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
        disc(
            ui.painter(),
            dr.center(),
            20.0,
            dc,
            if playing && g.is_some() {
                app.engine.time as f32 * (360.0 / 3.2)
            } else {
                0.0
            },
        );
    }
    let lw = if tiny || compact { 130.0 } else { 170.0 };
    let (lr, lresp) = ui.allocate_exact_size(vec2(lw, 30.0), Sense::click());
    let p = ui.painter();
    p.rect_filled(lr, 3.0, rgb(0x05070b));
    paint::inner_shade(p, lr, 3.0, 0.5);
    let (line1, line2) = match &g {
        Some(g) => (
            format!(
                "GENESIS · {} · {}",
                g.header.domain.to_uppercase(),
                g.header.lifecycle.to_uppercase()
            ),
            g.header.name.clone(),
        ),
        None => ("NO GENESIS MOUNTED".into(), "— LOAD A CONTAINER —".into()),
    };
    text_clipped(
        p,
        pos2(lr.left() + 10.0, lr.top() + 9.0),
        Align2::LEFT_CENTER,
        &line1,
        mono(7.0),
        INK_DIM,
        0.14,
        lw - 16.0,
    );
    text_clipped(
        p,
        pos2(lr.left() + 10.0, lr.top() + 20.0),
        Align2::LEFT_CENTER,
        &line2,
        display(12.0),
        dc,
        0.08,
        lw - 16.0,
    );
    let lresp = lresp
        .on_hover_text("Genesis Container · click for contents")
        .on_hover_cursor(CursorIcon::PointingHand);

    let (br, bresp) = ui.allocate_exact_size(vec2(56.0, 24.0), Sense::click());
    let p = ui.painter();
    grad_v(p, br, 3.0, rgb(0x252a34), rgb(0x171b22));
    outline(p, br, 3.0, alpha(SIGNAL_COOL, 0.35));
    icons::paint(
        p,
        "export",
        pos2(br.left() + 13.0, br.center().y),
        11.0,
        SIGNAL_COOL,
    );
    text(
        p,
        pos2(br.left() + 23.0, br.center().y),
        Align2::LEFT_CENTER,
        "LOAD",
        mono(8.0),
        SIGNAL_COOL,
        0.1,
    );
    let bresp = bresp.on_hover_cursor(CursorIcon::PointingHand);

    let frame = Frame::new()
        .fill(rgb(0x181c24))
        .stroke(Stroke::new(1.0, Color32::BLACK))
        .corner_radius(8)
        .inner_margin(Margin::same(10));
    let content = |ui: &mut Ui, app: &mut ForgeApp| {
        ui.set_width(440.0);
        ui.horizontal_wrapped(|ui| {
            if widgets::cv_btn(ui, "OROS-V7 · WORLD", true, false).clicked() {
                app.actions
                    .push(Action::Mount(Box::new(Genesis::oros_v7())));
            }
            let (hr, _) = ui.allocate_exact_size(vec2(230.0, 24.0), Sense::hover());
            text(
                ui.painter(),
                hr.left_center(),
                Align2::LEFT_CENTER,
                "or drop a Genesis .json onto the window",
                body_font(11.0),
                INK_DIM,
                0.0,
            );
        });
        if let Some(e) = &app.picker.error {
            ui.colored_label(SIGNAL_HOT, egui::RichText::new(e).font(mono(11.0)));
        }
        let Some(g) = &g else { return };
        ui.label(
            egui::RichText::new(&g.header.description)
                .font(body_font(12.0))
                .color(INK_MID),
        );
        let caps = format!(
            "{}/{} CAPSULES · {}",
            g.header.capsule_count,
            g.header.capsule_capacity,
            g.header.wire_types.join(" ")
        );
        ui.label(egui::RichText::new(caps).font(mono(8.0)).color(INK_MID));
        if g.sealed() {
            ui.label(egui::RichText::new("SEALED · header only. The manifest is encrypted with the Seal file; unsealing runs in the qgcp core.").font(body_font(11.0)).color(SIGNAL_AMBER));
        }
        ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
            for m in g.mythos() {
                let d = DEPTS
                    .iter()
                    .find(|d| d.crest == m.crest)
                    .unwrap_or(&DEPTS[0]);
                let (hr, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
                let p = ui.painter();
                widgets::glyph_tile_at(
                    p,
                    Rect::from_min_size(pos2(hr.left(), hr.top() + 2.0), Vec2::splat(20.0)),
                    d.crest,
                    chc(d.ch),
                    true,
                    false,
                );
                text(
                    p,
                    pos2(hr.left() + 26.0, hr.center().y),
                    Align2::LEFT_CENTER,
                    &m.name,
                    display(10.0),
                    chc(d.ch),
                    0.1,
                );
                text(
                    p,
                    hr.right_center(),
                    Align2::RIGHT_CENTER,
                    &format!("{} · {} capsules", m.department, m.capsule_count()),
                    mono(8.0),
                    INK_DIM,
                    0.0,
                );
                for c in &m.containers {
                    let (cr, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 16.0), Sense::hover());
                    let p = ui.painter();
                    p.circle_filled(pos2(cr.left() + 29.0, cr.center().y), 3.0, dc);
                    text(
                        p,
                        pos2(cr.left() + 38.0, cr.center().y),
                        Align2::LEFT_CENTER,
                        &c.name,
                        mono(10.0),
                        INK,
                        0.0,
                    );
                    text(
                        p,
                        cr.right_center(),
                        Align2::RIGHT_CENTER,
                        &format!("{} capsules", c.capsules.len()),
                        mono(9.0),
                        INK_DIM,
                        0.0,
                    );
                }
            }
        });
    };
    Popup::menu(&lresp)
        .frame(frame)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| content(ui, app));
    Popup::menu(&bresp)
        .frame(frame)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| content(ui, app));
}

/// Compact disc that turns while the transport runs.
pub fn disc(p: &egui::Painter, c: egui::Pos2, r: f32, tone: Color32, angle: f32) {
    p.circle_filled(c + vec2(0.0, 2.0), r, black(0.5));
    radial(p, c, r, rgb(0x2a2f3a), rgb(0x0b0d12), vec2(-0.2, -0.3));
    for (i, (a, col)) in [(0.0, tone), (120.0, SIGNAL_ROSE), (240.0, SIGNAL_AMBER)]
        .into_iter()
        .enumerate()
    {
        let d = angle + a + i as f32 * 7.0;
        arc(p, c, r * 0.72, d, d + 46.0, r * 0.34, alpha(col, 0.5));
    }
    p.circle_stroke(c, r, Stroke::new(1.0, black(0.8)));
    p.circle_filled(c, r * 0.36, rgb(0x14181f));
    p.circle_stroke(c, r * 0.36, Stroke::new(1.0, white(0.18)));
    p.circle_filled(c, r * 0.14, rgb(0x03050a));
}

fn launcher_row(app: &mut ForgeApp, ui: &mut Ui, r: Rect, compact: bool, tiny: bool) {
    let right_w = row_rtl(ui, r, "r2-right", |ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        transport(app, ui, compact);
        widgets::sep(ui);
        chapter(app, ui, compact, tiny);
        ui.min_rect().width()
    });
    let left = Rect::from_min_max(r.min, pos2(r.right() - right_w - 8.0, r.bottom()));
    row(ui, left, "r2-left", |ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        let start = ui.cursor().min;
        let bg = ui.painter().add(egui::Shape::Noop);
        ui.add_space(3.0);
        for (key, icon, name) in APPS {
            let on = app.prefs.app == key;
            let lw = if on && !compact {
                measure(ui.painter(), name, mono(9.0), 0.14).x + 7.0
            } else {
                0.0
            };
            let (br, resp) = ui.allocate_exact_size(vec2(36.0 + lw, 30.0), Sense::click());
            let p = ui.painter();
            let ink = if on {
                FORGE_INK
            } else if resp.hovered() {
                INK
            } else {
                INK_DIM
            };
            if on {
                grad_v(p, br, 5.0, rgb(0x1f2530), rgb(0x161b23));
                outline(p, br, 5.0, alpha(rgb(0xff7a3c), 0.35));
            }
            icons::paint(p, icon, pos2(br.left() + 18.0, br.center().y), 18.0, ink);
            if lw > 0.0 {
                text(
                    p,
                    pos2(br.left() + 34.0, br.center().y),
                    Align2::LEFT_CENTER,
                    name,
                    mono(9.0),
                    ink,
                    0.14,
                );
            }
            if resp
                .on_hover_text(name)
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                app.actions.push(Action::SetApp(key));
            }
        }
        ui.add_space(3.0);
        let well = Rect::from_min_max(
            pos2(start.x, r.center().y - 18.0),
            pos2(ui.cursor().min.x - 3.0, r.center().y + 18.0),
        );
        ui.painter()
            .set(bg, egui::Shape::rect_filled(well, 7.0, rgb(0x05070a)));
    });
}

/// SWING · chapter / scene / beat with FOLLOW. Laid out right-to-left.
fn chapter(app: &mut ForgeApp, ui: &mut Ui, compact: bool, tiny: bool) {
    let bus = &mut app.bus;
    let chap = bus.num("rack.chapter", 0.0) as i32;
    let scene = bus.num("rack.scene", 0.0) as i32;
    let step = bus.num("transport.step", 0.0) as i32;
    let follow = bus.flag("rack.follow", true);
    let mut set_ch: Option<i32> = None;
    if !tiny && widgets::gate_btn(ui, "FOLLOW", Channel::Myth, follow).clicked() {
        bus.set_flag("rack.follow", !follow);
    }
    if widgets::tool_btn(ui, "chevR", None, false, None)
        .on_hover_text("Next chapter")
        .clicked()
    {
        set_ch = Some(chap + 1);
    }
    widgets::readout_box(
        ui,
        Some("BEAT"),
        &format!("{:02}", step + 1),
        Channel::Cool,
        38.0,
    );
    widgets::readout_box(
        ui,
        Some("SCENE"),
        &format!("{}/4", scene + 1),
        Channel::Myth,
        50.0,
    );
    widgets::readout_box(
        ui,
        Some("CH"),
        ROMAN[chap.rem_euclid(12) as usize],
        Channel::Myth,
        48.0,
    );
    if widgets::tool_btn(ui, "chev", None, false, None)
        .on_hover_text("Previous chapter")
        .clicked()
    {
        set_ch = Some(chap - 1);
    }
    if !compact {
        crate::util::knob(
            ui,
            bus,
            "chr.swing",
            Knob::new(24.0, Channel::Amber)
                .variant(KnobVariant::Arc)
                .default_value(0.12)
                .show_value(false),
        )
        .on_hover_text("Swing");
    }
    if let Some(c) = set_ch {
        bus.set_num("rack.chapter", c.rem_euclid(12) as f32);
        bus.set_num("rack.scene", 0.0);
    }
}

/// Transport, tempo and master. Laid out right-to-left.
fn transport(app: &mut ForgeApp, ui: &mut Ui, compact: bool) {
    crate::util::knob(
        ui,
        &mut app.bus,
        "forge.main",
        Knob::new(28.0, Channel::Forge)
            .variant(KnobVariant::Forge)
            .default_value(0.78)
            .show_value(false),
    )
    .on_hover_text("Master");
    if !compact {
        let (vr, _) = ui.allocate_exact_size(vec2(56.0, 15.0), Sense::hover());
        for (i, (l, v)) in [("L", app.engine.rms.l), ("R", app.engine.rms.r)]
            .into_iter()
            .enumerate()
        {
            let r =
                Rect::from_min_size(pos2(vr.left(), vr.top() + i as f32 * 9.0), vec2(56.0, 6.0));
            util_vu(ui, r, v, l);
        }
    }
    widgets::readout_box(
        ui,
        Some("BPM"),
        &format!("{:.1}", Engine::bpm_exact(&app.bus)),
        Channel::Amber,
        56.0,
    );
    crate::util::knob(
        ui,
        &mut app.bus,
        "chr.bpm",
        Knob::new(26.0, Channel::Amber)
            .variant(KnobVariant::Ringed)
            .default_value(0.53)
            .show_value(false),
    )
    .on_hover_text("Tempo");
    let (armed, rec, playing) = (
        app.engine.armed,
        app.bus.flag("engine.rec", false),
        app.engine.playing,
    );
    if widgets::gate_btn(ui, if armed { "ARMED" } else { "ARM" }, Channel::Hot, armed)
        .on_hover_text("Arm the output engine")
        .clicked()
    {
        app.engine.toggle_arm(&mut app.bus);
    }
    if widgets::pad(ui, Face::Icon("rec"), Channel::Hot, rec, 26.0)
        .on_hover_text("Record take")
        .clicked()
    {
        app.engine.toggle_rec(&mut app.bus);
    }
    if widgets::pad(ui, Face::Icon("stop"), Channel::Hot, false, 26.0)
        .on_hover_text("Stop")
        .clicked()
    {
        app.engine.stop(&mut app.bus);
        app.bus.set_num("rack.scene", 0.0);
    }
    if widgets::pad(
        ui,
        Face::Icon(if playing { "pause" } else { "play" }),
        Channel::Life,
        playing,
        26.0,
    )
    .on_hover_text("Play / pause")
    .clicked()
    {
        app.engine.toggle_play(&mut app.bus);
    }
}

fn tools_row(app: &mut ForgeApp, ui: &mut Ui, r: Rect, compact: bool, tiny: bool) {
    let label = |ui: &mut Ui, s: &str| {
        if compact {
            return;
        }
        let w = measure(ui.painter(), s, mono(8.0), 0.12).x;
        let (lr, _) = ui.allocate_exact_size(vec2(w + 12.0, 20.0), Sense::hover());
        text(
            ui.painter(),
            lr.center(),
            Align2::CENTER_CENTER,
            s,
            mono(8.0),
            INK_DIM,
            0.12,
        );
    };
    row(ui, r, "r3", |ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        match app.prefs.app.as_str() {
            "forge" => {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (key, icon, name, tip) in STAGES {
                    let on = app.prefs.stage == key;
                    let lw = if on && !tiny {
                        measure(ui.painter(), name, mono(8.5), 0.14).x + 6.0
                    } else {
                        0.0
                    };
                    let (br, resp) = ui.allocate_exact_size(vec2(32.0 + lw, 28.0), Sense::click());
                    let p = ui.painter();
                    let ink = if on {
                        GOLD
                    } else if resp.hovered() {
                        INK
                    } else {
                        INK_DIM
                    };
                    if on {
                        p.rect_filled(br, 5.0, alpha(GOLD, 0.1));
                    } else if resp.hovered() {
                        p.rect_filled(br, 5.0, white(0.04));
                    }
                    icons::paint(p, icon, pos2(br.left() + 16.0, br.center().y), 16.0, ink);
                    if lw > 0.0 {
                        text(
                            p,
                            pos2(br.left() + 30.0, br.center().y),
                            Align2::LEFT_CENTER,
                            name,
                            mono(8.5),
                            ink,
                            0.14,
                        );
                    }
                    if resp
                        .on_hover_text(format!("{name} · {tip}"))
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        app.actions.push(Action::SetStage(key));
                    }
                }
                ui.spacing_mut().item_spacing.x = 6.0;
                widgets::sep(ui);
                let d = catalog::dept(app.draft.dept);
                widgets::glyph_tile(ui, d.crest, chc(d.ch), 20.0, true, false).on_hover_text(
                    format!("Editing instrument · {} modules", app.draft.modules.len()),
                );
                if !compact {
                    let w = measure(ui.painter(), &app.draft.name, mono(9.0), 0.06)
                        .x
                        .min(140.0);
                    let (lr, _) = ui.allocate_exact_size(vec2(w + 4.0, 20.0), Sense::hover());
                    text_clipped(
                        ui.painter(),
                        lr.left_center(),
                        Align2::LEFT_CENTER,
                        &app.draft.name,
                        mono(9.0),
                        INK_MID,
                        0.06,
                        144.0,
                    );
                }
                widgets::sep(ui);
                crate::forge::tools(app, ui, compact);
            }
            "rack" => {
                icons::paint(
                    ui.painter(),
                    "rack",
                    pos2(ui.cursor().min.x + 8.0, r.center().y),
                    16.0,
                    INK_MID,
                );
                ui.add_space(18.0);
                label(ui, "MAIN RACK");
                widgets::sep(ui);
                label(
                    ui,
                    &format!(
                        "{} INSTRUMENTS · {} ROUTES",
                        app.state.rack.len(),
                        app.state.routes.len()
                    ),
                );
                if widgets::tool_btn(ui, "down", None, false, None)
                    .on_hover_text("Fold all")
                    .clicked()
                {
                    app.actions.push(Action::FoldAll(false));
                }
                if widgets::tool_btn(ui, "layouts", None, false, None)
                    .on_hover_text("Unfold all")
                    .clicked()
                {
                    app.actions.push(Action::FoldAll(true));
                }
                if widgets::tool_btn(ui, "euro", None, false, None)
                    .on_hover_text("Patch in Eurorack")
                    .clicked()
                {
                    app.actions.push(Action::SetApp("modular"));
                }
            }
            "modular" => {
                icons::paint(
                    ui.painter(),
                    "euro",
                    pos2(ui.cursor().min.x + 8.0, r.center().y),
                    16.0,
                    INK_MID,
                );
                ui.add_space(18.0);
                label(ui, "EURORACK · MODULAR");
                widgets::sep(ui);
                let active = app.state.routes.iter().filter(|c| c.on).count();
                label(
                    ui,
                    &format!("{} ROUTES · {} ACTIVE", app.state.routes.len(), active),
                );
                if widgets::tool_btn(ui, "rack", None, false, None)
                    .on_hover_text("Back to the rack")
                    .clicked()
                {
                    app.actions.push(Action::SetApp("rack"));
                }
            }
            _ => {
                icons::paint(
                    ui.painter(),
                    "perform",
                    pos2(ui.cursor().min.x + 8.0, r.center().y),
                    16.0,
                    INK_MID,
                );
                ui.add_space(18.0);
                label(ui, "PERFORMANCE · DJ");
            }
        }
    });
}

// ---- side rails ------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct SideItem {
    pub key: String,
    pub icon: &'static str,
    pub color: Option<Color32>,
    pub title: String,
    pub sub: String,
    pub badge: Option<usize>,
    pub sep: bool,
}

impl SideItem {
    pub fn new(key: &str, icon: &'static str, title: &str, sub: &str) -> Self {
        Self {
            key: key.into(),
            icon,
            title: title.into(),
            sub: sub.into(),
            ..Default::default()
        }
    }
    pub fn sep() -> Self {
        Self {
            sep: true,
            ..Default::default()
        }
    }
    pub fn color(mut self, c: Color32) -> Self {
        self.color = Some(c);
        self
    }
    pub fn badge(mut self, n: usize) -> Self {
        self.badge = (n > 0).then_some(n);
        self
    }
}

pub struct SideOut {
    /// Area left for the workspace after this side took its share.
    pub rest: Rect,
    /// Flyout body rect and the active tab key, when the panel is open.
    pub body: Option<(Rect, String)>,
}

/// Icon rail + flyout. Clicking the active icon collapses the panel; state is
/// remembered per `id`.
pub fn side(
    prefs: &mut Prefs,
    ui: &mut Ui,
    area: Rect,
    id: &str,
    right: bool,
    items: &[SideItem],
    width: f32,
    def_open: bool,
) -> SideOut {
    let first = items
        .iter()
        .find(|i| !i.sep)
        .map(|i| i.key.clone())
        .unwrap_or_default();
    let st = prefs
        .sides
        .entry(id.to_owned())
        .or_insert_with(|| SideState {
            tab: first.clone(),
            open: def_open,
        });
    if !items.iter().any(|i| !i.sep && i.key == st.tab) {
        st.tab = first;
    }
    let rail = if right {
        Rect::from_min_max(pos2(area.right() - RAIL_W, area.top()), area.max)
    } else {
        Rect::from_min_size(area.min, vec2(RAIL_W, area.height()))
    };
    let p = ui.painter().clone();
    p.rect_filled(rail, 0.0, rgb(0x05070b));
    let edge = if right { rail.left() } else { rail.right() };
    p.line_segment(
        [pos2(edge, rail.top()), pos2(edge, rail.bottom())],
        Stroke::new(1.0, white(0.06)),
    );

    let mut click: Option<String> = None;
    let mut toggle = false;
    col(
        ui,
        Rect::from_min_max(
            pos2(rail.left(), rail.top() + 8.0),
            pos2(rail.right(), rail.bottom() - 34.0),
        ),
        ("rail", id),
        |ui| {
            ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    for it in items {
                        if it.sep {
                            let (sr, _) = ui.allocate_exact_size(vec2(RAIL_W, 9.0), Sense::hover());
                            ui.painter().line_segment(
                                [
                                    pos2(sr.center().x - 10.0, sr.center().y),
                                    pos2(sr.center().x + 10.0, sr.center().y),
                                ],
                                Stroke::new(1.0, white(0.08)),
                            );
                            continue;
                        }
                        let (cell, resp) =
                            ui.allocate_exact_size(vec2(RAIL_W, 34.0), Sense::click());
                        let br = Rect::from_center_size(cell.center(), Vec2::splat(34.0));
                        let accent = it.color.unwrap_or(GOLD);
                        let cur = st.tab == it.key;
                        let p = ui.painter();
                        let ink = if cur && st.open {
                            p.rect_filled(br, 7.0, alpha(accent, 0.12));
                            let x = if right {
                                br.right() - 1.0
                            } else {
                                br.left() + 1.0
                            };
                            p.line_segment(
                                [pos2(x, br.top() + 2.0), pos2(x, br.bottom() - 2.0)],
                                Stroke::new(2.0, accent),
                            );
                            accent
                        } else if resp.hovered() {
                            p.rect_filled(br, 7.0, white(0.05));
                            INK
                        } else if cur {
                            mix(accent, rgb(0x6b7a90), 0.7)
                        } else {
                            rgb(0x6b7a90)
                        };
                        icons::paint(p, it.icon, br.center(), 18.0, ink);
                        if let Some(n) = it.badge {
                            widgets::badge(p, pos2(br.right(), br.top() + 1.0), n);
                        }
                        let tip = if it.sub.is_empty() {
                            it.title.clone()
                        } else {
                            format!("{} · {}", it.title, it.sub)
                        };
                        if resp
                            .on_hover_text(tip)
                            .on_hover_cursor(CursorIcon::PointingHand)
                            .clicked()
                        {
                            click = Some(it.key.clone());
                        }
                    }
                });
        },
    );
    // collapse / expand chevron
    let cb = Rect::from_center_size(
        pos2(rail.center().x, rail.bottom() - 18.0),
        vec2(34.0, 26.0),
    );
    let cresp = ui
        .interact(cb, egui::Id::unique(("rail-toggle", id)), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(if st.open {
            "Collapse panel"
        } else {
            "Expand panel"
        });
    if cresp.hovered() {
        p.rect_filled(cb, 7.0, white(0.05));
    }
    icons::paint(
        &p,
        if right != st.open { "chev" } else { "chevR" },
        cb.center(),
        14.0,
        if cresp.hovered() { INK } else { rgb(0x6b7a90) },
    );
    toggle |= cresp.clicked();

    if let Some(k) = click {
        if k == st.tab && st.open {
            st.open = false;
        } else {
            st.tab = k;
            st.open = true;
        }
    }
    if toggle {
        st.open = !st.open;
    }

    let mut rest = area;
    if right {
        rest.max.x = rail.left();
    } else {
        rest.min.x = rail.right();
    }
    if !st.open {
        return SideOut { rest, body: None };
    }
    let width = width.min((rest.width() - 200.0).max(160.0));
    let fly = if right {
        Rect::from_min_max(pos2(rest.right() - width, rest.top()), rest.max)
    } else {
        Rect::from_min_size(rest.min, vec2(width, rest.height()))
    };
    if right {
        rest.max.x = fly.left();
    } else {
        rest.min.x = fly.right();
    }
    grad_v(&p, fly, 0.0, rgb(0x0b0e14), rgb(0x07090f));
    let fe = if right { fly.left() } else { fly.right() };
    p.line_segment(
        [pos2(fe, fly.top()), pos2(fe, fly.bottom())],
        Stroke::new(1.0, white(0.06)),
    );
    let cur = items.iter().find(|i| !i.sep && i.key == st.tab);
    let hd = Rect::from_min_size(fly.min, vec2(fly.width(), 32.0));
    if let Some(cur) = cur {
        let tr = text_clipped(
            &p,
            pos2(hd.left() + 12.0, hd.bottom() - 9.0),
            Align2::LEFT_BOTTOM,
            &cur.title,
            mono(9.0),
            INK,
            0.16,
            hd.width() - 24.0,
        );
        if !cur.sub.is_empty() {
            text_clipped(
                &p,
                pos2(tr.right() + 8.0, hd.bottom() - 9.0),
                Align2::LEFT_BOTTOM,
                &cur.sub,
                mono(8.0),
                INK_DIM,
                0.1,
                (hd.right() - tr.right() - 20.0).max(1.0),
            );
        }
    }
    p.line_segment(
        [hd.left_bottom(), hd.right_bottom()],
        Stroke::new(1.0, white(0.05)),
    );
    let body = Rect::from_min_max(pos2(fly.left(), hd.bottom() + 1.0), fly.max);
    SideOut {
        rest,
        body: Some((body, st.tab.clone())),
    }
}

/// Placeholder for a workspace that is not built yet.
pub fn stub(ui: &mut Ui, area: Rect, icon: &str, name: &str, blurb: &str, note: &str) {
    let p = ui.painter();
    let c = area.center();
    icons::paint(p, icon, pos2(c.x, c.y - 58.0), 56.0, rgb(0x3c4656));
    let lock = pos2(c.x + 30.0, c.y - 30.0);
    p.circle_filled(lock, 12.0, rgb(0x0d1118));
    p.circle_stroke(lock, 12.0, Stroke::new(1.0, white(0.1)));
    icons::paint(p, "lock", lock, 14.0, INK_MID);
    text(
        p,
        pos2(c.x, c.y + 6.0),
        Align2::CENTER_CENTER,
        name,
        serif(16.0),
        INK,
        0.24,
    );
    text(
        p,
        pos2(c.x, c.y + 32.0),
        Align2::CENTER_CENTER,
        blurb,
        body_font(13.0),
        INK_MID,
        0.0,
    );
    text(
        p,
        pos2(c.x, c.y + 58.0),
        Align2::CENTER_CENTER,
        note,
        mono(8.0),
        INK_DIM,
        0.16,
    );
}

fn util_vu(ui: &mut Ui, r: Rect, level: f32, label: &str) {
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(egui::Id::unique(("hd-vu", label)))
            .max_rect(r),
    );
    widgets::vu(&mut child, level, r.width(), r.height(), Some(label));
}
