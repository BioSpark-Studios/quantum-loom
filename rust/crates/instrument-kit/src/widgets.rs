//! Atomic instrument controls. Every control is a physical object: it
//! recesses, rises, glows and clicks. Values are plain numbers the caller
//! binds to whatever store it uses; the kit never knows about an app.

use crate::icons;
use crate::paint::{self, *};
use crate::theme::*;
use egui::{
    pos2, vec2, Align2, Color32, CursorIcon, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2,
};

// ---- knob -----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum KnobVariant {
    Classic,
    #[default]
    Arc,
    Dotted,
    Forge,
    Ringed,
    Pip,
}

impl KnobVariant {
    pub fn from_key(k: &str) -> Self {
        match k {
            "classic" => Self::Classic,
            "dotted" => Self::Dotted,
            "forge" => Self::Forge,
            "ringed" => Self::Ringed,
            "pip" => Self::Pip,
            _ => Self::Arc,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Knob<'a> {
    pub size: f32,
    pub ch: Channel,
    /// Overrides the channel colour (department hues, custom accents).
    pub color: Option<Color32>,
    pub variant: KnobVariant,
    pub label: Option<&'a str>,
    pub label_size: f32,
    pub bipolar: bool,
    pub default: f32,
    pub show_value: bool,
}

impl Default for Knob<'_> {
    fn default() -> Self {
        Self {
            size: 64.0,
            ch: Channel::Cool,
            color: None,
            variant: KnobVariant::Arc,
            label: None,
            label_size: 9.0,
            bipolar: false,
            default: 0.5,
            show_value: true,
        }
    }
}

const SWEEP: f32 = 270.0;

impl<'a> Knob<'a> {
    pub fn new(size: f32, ch: Channel) -> Self {
        Self {
            size,
            ch,
            ..Default::default()
        }
    }
    pub fn variant(mut self, v: KnobVariant) -> Self {
        self.variant = v;
        self
    }
    pub fn label(mut self, l: &'a str) -> Self {
        self.label = Some(l);
        self
    }
    pub fn label_size(mut self, s: f32) -> Self {
        self.label_size = s;
        self
    }
    pub fn default_value(mut self, d: f32) -> Self {
        self.default = d;
        self
    }
    pub fn bipolar(mut self, b: bool) -> Self {
        self.bipolar = b;
        self
    }
    pub fn show_value(mut self, s: bool) -> Self {
        self.show_value = s;
        self
    }
    pub fn color(mut self, c: Color32) -> Self {
        self.color = Some(c);
        self
    }

    /// Footprint of the control including label and value rows.
    pub fn footprint(&self) -> Vec2 {
        let mut h = self.size;
        if self.label.is_some_and(|l| !l.is_empty()) {
            h += 4.0 + self.label_size + 4.0;
        }
        if self.show_value {
            h += 14.0;
        }
        vec2(self.size + 12.0, h)
    }

    /// Drag ↕ to set · Shift = fine · double-click = reset.
    pub fn show(self, ui: &mut Ui, value: &mut f32) -> Response {
        let (rect, mut resp) = ui.allocate_exact_size(self.footprint(), Sense::click_and_drag());
        let knob = Rect::from_min_size(
            pos2(rect.center().x - self.size / 2.0, rect.top()),
            Vec2::splat(self.size),
        );
        if resp.dragged() {
            let fine = ui.input(|i| i.modifiers.shift);
            let nv =
                (*value - resp.drag_delta().y / if fine { 800.0 } else { 200.0 }).clamp(0.0, 1.0);
            if nv != *value {
                *value = nv;
                resp.mark_changed();
            }
        }
        if resp.double_clicked() {
            *value = self.default;
            resp.mark_changed();
        }
        if resp.hovered() || resp.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
        }
        if ui.is_rect_visible(rect) {
            let p = ui.painter();
            self.paint(p, knob, *value);
            let mut y = knob.bottom() + 4.0;
            if let Some(l) = self.label.filter(|l| !l.is_empty()) {
                engrave(
                    p,
                    pos2(rect.center().x, y),
                    Align2::CENTER_TOP,
                    l,
                    self.label_size,
                    ETCH,
                );
                y += self.label_size + 4.0;
            }
            if self.show_value {
                let v = if self.bipolar {
                    (*value - 0.5) * 2.0
                } else {
                    *value
                };
                readout(
                    p,
                    pos2(rect.center().x, y),
                    Align2::CENTER_TOP,
                    &format!("{v:.2}"),
                    10.0,
                    alpha(self.accent(), 0.85),
                );
            }
        }
        resp
    }

    fn accent(&self) -> Color32 {
        self.color.unwrap_or_else(|| self.ch.color())
    }

