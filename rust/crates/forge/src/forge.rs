//! FORGE UI DESIGNER — one instrument draft, five stages:
//! LIBRARY (every asset, live) · MODULES (the eurorack case) · PANEL (the
//! pane editor) · LAYOUTS (partition templates) · NODE GRAPH.

use crate::app::{Action, ForgeApp};
use crate::assets::{self, ASSETS, FAMILIES, MATERIALS};
use crate::rack::sub_head;
use crate::shell::{self, SideItem};
use crate::util::{self, chc, col};
use egui::{
    pos2, vec2, Align2, Color32, CursorIcon, Id, Rect, ScrollArea, Sense, Stroke, Ui, Vec2,
};
use instrument_kit::icons;
use instrument_kit::paint::{self, *};
use instrument_kit::theme::*;
use instrument_kit::widgets::{self, Knob};
use rack_core::catalog::{self, DEPTS, FACTIONS, MODULE_GROUPS, SKINS};
use rack_core::graph::{self, Graph, Wire};
use rack_core::model::{Instrument, MacroRef, ModuleRef, MAX_MACROS};
use rack_core::panel::{self, Dir, PRESETS, TEMPLATES};

/// Transient editor state.
#[derive(Default)]
pub struct ForgeUi {
    pub family: Option<&'static str>,
    pub search: String,
    pub pane: Option<String>,
    pub play: bool,
    pub node: Option<String>,
    /// Output port a wire is being pulled from.
    pub wire_from: Option<(String, usize)>,
    pub msg: String,
}

pub fn body(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    ui.painter().rect_filled(area, 0.0, SHELL);
    match app.prefs.stage.as_str() {
        "case" => modules_stage(app, ui, area),
        "panel" => panel_stage(app, ui, area),
        "layouts" => layouts_stage(app, ui, area),
        "graph" => graph_stage(app, ui, area),
        _ => library_stage(app, ui, area),
    }
}

fn name_field(ui: &mut Ui, value: &mut String, hint: &str, width: f32) {
    widgets::text_field(ui, value, hint, width, 26.0, GOLD);
}

/// Stage tools in the workspace row.
pub fn tools(app: &mut ForgeApp, ui: &mut Ui, compact: bool) {
    match app.prefs.stage.as_str() {
        "panel" | "layouts" => {
            let panel_stage = app.prefs.stage == "panel";
            name_field(
                ui,
                &mut app.forge_doc.current.name,
                "Faceplate name",
                if compact { 120.0 } else { 160.0 },
            );
            if panel_stage {
                if widgets::tool_btn(ui, "edit", None, !app.fui.play, Some(SIGNAL_COOL))
                    .on_hover_text("Edit · move and remove components")
                    .clicked()
                {
                    app.fui.play = false;
                }
                if widgets::tool_btn(ui, "playm", None, app.fui.play, Some(SIGNAL_LIFE))
                    .on_hover_text("Play · the controls are live")
                    .clicked()
                {
                    app.fui.play = true;
                }
                widgets::sep(ui);
            }
            let sel = app
                .fui
                .pane
                .clone()
                .filter(|p| app.forge_doc.current.tree.pane(p).is_some());
            ui.add_enabled_ui(sel.is_some(), |ui| {
                for (icon, dir, tip) in [
                    ("splitV", Dir::Row, "Split pane left / right"),
                    ("splitH", Dir::Col, "Split pane top / bottom"),
                ] {
                    if widgets::tool_btn(ui, icon, None, false, None)
                        .on_hover_text(tip)
                        .on_disabled_hover_text("Select a pane first")
                        .clicked()
                    {
                        if let Some(p) = &sel {
                            app.forge_doc.current.split(p, dir);
                        }
                    }
                }
                if widgets::tool_btn(ui, "unpane", None, false, None)
                    .on_hover_text("Remove pane")
                    .on_disabled_hover_text("Select a pane first")
                    .clicked()
                {
                    if let Some(p) = &sel {
                        if app.forge_doc.current.tree.remove_pane(p) {
                            app.fui.pane = None;
                        }
                    }
                }
            });
            let n = app.forge_doc.current.tree.panes().len();
            let pi = PRESETS
                .iter()
                .position(|p| p.0 == app.forge_doc.current.preset)
                .unwrap_or(1);
            if widgets::cv_btn(ui, PRESETS[pi].1, false, false)
                .on_hover_text("Faceplate format · click to change")
                .clicked()
            {
                app.forge_doc.current.preset = PRESETS[(pi + 1) % PRESETS.len()].0.into();
            }
            if !compact {
                let (lr, _) = ui.allocate_exact_size(vec2(58.0, 20.0), Sense::hover());
                text(
                    ui.painter(),
                    lr.center(),
                    Align2::CENTER_CENTER,
                    &format!("{n} PANES"),
                    mono(8.0),
                    INK_DIM,
                    0.12,
                );
            }
            if panel_stage {
                if widgets::tool_btn(ui, "newdoc", None, false, None)
                    .on_hover_text("New faceplate")
                    .clicked()
                {
                    app.forge_doc.current = panel::template("blank");
                    app.forge_doc.current.name = "Untitled Faceplate".into();
                    app.fui.pane = None;
                }
                let bound = app.forge_doc.current.bind.clone();
                if widgets::tool_btn(ui, "save", Some("SAVE"), false, Some(SIGNAL_LIFE))
                    .on_hover_text(
                        "Save the faceplate; a bound instrument uses it as its front panel",
                    )
                    .clicked()
                {
                    let id = app.forge_doc.save_current();
                    if let Some(inst) =
                        bound.and_then(|b| app.state.lib.iter_mut().find(|i| i.id == b))
                    {
                        inst.faceplate = Some(id.clone());
                    }
                    if app.draft.id == app.forge_doc.current.bind.clone().unwrap_or_default() {
                        app.draft.faceplate = Some(id);
                    }
                    app.fui.msg = "Faceplate saved".into();
                }
            } else if widgets::tool_btn(ui, "panel", Some("OPEN IN PANEL"), false, Some(GOLD))
                .clicked()
            {
                app.actions.push(Action::SetStage("panel"));
            }
        }
        "graph" => {
            if widgets::tool_btn(ui, "playm", Some("RUN"), false, Some(SIGNAL_LIFE))
                .on_hover_text("Walk the graph in dependency order")
                .clicked()
            {
                app.run_graph();
            }
            if widgets::cv_btn(ui, "CLEAR OUT", false, false).clicked() {
                app.graph.nodes.iter_mut().for_each(|n| {
                    n.out = None;
                    n.skipped = false;
                });
            }
            if widgets::cv_btn(ui, "DEMO", false, false).clicked() {
                app.graph = Graph::demo();
            }
            if widgets::cv_btn(ui, "EMPTY", false, true).clicked() {
                app.graph = Graph::default();
            }
            if !compact {
                let hint = if app.fui.wire_from.is_some() {
                    "Now click an in-port · Esc cancels".to_owned()
                } else if app.fui.msg.is_empty() {
                    "Click an out-port, then an in-port · click a wire to cut".to_owned()
                } else {
                    app.fui.msg.clone()
                };
                let (lr, _) = ui.allocate_exact_size(vec2(340.0, 20.0), Sense::hover());
                text_clipped(
                    ui.painter(),
                    lr.left_center(),
                    Align2::LEFT_CENTER,
                    &hint,
                    instrument_kit::theme::body(11.0),
                    INK_DIM,
                    0.0,
                    340.0,
                );
            }
        }
        "case" => scope_tools(app, ui),
        _ => {}
    }
}

fn scope_tools(app: &mut ForgeApp, ui: &mut Ui) {
    let avail = ui.available_width();
    if avail < 17.0 * 25.0 {
        return;
    }
    ui.add_space(avail - 17.0 * 25.0);
    ui.spacing_mut().item_spacing.x = 1.0;
    for (i, d) in DEPTS.iter().enumerate() {
        let on = app.prefs.scope == i as i32;
        let (r, resp) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
        let c = chc(d.ch);
        if on {
            ui.painter().rect_filled(r, 4.0, alpha(c, 0.15));
            outline(ui.painter(), r, 4.0, c);
        }
        icons::paint(
            ui.painter(),
            d.crest,
            r.center(),
            16.0,
            if on || resp.hovered() {
                c
            } else {
                rgb(0x3c4656)
            },
        );
        if resp
            .on_hover_text(format!("Module scope · {} {} · {}", d.num, d.name, d.law))
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            app.prefs.scope = if on { -1 } else { i as i32 };
        }
    }
}

// ---- ① LIBRARY ---------------------------------------------------------------

