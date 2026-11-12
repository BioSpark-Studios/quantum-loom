//! CPU reference for the compositor's blend maths. The GPU output is checked
//! against this, so it must follow the same formulas as `composite.wgsl`.

pub use stencil_core::manifest::BlendMode;

/// Index the shader switches on.
pub fn mode_index(m: BlendMode) -> u32 {
    match m {
        BlendMode::Normal => 0,
        BlendMode::Multiply => 1,
        BlendMode::Screen => 2,
        BlendMode::Overlay => 3,
        BlendMode::Dodge => 4,
        BlendMode::Burn => 5,
    }
}

fn screen(cb: f32, cs: f32) -> f32 {
    cb + cs - cb * cs
}

fn hard_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        cb * 2.0 * cs
    } else {
        screen(cb, 2.0 * cs - 1.0)
    }
}

/// Per-channel blend function B(Cb, Cs).
pub fn blend_channel(mode: BlendMode, cb: f32, cs: f32) -> f32 {
    match mode {
        BlendMode::Normal => cs,
        BlendMode::Multiply => cb * cs,
        BlendMode::Screen => screen(cb, cs),
        // overlay is hard-light with the layers swapped
        BlendMode::Overlay => hard_light(cs, cb),
        BlendMode::Dodge => {
            if cb == 0.0 {
                0.0
            } else if cs >= 1.0 {
                1.0
            } else {
                (cb / (1.0 - cs)).min(1.0)
            }
        }
        BlendMode::Burn => {
            if cb >= 1.0 {
                1.0
            } else if cs <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - cb) / cs).min(1.0)
            }
        }
    }
}

/// Composite `src` (straight RGBA) over `dst` with `mode` and layer `opacity`.
pub fn composite(dst: [f32; 4], src: [f32; 4], mode: BlendMode, opacity: f32) -> [f32; 4] {
    let (ab, a_s) = (dst[3], src[3] * opacity.clamp(0.0, 1.0));
    let ao = a_s + ab * (1.0 - a_s);
    if ao <= 0.0 {
        return [0.0; 4];
    }
    let mut out = [0.0, 0.0, 0.0, ao];
    for c in 0..3 {
        let (cb, cs) = (dst[c], src[c]);
        let b = blend_channel(mode, cb, cs);
        out[c] = (a_s * (1.0 - ab) * cs + a_s * ab * b + (1.0 - a_s) * ab * cb) / ao;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use BlendMode::*;

    #[test]
    fn textbook_values() {
        assert_eq!(blend_channel(Multiply, 0.5, 0.5), 0.25);
        assert_eq!(blend_channel(Screen, 0.5, 0.5), 0.75);
        assert_eq!(blend_channel(Normal, 0.2, 0.9), 0.9);
        assert_eq!(blend_channel(Dodge, 0.25, 0.5), 0.5);
        assert_eq!(blend_channel(Dodge, 0.0, 0.5), 0.0);
        assert_eq!(blend_channel(Dodge, 0.5, 1.0), 1.0);
        assert_eq!(blend_channel(Burn, 0.75, 0.5), 0.5);
        assert_eq!(blend_channel(Burn, 1.0, 0.0), 1.0);
        assert_eq!(blend_channel(Burn, 0.5, 0.0), 0.0);
        // overlay darkens dark backdrops and lightens light ones
        assert_eq!(blend_channel(Overlay, 0.25, 0.5), 0.25);
        assert!((blend_channel(Overlay, 0.75, 0.75) - 0.875).abs() < 1e-6);
    }

    #[test]
    fn opaque_over_opaque_and_transparent_cases() {
        let red = [1.0, 0.0, 0.0, 1.0];
        let blue = [0.0, 0.0, 1.0, 1.0];
        assert_eq!(composite(red, blue, Normal, 1.0), blue);
        assert_eq!(composite(red, blue, Normal, 0.0), red);
        let half = composite(red, blue, Normal, 0.5);
        assert!((half[0] - 0.5).abs() < 1e-6 && (half[2] - 0.5).abs() < 1e-6 && half[3] == 1.0);
        // over nothing the layer is just itself at its own alpha
        assert_eq!(
            composite([0.0; 4], [0.2, 0.4, 0.6, 0.5], Multiply, 1.0),
            [0.2, 0.4, 0.6, 0.5]
        );
        assert_eq!(composite([0.0; 4], [1.0; 4], Screen, 0.0), [0.0; 4]);
    }
}
