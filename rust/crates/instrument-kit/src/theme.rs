//! Design tokens. Names mirror `tokens.css` in the design project so the
//! design document stays the single source of truth across stacks.

use egui::{Color32, FontFamily, FontId};
use std::sync::Arc;

pub const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// White at `a` (0..1) alpha — the hairlines and highlights of the system.
pub fn white(a: f32) -> Color32 {
    Color32::from_white_alpha((a * 255.0).round() as u8)
}

pub fn black(a: f32) -> Color32 {
    Color32::from_black_alpha((a * 255.0).round() as u8)
}

/// `c` at `a` (0..1) opacity.
pub fn alpha(c: Color32, a: f32) -> Color32 {
    c.gamma_multiply(a.clamp(0.0, 1.0))
}

/// CSS `color-mix(in srgb, a t%, b)`.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 * t + y as f32 * (1.0 - t)).round() as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

pub fn hex(s: &str) -> Color32 {
    let s = s.trim_start_matches('#');
    match u32::from_str_radix(s, 16) {
        Ok(v) if s.len() == 6 => rgb(v),
        _ => SIGNAL_COOL,
    }
}

// --- chassis / substrate ---
pub const VOID: Color32 = rgb(0x05060a);
pub const ABYSS: Color32 = rgb(0x0a0c12);
pub const PITCH: Color32 = rgb(0x0e1117);
pub const CHASSIS: Color32 = rgb(0x14181f);
pub const CHASSIS_HI: Color32 = rgb(0x1a1f28);
pub const CHASSIS_LO: Color32 = rgb(0x0a0d12);
pub const BEZEL: Color32 = rgb(0x2a2f3a);
pub const BEZEL_HI: Color32 = rgb(0x404654);
pub const BEZEL_LO: Color32 = rgb(0x1a1d24);
pub const SCREEN_BG: Color32 = rgb(0x050810);
/// FORGE shell background.
pub const SHELL: Color32 = rgb(0x03050a);

// --- ink ---
pub const INK: Color32 = rgb(0xe6ebf2);
pub const INK_MID: Color32 = rgb(0x8e96a8);
pub const INK_DIM: Color32 = rgb(0x4f5566);
pub const INK_FAINT: Color32 = rgb(0x2a2e38);
pub const ETCH: Color32 = rgb(0x6b7388);

// --- signal colours ---
pub const SIGNAL_HOT: Color32 = rgb(0xff5a3c);
pub const SIGNAL_WARM: Color32 = rgb(0xff9028);
pub const SIGNAL_AMBER: Color32 = rgb(0xffb83a);
pub const SIGNAL_COOL: Color32 = rgb(0x2cd1ff);
pub const SIGNAL_LIFE: Color32 = rgb(0x00e070);
pub const SIGNAL_MYTH: Color32 = rgb(0xb070ff);
pub const SIGNAL_BONE: Color32 = rgb(0xf4f6ff);
pub const SIGNAL_ROSE: Color32 = rgb(0xff4488);
pub const SIGNAL_FORGE: Color32 = rgb(0xff6400);
/// FORGE brand ink (wordmark, active app).
pub const FORGE_INK: Color32 = rgb(0xffb27a);
/// Shell accent for active stages and rails.
pub const GOLD: Color32 = rgb(0xfbbf24);

/// The sixteen department hues, in registry order (Atlas … Nexus).
pub const DEPT_HUES: [Color32; 16] = [
    rgb(0x1e8cff),
    rgb(0x8050e0),
    rgb(0x64b4ff),
    rgb(0xe0d8ff),
    rgb(0xf4c025),
    rgb(0xdc3c78),
    rgb(0x9030d0),
    rgb(0xc8a860),
    rgb(0xb08030),
    rgb(0x8c50ff),
    rgb(0x00c060),
    rgb(0xdc8c1e),
    rgb(0x20c8d0),
    rgb(0x30e060),
    rgb(0xff6400),
    rgb(0xf4f6ff),
];

/// Functional signal channel a control inherits from its module.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Channel {
    #[default]
    Cool,
    Life,
    Myth,
    Amber,
    Warm,
    Hot,
    Rose,
    Bone,
    Forge,
}

impl Channel {
    pub const ALL: [Channel; 9] = [
        Channel::Cool,
        Channel::Life,
        Channel::Myth,
        Channel::Amber,
        Channel::Warm,
        Channel::Hot,
        Channel::Rose,
        Channel::Bone,
        Channel::Forge,
    ];

