//! Layout helpers and bus-bound controls shared by every workspace.

use egui::{pos2, Align, Color32, Id, Layout, Rect, Response, Ui, UiBuilder};
use instrument_kit::theme::Channel;
use instrument_kit::widgets::{self, Knob};
use rack_core::bus::{Bus, Value};

/// Run `f` in a left-to-right, vertically centred child UI clipped to `rect`.
pub fn row<R>(
    ui: &mut Ui,
    rect: Rect,
    salt: impl std::hash::Hash + std::fmt::Debug,
    f: impl FnOnce(&mut Ui) -> R,
) -> R {
    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(Id::unique(salt))
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    f(&mut child)
}

/// Like [`row`], laid out from the right edge.
pub fn row_rtl<R>(
    ui: &mut Ui,
    rect: Rect,
    salt: impl std::hash::Hash + std::fmt::Debug,
    f: impl FnOnce(&mut Ui) -> R,
) -> R {
    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(Id::unique(salt))
            .max_rect(rect)
            .layout(Layout::right_to_left(Align::Center)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    f(&mut child)
}

/// Top-down child UI clipped to `rect`.
pub fn col<R>(
    ui: &mut Ui,
    rect: Rect,
    salt: impl std::hash::Hash + std::fmt::Debug,
    f: impl FnOnce(&mut Ui) -> R,
) -> R {
    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(Id::unique(salt))
            .max_rect(rect)
            .layout(Layout::top_down(Align::Min)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    f(&mut child)
}

/// A row whose content is centred in `rect`. The content width is measured and
/// remembered, so it settles on the frame after it first appears.
pub fn centered<R>(
    ui: &mut Ui,
    rect: Rect,
    salt: impl std::hash::Hash + std::fmt::Debug,
    gap: f32,
    f: impl FnOnce(&mut Ui) -> R,
) -> R {
    let id = Id::unique(("centered", salt));
    let w: f32 = ui.data(|d| d.get_temp(id)).unwrap_or(rect.width());
    let x0 = rect.center().x - w.min(rect.width()) / 2.0;
    let r = Rect::from_min_max(pos2(x0, rect.top()), rect.max);
    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(id)
            .max_rect(r)
            .layout(Layout::left_to_right(Align::Center)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    child.spacing_mut().item_spacing.x = gap;
    let out = f(&mut child);
    let used = child.min_rect().width();
    if (used - w).abs() > 0.5 {
        ui.data_mut(|d| d.insert_temp(id, used));
        ui.ctx().request_repaint();
    }
    out
}

/// Knob bound to a bus address.
pub fn knob(ui: &mut Ui, bus: &mut Bus, key: &str, k: Knob<'_>) -> Response {
    let mut v = bus.num(key, k.default);
    let r = k.show(ui, &mut v);
    if r.changed() {
        bus.set_num(key, v);
    }
    r.on_hover_text_at_pointer("Drag ↕ · Shift = fine · Double-click = reset")
}

pub fn fader(
    ui: &mut Ui,
    bus: &mut Bus,
    key: &str,
    default: f32,
    height: f32,
    width: f32,
    ch: Channel,
    label: Option<&str>,
    scale: bool,
) -> Response {
    let mut v = bus.num(key, default);
    let r = widgets::fader(ui, &mut v, height, width, ch, label, scale);
    if r.changed() {
        bus.set_num(key, v);
    }
    r
}

/// Latching gate button bound to a boolean address.
pub fn gate(
    ui: &mut Ui,
    bus: &mut Bus,
    key: &str,
    label: &str,
    ch: Channel,
    default: bool,
) -> Response {
    let lit = bus.flag(key, default);
    let r = widgets::gate_btn(ui, label, ch, lit);
    if r.clicked() {
        bus.set_flag(key, !lit);
    }
    r
}

/// XY pad bound to `key.x` / `key.y`.
pub fn xy(
    ui: &mut Ui,
    bus: &mut Bus,
    key: &str,
    size: f32,
    ch: Channel,
    label: Option<&str>,
) -> Response {
    let (kx, ky) = (format!("{key}.x"), format!("{key}.y"));
    let (mut x, mut y) = (bus.num(&kx, 0.5), bus.num(&ky, 0.5));
    let r = widgets::xy_pad(ui, &mut x, &mut y, size, ch, label);
    if r.changed() {
        bus.set_num(&kx, x);
        bus.set_num(&ky, y);
    }
    r
}

/// Sixteen-step row bound to a pattern address; seeds a pattern on first use.
pub fn steps(
    ui: &mut Ui,
    bus: &mut Bus,
    key: &str,
    current: Option<usize>,
    ch: Channel,
    cell: f32,
) {
    let mut pat: Vec<bool> = match bus.steps(key) {
        Some(s) => s.to_vec(),
        None => {
            // Deterministic starter pattern so a fresh row is not empty.
            let seed = key
                .bytes()
                .fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32));
            let p: Vec<bool> = (0..16).map(|i| (seed >> (i % 24)) & 3 == 0).collect();
            bus.init(key, Value::Steps(p.clone()));
            p
        }
    };
    if widgets::step_row(ui, &mut pat, current, ch, cell) {
        bus.set(key, Value::Steps(pat));
    }
}

pub fn ch(key: &str) -> Channel {
    Channel::from_key(key)
}

pub fn chc(key: &str) -> Color32 {
    Channel::from_key(key).color()
}
