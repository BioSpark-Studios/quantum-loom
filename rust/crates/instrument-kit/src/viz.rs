//! Glass-screen visualizers. Every screen draws real per-frame data handed in
//! by the caller; the ones that need history keep it in egui's temp storage,
//! keyed by the widget id.

use crate::paint::*;
use crate::theme::*;
use egui::epaint::Mesh;
use egui::{pos2, vec2, Align2, Color32, Id, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};
use std::f32::consts::TAU;

fn screen(ui: &mut Ui, size: Vec2, sense: Sense) -> (Rect, Response, egui::Painter) {
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    let g = mat_screen(ui.painter(), rect, 6.0);
    (rect, resp, g)
}

fn corner_label(g: &egui::Painter, rect: Rect, label: &str) {
    if !label.is_empty() {
        text(
            g,
            pos2(rect.left() + 6.0, rect.bottom() - 4.0),
            Align2::LEFT_BOTTOM,
            label,
            display(8.0),
            white(0.35),
            0.18,
        );
    }
}

/// `● LIVE` / `○ SIM` tag: whether the trace is real audio or the synthetic model.
fn live_tag(g: &egui::Painter, rect: Rect, live: bool) {
    let (s, c) = if live {
        ("LIVE", SIGNAL_LIFE)
    } else {
        ("SIM", INK_DIM)
    };
    let r = text(
        g,
        pos2(rect.right() - 6.0, rect.top() + 4.0),
        Align2::RIGHT_TOP,
        s,
        mono(8.0),
        c,
        0.1,
    );
    let dot = pos2(r.left() - 5.0, r.center().y);
    if live {
        g.circle_filled(dot, 2.0, c);
    } else {
        g.circle_stroke(dot, 2.0, Stroke::new(1.0, c));
    }
}

fn tiny_noise(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed >> 8) as f32 / (1u32 << 24) as f32
}

/// Oscilloscope. `sample(u)` returns the trace at `u` (0..1 across), in −1..1.
pub fn scope(
    ui: &mut Ui,
    size: Vec2,
    ch: Channel,
    label: &str,
    live: Option<bool>,
    sample: impl Fn(f32) -> f32,
) -> Response {
    let (rect, resp, g) = screen(ui, size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    for i in 1..6 {
        let x = rect.left() + rect.width() * i as f32 / 6.0;
        g.line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            Stroke::new(1.0, white(0.04)),
        );
    }
    for i in 1..4 {
        let y = rect.top() + rect.height() * i as f32 / 4.0;
        g.line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            Stroke::new(1.0, white(if i == 2 { 0.08 } else { 0.04 })),
        );
    }
    let n = (rect.width() as usize).max(2);
    let pts: Vec<Pos2> = (0..n)
        .map(|i| {
            let u = i as f32 / (n - 1) as f32;
            pos2(
                rect.left() + u * rect.width(),
                rect.center().y - sample(u).clamp(-1.2, 1.2) * rect.height() * 0.42,
            )
        })
        .collect();
    polyline_glow(&g, pts, 1.6, ch.color());
    corner_label(&g, rect, label);
    if let Some(l) = live {
        live_tag(&g, rect, l);
    }
    resp
}

/// Spectrum bars with falling peak caps. `bands` are 0..1.
pub fn spectrum(
    ui: &mut Ui,
    id: Id,
    size: Vec2,
    ch: Channel,
    label: &str,
    bands: &[f32],
) -> Response {
    let (rect, resp, g) = screen(ui, size, Sense::hover());
    if !ui.is_rect_visible(rect) || bands.is_empty() {
        return resp;
    }
    let n = bands.len();
    let mut peaks: Vec<f32> = ui.data(|d| d.get_temp(id)).unwrap_or_default();
    peaks.resize(n, 0.0);
    let accent = ch.color();
    let bw = (rect.width() - n as f32) / n as f32;
    let mut mesh = Mesh::default();
    for (i, v) in bands.iter().enumerate() {
        let v = v.clamp(0.0, 1.0);
        peaks[i] = (peaks[i] * 0.97).max(v);
        let x = rect.left() + i as f32 * (bw + 1.0);
        let top = rect.bottom() - v * rect.height();
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(pos2(x, top), accent);
        mesh.colored_vertex(pos2(x + bw, top), accent);
        mesh.colored_vertex(pos2(x + bw, rect.bottom()), alpha(accent, 0.19));
        mesh.colored_vertex(pos2(x, rect.bottom()), alpha(accent, 0.19));
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
        let py = rect.bottom() - peaks[i] * rect.height() - 1.0;
        mesh.add_colored_rect(Rect::from_min_size(pos2(x, py), vec2(bw, 1.0)), accent);
    }
    g.add(Shape::mesh(mesh));
    ui.data_mut(|d| d.insert_temp(id, peaks));
    corner_label(&g, rect, label);
    resp
}