    /// Paint just the knob body into `rect` (square).
    pub fn paint(&self, p: &egui::Painter, rect: Rect, value: f32) {
        let (c, s) = (rect.center(), rect.width());
        let accent = self.accent();
        let norm = value.clamp(0.0, 1.0);
        let angle = -SWEEP / 2.0 + norm * SWEEP;
        match self.variant {
            KnobVariant::Arc | KnobVariant::Ringed | KnobVariant::Forge => {
                let r = s / 2.0 - 4.0;
                let sw = if self.variant == KnobVariant::Forge {
                    4.0
                } else {
                    2.5
                };
                arc(p, c, r, -SWEEP / 2.0, SWEEP / 2.0, sw, white(0.07));
                if self.bipolar {
                    p.circle_filled(polar(c, r, 0.0), 1.4, white(0.4));
                }
                let from = if self.bipolar { 0.0 } else { -SWEEP / 2.0 };
                arc_glow(p, c, r, from, angle, sw, accent);
            }
            KnobVariant::Dotted => {
                let r = (s / 2.0 + 2.0) * 0.66;
                let lit = (norm * 38.0).round() as i32;
                for i in 0..=38 {
                    let d = -135.0 + i as f32 * (270.0 / 38.0);
                    let col = if i <= lit {
                        alpha(accent, 0.85)
                    } else {
                        white(0.08)
                    };
                    p.circle_filled(polar(c, r, d), 0.9, col);
                }
            }
            _ => {}
        }
        // cap
        let cap = (s / 2.0 - 8.0).max(3.0);
        p.circle_filled(c + vec2(0.0, 2.0), cap + 1.0, black(0.45));
        if self.variant == KnobVariant::Forge {
            let kr = cap + 2.0;
            p.circle_stroke(c, kr - 1.0, Stroke::new(2.0, rgb(0x1a1d24)));
            let teeth = ((kr * 1.6) as usize).clamp(12, 60);
            for i in 0..teeth {
                let d = i as f32 * 360.0 / teeth as f32 + angle;
                p.line_segment(
                    [polar(c, kr - 2.0, d), polar(c, kr, d)],
                    Stroke::new(1.0, rgb(0x3a4050)),
                );
            }
        }
        radial(p, c, cap, rgb(0x2a2f3a), rgb(0x0a0c12), vec2(-0.3, -0.4));
        p.circle_stroke(c, cap, Stroke::new(1.0, black(0.55)));
        arc(p, c, cap - 0.5, -70.0, 10.0, 1.0, white(0.09));
        // indicator
        if self.variant == KnobVariant::Pip {
            glow_dot(
                p,
                polar(c, cap * 0.62, angle),
                (cap * 0.16).clamp(1.5, 3.0),
                accent,
            );
        } else {
            let (a, b) = (polar(c, cap * 0.88, angle), polar(c, cap * 0.38, angle));
            p.line_segment([a, b], Stroke::new(4.0, alpha(accent, 0.22)));
            p.line_segment([a, b], Stroke::new(2.0, accent));
        }
        if self.variant == KnobVariant::Ringed {
            glow_dot(p, c, (cap * 0.22).clamp(1.5, 3.0), accent);
        }
    }
}

// ---- faders ---------------------------------------------------------------

/// Vertical fader with scale, lit fill and a metal cap.
pub fn fader(
    ui: &mut Ui,
    value: &mut f32,
    height: f32,
    width: f32,
    ch: Channel,
    label: Option<&str>,
    show_scale: bool,
) -> Response {
    let label_h = if label.is_some() { 15.0 } else { 0.0 };
    let (rect, mut resp) = ui.allocate_exact_size(
        vec2(width + 28.0, height + label_h),
        Sense::click_and_drag(),
    );
    let body = Rect::from_min_size(
        pos2(rect.center().x - width / 2.0, rect.top() + label_h),
        vec2(width, height),
    );
    let track = Rect::from_center_size(body.center(), vec2(8.0, height));
    if resp.is_pointer_button_down_on() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let nv = (1.0 - (pos.y - track.top()) / track.height()).clamp(0.0, 1.0);
            if nv != *value {
                *value = nv;
                resp.mark_changed();
            }
        }
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
    }
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let p = ui.painter();
    let accent = ch.color();
    if let Some(l) = label {
        engrave(
            p,
            pos2(rect.center().x, rect.top()),
            Align2::CENTER_TOP,
            l,
            9.0,
            ETCH,
        );
    }
    if show_scale {
        for i in 0..11 {
            let y = body.top() + height * i as f32 / 10.0;
            let (w, a) = if i % 5 == 0 { (6.0, 0.3) } else { (4.0, 0.15) };
            p.line_segment(
                [pos2(body.left(), y), pos2(body.left() + w, y)],
                Stroke::new(1.0, white(a)),
            );
        }
    }
    grad_v(p, track, 4.0, rgb(0x050810), rgb(0x0a0d14));
    outline(p, track, 4.0, black(0.6));
    let v = value.clamp(0.0, 1.0);
    let fill = Rect::from_min_max(
        pos2(track.left(), track.bottom() - track.height() * v),
        track.max,
    );
    if fill.height() > 1.0 {
        glow_rect(p, fill, 4.0, 8.0, alpha(accent, 0.5));
        grad_v(p, fill, 4.0, alpha(accent, 0.6), accent);
    }
    let cap = Rect::from_center_size(
        pos2(
            track.center().x,
            (track.bottom() - track.height() * v).clamp(track.top() + 2.0, track.bottom() - 2.0),
        ),
        vec2(22.0, 16.0),
    );
    paint::shadow(p, cap, 3.0, 2.0, 4.0, black(0.7));
    grad_stops(
        p,
        cap,
        3.0,
        &[
            (0.0, rgb(0x2a2f3a)),
            (0.5, rgb(0x14181f)),
            (1.0, rgb(0x0a0c12)),
        ],
        true,
    );
    bevel(p, cap, 3.0, 0.15, 0.6);
    outline_outside(p, cap, 3.0, black(0.5));
    let line = Rect::from_center_size(cap.center(), vec2(14.0, 2.0));
    glow_rect(p, line, 1.0, 4.0, alpha(accent, 0.6));
    p.rect_filled(line, 1.0, accent);
    resp
}