fn library_stage(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    let items = [SideItem::new(
        "fam",
        "library",
        "FAMILIES",
        &format!("{} / {}", FAMILIES.len(), ASSETS.len()),
    )];
    let left = shell::side(
        &mut app.prefs,
        ui,
        area,
        "lib.L",
        false,
        &items,
        220.0,
        true,
    );
    if let Some((r, _)) = &left.body {
        col(ui, r.shrink2(vec2(12.0, 0.0)), "lib-fam", |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.add_space(12.0);
            let w = ui.available_width();
            widgets::text_field(ui, &mut app.fui.search, "Search assets…", w, 28.0, GOLD);
            ui.add_space(6.0);
            let fam_row = |ui: &mut Ui, label: &str, n: usize, on: bool| -> bool {
                let (fr, resp) = ui.allocate_exact_size(vec2(w, 23.0), Sense::click());
                let p = ui.painter();
                p.rect_filled(
                    fr,
                    3.0,
                    if on {
                        alpha(SIGNAL_FORGE, 0.16)
                    } else if resp.hovered() {
                        white(0.04)
                    } else {
                        rgb(0x0a0d12)
                    },
                );
                text(
                    p,
                    pos2(fr.left() + 9.0, fr.center().y),
                    Align2::LEFT_CENTER,
                    label,
                    mono(8.0),
                    if on { INK } else { INK_MID },
                    0.14,
                );
                text(
                    p,
                    pos2(fr.right() - 8.0, fr.center().y),
                    Align2::RIGHT_CENTER,
                    &n.to_string(),
                    mono(8.0),
                    INK_DIM,
                    0.0,
                );
                resp.on_hover_cursor(CursorIcon::PointingHand).clicked()
            };
            if fam_row(ui, "ALL", ASSETS.len(), app.fui.family.is_none()) {
                app.fui.family = None;
            }
            for f in FAMILIES {
                let n = ASSETS.iter().filter(|a| a.family == f).count();
                if fam_row(ui, f, n, app.fui.family == Some(f)) {
                    app.fui.family = Some(f);
                }
            }
        });
    }
    let q = app.fui.search.to_lowercase();
    col(ui, left.rest, "lib-grid", |ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let w = ui.available_width() - 32.0;
            let x0 = ui.cursor().left() + 16.0;
            let cols = (w / 230.0).floor().max(1.0) as usize;
            let cw = (w - (cols as f32 - 1.0) * 2.0) / cols as f32;
            for fam in FAMILIES {
                if app.fui.family.is_some_and(|f| f != fam) {
                    continue;
                }
                let list: Vec<&assets::Asset> = ASSETS
                    .iter()
                    .filter(|a| {
                        a.family == fam && (q.is_empty() || a.label.to_lowercase().contains(&q))
                    })
                    .collect();
                if list.is_empty() {
                    continue;
                }
                ui.add_space(14.0);
                let (hr, _) = ui.allocate_exact_size(vec2(w + 32.0, 22.0), Sense::hover());
                let t = engrave(
                    ui.painter(),
                    pos2(x0, hr.center().y),
                    Align2::LEFT_CENTER,
                    fam,
                    10.0,
                    INK_MID,
                );
                text(
                    ui.painter(),
                    pos2(t.right() + 8.0, hr.center().y + 1.0),
                    Align2::LEFT_CENTER,
                    &list.len().to_string(),
                    mono(8.0),
                    INK_DIM,
                    0.0,
                );
                for chunk in list.chunks(cols) {
                    let (rr, _) = ui.allocate_exact_size(vec2(w + 32.0, 178.0), Sense::hover());
                    for (i, a) in chunk.iter().enumerate() {
                        let tile = Rect::from_min_size(
                            pos2(x0 + i as f32 * (cw + 2.0), rr.top()),
                            vec2(cw, 176.0),
                        );
                        if !ui.is_rect_visible(tile) {
                            continue;
                        }
                        let p = ui.painter().clone();
                        grad(&p, tile, 0, rgb(0x232845), rgb(0x18333f), false);
                        brushed(&p, tile, 0.012);
                        let foot =
                            Rect::from_min_max(pos2(tile.left(), tile.bottom() - 28.0), tile.max);
                        p.rect_filled(foot, 0.0, rgb(0x0b0e14));
                        let tag = Rect::from_min_size(
                            pos2(foot.left() + 8.0, foot.center().y - 6.0),
                            vec2(18.0, 12.0),
                        );
                        p.rect_filled(tag, 2.0, SIGNAL_COOL);
                        text(
                            &p,
                            tag.center(),
                            Align2::CENTER_CENTER,
                            "DS",
                            mono(7.0),
                            rgb(0x04121a),
                            0.0,
                        );
                        text_clipped(
                            &p,
                            pos2(tag.right() + 7.0, foot.center().y),
                            Align2::LEFT_CENTER,
                            a.label,
                            mono(8.0),
                            INK,
                            0.1,
                            cw - 100.0,
                        );
                        let stage = Rect::from_min_max(tile.min, pos2(tile.right(), foot.top()));
                        let mut child = ui.new_child(
                            egui::UiBuilder::new()
                                .id_salt(Id::unique(("libtile", a.key)))
                                .max_rect(stage),
                        );
                        child.set_clip_rect(stage.intersect(ui.clip_rect()));
                        assets::draw(
                            &mut child,
                            &mut app.bus,
                            &mut app.engine,
                            a.key,
                            &format!("forge.lib.{}", a.key),
                            stage,
                            if a.family == "ROTARIES" { "TONE" } else { "" },
                            util::ch(a.ch),
                            true,
                        );
                        let br = Rect::from_min_max(
                            pos2(foot.right() - 58.0, foot.top() + 5.0),
                            pos2(foot.right() - 6.0, foot.bottom() - 5.0),
                        );
                        let resp = ui
                            .interact(br, Id::unique(("lib-add", a.key)), Sense::click())
                            .on_hover_text("Place on the selected pane of the faceplate")
                            .on_hover_cursor(CursorIcon::PointingHand);
                        outline(
                            &p,
                            br,
                            3.0,
                            alpha(SIGNAL_COOL, if resp.hovered() { 0.9 } else { 0.4 }),
                        );
                        text(
                            &p,
                            br.center(),
                            Align2::CENTER_CENTER,
                            "+ PANEL",
                            mono(7.5),
                            SIGNAL_COOL,
                            0.1,
                        );
                        if resp.clicked() {
                            add_to_panel(app, a.key);
                            app.actions.push(Action::SetStage("panel"));
                        }
                    }
                }
            }
            ui.add_space(80.0);
        });
    });
}

fn add_to_panel(app: &mut ForgeApp, key: &str) {
    let fp = &mut app.forge_doc.current;
    let pane = app
        .fui
        .pane
        .clone()
        .filter(|p| fp.tree.pane(p).is_some())
        .or_else(|| fp.tree.panes().first().map(|p| p.id.clone()));
    if let Some(pane) = pane {
        let label = assets::asset(key)
            .map(|a| a.label.split(" · ").next().unwrap_or(a.label))
            .unwrap_or("");
        fp.add_component(
            &pane,
            key,
            if key.starts_with("knob") {
                "TONE"
            } else {
                label
            },
        );
        app.fui.pane = Some(pane);
    }
}

// ---- ② MODULES ---------------------------------------------------------------

fn add_module(app: &mut ForgeApp, key: &str, at: Option<usize>) {
    let uid = app.state.ids.next("m");
    let at = at
        .unwrap_or(app.draft.modules.len())
        .min(app.draft.modules.len());
    app.draft.modules.insert(
        at,
        ModuleRef {
            uid,
            kind: key.into(),
        },
    );
}

fn modules_stage(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    let mut items = vec![
        SideItem::new("all", "library", "ALL MODULES", ""),
        SideItem::sep(),
    ];
    for (i, d) in DEPTS.iter().enumerate() {
        items.push(
            SideItem::new(
                &format!("d{i}"),
                d.crest,
                &format!("{} · {}", d.num, d.name),
                d.law,
            )
            .color(chc(d.ch)),
        );
    }
    let left = shell::side(
        &mut app.prefs,
        ui,
        area,
        "case.L",
        false,
        &items,
        250.0,
        true,
    );
    if let Some((r, tab)) = &left.body {
        let crest = tab
            .strip_prefix('d')
            .and_then(|n| n.parse::<usize>().ok())
            .map(|i| DEPTS[i].crest);
        palette(app, ui, *r, crest);
    }
    let right = shell::side(
        &mut app.prefs,
        ui,
        left.rest,
        "case.R",
        true,
        &[SideItem::new(
            "inst",
            "device",
            "INSTRUMENT",
            &format!("{} MOD", app.draft.modules.len()),
        )],
        300.0,
        true,
    );
    if let Some((r, _)) = &right.body {
        props(app, ui, *r);
    }
    case(app, ui, right.rest);
}