/// Scrolling spectrogram. One column of `bands` is appended per call.
pub fn spectrogram(ui: &mut Ui, id: Id, size: Vec2, label: &str, bands: &[f32]) -> Response {
    let (rect, resp, g) = screen(ui, size, Sense::hover());
    if !ui.is_rect_visible(rect) || bands.is_empty() {
        return resp;
    }
    let cols = ((rect.width() / 3.0) as usize).max(1);
    let rows = bands.len();
    let mut hist: Vec<Vec<f32>> = ui.data(|d| d.get_temp(id)).unwrap_or_default();
    hist.push(bands.to_vec());
    if hist.len() > cols {
        let cut = hist.len() - cols;
        hist.drain(..cut);
    }
    let rh = rect.height() / rows as f32;
    let cw = rect.width() / cols as f32;
    let mut mesh = Mesh::default();
    let off = cols - hist.len();
    for (ci, col) in hist.iter().enumerate() {
        for (r, v) in col.iter().enumerate().take(rows) {
            let a = (v * 1.2).min(1.0);
            if a < 0.04 {
                continue;
            }
            let c = Color32::from_rgb(
                (20.0 + a * 235.0) as u8,
                (10.0 + a * a * 120.0) as u8,
                (50.0 + (1.0 - a) * 120.0 * a + a * 60.0) as u8,
            );
            let x = rect.left() + (off + ci) as f32 * cw;
            let y = rect.bottom() - (r + 1) as f32 * rh;
            mesh.add_colored_rect(Rect::from_min_size(pos2(x, y), vec2(cw + 0.5, rh + 0.5)), c);
        }
    }
    g.add(Shape::mesh(mesh));
    ui.data_mut(|d| d.insert_temp(id, hist));
    corner_label(&g, rect, label);
    resp
}

/// Rolling level history as a mirrored envelope.
pub fn waveform(
    ui: &mut Ui,
    size: Vec2,
    ch: Channel,
    label: &str,
    hist: &[f32],
    head: usize,
) -> Response {
    let (rect, resp, g) = screen(ui, size, Sense::hover());
    if !ui.is_rect_visible(rect) || hist.is_empty() {
        return resp;
    }
    let n = hist.len();
    let accent = ch.color();
    let bw = rect.width() / n as f32;
    let mut mesh = Mesh::default();
    for i in 0..n {
        let v = hist[(head + i) % n].clamp(0.0, 1.0);
        let h = (v * rect.height() * 0.9).max(1.0);
        let x = rect.left() + i as f32 * bw;
        mesh.add_colored_rect(
            Rect::from_min_size(
                pos2(x, rect.center().y - h / 2.0),
                vec2((bw - 0.6).max(0.6), h),
            ),
            alpha(accent, 0.35 + 0.65 * i as f32 / n as f32),
        );
    }
    g.add(Shape::mesh(mesh));
    corner_label(&g, rect, label);
    resp
}

