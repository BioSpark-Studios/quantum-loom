//! SETTINGS drawer tab: the local LLM (Ollama), audio / MIDI status, storage.
//! Local only — there is deliberately no cloud provider list.

use crate::app::ForgeApp;
use crate::util::col;
use egui::{pos2, vec2, Align2, Rect, ScrollArea, Sense, Ui};
use instrument_kit::paint::*;
use instrument_kit::theme::*;
use instrument_kit::widgets;

fn heading(ui: &mut Ui, s: &str) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 20.0), Sense::hover());
    engrave(
        ui.painter(),
        pos2(r.left(), r.bottom() - 4.0),
        Align2::LEFT_BOTTOM,
        s,
        9.0,
        INK_MID,
    );
}

fn label(ui: &mut Ui, s: &str) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 12.0), Sense::hover());
    text(
        ui.painter(),
        r.left_center(),
        Align2::LEFT_CENTER,
        s,
        mono(8.0),
        INK_DIM,
        0.16,
    );
}

pub fn show(app: &mut ForgeApp, ui: &mut Ui, r: Rect) {
    col(ui, r, "settings", |ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            let w = ui.available_width().min(760.0);
            heading(ui, "LOCAL LLM · OLLAMA");
            label(ui, "ENDPOINT");
            widgets::text_field(ui, &mut app.settings.llm.endpoint, "http://127.0.0.1:11434", w, 28.0, GOLD);
            label(ui, "MODEL");
            widgets::text_field(ui, &mut app.settings.llm.model, "llama3.2", w, 28.0, GOLD);
            if !app.llm_models.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                    for m in app.llm_models.clone() {
                        let on = app.settings.llm.model == m;
                        if widgets::chip(ui, &m, on, instrument_kit::Channel::Cool).clicked() {
                            app.settings.llm.model = m;
                        }
                    }
                });
            }
            label(ui, "SYSTEM PROMPT (OPTIONAL)");
            widgets::text_field(ui, &mut app.settings.llm.system, "You are the rack's narrator…", w, 28.0, GOLD);
            ui.horizontal(|ui| {
                if widgets::cv_btn(ui, "TEST CONNECTION", true, false).clicked() {
                    app.refresh_models();
                }
                let ok = app.llm_status.starts_with("connected");
                let c = if ok { SIGNAL_LIFE } else if app.llm_status.is_empty() { INK_DIM } else { SIGNAL_AMBER };
                let msg = if app.llm_status.is_empty() { "not checked" } else { app.llm_status.as_str() };
                let (sr, _) = ui.allocate_exact_size(vec2((w - 150.0).max(100.0), 24.0), Sense::hover());
                widgets::led_at(ui.painter(), pos2(sr.left() + 6.0, sr.center().y), 6.0, ok, SIGNAL_LIFE);
                text_clipped(ui.painter(), pos2(sr.left() + 18.0, sr.center().y), Align2::LEFT_CENTER, msg, mono(9.0), c, 0.04, sr.width() - 20.0);
            });
            ui.label(egui::RichText::new("Everything runs on your own machine. No keys, no cloud providers; the Node Graph's MODEL nodes call this server.").font(instrument_kit::theme::body(11.0)).color(INK_DIM));

            ui.add_space(8.0);
            heading(ui, "AUDIO · MIDI");
            for (name, state) in [("AUDIO OUTPUT", "not available in this build — the engine runs the clock and note logic silently"), ("MIDI INPUT", "not available in this build — MIDI messages are decoded and mapped, but no device backend is compiled in")] {
                let (rr, _) = ui.allocate_exact_size(vec2(w, 16.0), Sense::hover());
                widgets::led_at(ui.painter(), pos2(rr.left() + 6.0, rr.center().y), 6.0, false, SIGNAL_AMBER);
                let t = text(ui.painter(), pos2(rr.left() + 18.0, rr.center().y), Align2::LEFT_CENTER, name, mono(9.0), INK, 0.1);
                text_clipped(ui.painter(), pos2(t.right() + 10.0, rr.center().y), Align2::LEFT_CENTER, state, instrument_kit::theme::body(11.0), INK_DIM, 0.0, w - t.width() - 30.0);
            }

            ui.add_space(8.0);
            heading(ui, "STORAGE");
            let root = app.ws.root().display().to_string();
            let (rr, _) = ui.allocate_exact_size(vec2(w, 16.0), Sense::hover());
            text_clipped(ui.painter(), rr.left_center(), Align2::LEFT_CENTER, &root, mono(9.0), INK_MID, 0.0, w);
            ui.add_space(30.0);
        });
    });
}
