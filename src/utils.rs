// src/utils.rs
use std::fs;
use std::path::{Path};
use crate::constants::KNOWN_DIR;
use anyhow::{Context, Result};

pub fn move_file(source_path: &str, identity_name: &str) -> Result<()> {
    let source = Path::new(source_path);
    let file_name = source.file_name()
        .context("Falha ao extrair nome do arquivo")?;

    // Destino: conhecidos/{identidade}/{nome_original}
    let dest_dir = Path::new(KNOWN_DIR).join(identity_name);
    let dest_path = dest_dir.join(file_name);

    if !dest_dir.exists() {
        fs::create_dir_all(&dest_dir)?;
    }

    // fs::rename é eficiente (apenas troca de inode se no mesmo FS)
    fs::rename(source, dest_path).context("Erro ao mover arquivo")?;
    
    Ok(())
}