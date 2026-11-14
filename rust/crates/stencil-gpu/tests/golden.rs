//! GPU output vs the CPU reference. Needs a Vulkan device (software is fine):
//! `FORGE_GPU_TESTS=1 VK_ICD_FILENAMES=.../vk_swiftshader_icd.json cargo test -p stencil-gpu`

use stencil_gpu::blend::{composite, BlendMode};
use stencil_gpu::{read_rgba8, Compositor, Fill, Gpu, Layer, Source};

fn gpu() -> Option<Gpu> {
    if std::env::var("FORGE_GPU_TESTS").is_err() {
        return None;
    }
    Some(Gpu::headless().expect("no GPU adapter"))
}

fn to_f(p: [u8; 4]) -> [f32; 4] {
    p.map(|v| v as f32 / 255.0)
}

fn close(a: [u8; 4], b: [f32; 4], tol: f32) -> bool {
    let a = to_f(a);
    (0..4).all(|i| (a[i] - b[i]).abs() <= tol)
}

const MODES: [BlendMode; 6] = [
    BlendMode::Normal,
    BlendMode::Multiply,
    BlendMode::Screen,
    BlendMode::Overlay,
    BlendMode::Dodge,
    BlendMode::Burn,
];

#[test]
fn every_blend_mode_matches_cpu() {
    let Some(gpu) = gpu() else { return };
    let comp = Compositor::new(&gpu.device);
    let base = [0.6, 0.3, 0.8, 1.0];
    let top = [0.4, 0.7, 0.2, 0.8];
    for mode in MODES {
        for opacity in [1.0, 0.5] {
            let tex = comp.compose(
                &gpu,
                4,
                4,
                &[
                    Layer {
                        source: Source::Fill(Fill::Solid(base)),
                        blend: BlendMode::Normal,
                        opacity: 1.0,
                    },
                    Layer {
                        source: Source::Fill(Fill::Solid(top)),
                        blend: mode,
                        opacity,
                    },
                ],
            );
            let px = read_rgba8(&gpu, &tex);
            let want = composite(base, top, mode, opacity);
            assert!(
                close(px[5], want, 2.5 / 255.0),
                "{mode:?} @ {opacity}: gpu {:?} cpu {want:?}",
                to_f(px[5])
            );
        }
    }
}

#[test]
fn gradient_runs_top_to_bottom() {
    let Some(gpu) = gpu() else { return };
    let comp = Compositor::new(&gpu.device);
    let tex = comp.compose(
        &gpu,
        8,
        8,
        &[Layer {
            source: Source::Fill(Fill::VGradient {
                top: [1.0, 0.0, 0.0, 1.0],
                bottom: [0.0, 0.0, 1.0, 1.0],
            }),
            blend: BlendMode::Normal,
            opacity: 1.0,
        }],
    );
    let px = read_rgba8(&gpu, &tex);
    assert!(px[0][0] > 200 && px[0][2] < 60, "top row red: {:?}", px[0]);
    assert!(
        px[56][2] > 200 && px[56][0] < 60,
        "bottom row blue: {:?}",
        px[56]
    );
}

#[test]
fn glow_peaks_at_center_and_fades() {
    let Some(gpu) = gpu() else { return };
    let comp = Compositor::new(&gpu.device);
    let tex = comp.compose(
        &gpu,
        33,
        33,
        &[Layer {
            source: Source::Fill(Fill::Glow {
                color: [1.0, 1.0, 1.0, 1.0],
                center: [0.5, 0.5],
                radius: 0.4,
                softness: 1.0,
            }),
            blend: BlendMode::Normal,
            opacity: 1.0,
        }],
    );
    let px = read_rgba8(&gpu, &tex);
    let (centre, corner) = (px[16 * 33 + 16][3], px[0][3]);
    assert!(centre > 240, "centre alpha {centre}");
    assert!(corner < 8, "corner alpha {corner}");
}

#[test]
fn empty_stack_is_transparent_and_texture_layers_work() {
    let Some(gpu) = gpu() else { return };
    let comp = Compositor::new(&gpu.device);
    let empty = comp.compose(&gpu, 4, 4, &[]);
    assert!(read_rgba8(&gpu, &empty).iter().all(|p| *p == [0, 0, 0, 0]));
    // A pre-rendered texture composites the same as the fill that made it.
    let src = stencil_gpu::layer_texture(&gpu.device, 4, 4, "src");
    comp.render_fill(&gpu, &Fill::Solid([0.2, 0.4, 0.6, 1.0]), &src);
    let view = src.create_view(&Default::default());
    let out = comp.compose(
        &gpu,
        4,
        4,
        &[Layer {
            source: Source::Texture(&view),
            blend: BlendMode::Normal,
            opacity: 1.0,
        }],
    );
    assert!(close(
        read_rgba8(&gpu, &out)[0],
        [0.2, 0.4, 0.6, 1.0],
        2.0 / 255.0
    ));
}
