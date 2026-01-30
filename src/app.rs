use eframe::egui;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;
use std::fs;
use std::path::Path;

use crate::constants::{KNOWN_DIR, UNKNOWN_DIR, COSINE_THRESHOLD};
use crate::types::DetectionResult;
use crate::worker::worker_logic;
use crate::utils::move_file;

struct MatchItem {
    data: DetectionResult,
    texture: Option<egui::TextureHandle>,
    is_editing: bool,
    input_name: String,
}

impl MatchItem {
    fn new(data: DetectionResult, texture: Option<egui::TextureHandle>) -> Self {
        Self {
            data,
            texture,
            is_editing: false,
            input_name: String::new(),
        }
    }
}

pub struct FaceApp {
    rx: Receiver<DetectionResult>,
    matches: Vec<MatchItem>,
    is_processing: bool,
    status_msg: String,
}

impl FaceApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let (_, rx) = channel();
        let _ = fs::create_dir_all(KNOWN_DIR);
        let _ = fs::create_dir_all(UNKNOWN_DIR);

        Self {
            rx,
            matches: Vec::new(),
            is_processing: false,
            status_msg: "Pronto.".to_owned(),
        }
    }

    fn start_processing(&mut self, ctx: egui::Context) {
        self.is_processing = true;
        self.matches.clear();
        
        let (tx, rx) = channel();
        self.rx = rx;

        thread::spawn(move || {
            if let Err(e) = worker_logic(tx, ctx) {
                eprintln!("Worker error: {:?}", e);
            }
        });
    }

    fn add_images_dialog(&mut self) {
        if let Some(files) = rfd::FileDialog::new()
            .add_filter("Imagens", &["jpg", "png", "jpeg", "bmp", "webp"])
            .set_title("Importar para Desconhecidos")
            .pick_files() 
        {
            let count = files.iter().filter_map(|file| {
                let name = file.file_name()?;
                let dest = Path::new(UNKNOWN_DIR).join(name);
                fs::copy(file, dest).ok()
            }).count();
            
            self.status_msg = format!("{} imagens adicionadas. Clique em RUN.", count);
        }
    }
}

impl eframe::App for FaceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Ingestão de resultados do Worker
        while let Ok(res) = self.rx.try_recv() {
            let texture = res.face_image.as_ref().map(|img| {
                ctx.load_texture(&res.path, img.clone(), egui::TextureOptions::LINEAR)
            });
            self.matches.push(MatchItem::new(res, texture));
        }

        if self.is_processing {
            if let Err(TryRecvError::Disconnected) = self.rx.try_recv() {
                self.is_processing = false;
                self.status_msg = format!("Busca finalizada. {} itens.", self.matches.len());
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("RUST Face ID - High Performance Manager");
            
            ui.horizontal(|ui| {
                if ui.add_enabled(!self.is_processing, egui::Button::new("▶ RUN")).clicked() {
                    self.start_processing(ctx.clone());
                }
                if ui.add_enabled(!self.is_processing, egui::Button::new("📂 ADD")).clicked() {
                    self.add_images_dialog();
                }
                if self.is_processing { ui.spinner(); }
                ui.label(&self.status_msg);
            });

            ui.separator();

            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                egui::Grid::new("face_grid").striped(true).spacing([10.0, 10.0]).show(ui, |ui| {
                    ui.strong("Face");
                    ui.strong("Identidade");
                    ui.strong("Confiança");
                    ui.strong("Ações");
                    ui.end_row();

                    let mut to_remove = None;

                    for (i, item) in self.matches.iter_mut().enumerate().rev() {
                        let res = &item.data;

                        // Coluna 1: Thumbnail
                        if let Some(tex) = &item.texture {
                            ui.add(egui::Image::new(tex).max_height(60.0));
                        } else {
                            ui.label("N/A");
                        }
                        
                        // Coluna 2: Nome Detectado
                        let is_match = res.score > COSINE_THRESHOLD;
                        let color = if is_match { egui::Color32::LIGHT_GREEN } else { egui::Color32::LIGHT_RED };
                        ui.colored_label(color, &res.name);
                        
                        // Coluna 3: Score
                        ui.add(egui::ProgressBar::new(res.score as f32).text(format!("{:.1}%", res.score * 100.0)));

                        // Coluna 4: Ações Dinâmicas
                        ui.horizontal(|ui| {
                            if item.is_editing {
                                ui.text_edit_singleline(&mut item.input_name);
                                if ui.button("✅").clicked() {
                                    let clean_name = item.input_name.trim();
                                    if !clean_name.is_empty() && move_file(&res.path, clean_name).is_ok() {
                                        to_remove = Some(i);
                                    }
                                }
                                if ui.button("❌").clicked() { item.is_editing = false; }
                            } else {
                                // Mover para pasta reconhecida (Identidade)
                                if is_match && res.name != "Desconhecido" {
                                    if ui.button(format!("📁 Aceitar {}", res.name)).clicked() {
                                        if move_file(&res.path, &res.name).is_ok() {
                                            to_remove = Some(i);
                                        }
                                    }
                                }

                                if ui.button(if is_match { "✏️ Corrigir" } else { "➕ Novo" }).clicked() {
                                    item.is_editing = true;
                                    item.input_name = if res.name != "Desconhecido" { res.name.clone() } else { "".into() };
                                }
                            }
                        });
                        ui.end_row();
                    }

                    if let Some(idx) = to_remove {
                        self.matches.remove(idx);
                        self.status_msg = "Organizado com sucesso.".into();
                    }
                });
            });
        });
    }
}