fn palette(app: &mut ForgeApp, ui: &mut Ui, area: Rect, crest: Option<&str>) {
    col(ui, area, "palette", |ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            let w = ui.available_width() - 20.0;
            let mut any = false;
            for (group, keys) in MODULE_GROUPS {
                let list: Vec<&catalog::ModuleDef> = keys
                    .iter()
                    .filter_map(|k| catalog::module(k))
                    .filter(|m| crest.is_none_or(|c| m.crest == c))
                    .collect();
                if list.is_empty() {
                    continue;
                }
                any = true;
                ui.add_space(6.0);
                let (hr, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 16.0), Sense::hover());
                engrave(
                    ui.painter(),
                    pos2(hr.left() + 10.0, hr.center().y),
                    Align2::LEFT_CENTER,
                    group,
                    9.0,
                    INK_MID,
                );
                for m in list {
                    let (cell, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::hover());
                    let r =
                        Rect::from_min_size(pos2(cell.left() + 10.0, cell.top()), vec2(w, 40.0));
                    let resp = ui
                        .interact(r, Id::unique(("pal", m.key)), Sense::click())
                        .on_hover_text(m.blurb)
                        .on_hover_cursor(CursorIcon::PointingHand);
                    let p = ui.painter();
                    grad_v(
                        p,
                        r,
                        6.0,
                        if resp.hovered() {
                            rgb(0x222733)
                        } else {
                            rgb(0x1a1e26)
                        },
                        rgb(0x14171d),
                    );
                    outline_outside(p, r, 6.0, black(0.6));
                    widgets::glyph_tile_at(
                        p,
                        Rect::from_min_size(pos2(r.left() + 6.0, r.top() + 5.0), Vec2::splat(30.0)),
                        m.crest,
                        chc(m.ch),
                        false,
                        false,
                    );
                    text_clipped(
                        p,
                        pos2(r.left() + 44.0, r.top() + 14.0),
                        Align2::LEFT_CENTER,
                        m.name,
                        mono(10.0),
                        INK,
                        0.08,
                        w - 120.0,
                    );
                    text_clipped(
                        p,
                        pos2(r.left() + 44.0, r.top() + 27.0),
                        Align2::LEFT_CENTER,
                        m.blurb,
                        instrument_kit::theme::body(10.0),
                        INK_DIM,
                        0.0,
                        w - 120.0,
                    );
                    text(
                        p,
                        pos2(r.right() - 36.0, r.center().y),
                        Align2::RIGHT_CENTER,
                        &format!("{}HP", m.hp),
                        mono(8.0),
                        INK_MID,
                        0.0,
                    );
                    let plus = Rect::from_center_size(
                        pos2(r.right() - 16.0, r.center().y),
                        Vec2::splat(20.0),
                    );
                    p.rect_filled(plus, 4.0, rgb(0x0b0e13));
                    text(
                        p,
                        plus.center(),
                        Align2::CENTER_CENTER,
                        "+",
                        mono(13.0),
                        SIGNAL_LIFE,
                        0.0,
                    );
                    if resp.clicked() {
                        add_module(app, m.key, None);
                    }
                }
            }
            if !any {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("   No modules carry this crest yet.")
                        .font(instrument_kit::theme::body(11.0))
                        .color(INK_DIM),
                );
            }
            ui.add_space(70.0);
        });
    });
}

const CASE_HP: u32 = 104;

fn case(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    let p = ui.painter().clone();
    let bar = Rect::from_min_size(area.min, vec2(area.width(), 44.0));
    p.rect_filled(bar, 0.0, rgb(0x0f1218));
    p.line_segment(
        [bar.left_bottom(), bar.right_bottom()],
        Stroke::new(1.0, Color32::BLACK),
    );
    let t = engrave(
        &p,
        pos2(bar.left() + 14.0, bar.center().y),
        Align2::LEFT_CENTER,
        "EURORACK CASE",
        9.0,
        INK_MID,
    );
    let hp = app.draft.hp();
    let over = hp > CASE_HP;
    let meter = Rect::from_min_max(
        pos2(t.right() + 12.0, bar.center().y - 3.0),
        pos2(bar.right() - 150.0, bar.center().y + 3.0),
    );
    p.rect_filled(meter, 3.0, rgb(0x07090d));
    p.rect_filled(
        Rect::from_min_size(
            meter.min,
            vec2(meter.width() * (hp as f32 / CASE_HP as f32).min(1.0), 6.0),
        ),
        3.0,
        if over { SIGNAL_HOT } else { SIGNAL_COOL },
    );
    readout(
        &p,
        pos2(bar.right() - 76.0, bar.center().y),
        Align2::RIGHT_CENTER,
        &format!("{hp} / {CASE_HP} HP"),
        11.0,
        if over { SIGNAL_HOT } else { SIGNAL_COOL },
    );
    util::row_rtl(ui, bar.shrink2(vec2(12.0, 0.0)), "case-bar", |ui| {
        if widgets::cv_btn(ui, "CLEAR", false, true).clicked() {
            app.draft.modules.clear();
            app.draft.macros.clear();
        }
    });
    let body_r = Rect::from_min_max(pos2(area.left(), bar.bottom() + 1.0), area.max);
    grad_v(&p, body_r, 0.0, rgb(0x050810), rgb(0x08090f));
    col(ui, body_r, "case-scroll", |ui| {
        ScrollArea::both().auto_shrink(false).show(ui, |ui| {
            if app.draft.modules.is_empty() {
                let r = ui.max_rect().shrink(26.0);
                let r = Rect::from_min_size(r.min, vec2(r.width(), 420.0_f32.min(r.height())));
                dashed(ui.painter(), r, white(0.12));
                text(ui.painter(), r.center() - vec2(0.0, 10.0), Align2::CENTER_CENTER, "CLICK MODULES ON THE LEFT TO ADD THEM TO THE CASE", mono(11.0), INK_DIM, 0.14);
                text(ui.painter(), r.center() + vec2(0.0, 12.0), Align2::CENTER_CENTER, "Engine modules share the global sound engine · utility modules get their own parameters in every rack copy", instrument_kit::theme::body(11.0), INK_DIM, 0.0);
                ui.allocate_rect(r, Sense::hover());
                return;
            }
            ui.add_space(16.0);
            let mut remove: Option<usize> = None;
            let mut swap: Option<(usize, usize)> = None;
            let n = app.draft.modules.len();
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add_space(16.0);
                let mods = app.draft.modules.clone();
                for (i, m) in mods.iter().enumerate() {
                    let Some(def) = catalog::module(&m.kind) else { continue };
                    let w = (def.hp as f32 * HP).clamp(112.0, 420.0);
                    let (r, _) = ui.allocate_exact_size(vec2(w, 404.0), Sense::hover());
                    let bar = Rect::from_min_size(r.min, vec2(w - 2.0, 20.0));
                    let p = ui.painter().clone();
                    p.rect_filled(bar, 3.0, rgb(0x161a21));
                    text_clipped(&p, pos2(bar.left() + 6.0, bar.center().y), Align2::LEFT_CENTER, def.name, mono(9.0), INK_MID, 0.1, w - 70.0);
                    for (k, (icon, tip)) in [("close", "Remove"), ("chevR", "Move right"), ("chev", "Move left")].into_iter().enumerate() {
                        let b = Rect::from_center_size(pos2(bar.right() - 10.0 - k as f32 * 17.0, bar.center().y), Vec2::splat(16.0));
                        let resp = ui.interact(b, Id::unique(("case-btn", &m.uid, k)), Sense::click()).on_hover_text(tip).on_hover_cursor(CursorIcon::PointingHand);
                        icons::paint(&p, icon, b.center(), 9.0, if resp.hovered() { if k == 0 { SIGNAL_HOT } else { SIGNAL_COOL } } else { INK_DIM });
                        if resp.clicked() {
                            match k {
                                0 => remove = Some(i),
                                1 if i + 1 < n => swap = Some((i, i + 1)),
                                2 if i > 0 => swap = Some((i, i - 1)),
                                _ => {}
                            }
                        }
                    }
                    module_face(app, ui, Rect::from_min_max(pos2(r.left(), bar.bottom() + 4.0), pos2(r.right() - 2.0, r.bottom())), def, &m.uid);
                }
                ui.add_space(16.0);
            });
            if let Some(i) = remove {
                let uid = app.draft.modules.remove(i).uid;
                app.draft.macros.retain(|m| m.module != uid);
            }
            if let Some((a, b)) = swap {
                app.draft.modules.swap(a, b);
            }
            ui.add_space(80.0);
        });
    });
}

