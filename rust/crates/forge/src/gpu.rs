//! Bridge between the Stencil compositor and egui. The compositor renders a
//! layer stack into a texture on eframe's own device; the texture is then
//! registered with egui's renderer so any `ui` can draw it as an image.

use std::collections::HashMap;

use eframe::egui_wgpu::RenderState;
use stencil_gpu::{Compositor, Gpu, Layer};

pub struct GpuView {
    gpu: Gpu,
    comp: Compositor,
    state: RenderState,
    /// One registered texture per caller key, re-composed in place each call.
    slots: HashMap<String, (egui::TextureId, (u32, u32))>,
}

impl GpuView {
    pub fn new(state: &RenderState) -> Self {
        let gpu = Gpu::from_parts(state.device.clone(), state.queue.clone());
        Self {
            comp: Compositor::new(&gpu.device),
            gpu,
            state: state.clone(),
            slots: HashMap::new(),
        }
    }

    /// Compose `layers` at `size` pixels and return the egui texture holding
    /// the result. Call every frame the stack changes; the id stays stable
    /// while the size does.
    pub fn compose(
        &mut self,
        key: &str,
        size: (u32, u32),
        layers: &[Layer<'_>],
    ) -> egui::TextureId {
        let tex = self
            .comp
            .compose(&self.gpu, size.0.max(1), size.1.max(1), layers);
        let view = tex.create_view(&Default::default());
        let mut r = self.state.renderer.write();
        match self.slots.get(key) {
            Some(&(id, s)) if s == size => {
                r.update_egui_texture_from_wgpu_texture(&self.gpu.device, &view, wgpu_filter(), id);
                id
            }
            old => {
                if let Some(&(id, _)) = old {
                    r.free_texture(&id);
                }
                let id = r.register_native_texture(&self.gpu.device, &view, wgpu_filter());
                self.slots.insert(key.to_owned(), (id, size));
                id
            }
        }
    }

    pub fn compositor(&self) -> (&Gpu, &Compositor) {
        (&self.gpu, &self.comp)
    }
}

fn wgpu_filter() -> eframe::wgpu::FilterMode {
    eframe::wgpu::FilterMode::Linear
}