/// Lissajous phase scope. `a1`, `a2` are the frequency ratios, `ph` the phase offset.
#[allow(clippy::too_many_arguments)]
pub fn phase_scope(
    ui: &mut Ui,
    size: f32,
    ch: Channel,
    label: &str,
    a1: f32,
    a2: f32,
    amp: f32,
    ph: f32,
    t: f32,
) -> Response {
    let (rect, resp, g) = screen(ui, Vec2::splat(size), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    g.line_segment(
        [rect.center_top(), rect.center_bottom()],
        Stroke::new(1.0, white(0.06)),
    );
    g.line_segment(
        [rect.left_center(), rect.right_center()],
        Stroke::new(1.0, white(0.06)),
    );
    let c = rect.center();
    let mut pts: Vec<Pos2> = (0..=220)
        .map(|i| {
            let a = i as f32 / 220.0 * TAU;
            pos2(
                c.x + (a * a1 + t * 0.7).sin() * size * 0.4 * amp,
                c.y + (a * a2 + ph + t).sin() * size * 0.4 * amp,
            )
        })
        .collect();
    pts.dedup();
    polyline_glow(&g, pts, 1.2, ch.color());
    corner_label(&g, rect, label);
    resp
}

#[derive(Clone, Copy, Debug)]
pub struct FieldParams {
    pub scale: f32,
    pub drift: f32,
    pub speed: f32,
    pub damp: f32,
    pub run: bool,
    /// Signal level, 0..1: particles speed up and swell with it.
    pub level: f32,
}

impl Default for FieldParams {
    fn default() -> Self {
        Self {
            scale: 0.4,
            drift: 0.5,
            speed: 0.3,
            damp: 0.3,
            run: true,
            level: 0.0,
        }
    }
}

#[derive(Clone, Default)]
struct FieldState {
    ps: Vec<[f32; 5]>,
    t: f32,
    seed: u32,
}

/// Flow field of drifting particles.
pub fn particle_field(
    ui: &mut Ui,
    id: Id,
    size: Vec2,
    ch: Channel,
    label: &str,
    count: usize,
    fp: FieldParams,
) -> Response {
    let (rect, resp, g) = screen(ui, size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let (w, h) = (rect.width(), rect.height());
    let mut st: FieldState = ui.data(|d| d.get_temp(id)).unwrap_or_default();
    if st.seed == 0 {
        st.seed = (id.value() as u32) | 1;
    }
    if st.ps.len() != count {
        st.ps = (0..count)
            .map(|_| {
                [
                    tiny_noise(&mut st.seed) * w,
                    tiny_noise(&mut st.seed) * h,
                    0.0,
                    0.0,
                    tiny_noise(&mut st.seed),
                ]
            })
            .collect();
    }
    let sc = 1.0 + fp.scale * 6.0;
    let sp = 0.3 + fp.speed * 1.6 + fp.level * 0.8;
    let damp = 0.85 + (1.0 - fp.damp) * 0.12;
    if fp.run {
        st.t += 0.016 * (0.5 + fp.drift);
    }
    let t = st.t;
    let accent = ch.color();
    for q in st.ps.iter_mut() {
        if fp.run {
            let (fx, fy) = (q[0] / w * sc, q[1] / h * sc * 0.75);
            let ang = (fx + t * 0.4).sin()
                + (fy + t * 0.3).cos()
                + (fx * 2.0 - fy + t * 0.2).sin()
                + (fp.drift - 0.5) * 2.0;
            q[2] = q[2] * damp + ang.cos() * sp * 0.6;
            q[3] = q[3] * damp + ang.sin() * sp * 0.6;
            // Short trail in place of the canvas's persistence fade.
            let tail = pos2(rect.left() + q[0], rect.top() + q[1]);
            q[0] += q[2];
            q[1] += q[3];
            q[4] -= 0.005;
            if q[0] < 0.0 || q[0] > w || q[1] < 0.0 || q[1] > h || q[4] <= 0.0 {
                q[0] = tiny_noise(&mut st.seed) * w;
                q[1] = tiny_noise(&mut st.seed) * h;
                q[4] = 1.0;
            } else {
                let head = pos2(rect.left() + q[0], rect.top() + q[1]);
                g.line_segment(
                    [tail - (head - tail) * 2.0, head],
                    Stroke::new(1.0, alpha(accent, q[4] * 0.3)),
                );
            }
        }
        let c = pos2(rect.left() + q[0], rect.top() + q[1]);
        g.circle_filled(c, 2.0 + fp.level, alpha(accent, q[4] * 0.25));
        g.circle_filled(c, 0.7 + fp.level * 0.5, alpha(accent, q[4] * 0.9));
    }
    ui.data_mut(|d| d.insert_temp(id, st));
    corner_label(&g, rect, label);
    resp
}

/// Radar sweep with orbiting blips.
#[allow(clippy::too_many_arguments)]
pub fn polar_screen(
    ui: &mut Ui,
    id: Id,
    size: f32,
    ch: Channel,
    label: &str,
    speed: f32,
    count: f32,
    spread: f32,
    run: bool,
    level: f32,
) -> Response {
    let (rect, resp, g) = screen(ui, Vec2::splat(size), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let mut st: [f32; 2] = ui.data(|d| d.get_temp(id)).unwrap_or([0.0, 0.0]);
    if run {
        st[0] += 0.008 + speed * 0.06;
        st[1] += 0.016;
    }
    let (c, r) = (rect.center(), size / 2.0 - 4.0);
    for i in 1..=4 {
        g.circle_stroke(c, r / 4.0 * i as f32, Stroke::new(1.0, white(0.06)));
    }
    for i in 0..12 {
        let a = i as f32 / 12.0 * TAU;
        g.line_segment(
            [c, c + r * vec2(a.cos(), a.sin())],
            Stroke::new(1.0, white(0.06)),
        );
    }
    let accent = ch.color();
    // Sweep wedge: bright leading edge fading behind it.
    let mut mesh = Mesh::default();
    mesh.colored_vertex(c, alpha(accent, 0.25));
    let steps = 18;
    for i in 0..=steps {
        let f = i as f32 / steps as f32;
        let a = st[0] - 0.9 * (1.0 - f);
        mesh.colored_vertex(c + r * vec2(a.cos(), a.sin()), alpha(accent, f * f * 0.55));
    }
    for i in 0..steps {
        mesh.add_triangle(0, 1 + i, 2 + i);
    }
    g.add(Shape::mesh(mesh));
    g.line_segment(
        [c, c + r * vec2(st[0].cos(), st[0].sin())],
        Stroke::new(1.2, accent),
    );
    let n = 3 + (count * 9.0).round() as usize;
    for i in 0..n {
        let fi = i as f32;
        let a = fi / n as f32 * TAU + (st[1] * 0.3 + fi).sin() * 0.4;
        let rr = r
            * (0.2 + ((st[1] + fi * 1.7).sin() * 0.5 + 0.5) * (0.3 + spread * 0.5) + level * 0.2)
                .clamp(0.0, 1.0);
        glow_dot(
            &g,
            c + rr * vec2(a.cos(), a.sin()),
            1.0 + level * 1.5,
            accent,
        );
    }
    ui.data_mut(|d| d.insert_temp(id, st));
    corner_label(&g, rect, label);
    resp
}

/// Notes the engine played over the last 32 steps: `(absolute step, midi, accent)`.
pub fn piano_roll(
    ui: &mut Ui,
    size: Vec2,
    ch: Channel,
    label: &str,
    notes: &[(u64, i32, bool)],
    now: u64,
) -> Response {
    let (rect, resp, g) = screen(ui, size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    const WIN: u64 = 32;
    let vis: Vec<&(u64, i32, bool)> = notes.iter().filter(|n| n.0 + WIN > now).collect();
    let lo = vis.iter().map(|n| n.1).min().unwrap_or(36).min(36);
    let hi = vis
        .iter()
        .map(|n| n.1 + 1)
        .max()
        .unwrap_or(lo + 12)
        .max(lo + 12);
    let rows = (hi - lo).max(1);
    let rh = rect.height() / rows as f32;
    let cw = rect.width() / WIN as f32;
    for i in 0..rows {
        if [1, 3, 6, 8, 10].contains(&(hi - 1 - i).rem_euclid(12)) {
            g.rect_filled(
                Rect::from_min_size(
                    pos2(rect.left(), rect.top() + i as f32 * rh),
                    vec2(rect.width(), rh),
                ),
                0.0,
                black(0.25),
            );
        }
    }
    for i in 0..(WIN / 4) {
        let x = rect.right() - ((now % 4) + i * 4) as f32 * cw;
        g.line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            Stroke::new(1.0, white(0.05)),
        );
    }
    let accent = ch.color();
    for n in vis {
        let x = rect.right() - (now - n.0 + 1) as f32 * cw;
        let y = rect.top() + (hi - 1 - n.1) as f32 * rh + 0.5;
        let r = Rect::from_min_size(pos2(x, y), vec2(cw * 1.8, (rh - 1.0).max(2.0)));
        glow_rect(&g, r, 1.5, 3.0, alpha(accent, 0.5));
        g.rect_filled(r, 1.5, alpha(accent, if n.2 { 1.0 } else { 0.65 }));
    }
    g.line_segment(
        [
            pos2(rect.right() - cw, rect.top()),
            pos2(rect.right() - cw, rect.bottom()),
        ],
        Stroke::new(1.5, alpha(accent, 0.8)),
    );
    corner_label(&g, rect, label);
    resp
}

/// Draggable envelope curve. Points are `[x, y]` in 0..1; the end points keep
/// their x. `playhead` (0..1) shows the running envelope position.
pub fn curve_editor(
    ui: &mut Ui,
    id: Id,
    pts: &mut [[f32; 2]],
    size: Vec2,
    ch: Channel,
    label: &str,
    playhead: Option<f32>,
) -> bool {
    let (rect, _resp, g) = screen(ui, size, Sense::hover());
    let accent = ch.color();
    let to_screen = |q: &[f32; 2]| {
        pos2(
            rect.left() + q[0] * rect.width(),
            rect.top() + (1.0 - q[1]) * rect.height(),
        )
    };
    for i in 1..8 {
        let x = rect.left() + rect.width() * i as f32 / 8.0;
        g.line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            Stroke::new(1.0, white(0.05)),
        );
    }
    for i in 1..4 {
        let y = rect.top() + rect.height() * i as f32 / 4.0;
        g.line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            Stroke::new(1.0, white(0.05)),
        );
    }
    let mut changed = false;
    let last = pts.len().saturating_sub(1);
    for i in 0..pts.len() {
        let c = to_screen(&pts[i]);
        let resp = ui
            .interact(
                Rect::from_center_size(c, Vec2::splat(16.0)),
                id.with(i),
                Sense::drag(),
            )
            .on_hover_cursor(egui::CursorIcon::Grab);
        if resp.dragged() {
            if let Some(pos) = resp.interact_pointer_pos() {
                let x = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                let y = (1.0 - (pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
                pts[i] = [if i == 0 || i == last { pts[i][0] } else { x }, y];
                changed = true;
            }
        }
    }
    let mut sorted = pts.to_vec();
    sorted.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let line: Vec<Pos2> = sorted.iter().map(to_screen).collect();
    // Filled area under the curve, fading down.
    let mut mesh = Mesh::default();
    for w in line.windows(2) {
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(
            w[0],
            alpha(accent, 0.4 * (1.0 - (w[0].y - rect.top()) / rect.height())),
        );
        mesh.colored_vertex(
            w[1],
            alpha(accent, 0.4 * (1.0 - (w[1].y - rect.top()) / rect.height())),
        );
        mesh.colored_vertex(pos2(w[1].x, rect.bottom()), alpha(accent, 0.0));
        mesh.colored_vertex(pos2(w[0].x, rect.bottom()), alpha(accent, 0.0));
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
    }
    g.add(Shape::mesh(mesh));
    polyline_glow(&g, line, 1.8, accent);
    if let Some(ph) = playhead.filter(|p| (0.0..=1.0).contains(p)) {
        let x = rect.left() + ph * rect.width();
        g.line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            Stroke::new(1.0, white(0.35)),
        );
    }
    for q in pts.iter() {
        let c = to_screen(q);
        glow_dot(&g, c, 4.0, accent);
        g.circle_stroke(c, 4.5, Stroke::new(1.2, Color32::WHITE));
    }
    corner_label(&g, rect, label);
    changed
}

/// Deterministic circuit-trace art for idle VIEW sections.
pub fn circuit_art(ui: &mut Ui, size: Vec2, seed: u32, color: Color32) -> Response {
    let (rect, resp, g) = screen(ui, size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let mut s = seed | 1;
    let cell = 12.0;
    let (cols, rows) = ((rect.width() / cell) as i32, (rect.height() / cell) as i32);
    for _ in 0..((cols * rows) / 9).clamp(4, 60) {
        let mut x = (tiny_noise(&mut s) * cols as f32) as i32;
        let mut y = (tiny_noise(&mut s) * rows as f32) as i32;
        let at = |x: i32, y: i32| {
            pos2(
                rect.left() + (x as f32 + 0.5) * cell,
                rect.top() + (y as f32 + 0.5) * cell,
            )
        };
        let mut pts = vec![at(x, y)];
        for _ in 0..(2 + (tiny_noise(&mut s) * 4.0) as i32) {
            let len = 1 + (tiny_noise(&mut s) * 4.0) as i32;
            if tiny_noise(&mut s) > 0.5 {
                x = (x + if tiny_noise(&mut s) > 0.5 { len } else { -len })
                    .clamp(0, cols.max(1) - 1);
            } else {
                y = (y + if tiny_noise(&mut s) > 0.5 { len } else { -len })
                    .clamp(0, rows.max(1) - 1);
            }
            pts.push(at(x, y));
        }
        let (a, b) = (pts[0], pts[pts.len() - 1]);
        polyline(&g, pts, 1.0, alpha(color, 0.45));
        g.circle_stroke(a, 2.0, Stroke::new(1.0, alpha(color, 0.8)));
        g.circle_filled(b, 1.6, alpha(color, 0.9));
    }
    resp
}

/// Deterministic node-network art with a travelling pulse.
pub fn network_art(ui: &mut Ui, size: Vec2, seed: u32, color: Color32, t: f32) -> Response {
    let (rect, resp, g) = screen(ui, size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let mut s = seed | 1;
    let n = ((rect.width() * rect.height() / 2600.0) as usize).clamp(5, 28);
    let nodes: Vec<Pos2> = (0..n)
        .map(|_| {
            pos2(
                rect.left() + 8.0 + tiny_noise(&mut s) * (rect.width() - 16.0),
                rect.top() + 8.0 + tiny_noise(&mut s) * (rect.height() - 16.0),
            )
        })
        .collect();
    for (i, a) in nodes.iter().enumerate() {
        let mut near: Vec<(f32, usize)> = nodes
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(j, b)| (a.distance(*b), j))
            .collect();
        near.sort_by(|x, y| x.0.total_cmp(&y.0));
        for (k, (_, j)) in near.iter().take(2).enumerate() {
            let b = nodes[*j];
            g.line_segment([*a, b], Stroke::new(1.0, alpha(color, 0.22)));
            let f = (t * 0.35 + i as f32 * 0.37 + k as f32 * 0.5).fract();
            g.circle_filled(*a + (b - *a) * f, 1.2, alpha(color, 0.8));
        }
    }
    for (i, a) in nodes.iter().enumerate() {
        let pulse = 0.5 + 0.5 * (t * 1.3 + i as f32).sin();
        g.circle_filled(*a, 2.0 + pulse, alpha(color, 0.35 + 0.5 * pulse));
    }
    resp
}

/// Bar-per-macro view: `(label, value 0..1)`.
pub fn macro_bars(ui: &mut Ui, size: Vec2, color: Color32, items: &[(&str, f32)]) -> Response {
    let (rect, resp, g) = screen(ui, size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    if items.is_empty() {
        text(
            &g,
            rect.center(),
            Align2::CENTER_CENTER,
            "NO MACROS",
            mono(8.0),
            INK_DIM,
            0.14,
        );
        return resp;
    }
    let n = items.len().min(8);
    let w = rect.width() / n as f32;
    for (i, (label, v)) in items.iter().take(8).enumerate() {
        let col = Rect::from_min_size(
            pos2(rect.left() + i as f32 * w, rect.top()),
            vec2(w, rect.height()),
        );
        let track = Rect::from_min_max(
            pos2(col.center().x - 3.0, col.top() + 8.0),
            pos2(col.center().x + 3.0, col.bottom() - 22.0),
        );
        g.rect_filled(track, 2.0, white(0.05));
        let v = v.clamp(0.0, 1.0);
        let fill = Rect::from_min_max(
            pos2(track.left(), track.bottom() - track.height() * v),
            track.max,
        );
        glow_rect(&g, fill, 2.0, 8.0, alpha(color, 0.5));
        g.rect_filled(fill, 2.0, color);
        text_clipped(
            &g,
            pos2(col.center().x, col.bottom() - 12.0),
            Align2::CENTER_CENTER,
            label,
            display(7.0),
            INK_DIM,
            0.1,
            w - 2.0,
        );
        text(
            &g,
            pos2(col.center().x, col.bottom() - 3.0),
            Align2::CENTER_BOTTOM,
            &format!("{}", (v * 100.0).round()),
            mono(7.0),
            INK_MID,
            0.0,
        );
    }
    resp
}

// ---- algorithmic screens --------------------------------------------------

#[derive(Clone, Default)]
struct LifeState {
    cells: Vec<bool>,
    w: usize,
    h: usize,
    acc: f32,
    seed: u32,
}

/// Conway's Game of Life; `density` reseeds sparse boards and sets the tick rate.
pub fn life_cells(
    ui: &Ui,
    id: Id,
    p: &egui::Painter,
    rect: Rect,
    density: f32,
    color: Color32,
    round: bool,
) {
    let cell = 4.0;
    let (w, h) = (
        ((rect.width() / cell) as usize).max(4),
        ((rect.height() / cell) as usize).max(4),
    );
    let mut st: LifeState = ui.data(|d| d.get_temp(id)).unwrap_or_default();
    if st.seed == 0 {
        st.seed = (id.value() as u32) | 1;
    }
    if st.w != w || st.h != h {
        st.w = w;
        st.h = h;
        st.cells = (0..w * h)
            .map(|_| tiny_noise(&mut st.seed) < density)
            .collect();
    }
    st.acc += ui.input(|i| i.stable_dt).min(0.1) * (2.0 + density * 14.0);
    if st.acc >= 1.0 {
        st.acc = 0.0;
        let mut next = vec![false; w * h];
        let mut alive = 0;
        for y in 0..h {
            for x in 0..w {
                let mut n = 0;
                for (dx, dy) in [
                    (-1i32, -1i32),
                    (0, -1),
                    (1, -1),
                    (-1, 0),
                    (1, 0),
                    (-1, 1),
                    (0, 1),
                    (1, 1),
                ] {
                    let (nx, ny) = (
                        (x as i32 + dx).rem_euclid(w as i32) as usize,
                        (y as i32 + dy).rem_euclid(h as i32) as usize,
                    );
                    n += st.cells[ny * w + nx] as u8;
                }
                let on = matches!((st.cells[y * w + x], n), (true, 2) | (true, 3) | (false, 3));
                next[y * w + x] = on;
                alive += on as usize;
            }
        }
        // Keep the board alive: sprinkle new cells when it thins out.
        if (alive as f32) < (w * h) as f32 * density * 0.25 {
            for c in next.iter_mut() {
                if tiny_noise(&mut st.seed) < density * 0.3 {
                    *c = true;
                }
            }
        }
        st.cells = next;
    }
    let c0 = rect.center();
    let rad = rect.width().min(rect.height()) / 2.0;
    let mut mesh = Mesh::default();
    for y in 0..h {
        for x in 0..w {
            if st.cells[y * w + x] {
                let q = pos2(rect.left() + x as f32 * cell, rect.top() + y as f32 * cell);
                if round && q.distance(c0) > rad - 3.0 {
                    continue;
                }
                mesh.add_colored_rect(Rect::from_min_size(q, Vec2::splat(cell - 1.0)), color);
            }
        }
    }
    p.add(Shape::mesh(mesh));
    ui.data_mut(|d| d.insert_temp(id, st));
}

/// Mandelbrot escape-time field; `depth` sets the iteration count and zoom.
pub fn mandel_cells(p: &egui::Painter, rect: Rect, depth: f32, color: Color32, round: bool) {
    let cell = 3.0;
    let (w, h) = (
        (rect.width() / cell) as usize,
        (rect.height() / cell) as usize,
    );
    let iters = 6 + (depth * 34.0) as u32;
    let zoom = 1.35 - depth * 0.55;
    let c0 = rect.center();
    let rad = rect.width().min(rect.height()) / 2.0;
    let mut mesh = Mesh::default();
    for y in 0..h {
        for x in 0..w {
            let q = pos2(rect.left() + x as f32 * cell, rect.top() + y as f32 * cell);
            if round && q.distance(c0) > rad - 3.0 {
                continue;
            }
            let cx = (x as f32 / w as f32 - 0.5) * 3.0 * zoom - 0.6;
            let cy = (y as f32 / h as f32 - 0.5) * 3.0 * zoom;
            let (mut zx, mut zy, mut i) = (0.0f32, 0.0f32, 0);
            while i < iters && zx * zx + zy * zy < 4.0 {
                let t = zx * zx - zy * zy + cx;
                zy = 2.0 * zx * zy + cy;
                zx = t;
                i += 1;
            }
            if i < iters && i > 1 {
                mesh.add_colored_rect(
                    Rect::from_min_size(q, Vec2::splat(cell)),
                    alpha(color, (i as f32 / iters as f32).powf(0.7)),
                );
            }
        }
    }
    p.add(Shape::mesh(mesh));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RimScreen {
    Life,
    Mandel,
}

/// Large knob whose face is a live algorithmic screen and whose rim is the
/// value arc ("heat" runs amber → red, "cool" stays cyan).
pub fn rim_knob(
    ui: &mut Ui,
    id: Id,
    value: &mut f32,
    size: f32,
    hot_rim: bool,
    label: &str,
    screen_kind: RimScreen,
    default: f32,
) -> Response {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(size, size + 16.0), Sense::click_and_drag());
    if resp.dragged() {
        let fine = ui.input(|i| i.modifiers.shift);
        *value = (*value - resp.drag_delta().y / if fine { 800.0 } else { 200.0 }).clamp(0.0, 1.0);
        resp.mark_changed();
    }
    if resp.double_clicked() {
        *value = default;
        resp.mark_changed();
    }
    if resp.hovered() || resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
    }
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let p = ui.painter();
    let c = pos2(rect.center().x, rect.top() + size / 2.0);
    let r = size / 2.0;
    let rim = if hot_rim {
        mix(SIGNAL_HOT, SIGNAL_AMBER, *value)
    } else {
        SIGNAL_COOL
    };
    p.circle_filled(c + vec2(0.0, 3.0), r, black(0.5));
    radial(p, c, r, rgb(0x2a2f3a), rgb(0x0e1117), vec2(-0.2, -0.3));
    p.circle_stroke(c, r, Stroke::new(1.0, black(0.7)));
    arc(p, c, r - 5.0, -135.0, 135.0, 3.0, white(0.07));
    arc_glow(
        p,
        c,
        r - 5.0,
        -135.0,
        -135.0 + 270.0 * value.clamp(0.0, 1.0),
        3.0,
        rim,
    );
    let face = r - 11.0;
    p.circle_filled(c, face, rgb(0x04060b));
    let inner = Rect::from_center_size(c, Vec2::splat(face * 2.0));
    let g = p.with_clip_rect(inner.intersect(p.clip_rect()));
    match screen_kind {
        RimScreen::Life => life_cells(ui, id, &g, inner, 0.1 + *value * 0.5, rim, true),
        RimScreen::Mandel => mandel_cells(&g, inner, *value, rim, true),
    }
    p.circle_stroke(c, face, Stroke::new(1.5, black(0.8)));
    engrave(
        p,
        pos2(rect.center().x, rect.bottom() - 10.0),
        Align2::CENTER_TOP,
        label,
        8.0,
        ETCH,
    );
    resp
}
