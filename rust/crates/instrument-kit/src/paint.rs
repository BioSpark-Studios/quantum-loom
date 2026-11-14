//! Drawing primitives the materials and controls are built from: gradients,
//! glows, arcs, brushed metal, screws and tracked text.

use crate::theme::*;
use egui::epaint::{Mesh, PathShape, PathStroke, Shadow};
use egui::text::{LayoutJob, TextFormat};
use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Shape, Stroke,
    StrokeKind, Vec2,
};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
        l(a.a(), b.a()),
    )
}

/// Outline of a rounded rectangle, clockwise from the top-left corner.
fn rounded_outline(rect: Rect, r: CornerRadius) -> Vec<Pos2> {
    let max = rect.width().min(rect.height()) / 2.0;
    let corner = |pts: &mut Vec<Pos2>, c: Pos2, rad: f32, start: f32| {
        if rad <= 0.5 {
            pts.push(c);
            return;
        }
        let n = ((rad / 2.0).ceil() as usize).clamp(2, 8);
        for i in 0..=n {
            let a = start + FRAC_PI_2 * i as f32 / n as f32;
            pts.push(c + rad * vec2(a.cos(), a.sin()));
        }
    };
    let mut pts = Vec::with_capacity(36);
    let (nw, ne, se, sw) = (
        (r.nw as f32).min(max),
        (r.ne as f32).min(max),
        (r.se as f32).min(max),
        (r.sw as f32).min(max),
    );
    corner(&mut pts, rect.left_top() + vec2(nw, nw), nw, PI);
    corner(&mut pts, rect.right_top() + vec2(-ne, ne), ne, -FRAC_PI_2);
    corner(&mut pts, rect.right_bottom() + vec2(-se, -se), se, 0.0);
    corner(&mut pts, rect.left_bottom() + vec2(sw, -sw), sw, FRAC_PI_2);
    pts
}

/// Two-stop linear gradient in a rounded rectangle. `vertical` runs top → bottom,
/// otherwise left → right.
pub fn grad(
    p: &Painter,
    rect: Rect,
    radius: impl Into<CornerRadius>,
    a: Color32,
    b: Color32,
    vertical: bool,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    let at = |q: Pos2| {
        let t = if vertical {
            (q.y - rect.top()) / rect.height()
        } else {
            (q.x - rect.left()) / rect.width()
        };
        lerp_color(a, b, t)
    };
    let pts = rounded_outline(rect, radius.into());
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.center(), at(rect.center()));
    for q in &pts {
        mesh.colored_vertex(*q, at(*q));
    }
    let n = pts.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    p.add(Shape::mesh(mesh));
}

pub fn grad_v(p: &Painter, rect: Rect, radius: f32, top: Color32, bottom: Color32) {
    grad(p, rect, CornerRadius::same(radius as u8), top, bottom, true);
}

pub fn grad_h(p: &Painter, rect: Rect, radius: f32, left: Color32, right: Color32) {
    grad(
        p,
        rect,
        CornerRadius::same(radius as u8),
        left,
        right,
        false,
    );
}

/// Multi-stop gradient: `stops` are `(position 0..1, colour)` in order.
pub fn grad_stops(p: &Painter, rect: Rect, radius: f32, stops: &[(f32, Color32)], vertical: bool) {
    let r = radius as u8;
    for (i, w) in stops.windows(2).enumerate() {
        let (first, last) = (i == 0, i == stops.len() - 2);
        let (band, cr) = if vertical {
            let band = Rect::from_min_max(
                pos2(rect.left(), rect.top() + rect.height() * w[0].0),
                pos2(rect.right(), rect.top() + rect.height() * w[1].0),
            );
            (
                band,
                CornerRadius {
                    nw: if first { r } else { 0 },
                    ne: if first { r } else { 0 },
                    sw: if last { r } else { 0 },
                    se: if last { r } else { 0 },
                },
            )
        } else {
            let band = Rect::from_min_max(
                pos2(rect.left() + rect.width() * w[0].0, rect.top()),
                pos2(rect.left() + rect.width() * w[1].0, rect.bottom()),
            );
            (
                band,
                CornerRadius {
                    nw: if first { r } else { 0 },
                    sw: if first { r } else { 0 },
                    ne: if last { r } else { 0 },
                    se: if last { r } else { 0 },
                },
            )
        };
        grad(p, band, cr, w[0].1, w[1].1, vertical);
    }
}