/// Eurorack module face: chassis, crest, name and one control per parameter.
fn module_face(
    app: &mut ForgeApp,
    ui: &mut Ui,
    r: Rect,
    def: &'static catalog::ModuleDef,
    uid: &str,
) {
    let p = ui.painter().clone();
    let c = chc(def.ch);
    paint::shadow(&p, r, 6.0, 8.0, 18.0, black(0.6));
    grad_stops(
        &p,
        r,
        6.0,
        &[
            (0.0, rgb(0x1c2028)),
            (0.18, rgb(0x14181f)),
            (1.0, rgb(0x0f131a)),
        ],
        true,
    );
    bevel(&p, r, 6.0, 0.07, 0.6);
    outline(&p, r, 6.0, black(0.5));
    for q in [
        r.left_top() + vec2(8.0, 8.0),
        r.right_top() + vec2(-8.0, 8.0),
        r.left_bottom() + vec2(8.0, -8.0),
        r.right_bottom() + vec2(-8.0, -8.0),
    ] {
        screw(&p, q, 8.0);
    }
    let head = Rect::from_min_size(
        pos2(r.left() + 16.0, r.top() + 14.0),
        vec2(r.width() - 32.0, 30.0),
    );
    icons::paint(
        &p,
        def.crest,
        pos2(head.left() + 13.0, head.center().y),
        24.0,
        c,
    );
    text_clipped(
        &p,
        pos2(head.left() + 34.0, head.center().y),
        Align2::LEFT_CENTER,
        def.name,
        display(11.0),
        INK,
        0.2,
        head.width() - 36.0,
    );
    let ly = head.bottom() + 1.0;
    grad_stops(
        &p,
        Rect::from_min_size(pos2(head.left(), ly), vec2(head.width(), 1.0)),
        0.0,
        &[(0.0, alpha(c, 0.0)), (0.5, c), (1.0, alpha(c, 0.0))],
        false,
    );
    let sec = Rect::from_min_max(
        pos2(r.left() + 12.0, ly + 12.0),
        pos2(r.right() - 12.0, r.bottom() - 46.0),
    );
    p.rect_filled(sec, 4.0, black(0.2));
    outline(&p, sec, 4.0, black(0.3));
    // Draft parameters preview on a scratch prefix so the case is playable.
    let prefix = format!("draft.{uid}");
    let n = def.params.len().max(1);
    let per_row = ((sec.width() / 58.0).floor() as usize).clamp(1, 6);
    let rows = n.div_ceil(per_row);
    let rh = (sec.height() / rows as f32).min(86.0);
    let size = if rh < 60.0 { 26.0 } else { 38.0 };
    let top = sec.center().y - rh * rows as f32 / 2.0;
    for (ri, chunk) in def.params.chunks(per_row).enumerate() {
        let rr = Rect::from_min_size(
            pos2(sec.left(), top + ri as f32 * rh),
            vec2(sec.width(), rh),
        );
        util::centered(ui, rr, ("mface", uid, ri), 2.0, |ui| {
            for (k, l) in chunk {
                let key = if def.shared {
                    (*k).to_owned()
                } else {
                    format!("{prefix}.{k}")
                };
                util::knob(
                    ui,
                    &mut app.bus,
                    &key,
                    Knob::new(size, util::ch(def.ch))
                        .label(l)
                        .label_size(7.0)
                        .show_value(false),
                );
            }
        });
    }
    text(
        &p,
        pos2(r.center().x, r.bottom() - 24.0),
        Align2::CENTER_CENTER,
        &format!(
            "{} HP · {}",
            def.hp,
            if def.shared { "ENGINE" } else { "PER INSTANCE" }
        ),
        mono(7.5),
        INK_DIM,
        0.14,
    );
}

fn props(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    col(ui, area.shrink2(vec2(12.0, 0.0)), "props", |ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            let w = ui.available_width();
            ui.add_space(6.0);
            sub_head(ui, "INSTRUMENT");
            widgets::text_field(ui, &mut app.draft.name, "Instrument name", w, 28.0, GOLD);
            let d = catalog::dept(app.draft.dept);
            sub_head(ui, &format!("DEPARTMENT · {} {}", d.num, d.name));
            let cols = 4;
            let cw = w / cols as f32;
            for chunk in DEPTS.iter().enumerate().collect::<Vec<_>>().chunks(cols) {
                let (rr, _) = ui.allocate_exact_size(vec2(w, 36.0), Sense::hover());
                for (k, (i, dd)) in chunk.iter().enumerate() {
                    let tr = Rect::from_center_size(pos2(rr.left() + cw * (k as f32 + 0.5), rr.center().y), Vec2::splat(30.0));
                    let on = app.draft.dept == *i;
                    let resp = ui.interact(tr, Id::unique(("pdept", *i)), Sense::click()).on_hover_text(format!("{} {} · {}", dd.num, dd.name, dd.law)).on_hover_cursor(CursorIcon::PointingHand);
                    if on {
                        glow_rect(ui.painter(), tr, 15.0, 10.0, alpha(chc(dd.ch), 0.5));
                    }
                    widgets::glyph_tile_at(ui.painter(), tr, dd.crest, chc(dd.ch), true, !on);
                    if on {
                        ui.painter().circle_stroke(tr.center(), 15.5, Stroke::new(1.5, chc(dd.ch)));
                    }
                    if resp.clicked() {
                        app.draft.dept = *i;
                    }
                }
            }
            sub_head(ui, &format!("PANEL SKIN · {}", app.draft.skin.to_uppercase()));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                for (name, hx) in SKINS {
                    let (sr, resp) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
                    let on = app.draft.skin == *name;
                    if on {
                        glow_rect(ui.painter(), sr, 5.0, 10.0, alpha(SIGNAL_COOL, 0.4));
                    }
                    ui.painter().rect_filled(sr, 5.0, hex(hx));
                    outline_outside(ui.painter(), sr, 5.0, if on { SIGNAL_COOL } else { black(0.8) });
                    if resp.on_hover_text(*name).on_hover_cursor(CursorIcon::PointingHand).clicked() {
                        app.draft.skin = (*name).into();
                    }
                }
            });
            sub_head(ui, "PANEL · FACE IN THE RACK");
            let face = app.draft.faceplate.as_ref().and_then(|f| app.forge_doc.plate(f)).map(|f| f.name.clone());
            let (fr, _) = ui.allocate_exact_size(vec2(w, 28.0), Sense::hover());
            ui.painter().rect_filled(fr, 3.0, rgb(0x06080c));
            outline(ui.painter(), fr, 3.0, white(0.08));
            text_clipped(ui.painter(), pos2(fr.left() + 10.0, fr.center().y), Align2::LEFT_CENTER, face.as_deref().unwrap_or("— macro knobs (default) —"), instrument_kit::theme::body(12.0), INK, 0.0, w - 20.0);
            ui.label(egui::RichText::new("Shown as the folded front panel in the rack. Save the instrument first, then build its face on the PANEL stage — saving there attaches it here.").font(instrument_kit::theme::body(11.0)).color(INK_DIM));
            ui.horizontal(|ui| {
                if widgets::cv_btn(ui, "OPEN PANEL STAGE", false, false).clicked() {
                    if !app.draft.id.is_empty() {
                        app.forge_doc.current.bind = Some(app.draft.id.clone());
                    }
                    app.actions.push(Action::SetStage("panel"));
                }
                if face.is_some() && widgets::cv_btn(ui, "DETACH", false, true).clicked() {
                    app.draft.faceplate = None;
                }
            });
            sub_head(ui, &format!("FRONT-PANEL MACROS · {}/{}", app.draft.macros.len(), MAX_MACROS));
            let params = app.draft.params("draft");
            if params.is_empty() {
                ui.label(egui::RichText::new("Add modules to expose their parameters.").font(instrument_kit::theme::body(11.0)).color(INK_DIM));
            }
            for chunk in params.chunks(2) {
                let (rr, _) = ui.allocate_exact_size(vec2(w, 22.0), Sense::hover());
                for (k, prm) in chunk.iter().enumerate() {
                    let cr = Rect::from_min_size(pos2(rr.left() + k as f32 * (w / 2.0 + 1.5), rr.top()), vec2(w / 2.0 - 1.5, 22.0));
                    let idx = app.draft.macros.iter().position(|m| m.module == prm.module && m.key == prm.key);
                    let resp = ui.interact(cr, Id::unique(("macro", &prm.module, prm.key)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
                    let p = ui.painter();
                    p.rect_filled(cr, 3.0, if idx.is_some() { rgb(0x151b24) } else { rgb(0x0b0e13) });
                    if idx.is_some() {
                        glow_dot(p, pos2(cr.left() + 9.0, cr.center().y), 3.0, SIGNAL_COOL);
                    } else {
                        p.circle_filled(pos2(cr.left() + 9.0, cr.center().y), 3.0, rgb(0x2a2f3a));
                    }
                    text_clipped(p, pos2(cr.left() + 18.0, cr.center().y), Align2::LEFT_CENTER, prm.label, mono(8.0), if idx.is_some() { INK } else { INK_DIM }, 0.0, cr.width() - 36.0);
                    if let Some(i) = idx {
                        text(p, pos2(cr.right() - 6.0, cr.center().y), Align2::RIGHT_CENTER, &(i + 1).to_string(), mono(9.0), SIGNAL_COOL, 0.0);
                    }
                    if resp.clicked() {
                        match idx {
                            Some(i) => {
                                app.draft.macros.remove(i);
                            }
                            None if app.draft.macros.len() < MAX_MACROS => app.draft.macros.push(MacroRef { module: prm.module.clone(), key: prm.key.into() }),
                            None => {}
                        }
                    }
                }
            }
            ui.add_space(6.0);
            let saved = !app.draft.id.is_empty() && app.state.inst(&app.draft.id).is_some();
            let can = !app.draft.modules.is_empty();
            let half = (w - 6.0) / 2.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.add_enabled_ui(can, |ui| {
                    if widgets::cv_btn_wide(ui, if saved { "UPDATE LIBRARY" } else { "SAVE TO LIBRARY" }, half, true, false).clicked() {
                        save_draft(app);
                    }
                    if widgets::cv_btn_wide(ui, "SAVE + ADD TO RACK", half, true, false).clicked() {
                        let id = save_draft(app);
                        app.actions.push(Action::AddDevice { inst: id, at: None });
                        app.actions.push(Action::SetApp("rack"));
                    }
                });
            });
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                if widgets::cv_btn_wide(ui, "NEW", half, false, false).clicked() {
                    app.draft = Instrument::blank(app.draft.dept);
                }
                if saved && widgets::cv_btn_wide(ui, "DELETE", half, false, true).on_hover_text("Remove from the library and the rack").clicked() {
                    app.state.delete_instrument(&app.draft.id.clone());
                    app.draft = Instrument::blank(app.draft.dept);
                }
            });
            ui.add_space(70.0);
        });
    });
}

