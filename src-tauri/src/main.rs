#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct SaveRequest {
    path: String,
    data: String,
}

#[tauri::command]
fn save_records(req: SaveRequest) -> Result<(), String> {
    fs::write(req.path, req.data).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_records(path: String) -> Result<String, String> {
    let p = PathBuf::from(path);
    if p.exists() {
        fs::read_to_string(p).map_err(|e| e.to_string())
    } else {
        Ok("[]".to_string())
    }
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![save_records, load_records])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