/// Radial gradient disc; `focus` offsets the bright centre (−1..1 of the radius).
pub fn radial(p: &Painter, c: Pos2, r: f32, inner: Color32, outer: Color32, focus: Vec2) {
    if r <= 0.0 {
        return;
    }
    let n = ((r * 1.2) as u32).clamp(12, 48);
    let mut mesh = Mesh::default();
    mesh.colored_vertex(c + focus * r, inner);
    for i in 0..n {
        let a = TAU * i as f32 / n as f32;
        mesh.colored_vertex(c + r * vec2(a.cos(), a.sin()), outer);
    }
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    p.add(Shape::mesh(mesh));
}

/// Soft drop shadow under a rounded rect.
pub fn shadow(p: &Painter, rect: Rect, radius: f32, offset_y: f32, blur: f32, color: Color32) {
    let s = Shadow {
        offset: [0, offset_y.round() as i8],
        blur: blur.round() as u8,
        spread: 0,
        color,
    };
    p.add(s.as_shape(rect, CornerRadius::same(radius as u8)));
}

/// Halo around a rounded rect (`box-shadow: 0 0 blur color`).
pub fn glow_rect(p: &Painter, rect: Rect, radius: f32, blur: f32, color: Color32) {
    shadow(p, rect, radius, 0.0, blur, color);
}

/// Lit dot with a halo — LEDs, pips, cursors.
pub fn glow_dot(p: &Painter, c: Pos2, r: f32, color: Color32) {
    radial(
        p,
        c,
        r * 3.2,
        alpha(color, 0.38),
        alpha(color, 0.0),
        Vec2::ZERO,
    );
    p.circle_filled(c, r, color);
}

/// Angle convention for arcs and knobs: degrees clockwise from 12 o'clock.
pub fn polar(c: Pos2, r: f32, deg: f32) -> Pos2 {
    let a = (deg - 90.0).to_radians();
    c + r * vec2(a.cos(), a.sin())
}

pub fn arc_points(c: Pos2, r: f32, d0: f32, d1: f32) -> Vec<Pos2> {
    let (lo, hi) = (d0.min(d1), d0.max(d1));
    let n = (((hi - lo).abs() / 6.0).ceil() as usize).max(2);
    (0..=n)
        .map(|i| polar(c, r, lo + (hi - lo) * i as f32 / n as f32))
        .collect()
}

/// Stroked arc with round caps.
pub fn arc(p: &Painter, c: Pos2, r: f32, d0: f32, d1: f32, width: f32, color: Color32) {
    if (d1 - d0).abs() < 0.5 {
        return;
    }
    let pts = arc_points(c, r, d0, d1);
    let (a, b) = (pts[0], pts[pts.len() - 1]);
    p.add(Shape::Path(PathShape::line(
        pts,
        PathStroke::new(width, color),
    )));
    p.circle_filled(a, width / 2.0, color);
    p.circle_filled(b, width / 2.0, color);
}

/// Arc with a neon halo underneath.
pub fn arc_glow(p: &Painter, c: Pos2, r: f32, d0: f32, d1: f32, width: f32, color: Color32) {
    if (d1 - d0).abs() < 0.5 {
        return;
    }
    p.add(Shape::Path(PathShape::line(
        arc_points(c, r, d0, d1),
        PathStroke::new(width + 5.0, alpha(color, 0.16)),
    )));
    p.add(Shape::Path(PathShape::line(
        arc_points(c, r, d0, d1),
        PathStroke::new(width + 2.0, alpha(color, 0.28)),
    )));
    arc(p, c, r, d0, d1, width, color);
}

pub fn polyline(p: &Painter, pts: Vec<Pos2>, width: f32, color: Color32) {
    if pts.len() >= 2 {
        p.add(Shape::Path(PathShape::line(
            pts,
            PathStroke::new(width, color),
        )));
    }
}

/// Polyline with a halo, for scope traces and curves.
pub fn polyline_glow(p: &Painter, pts: Vec<Pos2>, width: f32, color: Color32) {
    if pts.len() < 2 {
        return;
    }
    p.add(Shape::Path(PathShape::line(
        pts.clone(),
        PathStroke::new(width + 4.0, alpha(color, 0.18)),
    )));
    p.add(Shape::Path(PathShape::line(
        pts,
        PathStroke::new(width, color),
    )));
}

