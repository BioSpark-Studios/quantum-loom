// Procedural layer content: shaders that generate a layer instead of drawing widgets.

struct F {
    kind: u32,
    seed: u32,
    width: f32,
    height: f32,
    a: vec4<f32>,
    b: vec4<f32>,
    // x, y = centre (0..1), z = radius (0..1), w = softness
    shape: vec4<f32>,
};

@group(0) @binding(0) var<uniform> f: F;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
}

fn hash(n: u32) -> f32 {
    var x = n * 747796405u + 2891336453u;
    x = ((x >> ((x >> 28u) + 4u)) ^ x) * 277803737u;
    x = (x >> 22u) ^ x;
    return f32(x) / 4294967295.0;
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy / vec2<f32>(f.width, f.height);
    switch (f.kind) {
        // 1: vertical gradient a (top) -> b (bottom)
        case 1u: { return mix(f.a, f.b, uv.y); }
        // 2: round glow: colour a at the centre fading to transparent
        case 2u: {
            let d = distance(uv * vec2<f32>(f.width, f.height), f.shape.xy * vec2<f32>(f.width, f.height)) / (f.shape.z * f.height);
            let t = 1.0 - smoothstep(max(0.0, 1.0 - f.shape.w), 1.0, d);
            return vec4<f32>(f.a.rgb, f.a.a * t);
        }
        // 3: brushed metal: gradient a -> b with fine vertical grain (strength in shape.x)
        case 3u: {
            let g = hash(u32(pos.x) + f.seed * 7919u) - 0.5;
            let base = mix(f.a, f.b, uv.y);
            return vec4<f32>(clamp(base.rgb + vec3<f32>(g * f.shape.x), vec3<f32>(0.0), vec3<f32>(1.0)), base.a);
        }
        // 0: solid
        default: { return f.a; }
    }
}
