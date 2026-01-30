use anyhow::Result;
use eframe::egui;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use walkdir::WalkDir;
use opencv::prelude::*; 

use crate::constants::{KNOWN_DIR, UNKNOWN_DIR, COSINE_THRESHOLD};
use crate::engine::FaceEngine;
use crate::types::{DetectionResult, KnownFace};


pub fn worker_logic(tx: Sender<DetectionResult>, ctx: egui::Context) -> Result<()> {
    // Coleta arquivos com extensões comuns de imagem (case-insensitive)
    let valid_ext = ["jpg", "jpeg", "png", "bmp", "webp"];
    
    let get_paths = |dir: &str| {
        WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.into_path())
            .filter(|p| {
                p.is_file() && p.extension()
                    .and_then(|s| s.to_str())
                    .map(|s| valid_ext.contains(&s.to_lowercase().as_str()))
                    .unwrap_or(false)
            })
            .collect::<Vec<PathBuf>>()
    };

    let known_paths = get_paths(KNOWN_DIR);
    let unknown_paths = get_paths(UNKNOWN_DIR);

    // Carrega DB de faces conhecidas
    let db: Vec<KnownFace> = known_paths
        .par_iter()
        .map_init(
            || FaceEngine::new().ok(), 
            |engine, path| {
                if let Some(eng) = engine {
                    if let Ok(Some((embedding, _))) = eng.process_and_crop(path) {
                        let name = path.parent()
                                    .and_then(|p| p.file_name())
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "Desconhecido".to_string());
                                    
                                return Some(KnownFace { name, embedding });
                    }
                }
                None
            }
        )
        .filter_map(|x| x)
        .collect();

    // Processa Desconhecidos
    unknown_paths.par_iter().map_init(
        || FaceEngine::new().ok(),
        |engine, path| {
            let eng = match engine {
                Some(e) => e,
                None => return,
            };

            let result = match eng.process_and_crop(path) {
                Ok(Some((target_emb, face_img))) => {
                    let mut best_match_name = "Desconhecido".to_string();
                    let mut max_score = 0.0;

                    for known in &db {
                        if let Ok(score) = eng.recognizer.match_(&known.embedding, &target_emb, 0) {
                            if score > max_score {
                                max_score = score;
                                if score > COSINE_THRESHOLD {
                                    best_match_name = known.name.clone();
                                }
                            }
                        }
                    }

                    DetectionResult {
                        path: path.to_string_lossy().to_string(),
                        name: best_match_name,
                        score: max_score,
                        face_image: Some(face_img),
                    }
                }
                _ => {
                    // Caso não encontre rosto ou erro na imagem, envia um aviso
                    DetectionResult {
                        path: path.to_string_lossy().to_string(),
                        name: "Nenhum rosto detectado".to_string(),
                        score: 0.0,
                        face_image: None,
                    }
                }
            };

            let _ = tx.send(result);
            ctx.request_repaint();
        }
    ).count();

    Ok(())
}