/// Brushed-metal grain: one faint vertical line every 3 points.
pub fn brushed(p: &Painter, rect: Rect, strength: f32) {
    let c = white(strength);
    let mut mesh = Mesh::default();
    let mut x = rect.left();
    while x < rect.right() {
        mesh.add_colored_rect(
            Rect::from_min_max(
                pos2(x, rect.top()),
                pos2((x + 1.0).min(rect.right()), rect.bottom()),
            ),
            c,
        );
        x += 3.0;
    }
    p.add(Shape::mesh(mesh));
}

/// 1 px top highlight and bottom shade inside a rect (`inset 0 1px …`).
pub fn bevel(p: &Painter, rect: Rect, radius: f32, hi: f32, lo: f32) {
    let x0 = rect.left() + radius * 0.6;
    let x1 = rect.right() - radius * 0.6;
    if hi > 0.0 {
        p.line_segment(
            [pos2(x0, rect.top() + 0.5), pos2(x1, rect.top() + 0.5)],
            Stroke::new(1.0, white(hi)),
        );
    }
    if lo > 0.0 {
        p.line_segment(
            [pos2(x0, rect.bottom() - 0.5), pos2(x1, rect.bottom() - 0.5)],
            Stroke::new(1.0, black(lo)),
        );
    }
}

pub fn outline(p: &Painter, rect: Rect, radius: f32, color: Color32) {
    p.rect_stroke(
        rect,
        CornerRadius::same(radius as u8),
        Stroke::new(1.0, color),
        StrokeKind::Inside,
    );
}

pub fn outline_outside(p: &Painter, rect: Rect, radius: f32, color: Color32) {
    p.rect_stroke(
        rect,
        CornerRadius::same(radius as u8),
        Stroke::new(1.0, color),
        StrokeKind::Outside,
    );
}

/// Dashed rounded outline, for drop targets and empty slots.
pub fn dashed(p: &Painter, rect: Rect, color: Color32) {
    let c = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
        rect.left_top(),
    ];
    p.add(Shape::dashed_line(&c, Stroke::new(1.0, color), 4.0, 3.0));
}

// ---- materials ------------------------------------------------------------

/// `.mat-panel` — raised brushed-metal module face.
pub fn mat_panel(p: &Painter, rect: Rect, radius: f32) {
    shadow(p, rect, radius, 6.0, 16.0, black(0.5));
    grad_stops(
        p,
        rect,
        radius,
        &[
            (0.0, rgb(0x1f242e)),
            (0.5, rgb(0x14181f)),
            (1.0, rgb(0x0f131a)),
        ],
        true,
    );
    bevel(p, rect, radius, 0.07, 0.6);
    outline(p, rect, radius, black(0.4));
}

/// `.mat-cavity` — recessed well that holds screens and jacks.
pub fn mat_cavity(p: &Painter, rect: Rect, radius: f32) {
    grad_v(p, rect, radius, CHASSIS_LO, rgb(0x060810));
    inner_shade(p, rect, radius, 0.55);
    outline(p, rect, radius, black(0.6));
    p.line_segment(
        [
            pos2(rect.left() + radius, rect.bottom() - 0.5),
            pos2(rect.right() - radius, rect.bottom() - 0.5),
        ],
        Stroke::new(1.0, white(0.05)),
    );
}

/// `.mat-screen` — glass display. Returns a painter clipped to the glass.
pub fn mat_screen(p: &Painter, rect: Rect, radius: f32) -> Painter {
    grad_v(p, rect, radius, rgb(0x060a14), rgb(0x030509));
    inner_shade(p, rect, radius, 0.6);
    outline(p, rect, radius, black(0.6));
    p.with_clip_rect(rect.shrink(1.0).intersect(p.clip_rect()))
}

/// Darkening at the top edge of a recessed surface (`inset 0 2px 8px #000`).
pub fn inner_shade(p: &Painter, rect: Rect, radius: f32, strength: f32) {
    let h = (rect.height() * 0.35).min(8.0);
    if h < 1.0 {
        return;
    }
    let band = Rect::from_min_size(rect.min, vec2(rect.width(), h));
    let r = radius as u8;
    grad(
        p,
        band,
        CornerRadius {
            nw: r,
            ne: r,
            sw: 0,
            se: 0,
        },
        black(strength),
        black(0.0),
        true,
    );
}

