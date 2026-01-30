use eframe::egui;
use opencv::core::Mat;

#[derive(Clone)]
pub struct DetectionResult {
    pub path: String,
    pub name: String,
    pub score: f64,
    pub face_image: Option<egui::ColorImage>,
}

#[derive(Clone)]
pub struct KnownFace {
    pub name: String,
    pub embedding: Mat,
}