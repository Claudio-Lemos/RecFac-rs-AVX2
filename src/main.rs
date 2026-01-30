// Declaração dos módulos
mod constants;
mod types;
mod utils;
mod engine;
mod worker;
mod app;

use anyhow::Result;
use eframe::egui;
use app::FaceApp;

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 800.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Rust Face ID", 
        options, 
        Box::new(|cc| Ok(Box::new(FaceApp::new(cc))))
    ).map_err(|e| anyhow::anyhow!("{}", e))
}