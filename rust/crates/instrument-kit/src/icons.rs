//! Vector icon set: shell icons, glyph icons and the department crests.
//!
//! The SVG sources are the design project's own (exported to `assets/icons.json`).
//! Each is rasterised on demand at the exact pixel size it is drawn at, in
//! white, and tinted when painted — so strokes stay crisp at every size.

use egui::{pos2, vec2, Color32, Context, Id, Painter, Pos2, Rect, TextureHandle, TextureOptions};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const SOURCES: &str = include_str!("../assets/icons.json");

pub struct Icons {
    svg: HashMap<String, String>,
    cache: Mutex<HashMap<(String, u32, u32), Option<TextureHandle>>>,
}

impl Icons {
    fn new() -> Self {
        let svg: HashMap<String, String> = serde_json::from_str(SOURCES).unwrap_or_default();
        Self {
            svg,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// The shared icon store for this egui context.
    pub fn of(ctx: &Context) -> Arc<Icons> {
        let id = Id::unique("instrument-kit.icons");
        if let Some(i) = ctx.data(|d| d.get_temp::<Arc<Icons>>(id)) {
            return i;
        }
        let i = Arc::new(Icons::new());
        ctx.data_mut(|d| d.insert_temp(id, i.clone()));
        i
    }

    pub fn has(&self, name: &str) -> bool {
        self.svg.contains_key(name)
    }

    pub fn names(&self) -> Vec<&str> {
        let mut n: Vec<&str> = self.svg.keys().map(String::as_str).collect();
        n.sort_unstable();
        n
    }

    fn texture(
        &self,
        ctx: &Context,
        name: &str,
        px: u32,
        stroke_px: Option<f32>,
    ) -> Option<TextureHandle> {
        let key = (
            name.to_owned(),
            px,
            stroke_px.map(|s| (s * 100.0) as u32).unwrap_or(0),
        );
        let mut cache = self.cache.lock().ok()?;
        if let Some(t) = cache.get(&key) {
            return t.clone();
        }
        let tex = self
            .svg
            .get(name)
            .and_then(|src| rasterize(src, px, stroke_px))
            .map(|img| ctx.load_texture(format!("icon:{name}:{px}"), img, TextureOptions::LINEAR));
        cache.insert(key, tex.clone());
        tex
    }
}

fn rasterize(src: &str, px: u32, stroke_px: Option<f32>) -> Option<egui::ColorImage> {
    use resvg::{tiny_skia, usvg};
    let px = px.clamp(4, 512);
    let mut svg = src
        .replace("currentColor", "#ffffff")
        .replace("var(--chassis)", "#000000");
    if let Some(sw) = stroke_px {
        // Non-scaling stroke: keep the mark the same weight at every size.
        let w = sw * 24.0 / px as f32;
        if let Some(i) = svg.find("stroke-width=\"") {
            let start = i + "stroke-width=\"".len();
            if let Some(len) = svg[start..].find('"') {
                svg.replace_range(start..start + len, &format!("{w:.3}"));
            }
        } else {
            svg = svg.replacen("<svg ", &format!("<svg stroke-width=\"{w:.3}\" "), 1);
        }
    }
    if !svg.contains("xmlns=") {
        svg = svg.replacen("<svg ", "<svg xmlns=\"http://www.w3.org/2000/svg\" ", 1);
    }
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(px, px)?;
    let s = px as f32 / 24.0;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(s, s),
        &mut pixmap.as_mut(),
    );
    Some(egui::ColorImage::from_rgba_premultiplied(
        [px as usize, px as usize],
        pixmap.data(),
    ))
}

/// Paint icon `name` centred at `c`, `size` points square, tinted `color`.
pub fn paint(p: &Painter, name: &str, c: Pos2, size: f32, color: Color32) {
    paint_sw(p, name, c, size, color, None);
}

/// Like [`paint`], with a fixed stroke weight in points.
pub fn paint_sw(p: &Painter, name: &str, c: Pos2, size: f32, color: Color32, stroke: Option<f32>) {
    let ctx = p.ctx();
    let ppp = ctx.pixels_per_point();
    let px = (size * ppp).round().max(4.0) as u32;
    let icons = Icons::of(ctx);
    let key = if icons.has(name) { name } else { "node" };
    if let Some(tex) = icons.texture(ctx, key, px, stroke.map(|s| s * ppp)) {
        // Snap to the pixel grid so the 1:1 texture is not resampled.
        let half = px as f32 / ppp / 2.0;
        let min = pos2(
            ((c.x - half) * ppp).round() / ppp,
            ((c.y - half) * ppp).round() / ppp,
        );
        let rect = Rect::from_min_size(min, vec2(half * 2.0, half * 2.0));
        p.image(
            tex.id(),
            rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_rasterizes() {
        let icons = Icons::new();
        assert!(icons.svg.len() >= 90, "{} icons", icons.svg.len());
        for (name, src) in &icons.svg {
            let img = rasterize(src, 24, Some(1.4))
                .unwrap_or_else(|| panic!("{name} failed to rasterize"));
            assert!(img.pixels.iter().any(|c| c.a() > 0), "{name} is blank");
        }
    }

    #[test]
    fn crests_exist_for_every_department() {
        let icons = Icons::new();
        for c in [
            "atlas",
            "mythos",
            "architect",
            "prism",
            "animus",
            "loom",
            "instinct",
            "order",
            "chronicle",
            "quill",
            "codex",
            "composer",
            "axiom",
            "continuum",
            "forge",
            "nexus",
            "aetherforge",
        ] {
            assert!(icons.has(c), "{c}");
        }
    }
}