fn save_draft(app: &mut ForgeApp) -> String {
    let id = app.state.save_instrument(app.draft.clone());
    app.draft.id = id.clone();
    id
}

// ---- ③ PANEL / ④ LAYOUTS ------------------------------------------------------

fn fit(stage: Rect, aspect: f32) -> Rect {
    let s = stage.shrink(18.0);
    if aspect <= 0.0 {
        return s;
    }
    let (w, h) = if s.width() / s.height() > aspect {
        (s.height() * aspect, s.height())
    } else {
        (s.width(), s.width() / aspect)
    };
    Rect::from_center_size(s.center(), vec2(w, h))
}

/// Draw the faceplate being edited. Returns nothing; edits go straight to the doc.
fn plate_editor(app: &mut ForgeApp, ui: &mut Ui, stage: Rect, edit_components: bool) {
    let fp = app.forge_doc.current.clone();
    let r = fit(stage, fp.aspect());
    glow_rect(ui.painter(), r, 8.0, 40.0, alpha(SIGNAL_COOL, 0.12));
    let play = app.fui.play && edit_components;
    let mut remove: Option<String> = None;
    let mut moved: Option<(String, String, f32, f32)> = None;
    for (pane, nr) in fp.tree.layout([0.0, 0.0, 1.0, 1.0]) {
        let pr = Rect::from_min_size(
            pos2(r.left() + nr[0] * r.width(), r.top() + nr[1] * r.height()),
            vec2(nr[2] * r.width(), nr[3] * r.height()),
        );
        let sel = app.fui.pane.as_deref() == Some(pane.id.as_str());
        if !play
            && ui
                .interact(pr, Id::unique(("pane", &pane.id)), Sense::click())
                .clicked()
        {
            app.fui.pane = Some(pane.id.clone());
        }
        assets::pane_bg(ui.painter(), pr, pane, &fp.material, sel && !play);
        for c in &pane.components {
            let cr = assets::comp_rect(pr, &c.kind, c.x, c.y);
            let addr = fp.address(c);
            let ch = util::ch(assets::asset(&c.kind).map(|a| a.ch).unwrap_or("cool"));
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt(Id::unique(("pc", &c.id)))
                    .max_rect(pr),
            );
            child.set_clip_rect(pr.intersect(ui.clip_rect()));
            assets::draw(
                &mut child,
                &mut app.bus,
                &mut app.engine,
                &c.kind,
                &addr,
                cr,
                &c.label,
                ch,
                play,
            );
            if !play && edit_components {
                let hit = cr.expand(4.0);
                let resp = ui
                    .interact(hit, Id::unique(("pc-drag", &c.id)), Sense::click_and_drag())
                    .on_hover_cursor(CursorIcon::Grab);
                dashed(
                    ui.painter(),
                    hit,
                    alpha(
                        SIGNAL_COOL,
                        if resp.hovered() || resp.dragged() {
                            0.6
                        } else {
                            0.18
                        },
                    ),
                );
                if resp.dragged() {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        moved = Some((
                            pane.id.clone(),
                            c.id.clone(),
                            ((pos.x - pr.left()) / pr.width()).clamp(0.04, 0.96),
                            ((pos.y - pr.top()) / pr.height()).clamp(0.06, 0.94),
                        ));
                    }
                }
                if resp.clicked() {
                    app.fui.pane = Some(pane.id.clone());
                }
                let xb = Rect::from_center_size(hit.right_top(), Vec2::splat(13.0));
                let xr = ui
                    .interact(xb, Id::unique(("pc-x", &c.id)), Sense::click())
                    .on_hover_text("Remove")
                    .on_hover_cursor(CursorIcon::PointingHand);
                ui.painter().circle_filled(xb.center(), 6.5, rgb(0x0d1118));
                ui.painter().circle_stroke(
                    xb.center(),
                    6.5,
                    Stroke::new(1.0, alpha(SIGNAL_HOT, if xr.hovered() { 1.0 } else { 0.6 })),
                );
                icons::paint(ui.painter(), "close", xb.center(), 7.0, SIGNAL_HOT);
                if xr.clicked() {
                    remove = Some(c.id.clone());
                }
            }
        }
    }
    if let Some(id) = remove {
        app.forge_doc.current.remove_component(&id);
    }
    if let Some((pane, comp, x, y)) = moved {
        if let Some(c) = app
            .forge_doc
            .current
            .tree
            .pane_mut(&pane)
            .and_then(|p| p.components.iter_mut().find(|c| c.id == comp))
        {
            c.x = x;
            c.y = y;
        }
    }
}

fn panel_stage(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    let left = shell::side(
        &mut app.prefs,
        ui,
        area,
        "panel.L",
        false,
        &[SideItem::new(
            "ctl",
            "knob",
            "CONTROLS",
            &ASSETS.len().to_string(),
        )],
        230.0,
        true,
    );
    if let Some((r, _)) = &left.body {
        col(ui, r.shrink2(vec2(10.0, 0.0)), "panel-pal", |ui| {
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 3.0;
                ui.add_space(10.0);
                let w = ui.available_width();
                widgets::text_field(ui, &mut app.fui.search, "Search…", w, 26.0, GOLD);
                ui.add_space(4.0);
                let q = app.fui.search.to_lowercase();
                let list: Vec<&assets::Asset> = ASSETS
                    .iter()
                    .filter(|a| q.is_empty() || a.label.to_lowercase().contains(&q))
                    .collect();
                for chunk in list.chunks(2) {
                    let (rr, _) = ui.allocate_exact_size(vec2(w, 24.0), Sense::hover());
                    for (k, a) in chunk.iter().enumerate() {
                        let cr = Rect::from_min_size(
                            pos2(rr.left() + k as f32 * (w / 2.0 + 1.5), rr.top()),
                            vec2(w / 2.0 - 1.5, 24.0),
                        );
                        let resp = ui
                            .interact(cr, Id::unique(("ppal", a.key)), Sense::click())
                            .on_hover_text(format!("{} · place on the selected pane", a.family))
                            .on_hover_cursor(CursorIcon::PointingHand);
                        let p = ui.painter();
                        p.rect_filled(cr, 4.0, rgb(0x0e1220));
                        outline(
                            p,
                            cr,
                            4.0,
                            if resp.hovered() {
                                chc(a.ch)
                            } else {
                                white(0.05)
                            },
                        );
                        glow_dot(p, pos2(cr.left() + 9.0, cr.center().y), 2.5, chc(a.ch));
                        text_clipped(
                            p,
                            pos2(cr.left() + 18.0, cr.center().y),
                            Align2::LEFT_CENTER,
                            a.label,
                            mono(7.5),
                            INK,
                            0.05,
                            cr.width() - 22.0,
                        );
                        if resp.clicked() {
                            add_to_panel(app, a.key);
                        }
                    }
                }
                ui.add_space(70.0);
            });
        });
    }
    let right = shell::side(
        &mut app.prefs,
        ui,
        left.rest,
        "panel.R",
        true,
        &[SideItem::new("dev", "device", "DEVICE", "BIOSPARK")],
        300.0,
        true,
    );
    if let Some((r, _)) = &right.body {
        pane_props(app, ui, *r);
    }
    plate_editor(app, ui, right.rest, true);
}

fn pane_props(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    col(ui, area.shrink2(vec2(12.0, 0.0)), "pane-props", |ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            let w = ui.available_width();
            ui.add_space(6.0);
            let sel = app.fui.pane.clone().filter(|p| app.forge_doc.current.tree.pane(p).is_some());
            sub_head(ui, "PANE");
            match &sel {
                None => {
                    ui.label(egui::RichText::new("Click a pane to select it. Split it from the toolbar, then place controls from the left.").font(instrument_kit::theme::body(11.0)).color(INK_DIM));
                }
                Some(id) => {
                    if let Some(p) = app.forge_doc.current.tree.pane_mut(id) {
                        widgets::text_field(ui, &mut p.title, "Pane title", w, 28.0, GOLD);
                        p.show_header = !p.title.is_empty();
                        sub_head(ui, "ACCENT");
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                            for c in Channel::ALL {
                                let (sr, resp) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::click());
                                let on = p.accent == c.key();
                                ui.painter().circle_filled(sr.center(), 7.0, c.color());
                                if on {
                                    ui.painter().circle_stroke(sr.center(), 10.0, Stroke::new(1.5, c.color()));
                                }
                                if resp.on_hover_text(c.key()).on_hover_cursor(CursorIcon::PointingHand).clicked() {
                                    p.accent = c.key().into();
                                }
                            }
                        });
                        sub_head(ui, "PANE MATERIAL");
                        let cur = p.material.clone();
                        if let Some(m) = material_grid(ui, &cur, true) {
                            p.material = m;
                        }
                    }
                }
            }
            sub_head(ui, "DEFAULT MATERIAL");
            let cur = app.forge_doc.current.material.clone();
            if let Some(m) = material_grid(ui, &cur, false) {
                app.forge_doc.current.material = m;
            }
            sub_head(ui, "BIND TO INSTRUMENT");
            let bound = app.forge_doc.current.bind.clone();
            let label = bound.as_ref().and_then(|b| app.state.inst(b)).map(|i| i.name.clone()).unwrap_or_else(|| "— none (layout only) —".into());
            let mut pick: Option<Option<String>> = None;
            egui::ComboBox::from_id_salt("bind").width(w - 8.0).selected_text(egui::RichText::new(label.clone()).font(mono(9.0))).show_ui(ui, |ui| {
                if ui.selectable_label(bound.is_none(), "— none (layout only) —").clicked() {
                    pick = Some(None);
                }
                for i in &app.state.lib {
                    if ui.selectable_label(bound.as_deref() == Some(i.id.as_str()), egui::RichText::new(&i.name).font(mono(9.0))).clicked() {
                        pick = Some(Some(i.id.clone()));
                    }
                }
            }).response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::ComboBox, true, &label));
            if let Some(b) = pick {
                app.forge_doc.current.bind = b;
            }
            ui.label(egui::RichText::new("Bind a faceplate to a library instrument, then SAVE: the instrument shows it as its folded front panel in the rack.").font(instrument_kit::theme::body(11.0)).color(INK_DIM));
            if !app.forge_doc.plates.is_empty() {
                sub_head(ui, &format!("SAVED FACEPLATES · {}", app.forge_doc.plates.len()));
                let mut open: Option<usize> = None;
                for (i, f) in app.forge_doc.plates.iter().enumerate() {
                    let (lr, resp) = ui.allocate_exact_size(vec2(w, 22.0), Sense::click());
                    let p = ui.painter();
                    if resp.hovered() {
                        p.rect_filled(lr, 3.0, white(0.05));
                    }
                    text_clipped(p, pos2(lr.left() + 6.0, lr.center().y), Align2::LEFT_CENTER, &f.name, mono(9.0), INK, 0.06, w - 70.0);
                    text(p, pos2(lr.right() - 6.0, lr.center().y), Align2::RIGHT_CENTER, &format!("{} ctl", f.component_count()), mono(8.0), INK_DIM, 0.0);
                    if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                        open = Some(i);
                    }
                }
                if let Some(i) = open {
                    app.forge_doc.current = app.forge_doc.plates[i].clone();
                    app.fui.pane = None;
                }
            }
            ui.add_space(70.0);
        });
    });
}