    pub fn from_key(key: &str) -> Self {
        match key {
            "life" => Channel::Life,
            "myth" => Channel::Myth,
            "amber" => Channel::Amber,
            "warm" => Channel::Warm,
            "hot" => Channel::Hot,
            "rose" => Channel::Rose,
            "bone" => Channel::Bone,
            "forge" => Channel::Forge,
            _ => Channel::Cool,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Channel::Cool => "cool",
            Channel::Life => "life",
            Channel::Myth => "myth",
            Channel::Amber => "amber",
            Channel::Warm => "warm",
            Channel::Hot => "hot",
            Channel::Rose => "rose",
            Channel::Bone => "bone",
            Channel::Forge => "forge",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            Channel::Cool => SIGNAL_COOL,
            Channel::Life => SIGNAL_LIFE,
            Channel::Myth => SIGNAL_MYTH,
            Channel::Amber => SIGNAL_AMBER,
            Channel::Warm => SIGNAL_WARM,
            Channel::Hot => SIGNAL_HOT,
            Channel::Rose => SIGNAL_ROSE,
            Channel::Bone => SIGNAL_BONE,
            Channel::Forge => SIGNAL_FORGE,
        }
    }
}

// --- radii ---
pub const R1: f32 = 2.0;
pub const R2: f32 = 4.0;
pub const R3: f32 = 6.0;
pub const R4: f32 = 8.0;
pub const R5: f32 = 12.0;

/// Eurorack HP unit in points.
pub const HP: f32 = 16.0;

// --- type ---
pub const F_DISPLAY: &str = "display";
pub const F_MONO: &str = "mono";
pub const F_TEXT: &str = "text";
pub const F_TEXT_SEMI: &str = "text-semi";
pub const F_SERIF: &str = "serif-display";

fn fam(name: &'static str) -> FontFamily {
    FontFamily::Name(Arc::from(name))
}

/// Orbitron — module names, hero marks, engraved chassis labels.
pub fn display(size: f32) -> FontId {
    FontId::new(size, fam(F_DISPLAY))
}

/// JetBrains Mono — values, readouts, ids.
pub fn mono(size: f32) -> FontId {
    FontId::new(size, fam(F_MONO))
}

/// Inter — body copy and hints.
pub fn body(size: f32) -> FontId {
    FontId::new(size, fam(F_TEXT))
}

pub fn body_semi(size: f32) -> FontId {
    FontId::new(size, fam(F_TEXT_SEMI))
}

/// Cinzel — the FORGE wordmark and vault titles.
pub fn serif(size: f32) -> FontId {
    FontId::new(size, fam(F_SERIF))
}

/// Install the bundled faces and a dark instrument-panel style.
pub fn install(ctx: &egui::Context) {
    use egui::{FontData, FontDefinitions};
    let mut fonts = FontDefinitions::default();
    let faces: [(&str, &'static [u8]); 5] = [
        (
            F_DISPLAY,
            include_bytes!("../assets/fonts/Orbitron-Bold.ttf"),
        ),
        (
            F_MONO,
            include_bytes!("../assets/fonts/JetBrainsMono-SemiBold.ttf"),
        ),
        (F_TEXT, include_bytes!("../assets/fonts/Inter-Regular.ttf")),
        (
            F_TEXT_SEMI,
            include_bytes!("../assets/fonts/Inter-SemiBold.ttf"),
        ),
        (F_SERIF, include_bytes!("../assets/fonts/Cinzel-Bold.ttf")),
    ];
    // egui's built-in faces stay as fallbacks so symbols and emoji still resolve.
    let fallback: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for (name, bytes) in faces {
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
        let mut chain = vec![name.to_owned()];
        chain.extend(fallback.iter().cloned());
        fonts.families.insert(fam(name), chain);
    }
    if let Some(p) = fonts.families.get_mut(&FontFamily::Proportional) {
        p.insert(0, F_TEXT.to_owned());
    }
    if let Some(m) = fonts.families.get_mut(&FontFamily::Monospace) {
        m.insert(0, F_MONO.to_owned());
    }
    ctx.set_fonts(fonts);

    ctx.global_style_mut(|style| {
        let v = &mut style.visuals;
        *v = egui::Visuals::dark();
        v.panel_fill = SHELL;
        v.window_fill = rgb(0x0d1118);
        v.extreme_bg_color = rgb(0x05070a);
        v.override_text_color = Some(INK);
        v.selection.bg_fill = alpha(SIGNAL_COOL, 0.35);
        v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, white(0.08));
        v.window_stroke = egui::Stroke::new(1.0, white(0.1));
        v.window_corner_radius = egui::CornerRadius::same(8);
        v.popup_shadow = egui::Shadow {
            offset: [0, 14],
            blur: 40,
            spread: 0,
            color: black(0.7),
        };
        style.spacing.item_spacing = egui::vec2(6.0, 6.0);
        style.spacing.scroll = egui::style::ScrollStyle::thin();
        style.interaction.selectable_labels = false;
    });
}