/// Horizontal mixer fader (device level).
pub fn hfader(ui: &mut Ui, value: &mut f32, width: f32, color: Color32, default: f32) -> Response {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(width, 16.0), Sense::click_and_drag());
    let track = Rect::from_min_max(
        pos2(rect.left() + 6.0, rect.top() + 6.0),
        pos2(rect.right() - 6.0, rect.top() + 10.0),
    );
    if resp.is_pointer_button_down_on() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let nv = ((pos.x - track.left()) / track.width()).clamp(0.0, 1.0);
            if nv != *value {
                *value = nv;
                resp.mark_changed();
            }
        }
    }
    if resp.double_clicked() {
        *value = default;
        resp.mark_changed();
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
    }
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let p = ui.painter();
    for i in 0..11 {
        let x = track.left() + track.width() * i as f32 / 10.0;
        p.line_segment(
            [pos2(x, track.top() - 4.0), pos2(x, track.top() - 2.0)],
            Stroke::new(1.0, white(0.2)),
        );
    }
    p.rect_filled(track, 2.0, rgb(0x04060a));
    let v = value.clamp(0.0, 1.0);
    let fill = Rect::from_min_max(
        track.min,
        pos2(track.left() + track.width() * v, track.bottom()),
    );
    if fill.width() > 0.5 {
        glow_rect(p, fill, 2.0, 6.0, alpha(color, 0.55));
        p.rect_filled(fill, 2.0, color);
    }
    let cap = Rect::from_center_size(
        pos2(track.left() + track.width() * v, rect.center().y),
        vec2(12.0, 16.0),
    );
    paint::shadow(p, cap, 2.0, 2.0, 4.0, black(0.6));
    grad_stops(
        p,
        cap,
        2.0,
        &[
            (0.0, rgb(0x14181f)),
            (0.5, rgb(0x4a505c)),
            (1.0, rgb(0x14181f)),
        ],
        false,
    );
    outline_outside(p, cap, 2.0, black(0.7));
    p.line_segment(
        [
            pos2(cap.center().x, cap.top() + 2.0),
            pos2(cap.center().x, cap.bottom() - 2.0),
        ],
        Stroke::new(1.0, color),
    );
    resp
}

/// Bipolar amount strip (−1..1) for the routing matrix.
pub fn amount(ui: &mut Ui, value: &mut f32, width: f32, color: Color32) -> Response {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(width, 20.0), Sense::click_and_drag());
    if resp.is_pointer_button_down_on() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let nv = ((((pos.x - rect.left()) / rect.width()) * 2.0 - 1.0).clamp(-1.0, 1.0)
                * 100.0)
                .round()
                / 100.0;
            if nv != *value {
                *value = nv;
                resp.mark_changed();
            }
        }
    }
    if resp.double_clicked() {
        *value = 0.0;
        resp.mark_changed();
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
    }
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        p.rect_filled(rect, 3.0, rgb(0x05070a));
        outline(p, rect, 3.0, black(0.7));
        let mid = rect.center().x;
        let x = mid + *value * rect.width() / 2.0;
        let bar = Rect::from_min_max(
            pos2(mid.min(x), rect.top() + 3.0),
            pos2(mid.max(x), rect.bottom() - 3.0),
        );
        if bar.width() > 0.5 {
            p.rect_filled(bar, 1.0, alpha(color, 0.75));
        }
        p.line_segment(
            [pos2(mid, rect.top() + 2.0), pos2(mid, rect.bottom() - 2.0)],
            Stroke::new(1.0, white(0.25)),
        );
        let pct = (*value * 100.0).round() as i32;
        let label = if pct > 0 {
            format!("+{pct}")
        } else {
            pct.to_string()
        };
        text(
            p,
            pos2(rect.right() - 5.0, rect.center().y),
            Align2::RIGHT_CENTER,
            &label,
            mono(9.0),
            INK,
            0.04,
        );
    }
    resp
}

// ---- buttons --------------------------------------------------------------

