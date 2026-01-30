use anyhow::{Context, Result};
use eframe::egui;
use opencv::{
    core::{Mat, Ptr, Size},
    imgcodecs, imgproc,
    objdetect::{FaceDetectorYN, FaceRecognizerSF},
    prelude::*,
};
use std::fs;
use std::path::{Path};
use crate::constants::{DETECTOR_MODEL, RECOGNIZER_MODEL};

pub struct FaceEngine {
    pub detector: Ptr<FaceDetectorYN>,
    pub recognizer: Ptr<FaceRecognizerSF>,
}

impl FaceEngine {
    pub fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("rust_face_id_models");
        if !temp_dir.exists() { fs::create_dir_all(&temp_dir)?; }

        let det_path = temp_dir.join("det.onnx");
        let rec_path = temp_dir.join("rec.onnx");

        // Escrita atômica simples para garantir que os modelos existam no disco
        if !det_path.exists() { fs::write(&det_path, DETECTOR_MODEL)?; }
        if !rec_path.exists() { fs::write(&rec_path, RECOGNIZER_MODEL)?; }

        let det_ptr = det_path.to_str().context("Path error")?;
        let rec_ptr = rec_path.to_str().context("Path error")?;

        let detector = FaceDetectorYN::create(det_ptr, "", Size::new(320, 320), 0.5, 0.3, 5000, 0, 0)?;
        let recognizer = FaceRecognizerSF::create(rec_ptr, "", 0, 0)?;

        Ok(Self { detector, recognizer })
    }

    pub fn process_and_crop(&mut self, img_path: &Path) -> Result<Option<(Mat, egui::ColorImage)>> {
        let img = imgcodecs::imread(img_path.to_str().unwrap(), imgcodecs::IMREAD_COLOR)?;
        if img.empty() { return Ok(None); }

        self.detector.set_input_size(img.size()?)?;
        let mut faces = Mat::default();
        self.detector.detect(&img, &mut faces)?;

        if faces.rows() < 1 { return Ok(None); }

        let mut aligned_face = Mat::default();
        self.recognizer.align_crop(&img, &faces.row(0)?, &mut aligned_face)?;

        let mut feature = Mat::default();
        self.recognizer.feature(&aligned_face, &mut feature)?;

        let mut rgb_face = Mat::default();
        imgproc::cvt_color_def(&aligned_face, &mut rgb_face, imgproc::COLOR_BGR2RGB)?;
        
        let size = [rgb_face.cols() as usize, rgb_face.rows() as usize];
        let color_image = egui::ColorImage::from_rgb(size, rgb_face.data_bytes()?);

        Ok(Some((feature.clone(), color_image)))
    }
}