//! Thin binary: window setup and wiring only.

use forge::ForgeApp;
use rack_core::Workspace;

struct Host(ForgeApp);

impl eframe::App for Host {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
    fn on_exit(&mut self) {
        self.0.autosave();
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("FORGE · BioSpark")
            .with_inner_size([1680.0, 1000.0])
            .with_min_inner_size([760.0, 520.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "FORGE · BioSpark",
        options,
        Box::new(|cc| {
            let mut app = ForgeApp::new(&cc.egui_ctx, Workspace::default_root());
            app.gpu = cc.wgpu_render_state.as_ref().map(forge::gpu::GpuView::new);
            Ok(Box::new(Host(app)))
        }),
    )
}