pub enum Face<'a> {
    Text(&'a str),
    Icon(&'a str),
    None,
}

/// `.pad` — trigger pad that lights in its channel colour.
pub fn pad(ui: &mut Ui, face: Face<'_>, ch: Channel, lit: bool, size: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if ui.is_rect_visible(rect) {
        paint_pad(
            ui.painter(),
            rect,
            face,
            ch.color(),
            lit,
            resp.hovered(),
            resp.is_pointer_button_down_on(),
        );
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

pub fn paint_pad(
    p: &egui::Painter,
    rect: Rect,
    face: Face<'_>,
    accent: Color32,
    lit: bool,
    hovered: bool,
    down: bool,
) {
    let rect = if down {
        rect.translate(vec2(0.0, 1.0))
    } else {
        rect
    };
    paint::shadow(p, rect, 6.0, 2.0, 4.0, black(0.5));
    if lit {
        glow_rect(p, rect, 6.0, 12.0, alpha(accent, 0.55));
        grad_v(
            p,
            rect,
            6.0,
            mix(accent, rgb(0x1a1d24), 0.4),
            mix(accent, rgb(0x0e1117), 0.2),
        );
        outline(p, rect, 6.0, alpha(accent, 0.55));
        bevel(p, rect, 6.0, 0.15, 0.0);
    } else {
        let (a, b) = if hovered {
            (rgb(0x1f242e), rgb(0x14181f))
        } else {
            (rgb(0x1a1d24), rgb(0x0e1117))
        };
        grad_v(p, rect, 6.0, a, b);
        bevel(p, rect, 6.0, 0.08, 0.6);
        outline(p, rect, 6.0, black(0.4));
    }
    let ink = if lit { Color32::WHITE } else { INK_MID };
    match face {
        Face::Text(t) => {
            text(
                p,
                rect.center(),
                Align2::CENTER_CENTER,
                t,
                display(9.0),
                ink,
                0.1,
            );
        }
        Face::Icon(i) => icons::paint(p, i, rect.center(), (rect.width() * 0.5).round(), ink),
        Face::None => {}
    }
}

/// `.gatebtn` — latching gate with an LED.
pub fn gate_btn(ui: &mut Ui, label: &str, ch: Channel, lit: bool) -> Response {
    let tw = measure(ui.painter(), label, display(9.0), 0.18).x;
    let w = 10.0 + 6.0 + if label.is_empty() { 0.0 } else { 6.0 + tw } + 8.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(w.ceil(), 26.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let accent = ch.color();
        paint::shadow(p, rect, 4.0, 2.0, 4.0, black(0.5));
        let (a, b) = if resp.hovered() {
            (rgb(0x20242c), rgb(0x12151b))
        } else {
            (rgb(0x1a1d24), rgb(0x0e1117))
        };
        grad_v(p, rect, 4.0, a, b);
        bevel(p, rect, 4.0, 0.08, 0.0);
        outline(p, rect, 4.0, black(0.6));
        let c = pos2(rect.left() + 13.0, rect.center().y);
        if lit {
            glow_dot(p, c, 3.0, accent);
        } else {
            p.circle_filled(c, 3.0, white(0.1));
            p.circle_stroke(c, 3.0, Stroke::new(1.0, black(0.6)));
        }
        if !label.is_empty() {
            engrave(
                p,
                pos2(rect.left() + 22.0, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                9.0,
                ETCH,
            );
        }
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// `.cv-btn` — small engraved command button.
pub fn cv_btn(ui: &mut Ui, label: &str, primary: bool, danger: bool) -> Response {
    let tw = measure(ui.painter(), label, mono(8.5), 0.12).x;
    let (rect, resp) = ui.allocate_exact_size(vec2((tw + 18.0).ceil(), 24.0), Sense::click());
    paint_cv_btn(ui, rect, &resp, label, primary, danger);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// `.cv-btn` stretched to a given width.
pub fn cv_btn_wide(ui: &mut Ui, label: &str, width: f32, primary: bool, danger: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 26.0), Sense::click());
    paint_cv_btn(ui, rect, &resp, label, primary, danger);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

fn paint_cv_btn(ui: &Ui, rect: Rect, resp: &Response, label: &str, primary: bool, danger: bool) {
    if !ui.is_rect_visible(rect) {
        return;
    }
    let p = ui.painter();
    let enabled = ui.is_enabled();
    let hot = resp.hovered() && enabled;
    grad_v(
        p,
        rect,
        3.0,
        if hot { rgb(0x2a303b) } else { rgb(0x232831) },
        if hot { rgb(0x1a1e26) } else { rgb(0x15181e) },
    );
    bevel(p, rect, 3.0, 0.08, 0.0);
    outline(
        p,
        rect,
        3.0,
        if primary {
            alpha(SIGNAL_COOL, 0.35)
        } else {
            black(0.7)
        },
    );
    let mut ink = if primary {
        SIGNAL_COOL
    } else if hot {
        INK
    } else {
        INK_MID
    };
    if danger && hot {
        ink = SIGNAL_HOT;
    }
    if !enabled {
        ink = alpha(ink, 0.35);
    }
    text(
        p,
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        mono(8.5),
        ink,
        0.12,
    );
}

/// `.fx-go` — the primary forge-orange action.
pub fn go_btn(ui: &mut Ui, label: &str, width: f32, small: bool) -> Response {
    let h = if small { 28.0 } else { 40.0 };
    let (rect, resp) = ui.allocate_exact_size(vec2(width, h), Sense::click());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let on = ui.is_enabled();
        let k = if on { 1.0 } else { 0.35 };
        if on {
            glow_rect(p, rect, 6.0, 20.0, alpha(rgb(0xff7a3c), 0.25));
        }
        let lift = if resp.hovered() && on { 1.08 } else { 1.0 };
        let tone = |c: Color32| {
            alpha(
                Color32::from_rgb(
                    (c.r() as f32 * lift).min(255.0) as u8,
                    (c.g() as f32 * lift).min(255.0) as u8,
                    (c.b() as f32 * lift).min(255.0) as u8,
                ),
                k,
            )
        };
        grad_v(p, rect, 6.0, tone(rgb(0xff8a4c)), tone(rgb(0xe2622a)));
        text(
            p,
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            mono(if small { 8.5 } else { 10.0 }),
            alpha(rgb(0x1a0b04), if on { 1.0 } else { 0.8 }),
            if small { 0.14 } else { 0.2 },
        );
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// `.fx-tb` — flat toolbar button: icon, optional label, optional accent when on.
pub fn tool_btn(
    ui: &mut Ui,
    icon: &str,
    label: Option<&str>,
    on: bool,
    accent: Option<Color32>,
) -> Response {
    let tw = label
        .map(|l| measure(ui.painter(), l, mono(8.5), 0.12).x + 6.0)
        .unwrap_or(0.0);
    let (rect, resp) = ui.allocate_exact_size(vec2((28.0 + tw).ceil(), 28.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let a = accent.unwrap_or(GOLD);
        let enabled = ui.is_enabled();
        let mut ink = INK_MID;
        if on {
            p.rect_filled(rect, 5.0, alpha(a, 0.14));
            ink = a;
        } else if resp.hovered() && enabled {
            p.rect_filled(rect, 5.0, white(0.06));
            ink = INK;
        }
        if !enabled {
            ink = alpha(ink, 0.3);
        }
        icons::paint(
            p,
            icon,
            pos2(rect.left() + 14.0, rect.center().y),
            16.0,
            ink,
        );
        if let Some(l) = label {
            text(
                p,
                pos2(rect.left() + 28.0, rect.center().y),
                Align2::LEFT_CENTER,
                l,
                mono(8.5),
                ink,
                0.12,
            );
        }
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// Vertical hairline between toolbar groups.
pub fn sep(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(5.0, 20.0), Sense::hover());
    ui.painter().line_segment(
        [rect.center_top(), rect.center_bottom()],
        Stroke::new(1.0, white(0.08)),
    );
}

/// Small MUTE / SOLO style latch.
pub fn latch(ui: &mut Ui, label: &str, on: bool, on_color: Color32) -> Response {
    let tw = measure(ui.painter(), label, mono(8.0), 0.08).x;
    let (rect, resp) = ui.allocate_exact_size(vec2((tw + 12.0).ceil(), 16.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        if on {
            glow_rect(p, rect, 2.0, 6.0, alpha(on_color, 0.6));
            p.rect_filled(rect, 2.0, on_color);
        } else {
            grad_v(p, rect, 2.0, rgb(0x2a2f39), rgb(0x1a1e25));
            bevel(p, rect, 2.0, 0.08, 0.0);
        }
        text(
            p,
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            mono(8.0),
            if on { Color32::BLACK } else { INK_DIM },
            0.08,
        );
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// `.rack-chip` — toggle chip with an LED (output targets).
pub fn chip(ui: &mut Ui, label: &str, on: bool, ch: Channel) -> Response {
    let tw = measure(ui.painter(), label, mono(8.0), 0.1).x;
    let (rect, resp) = ui.allocate_exact_size(vec2((tw + 28.0).ceil(), 22.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        grad_v(
            p,
            rect,
            3.0,
            if on { rgb(0x1e232c) } else { rgb(0x15181e) },
            rgb(0x101318),
        );
        outline(
            p,
            rect,
            3.0,
            if on {
                alpha(ch.color(), 0.4)
            } else {
                black(0.7)
            },
        );
        led_at(
            p,
            pos2(rect.left() + 10.0, rect.center().y),
            6.0,
            on,
            ch.color(),
        );
        text(
            p,
            pos2(rect.left() + 19.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            mono(8.0),
            if on { INK } else { INK_DIM },
            0.1,
        );
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// Three-way segmented selector (`.fx-seg3`). Returns the clicked index.
pub fn segmented(
    ui: &mut Ui,
    width: f32,
    items: &[(&str, Color32)],
    current: usize,
) -> Option<usize> {
    let (rect, base) = ui.allocate_exact_size(vec2(width, 32.0), Sense::hover());
    let p = ui.painter().clone();
    p.rect_filled(rect, 6.0, rgb(0x05070a));
    outline(&p, rect, 6.0, white(0.08));
    let inner = rect.shrink(2.0);
    let w = (inner.width() - 2.0 * (items.len() as f32 - 1.0)) / items.len() as f32;
    let mut clicked = None;
    for (i, (label, color)) in items.iter().enumerate() {
        let r = Rect::from_min_size(
            pos2(inner.left() + i as f32 * (w + 2.0), inner.top()),
            vec2(w, inner.height()),
        );
        let resp = ui
            .interact(r, base.id.with(i), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        let on = i == current;
        if on {
            p.rect_filled(r, 4.0, rgb(0x161b23));
        }
        let ink = if on {
            *color
        } else if resp.hovered() {
            INK_MID
        } else {
            INK_DIM
        };
        let tw = measure(&p, label, mono(8.0), 0.1).x;
        let x0 = r.center().x - (tw + 12.0) / 2.0;
        if on {
            glow_dot(&p, pos2(x0 + 3.0, r.center().y), 3.0, ink);
        } else {
            p.circle_filled(pos2(x0 + 3.0, r.center().y), 3.0, alpha(ink, 0.5));
        }
        text(
            &p,
            pos2(x0 + 12.0, r.center().y),
            Align2::LEFT_CENTER,
            label,
            mono(8.0),
            ink,
            0.1,
        );
        if resp.clicked() {
            clicked = Some(i);
        }
    }
    clicked
}

// ---- displays -------------------------------------------------------------

/// `.ro-wrap` — labelled segmented readout.
pub fn readout_box(
    ui: &mut Ui,
    label: Option<&str>,
    value: &str,
    ch: Channel,
    width: f32,
) -> Response {
    let lh = if label.is_some() { 14.0 } else { 0.0 };
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 22.0 + lh), Sense::hover());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        if let Some(l) = label {
            engrave(p, rect.left_top(), Align2::LEFT_TOP, l, 9.0, INK_DIM);
        }
        let screen = Rect::from_min_size(pos2(rect.left(), rect.top() + lh), vec2(width, 22.0));
        grad_v(p, screen, 3.0, rgb(0x050810), rgb(0x03050a));
        outline(p, screen, 3.0, black(0.7));
        text(
            p,
            pos2(screen.left() + 6.0, screen.center().y),
            Align2::LEFT_CENTER,
            "88:88",
            mono(12.0),
            white(0.04),
            0.1,
        );
        text_clipped(
            p,
            pos2(screen.right() - 5.0, screen.center().y),
            Align2::RIGHT_CENTER,
            value,
            mono(12.0),
            ch.color(),
            0.1,
            width - 8.0,
        );
    }
    resp
}

pub fn led_at(p: &egui::Painter, c: Pos2, size: f32, on: bool, color: Color32) {
    let r = size / 2.0;
    if on {
        glow_dot(p, c, r, color);
    } else {
        p.circle_filled(c, r, white(0.05));
        p.circle_stroke(c, r, Stroke::new(1.0, black(0.6)));
    }
}

pub fn led(ui: &mut Ui, on: bool, ch: Channel, size: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size + 2.0), Sense::hover());
    led_at(ui.painter(), rect.center(), size, on, ch.color());
    resp
}

/// 24-segment VU bar: green → amber → red.
pub fn vu(ui: &mut Ui, level: f32, width: f32, height: f32, label: Option<&str>) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let mut bar = rect;
        if let Some(l) = label {
            text(
                p,
                rect.left_center(),
                Align2::LEFT_CENTER,
                l,
                mono(8.0),
                INK_MID,
                0.0,
            );
            bar.min.x += 12.0;
        }
        p.rect_filled(bar, 2.0, rgb(0x04060b));
        let inner = bar.shrink(1.5);
        let segs = 24;
        let n = (level.clamp(0.0, 1.0) * segs as f32).round() as usize;
        let sw = (inner.width() - (segs - 1) as f32) / segs as f32;
        for i in 0..segs {
            let f = i as f32 / segs as f32;
            let r = Rect::from_min_size(
                pos2(inner.left() + i as f32 * (sw + 1.0), inner.top()),
                vec2(sw, inner.height()),
            );
            if i < n {
                let c = if f < 0.6 {
                    SIGNAL_LIFE
                } else if f < 0.85 {
                    SIGNAL_AMBER
                } else {
                    SIGNAL_HOT
                };
                p.rect_filled(r, 1.0, c);
            } else {
                p.rect_filled(r, 1.0, white(0.04));
            }
        }
    }
    resp
}

/// 16-segment device meter with amber / red headroom.
pub fn meter16(p: &egui::Painter, rect: Rect, lit: usize, color: Color32) {
    let sw = (rect.width() - 15.0) / 16.0;
    for i in 0..16 {
        let r = Rect::from_min_size(
            pos2(rect.left() + i as f32 * (sw + 1.0), rect.top()),
            vec2(sw, rect.height()),
        );
        if i < lit {
            let c = if i > 12 {
                SIGNAL_HOT
            } else if i > 9 {
                SIGNAL_AMBER
            } else {
                color
            };
            glow_rect(p, r, 1.0, 4.0, alpha(c, 0.5));
            p.rect_filled(r, 1.0, c);
        } else {
            p.rect_filled(r, 1.0, white(0.05));
        }
    }
}

// ---- pattern / position controls -----------------------------------------

/// Row of latching steps. `current` highlights the playhead. Returns true when
/// a step was toggled.
pub fn step_row(
    ui: &mut Ui,
    steps: &mut [bool],
    current: Option<usize>,
    ch: Channel,
    cell: f32,
) -> bool {
    let n = steps.len().max(1);
    let (rect, base) = ui.allocate_exact_size(
        vec2(n as f32 * cell + (n as f32 - 1.0) * 3.0, cell),
        Sense::hover(),
    );
    let p = ui.painter().clone();
    let accent = ch.color();
    let mut changed = false;
    for (i, on) in steps.iter_mut().enumerate() {
        let r = Rect::from_min_size(
            pos2(rect.left() + i as f32 * (cell + 3.0), rect.top()),
            Vec2::splat(cell),
        );
        let resp = ui
            .interact(r, base.id.with(i), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        if resp.clicked() {
            *on = !*on;
            changed = true;
        }
        let cur = current == Some(i);
        if cur {
            glow_rect(&p, r, 3.0, 6.0, alpha(accent, 0.4));
        }
        grad_v(&p, r, 3.0, rgb(0x14181f), rgb(0x0a0c10));
        bevel(&p, r, 3.0, 0.06, 0.4);
        outline(&p, r, 3.0, if cur { accent } else { black(0.6) });
        let d = (cell * 0.18).max(2.0);
        if *on {
            glow_dot(&p, r.center(), d, accent);
        } else {
            p.circle_filled(r.center(), d, white(0.04));
        }
    }
    changed
}

/// `.xy-pad` — 2D modulation pad on a glass screen. Y grows downward, like the design.
pub fn xy_pad(
    ui: &mut Ui,
    x: &mut f32,
    y: &mut f32,
    size: f32,
    ch: Channel,
    label: Option<&str>,
) -> Response {
    let lh = if label.is_some_and(|l| !l.is_empty()) {
        16.0
    } else {
        0.0
    };
    let (rect, mut resp) = ui.allocate_exact_size(vec2(size, size + lh), Sense::click_and_drag());
    let padr = Rect::from_min_size(pos2(rect.left(), rect.top() + lh), Vec2::splat(size));
    if resp.is_pointer_button_down_on() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let nx = ((pos.x - padr.left()) / padr.width()).clamp(0.0, 1.0);
            let ny = ((pos.y - padr.top()) / padr.height()).clamp(0.0, 1.0);
            if nx != *x || ny != *y {
                *x = nx;
                *y = ny;
                resp.mark_changed();
            }
        }
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
    }
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        if let Some(l) = label.filter(|l| !l.is_empty()) {
            engrave(
                p,
                pos2(rect.center().x, rect.top()),
                Align2::CENTER_TOP,
                l,
                9.0,
                ETCH,
            );
        }
        let accent = ch.color();
        let g = mat_screen(p, padr, 6.0);
        for i in 1..4 {
            let t = i as f32 / 4.0;
            g.line_segment(
                [
                    pos2(padr.left() + padr.width() * t, padr.top()),
                    pos2(padr.left() + padr.width() * t, padr.bottom()),
                ],
                Stroke::new(1.0, white(0.04)),
            );
            g.line_segment(
                [
                    pos2(padr.left(), padr.top() + padr.height() * t),
                    pos2(padr.right(), padr.top() + padr.height() * t),
                ],
                Stroke::new(1.0, white(0.04)),
            );
        }
        let c = pos2(
            padr.left() + padr.width() * x.clamp(0.0, 1.0),
            padr.top() + padr.height() * y.clamp(0.0, 1.0),
        );
        g.line_segment(
            [pos2(c.x, padr.top()), pos2(c.x, padr.bottom())],
            Stroke::new(1.0, alpha(accent, 0.3)),
        );
        g.line_segment(
            [pos2(padr.left(), c.y), pos2(padr.right(), c.y)],
            Stroke::new(1.0, alpha(accent, 0.3)),
        );
        g.line_segment(
            [c - vec2(8.0, 0.0), c + vec2(8.0, 0.0)],
            Stroke::new(1.0, alpha(accent, 0.6)),
        );
        g.line_segment(
            [c - vec2(0.0, 8.0), c + vec2(0.0, 8.0)],
            Stroke::new(1.0, alpha(accent, 0.6)),
        );
        glow_dot(&g, c, 4.0, accent);
    }
    resp
}

/// Vertical multi-position slide switch with engraved position labels.
pub fn slide_switch(ui: &mut Ui, value: &mut usize, labels: &[&str], ch: Channel) -> Response {
    let n = labels.len().max(2);
    let lw = labels
        .iter()
        .map(|l| measure(ui.painter(), l, display(7.0), 0.16).x)
        .fold(0.0, f32::max);
    let (rect, mut resp) =
        ui.allocate_exact_size(vec2(24.0 + 6.0 + lw.ceil(), 56.0), Sense::click());
    if resp.clicked() {
        *value = (*value + 1) % n;
        resp.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let track = Rect::from_min_size(rect.min, vec2(24.0, 56.0));
        grad_v(p, track, 4.0, rgb(0x050810), rgb(0x0a0d14));
        outline(p, track, 4.0, black(0.6));
        let accent = ch.color();
        for i in 0..n {
            let y = track.top() + 8.0 + (track.height() - 16.0) * i as f32 / (n - 1) as f32;
            let on = i == *value;
            p.circle_filled(
                pos2(track.center().x, y),
                2.0,
                if on { accent } else { white(0.1) },
            );
            text(
                p,
                pos2(track.right() + 6.0, y),
                Align2::LEFT_CENTER,
                labels.get(i).copied().unwrap_or(""),
                display(7.0),
                if on { accent } else { INK_DIM },
                0.16,
            );
        }
        let ty = track.top()
            + 1.0
            + (track.height() - 24.0) * (*value).min(n - 1) as f32 / (n - 1) as f32;
        let thumb = Rect::from_min_size(pos2(track.left() + 1.0, ty), vec2(22.0, 22.0));
        paint::shadow(p, thumb, 3.0, 2.0, 4.0, black(0.7));
        grad_stops(
            p,
            thumb,
            3.0,
            &[
                (0.0, rgb(0x3a4050)),
                (0.5, rgb(0x1a1d24)),
                (1.0, rgb(0x0e1117)),
            ],
            true,
        );
        bevel(p, thumb, 3.0, 0.15, 0.6);
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

// ---- glyph tiles ----------------------------------------------------------

/// `.gt` — brushed-metal keycap with a recessed, lit glass well holding a glyph.
pub fn glyph_tile_at(
    p: &egui::Painter,
    rect: Rect,
    glyph: &str,
    tone: Color32,
    round: bool,
    off: bool,
) {
    let s = rect.width();
    let (rad, well_rad) = if round {
        (s / 2.0, s / 2.0)
    } else {
        (s * 0.22, s * 0.15)
    };
    let tone = if off { rgb(0x3c4656) } else { tone };
    paint::shadow(p, rect, rad, 3.0, 8.0, black(0.5));
    grad_stops(
        p,
        rect,
        rad,
        &[
            (0.0, rgb(0x2c323e)),
            (0.52, rgb(0x191d26)),
            (1.0, rgb(0x10131a)),
        ],
        true,
    );
    if round {
        p.circle_stroke(rect.center(), s / 2.0, Stroke::new(1.0, black(0.65)));
        arc(
            p,
            rect.center(),
            s / 2.0 - 1.0,
            -60.0,
            60.0,
            1.0,
            white(0.13),
        );
    } else {
        bevel(p, rect, rad, 0.13, 0.65);
        outline_outside(p, rect, rad, black(0.65));
    }
    let well = rect.shrink(s * 0.13);
    if round {
        radial(
            p,
            well.center(),
            well.width() / 2.0,
            mix(tone, rgb(0x070b13), if off { 0.0 } else { 0.16 }),
            rgb(0x04060b),
            vec2(0.0, -0.24),
        );
        p.circle_stroke(
            well.center(),
            well.width() / 2.0,
            Stroke::new(1.0, black(0.75)),
        );
    } else {
        grad_v(
            p,
            well,
            well_rad,
            mix(tone, rgb(0x070b13), if off { 0.0 } else { 0.14 }),
            rgb(0x04060b),
        );
        outline(p, well, well_rad, black(0.75));
    }
    let sw = if s >= 72.0 {
        1.8
    } else if s >= 44.0 {
        1.6
    } else if s >= 28.0 {
        1.4
    } else {
        1.2
    };
    let g = (well.width() * 0.74).round();
    if !off {
        icons::paint_sw(
            p,
            glyph,
            well.center(),
            g + 4.0,
            alpha(tone, 0.22),
            Some(sw + 2.0),
        );
    }
    icons::paint_sw(p, glyph, well.center(), g, tone, Some(sw));
    if s >= 56.0 && !round {
        for (dx, dy) in [
            (4.0, 4.0),
            (s - 4.0, 4.0),
            (4.0, s - 4.0),
            (s - 4.0, s - 4.0),
        ] {
            radial(
                p,
                rect.min + vec2(dx, dy),
                2.5,
                rgb(0x59606f),
                rgb(0x1b1f27),
                vec2(-0.3, -0.3),
            );
        }
    }
}

pub fn glyph_tile(
    ui: &mut Ui,
    glyph: &str,
    tone: Color32,
    size: f32,
    round: bool,
    off: bool,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if ui.is_rect_visible(rect) {
        glyph_tile_at(ui.painter(), rect, glyph, tone, round, off);
    }
    resp
}

/// Small count badge on the corner of a rail button or tile.
pub fn badge(p: &egui::Painter, top_right: Pos2, n: usize) {
    let s = n.to_string();
    let w = (measure(p, &s, mono(7.0), 0.0).x + 6.0).max(13.0);
    let r = Rect::from_min_size(pos2(top_right.x - w, top_right.y), vec2(w, 13.0));
    p.rect_filled(r, 6.5, rgb(0x1b2230));
    text(
        p,
        r.center(),
        Align2::CENTER_CENTER,
        &s,
        mono(7.0),
        rgb(0xcbd5e1),
        0.0,
    );
}

/// Single-line text field in the shell style. Returns the response of the edit.
pub fn text_field(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    width: f32,
    height: f32,
    focus_color: Color32,
) -> Response {
    let (rect, base) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let id = base.id.with("field");
    let focused = ui.memory(|m| m.has_focus(id));
    let p = ui.painter().clone();
    p.rect_filled(rect, 6.0, rgb(0x05070a));
    outline(
        &p,
        rect,
        6.0,
        if focused { focus_color } else { white(0.1) },
    );
    let inner = rect.shrink2(vec2(12.0, 0.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.add(
        egui::TextEdit::singleline(value)
            .id(id)
            .frame(egui::Frame::NONE)
            .desired_width(inner.width())
            .font(text_font(height))
            .text_color(INK)
            .hint_text(
                egui::RichText::new(hint)
                    .color(INK_DIM)
                    .font(text_font(height)),
            ),
    )
}

fn text_font(height: f32) -> egui::FontId {
    crate::theme::body(if height >= 34.0 { 13.0 } else { 11.0 })
}