/// Swatch grid. Returns the picked material key.
fn material_grid(ui: &mut Ui, current: &str, inherit: bool) -> Option<String> {
    let mut pick = None;
    let name = if current == "inherit" {
        "INHERIT · DEVICE"
    } else {
        MATERIALS
            .iter()
            .find(|m| m.0 == current)
            .map(|m| m.1)
            .unwrap_or("—")
    };
    let (hr, _) = ui.allocate_exact_size(vec2(ui.available_width(), 14.0), Sense::hover());
    text(
        ui.painter(),
        hr.left_center(),
        Align2::LEFT_CENTER,
        name,
        mono(9.0),
        INK,
        0.1,
    );
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        if inherit {
            let (sr, resp) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::click());
            ui.painter().rect_filled(sr, 4.0, rgb(0x0d121b));
            icons::paint(ui.painter(), "export", sr.center(), 12.0, INK_MID);
            outline(
                ui.painter(),
                sr,
                4.0,
                if current == "inherit" {
                    GOLD
                } else {
                    white(0.1)
                },
            );
            if resp
                .on_hover_text("Inherit device material")
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                pick = Some("inherit".to_owned());
            }
        }
        for (key, label, fam, a, b) in MATERIALS {
            let (sr, resp) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::click());
            let on = current == key;
            if on {
                glow_rect(ui.painter(), sr, 4.0, 10.0, alpha(GOLD, 0.4));
            }
            grad_v(ui.painter(), sr, 4.0, rgb(a), rgb(b));
            outline(ui.painter(), sr, 4.0, if on { GOLD } else { white(0.1) });
            if resp
                .on_hover_text(format!("{label} · {fam}"))
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                pick = Some(key.to_owned());
            }
        }
    });
    pick
}

fn layouts_stage(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    let left = shell::side(
        &mut app.prefs,
        ui,
        area,
        "lay.L",
        false,
        &[SideItem::new(
            "tpl",
            "rack",
            "INSTRUMENT",
            &TEMPLATES.len().to_string(),
        )],
        250.0,
        true,
    );
    if let Some((r, _)) = &left.body {
        col(ui, r.shrink2(vec2(12.0, 0.0)), "lay-tpl", |ui| {
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                ui.add_space(12.0);
                let w = ui.available_width();
                for (key, name) in TEMPLATES {
                    let t = panel::template(key);
                    let a = if t.aspect() <= 0.0 { 1.6 } else { t.aspect() };
                    let th = (w / a).clamp(34.0, 150.0);
                    let (tr, resp) = ui.allocate_exact_size(vec2(w, th + 26.0), Sense::click());
                    let p = ui.painter().clone();
                    p.rect_filled(tr, 4.0, rgb(0x0b0e14));
                    outline(
                        &p,
                        tr,
                        4.0,
                        if resp.hovered() {
                            SIGNAL_COOL
                        } else {
                            white(0.07)
                        },
                    );
                    let thumb =
                        Rect::from_min_size(tr.min + vec2(5.0, 5.0), vec2(w - 10.0, th - 4.0));
                    let tw = (thumb.height() * a).min(thumb.width());
                    let thumb = Rect::from_min_size(thumb.min, vec2(tw, thumb.height()));
                    for (pane, nr) in t.tree.layout([0.0, 0.0, 1.0, 1.0]) {
                        let pr = Rect::from_min_size(
                            pos2(
                                thumb.left() + nr[0] * thumb.width(),
                                thumb.top() + nr[1] * thumb.height(),
                            ),
                            vec2(nr[2] * thumb.width(), nr[3] * thumb.height()),
                        )
                        .shrink(1.0);
                        let (ca, cb) = assets::material("alloy");
                        grad_v(&p, pr, 2.0, ca, cb);
                        p.line_segment(
                            [
                                pos2(pr.left() + 3.0, pr.top() + 2.0),
                                pos2(pr.right() - 3.0, pr.top() + 2.0),
                            ],
                            Stroke::new(1.5, chc(&pane.accent)),
                        );
                    }
                    text(
                        &p,
                        pos2(tr.left() + 6.0, tr.bottom() - 10.0),
                        Align2::LEFT_CENTER,
                        name,
                        mono(8.0),
                        INK_MID,
                        0.12,
                    );
                    if resp
                        .on_hover_text("Start the faceplate from this partition")
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        let bind = app.forge_doc.current.bind.clone();
                        app.forge_doc.current = t;
                        app.forge_doc.current.bind = bind;
                        app.fui.pane = None;
                    }
                }
                ui.add_space(70.0);
            });
        });
    }
    let right = shell::side(
        &mut app.prefs,
        ui,
        left.rest,
        "lay.R",
        true,
        &[SideItem::new("res", "faction", "FACTION RESONANCE", "")],
        300.0,
        true,
    );
    if let Some((r, _)) = &right.body {
        col(ui, r.shrink2(vec2(12.0, 0.0)), "lay-res", |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.add_space(14.0);
            let w = ui.available_width();
            let mut top = (0usize, 0.0f32);
            for (i, (name, hx)) in FACTIONS.iter().enumerate() {
                let key = format!("fx.res.{}", name.to_lowercase());
                let mut v = app.bus.num(&key, [0.3, 0.35, 0.4, 0.8, 0.25][i]);
                if v > top.1 {
                    top = (i, v);
                }
                ui.horizontal(|ui| {
                    let (lr, _) = ui.allocate_exact_size(vec2(84.0, 16.0), Sense::hover());
                    text(
                        ui.painter(),
                        lr.left_center(),
                        Align2::LEFT_CENTER,
                        &name.to_uppercase(),
                        mono(8.0),
                        INK_MID,
                        0.12,
                    );
                    if widgets::hfader(ui, &mut v, w - 130.0, hex(hx), 0.5).changed() {
                        app.bus.set_num(&key, v);
                    }
                    let (vr, _) = ui.allocate_exact_size(vec2(24.0, 16.0), Sense::hover());
                    text(
                        ui.painter(),
                        vr.right_center(),
                        Align2::RIGHT_CENTER,
                        &format!("{}", (v * 100.0).round()),
                        mono(8.0),
                        INK_MID,
                        0.0,
                    );
                });
            }
            let (name, hx) = FACTIONS[top.0];
            let notes = [
                "solar · radiant order · bright register, clear cadence",
                "deep-space · precise · cool register, measured pulse",
                "verdant · growing · warm register, branching phrases",
                "fluid · abyssal pressure · deep register, flowing legato, tidal pulse",
                "obsidian · arc · hard register, syncopated strikes",
            ];
            let (br, _) = ui.allocate_exact_size(vec2(w, 58.0), Sense::hover());
            ui.painter().rect_filled(br, 4.0, rgb(0x0a0f18));
            outline(ui.painter(), br, 4.0, alpha(hex(hx), 0.6));
            text(
                ui.painter(),
                br.left_top() + vec2(9.0, 10.0),
                Align2::LEFT_CENTER,
                &format!("{} · {}%", name.to_uppercase(), (top.1 * 100.0).round()),
                mono(8.0),
                hex(hx),
                0.14,
            );
            paint::paragraph(
                ui.painter(),
                br.left_top() + vec2(9.0, 22.0),
                notes[top.0],
                instrument_kit::theme::body(11.0),
                INK,
                w - 18.0,
            );
            ui.label(egui::RichText::new("Resonance weights publish on the bus as fx.res.<faction>; route them like any other source.").font(instrument_kit::theme::body(11.0)).color(INK_DIM));
        });
    }
    plate_editor(app, ui, right.rest, false);
}