/// M2 screw head with a slot.
pub fn screw(p: &Painter, c: Pos2, d: f32) {
    let r = d / 2.0;
    radial(p, c, r, rgb(0x4a5060), rgb(0x15181f), vec2(-0.3, -0.3));
    p.circle_stroke(c, r, Stroke::new(1.0, black(0.5)));
    let k = r * 0.62;
    p.line_segment(
        [c + vec2(-k, k), c + vec2(k, -k)],
        Stroke::new(1.0, rgb(0x0a0c10)),
    );
}

/// Oval rack-ear mounting hole.
pub fn rack_hole(p: &Painter, c: Pos2) {
    let r = Rect::from_center_size(c, vec2(9.0, 5.0));
    p.rect_filled(r, 3.0, rgb(0x05070a));
    p.line_segment(
        [
            pos2(r.left() + 2.0, r.bottom() + 0.5),
            pos2(r.right() - 2.0, r.bottom() + 0.5),
        ],
        Stroke::new(1.0, white(0.12)),
    );
}

// ---- text -----------------------------------------------------------------

pub fn job(text: &str, font: FontId, color: Color32, tracking_em: f32) -> LayoutJob {
    let size = font.size;
    let mut j = LayoutJob::default();
    j.append(
        text,
        0.0,
        TextFormat {
            font_id: font,
            color,
            extra_letter_spacing: tracking_em * size,
            ..Default::default()
        },
    );
    j.wrap.max_width = f32::INFINITY;
    j
}

pub fn measure(p: &Painter, text: &str, font: FontId, tracking_em: f32) -> Vec2 {
    p.ctx()
        .fonts_mut(|f| f.layout_job(job(text, font, Color32::WHITE, tracking_em)))
        .size()
}

/// Draw tracked text anchored at `pos`. Returns the text rectangle.
pub fn text(
    p: &Painter,
    pos: Pos2,
    anchor: Align2,
    s: &str,
    font: FontId,
    color: Color32,
    tracking_em: f32,
) -> Rect {
    let galley = p
        .ctx()
        .fonts_mut(|f| f.layout_job(job(s, font, color, tracking_em)));
    // Tracking adds a trailing gap after the last glyph; ignore it when centring.
    let trail = tracking_em
        * galley
            .job
            .sections
            .first()
            .map(|s| s.format.font_id.size)
            .unwrap_or(0.0);
    let size = galley.size() - vec2(trail.max(0.0), 0.0);
    let rect = anchor.anchor_size(pos, size);
    p.galley(rect.min, galley, color);
    rect
}

/// Text clipped to `max_width` with a trailing ellipsis.
pub fn text_clipped(
    p: &Painter,
    pos: Pos2,
    anchor: Align2,
    s: &str,
    font: FontId,
    color: Color32,
    tracking_em: f32,
    max_width: f32,
) -> Rect {
    let mut j = job(s, font, color, tracking_em);
    j.wrap.max_width = max_width.max(1.0);
    j.wrap.max_rows = 1;
    j.wrap.break_anywhere = true;
    let galley = p.ctx().fonts_mut(|f| f.layout_job(j));
    let rect = anchor.anchor_size(pos, galley.size());
    p.galley(rect.min, galley, color);
    rect
}

/// Wrapped body text; returns the height used.
pub fn paragraph(p: &Painter, pos: Pos2, s: &str, font: FontId, color: Color32, width: f32) -> f32 {
    let mut j = job(s, font, color, 0.0);
    j.wrap.max_width = width.max(1.0);
    let galley = p.ctx().fonts_mut(|f| f.layout_job(j));
    let h = galley.size().y;
    p.galley(pos, galley, color);
    h
}

/// `.engrave` — uppercase etched chassis label.
pub fn engrave(p: &Painter, pos: Pos2, anchor: Align2, s: &str, size: f32, color: Color32) -> Rect {
    text(
        p,
        pos + vec2(0.0, 1.0),
        anchor,
        s,
        display(size),
        black(0.6),
        0.18,
    );
    text(p, pos, anchor, s, display(size), color, 0.18)
}

/// `.readout` — glowing mono value.
pub fn readout(p: &Painter, pos: Pos2, anchor: Align2, s: &str, size: f32, color: Color32) -> Rect {
    text(p, pos, anchor, s, mono(size), color, 0.06)
}
