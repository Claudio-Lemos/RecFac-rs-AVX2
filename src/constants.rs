pub const COSINE_THRESHOLD: f64 = 0.363;
pub const DETECTOR_MODEL: &[u8] = include_bytes!("../face_detection_yunet_2023mar.onnx");
pub const RECOGNIZER_MODEL: &[u8] = include_bytes!("../face_recognition_sface_2021dec.onnx");

pub const UNKNOWN_DIR: &str = "desconhecidos";
pub const KNOWN_DIR: &str = "conhecidos";