// ---- ⑤ NODE GRAPH -------------------------------------------------------------

const NODE_W: f32 = 176.0;

fn node_h(k: &graph::KindDef) -> f32 {
    22.0 + k.ins.len().max(k.outs.len()) as f32 * 18.0
        + if k.field.is_some() { 30.0 } else { 8.0 }
        + 6.0
}

fn wire_color(w: Wire) -> Color32 {
    match w {
        Wire::Text => SIGNAL_MYTH,
        Wire::Num => SIGNAL_COOL,
        Wire::Any => SIGNAL_BONE,
    }
}

fn graph_stage(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    let items: Vec<SideItem> = graph::FAMILIES
        .iter()
        .map(|(k, n, icon)| {
            SideItem::new(
                k,
                icon,
                n,
                &graph::KINDS
                    .iter()
                    .filter(|x| x.family == *k)
                    .count()
                    .to_string(),
            )
        })
        .collect();
    let left = shell::side(
        &mut app.prefs,
        ui,
        area,
        "graph.L",
        false,
        &items,
        220.0,
        true,
    );
    if let Some((r, fam)) = &left.body {
        col(ui, r.shrink2(vec2(12.0, 0.0)), "graph-pal", |ui| {
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                ui.add_space(12.0);
                let w = ui.available_width();
                let mut add: Option<(&str, usize)> = None;
                let item = |ui: &mut Ui, label: &str, sub: &str, c: Color32| -> bool {
                    let (lr, resp) = ui.allocate_exact_size(vec2(w, 23.0), Sense::click());
                    let p = ui.painter();
                    p.rect_filled(
                        lr,
                        3.0,
                        if resp.hovered() {
                            white(0.05)
                        } else {
                            rgb(0x0a0d12)
                        },
                    );
                    p.circle_filled(pos2(lr.left() + 9.0, lr.center().y), 3.0, c);
                    text(
                        p,
                        pos2(lr.left() + 19.0, lr.center().y),
                        Align2::LEFT_CENTER,
                        label,
                        mono(8.0),
                        INK,
                        0.12,
                    );
                    text(
                        p,
                        pos2(lr.right() - 6.0, lr.center().y),
                        Align2::RIGHT_CENTER,
                        sub,
                        mono(7.0),
                        INK_DIM,
                        0.0,
                    );
                    resp.on_hover_text("Add to the graph")
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                };
                if fam == "modules" {
                    for (i, d) in DEPTS.iter().take(16).enumerate() {
                        if item(ui, d.name, d.law, chc(d.ch)) {
                            add = Some(("dept", i));
                        }
                    }
                } else {
                    for k in graph::KINDS.iter().filter(|k| k.family == fam.as_str()) {
                        if item(
                            ui,
                            k.name,
                            &format!("{}-{}", k.ins.len(), k.outs.len()),
                            chc(k.ch),
                        ) {
                            add = Some((k.key, 0));
                        }
                    }
                }
                if let Some((key, dept)) = add {
                    let n = app.graph.nodes.len() as f32;
                    let id = app
                        .graph
                        .add(key, 40.0 + (n % 5.0) * 36.0, 40.0 + (n % 7.0) * 30.0);
                    if key == "dept" {
                        if let Some(nd) = app.graph.node_mut(&id) {
                            nd.dept = dept;
                            nd.text = format!("{}.out", DEPTS[dept].name.to_lowercase());
                        }
                    }
                    app.fui.node = Some(id);
                }
            });
        });
    }
    let right = shell::side(
        &mut app.prefs,
        ui,
        left.rest,
        "graph.R",
        true,
        &[SideItem::new(
            "node",
            "pane",
            "NODE",
            app.fui.node.as_deref().unwrap_or("none"),
        )],
        280.0,
        true,
    );
    if let Some((r, _)) = &right.body {
        node_inspector(app, ui, *r);
    }
    canvas(app, ui, right.rest);
}

