// Flatten one layer onto the accumulated image with a real blend mode.
// Straight (non-premultiplied) alpha; W3C compositing:
//   co = as(1-ab)Cs + as*ab*B(Cb,Cs) + (1-as)ab*Cb ,  ao = as + ab(1-as)

struct U {
    mode: u32,
    opacity: f32,
    pad0: f32,
    pad1: f32,
};

@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var dst_tex: texture_2d<f32>;
@group(0) @binding(2) var src_tex: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    // One oversized triangle covers the target.
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
}

fn screen(cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
    return cb + cs - cb * cs;
}

fn hard_light(cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
    let lo = cb * (2.0 * cs);
    let hi = screen(cb, 2.0 * cs - vec3<f32>(1.0));
    return select(hi, lo, cs <= vec3<f32>(0.5));
}

fn dodge1(cb: f32, cs: f32) -> f32 {
    if (cb == 0.0) { return 0.0; }
    if (cs >= 1.0) { return 1.0; }
    return min(1.0, cb / (1.0 - cs));
}

fn burn1(cb: f32, cs: f32) -> f32 {
    if (cb >= 1.0) { return 1.0; }
    if (cs <= 0.0) { return 0.0; }
    return 1.0 - min(1.0, (1.0 - cb) / cs);
}

fn blend(cb: vec3<f32>, cs: vec3<f32>, mode: u32) -> vec3<f32> {
    switch (mode) {
        case 1u: { return cb * cs; }                       // multiply
        case 2u: { return screen(cb, cs); }                // screen
        case 3u: { return hard_light(cs, cb); }            // overlay = hard-light, swapped
        case 4u: { return vec3<f32>(dodge1(cb.r, cs.r), dodge1(cb.g, cs.g), dodge1(cb.b, cs.b)); }
        case 5u: { return vec3<f32>(burn1(cb.r, cs.r), burn1(cb.g, cs.g), burn1(cb.b, cs.b)); }
        default: { return cs; }                            // normal
    }
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let p = vec2<i32>(pos.xy);
    let d = textureLoad(dst_tex, p, 0);
    let s = textureLoad(src_tex, p, 0);
    let ab = d.a;
    let a_s = s.a * clamp(u.opacity, 0.0, 1.0);
    let ao = a_s + ab * (1.0 - a_s);
    if (ao <= 0.0) {
        return vec4<f32>(0.0);
    }
    let b = blend(d.rgb, s.rgb, u.mode);
    let co = a_s * (1.0 - ab) * s.rgb + a_s * ab * b + (1.0 - a_s) * ab * d.rgb;
    return vec4<f32>(co / ao, ao);
}
