//! MSI Explorer GUI - A modern alternative to Microsoft Orca

#![windows_subsystem = "windows"]

use eframe::egui::{self, IconData};

mod app;
mod panels;
mod schema;
mod theme;
mod widgets;

use app::MsiExplorerApp;

/// Load icon from embedded PNG
fn load_icon() -> IconData {
    const ICON_BYTES: &[u8] = include_bytes!("../../assets/app-icon.png");

    let img = image::load_from_memory(ICON_BYTES)
        .expect("Failed to load embedded icon")
        .resize(64, 64, image::imageops::FilterType::Lanczos3)
        .to_rgba8();

    let (width, height) = img.dimensions();

    IconData {
        rgba: img.into_raw(),
        width,
        height,
    }
}

fn main() -> eframe::Result<()> {
    // Only init logger in debug builds (no console in release with windows subsystem)
    #[cfg(debug_assertions)]
    env_logger::init();

    let icon = load_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_drag_and_drop(true)
            .with_icon(std::sync::Arc::new(icon)),
        ..Default::default()
    };

    eframe::run_native(
        "MSI Explorer",
        options,
        Box::new(|cc| Ok(Box::new(MsiExplorerApp::new(cc)))),
    )
}
