mod app;

use paladin_core::error::{Error, Result};

pub fn run_app() -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([960.0, 640.0])
            .with_title("Paladin"),
        ..Default::default()
    };
    eframe::run_native(
        "Paladin",
        native_options,
        Box::new(|cc| Ok(Box::new(app::PaladinApp::new(cc)))),
    )
    .map_err(|e| Error::Other(format!("gui: {e}")))
}