fn canvas(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    let bg = ui.interact(area, Id::unique("graph-bg"), Sense::click_and_drag());
    let pan_id = Id::unique("graph-pan");
    let mut pan: Vec2 = ui.data(|d| d.get_temp(pan_id)).unwrap_or(vec2(20.0, 20.0));
    if bg.dragged() {
        pan += bg.drag_delta();
    }
    if bg.clicked() {
        app.fui.node = None;
        app.fui.wire_from = None;
    }
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.fui.wire_from = None;
    }
    ui.data_mut(|d| d.insert_temp(pan_id, pan));
    let p = ui.painter().with_clip_rect(area);
    // dotted grid
    let step = 22.0;
    let mut y = area.top() + pan.y.rem_euclid(step);
    while y < area.bottom() {
        let mut x = area.left() + pan.x.rem_euclid(step);
        while x < area.right() {
            p.circle_filled(pos2(x, y), 0.6, white(0.07));
            x += step;
        }
        y += step;
    }
    let origin = area.min + pan;
    let port_pos = |g: &Graph, node: &str, port: usize, output: bool| -> Option<egui::Pos2> {
        let n = g.node(node)?;
        Some(
            origin
                + vec2(
                    n.x + if output { NODE_W } else { 0.0 },
                    n.y + 22.0 + 12.0 + port as f32 * 18.0,
                ),
        )
    };
    // wires
    let mut cut: Option<usize> = None;
    let ptr = ui.ctx().pointer_latest_pos();
    for (i, w) in app.graph.wires.iter().enumerate() {
        let (Some(a), Some(b)) = (
            port_pos(&app.graph, &w.from, w.out, true),
            port_pos(&app.graph, &w.to, w.inp, false),
        ) else {
            continue;
        };
        let kd = app.graph.node(&w.from).and_then(|n| graph::kind(&n.kind));
        let c = kd
            .and_then(|k| k.outs.get(w.out))
            .map(|o| wire_color(o.1))
            .unwrap_or(SIGNAL_BONE);
        let dx = ((b.x - a.x).abs() * 0.5).max(30.0);
        let curve = egui::epaint::CubicBezierShape::from_points_stroke(
            [a, a + vec2(dx, 0.0), b - vec2(dx, 0.0), b],
            false,
            Color32::TRANSPARENT,
            Stroke::new(1.6, c),
        );
        let near = ptr.is_some_and(|q| {
            area.contains(q) && curve.flatten(Some(0.5)).iter().any(|s| s.distance(q) < 5.0)
        });
        if near {
            p.add(egui::epaint::CubicBezierShape::from_points_stroke(
                [a, a + vec2(dx, 0.0), b - vec2(dx, 0.0), b],
                false,
                Color32::TRANSPARENT,
                Stroke::new(5.0, alpha(SIGNAL_HOT, 0.5)),
            ));
            if bg.clicked() {
                cut = Some(i);
            }
        }
        p.add(curve);
    }
    if let Some(i) = cut {
        app.graph.wires.remove(i);
    }
    if let (Some((n, o)), Some(q)) = (&app.fui.wire_from, ptr) {
        if let Some(a) = port_pos(&app.graph, n, *o, true) {
            p.line_segment([a, q], Stroke::new(1.5, alpha(GOLD, 0.8)));
        }
    }
    // nodes
    let mut connect: Option<(String, usize)> = None;
    let mut remove: Option<String> = None;
    let nodes = app.graph.nodes.clone();
    for n in &nodes {
        let Some(k) = graph::kind(&n.kind) else {
            continue;
        };
        let d = &DEPTS[n.dept.min(16)];
        let (name, c, crest) = if n.kind == "dept" {
            (d.name, chc(d.ch), Some(d.crest))
        } else {
            (k.name, chc(k.ch), None)
        };
        let r = Rect::from_min_size(origin + vec2(n.x, n.y), vec2(NODE_W, node_h(k)));
        if !area.intersects(r) {
            continue;
        }
        let sel = app.fui.node.as_deref() == Some(n.id.as_str());
        let body_resp = ui.interact(r.intersect(area), Id::unique(("gn", &n.id)), Sense::click());
        if body_resp.clicked() {
            app.fui.node = Some(n.id.clone());
        }
        paint::shadow(&p, r, 5.0, 6.0, 14.0, black(0.6));
        p.rect_filled(r, 5.0, rgb(0x10141b));
        let head = Rect::from_min_size(r.min, vec2(NODE_W, 22.0));
        grad(
            &p,
            head,
            egui::CornerRadius {
                nw: 5,
                ne: 5,
                sw: 0,
                se: 0,
            },
            mix(c, rgb(0x14181f), 0.35),
            mix(c, rgb(0x14181f), 0.12),
            false,
        );
        outline(
            &p,
            r,
            5.0,
            if sel {
                GOLD
            } else if n.skipped || n.error.is_some() {
                alpha(SIGNAL_HOT, 0.6)
            } else {
                black(0.8)
            },
        );
        let hresp = ui
            .interact(
                Rect::from_min_max(head.min, pos2(head.right() - 74.0, head.bottom()))
                    .intersect(area),
                Id::unique(("gn-h", &n.id)),
                Sense::click_and_drag(),
            )
            .on_hover_cursor(CursorIcon::Grab);
        if hresp.dragged() && !n.lock {
            if let Some(nn) = app.graph.node_mut(&n.id) {
                nn.x += hresp.drag_delta().x;
                nn.y += hresp.drag_delta().y;
            }
        }
        if hresp.clicked() || hresp.drag_started() {
            app.fui.node = Some(n.id.clone());
        }
        let mut tx = head.left() + 8.0;
        if let Some(cr) = crest {
            icons::paint(&p, cr, pos2(tx + 7.0, head.center().y), 14.0, INK);
            tx += 20.0;
        }
        text_clipped(
            &p,
            pos2(tx, head.center().y),
            Align2::LEFT_CENTER,
            name,
            mono(8.0),
            INK,
            0.14,
            head.right() - 78.0 - tx,
        );
        // D / R / L flags and close
        for (fi, (l, on, col)) in [
            ("D", n.display, SIGNAL_COOL),
            ("R", n.bypass, SIGNAL_LIFE),
            ("L", n.lock, SIGNAL_HOT),
        ]
        .into_iter()
        .enumerate()
        {
            let fc = pos2(head.right() - 66.0 + fi as f32 * 15.0, head.center().y);
            let fr = ui
                .interact(
                    Rect::from_center_size(fc, Vec2::splat(13.0)).intersect(area),
                    Id::unique(("gn-f", &n.id, fi)),
                    Sense::click(),
                )
                .on_hover_text(
                    [
                        "Display flag",
                        "Bypass · pass input through",
                        "Lock position",
                    ][fi],
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            p.circle_filled(fc, 6.0, if on { col } else { rgb(0x1f2530) });
            text(
                &p,
                fc,
                Align2::CENTER_CENTER,
                l,
                mono(6.5),
                if on { rgb(0x05070a) } else { INK_DIM },
                0.0,
            );
            if fr.clicked() {
                if let Some(nn) = app.graph.node_mut(&n.id) {
                    match fi {
                        0 => nn.display = !nn.display,
                        1 => nn.bypass = !nn.bypass,
                        _ => nn.lock = !nn.lock,
                    }
                }
            }
        }
        let xc = pos2(head.right() - 12.0, head.center().y);
        let xr = ui
            .interact(
                Rect::from_center_size(xc, Vec2::splat(14.0)).intersect(area),
                Id::unique(("gn-x", &n.id)),
                Sense::click(),
            )
            .on_hover_text("Delete node")
            .on_hover_cursor(CursorIcon::PointingHand);
        icons::paint(
            &p,
            "close",
            xc,
            8.0,
            if xr.hovered() { SIGNAL_HOT } else { INK_DIM },
        );
        if xr.clicked() {
            remove = Some(n.id.clone());
        }
        // ports
        for (i, (label, wt)) in k.ins.iter().enumerate() {
            let c0 = pos2(r.left(), r.top() + 34.0 + i as f32 * 18.0);
            let pr = ui
                .interact(
                    Rect::from_center_size(c0, Vec2::splat(16.0)).intersect(area),
                    Id::unique(("gp-in", &n.id, i)),
                    Sense::click(),
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            let wired = app.graph.wires.iter().any(|w| w.to == n.id && w.inp == i);
            p.circle_filled(c0, 5.5, rgb(0x0b0e14));
            p.circle_stroke(c0, 5.0, Stroke::new(1.6, wire_color(*wt)));
            if wired || pr.hovered() {
                p.circle_filled(c0, 2.2, wire_color(*wt));
            }
            text(
                &p,
                c0 + vec2(10.0, 0.0),
                Align2::LEFT_CENTER,
                label,
                mono(7.5),
                INK_MID,
                0.06,
            );
            if pr.clicked() {
                connect = Some((n.id.clone(), i));
            }
        }
        for (i, (label, wt)) in k.outs.iter().enumerate() {
            let c0 = pos2(r.right(), r.top() + 34.0 + i as f32 * 18.0);
            let pr = ui
                .interact(
                    Rect::from_center_size(c0, Vec2::splat(16.0)).intersect(area),
                    Id::unique(("gp-out", &n.id, i)),
                    Sense::click(),
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            let active = app
                .fui
                .wire_from
                .as_ref()
                .is_some_and(|(a, b)| a == &n.id && *b == i);
            p.circle_filled(c0, 5.5, rgb(0x0b0e14));
            p.circle_stroke(
                c0,
                5.0,
                Stroke::new(1.6, if active { GOLD } else { wire_color(*wt) }),
            );
            if active || pr.hovered() {
                p.circle_filled(c0, 2.2, GOLD);
            }
            text(
                &p,
                c0 - vec2(10.0, 0.0),
                Align2::RIGHT_CENTER,
                label,
                mono(7.5),
                INK_MID,
                0.06,
            );
            if pr.clicked() {
                app.fui.wire_from = if active {
                    None
                } else {
                    Some((n.id.clone(), i))
                };
            }
        }
        // field / value line
        if let Some(fl) = k.field {
            let fy = r.top() + 22.0 + k.ins.len().max(k.outs.len()) as f32 * 18.0 + 6.0;
            text(
                &p,
                pos2(r.left() + 8.0, fy + 11.0),
                Align2::LEFT_CENTER,
                fl,
                mono(6.5),
                INK_DIM,
                0.12,
            );
            let fr =
                Rect::from_min_max(pos2(r.left() + 48.0, fy), pos2(r.right() - 8.0, fy + 22.0));
            p.rect_filled(fr, 3.0, rgb(0x05070a));
            let shown = if n.kind == "prompt" || n.out.is_none() {
                n.text.clone()
            } else {
                n.out.as_ref().map(|v| v.text()).unwrap_or_default()
            };
            text_clipped(
                &p,
                pos2(fr.left() + 6.0, fr.center().y),
                Align2::LEFT_CENTER,
                &shown.replace('\n', " ⏎ "),
                mono(9.0),
                if n.out.is_some() { c } else { INK },
                0.0,
                fr.width() - 10.0,
            );
        } else if let Some(v) = &n.out {
            text_clipped(
                &p,
                pos2(r.left() + 30.0, r.bottom() - 9.0),
                Align2::LEFT_CENTER,
                &v.text().replace('\n', " ⏎ "),
                mono(8.0),
                c,
                0.0,
                NODE_W - 38.0,
            );
        }
    }
    if let Some((to, inp)) = connect {
        if let Some((from, out)) = app.fui.wire_from.take() {
            app.fui.msg = match app.graph.connect(&from, out, &to, inp) {
                Ok(()) => String::new(),
                Err(e) => format!("Not connected: {e}"),
            };
        }
    }
    if let Some(id) = remove {
        app.graph.remove(&id);
        if app.fui.node.as_deref() == Some(id.as_str()) {
            app.fui.node = None;
        }
    }
    text(
        &p,
        pos2(area.left() + 12.0, area.bottom() - 14.0),
        Align2::LEFT_CENTER,
        &format!(
            "/obj/{}/graph · {} nodes · {} wires",
            app.vault()
                .map(|v| v.name.to_lowercase().replace(' ', "_"))
                .unwrap_or_default(),
            app.graph.nodes.len(),
            app.graph.wires.len()
        ),
        mono(8.0),
        INK_DIM,
        0.1,
    );
}

fn node_inspector(app: &mut ForgeApp, ui: &mut Ui, area: Rect) {
    col(ui, area.shrink2(vec2(12.0, 0.0)), "node-insp", |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        ui.add_space(10.0);
        let w = ui.available_width();
        let Some(n) = app.fui.node.clone().and_then(|id| app.graph.node_mut(&id)) else {
            ui.label(egui::RichText::new("Select a node. Drag headers to move, click an out-port then an in-port to wire, click a wire to cut.").font(instrument_kit::theme::body(12.0)).color(INK_DIM));
            return;
        };
        let Some(k) = graph::kind(&n.kind) else {
            return;
        };
        let title = if n.kind == "dept" {
            DEPTS[n.dept.min(16)].name
        } else {
            k.name
        };
        sub_head(ui, &format!("{title} · {}", n.id));
        if let Some(fl) = k.field {
            sub_head(ui, fl);
            if n.kind == "prompt" || n.kind == "template" {
                ui.add(
                    egui::TextEdit::multiline(&mut n.text)
                        .desired_width(w)
                        .desired_rows(5)
                        .font(instrument_kit::theme::body(12.0)),
                );
            } else {
                widgets::text_field(ui, &mut n.text, fl, w, 28.0, GOLD);
            }
        }
        if n.kind == "model" {
            ui.label(egui::RichText::new("Calls the local Ollama server from Settings. Set MODEL to a pulled model name, or leave \"local\" for the default.").font(instrument_kit::theme::body(11.0)).color(SIGNAL_AMBER));
        }
        sub_head(ui, "OUTPUT");
        let out = match (&n.out, n.skipped) {
            _ if n.error.is_some() => format!("failed: {}", n.error.clone().unwrap_or_default()),
            (_, true) => "skipped — an upstream node is grounded".to_owned(),
            (Some(v), _) => v.text(),
            (None, _) => "— run the graph —".to_owned(),
        };
        ui.label(egui::RichText::new(out).font(mono(10.0)).color(INK_MID));
    });
}
