// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use std::fs::{DirBuilder, OpenOptions};
use std::io::prelude::*;
use early_warning_system::data_model::{Farm, Status};
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Debug, Clone)]
enum LoadError {
    File,
    Format,
}

#[derive(Debug, Clone)]
enum SaveError {
    File,
    Write,
    Format,
}

impl SavedState {
    fn path() -> std::path::PathBuf {
        let mut path = std::env::current_dir().unwrap_or_default();
        path.push("rgo_early_warning.json");

        path
    }

    async fn load() -> Result<SavedState, LoadError> {
        use async_std::{self, prelude::*};

        let mut contents = String::new();

        let mut file = async_std::fs::File::open(Self::path())
            .await
            .map_err(|_| LoadError::File)?;

        file.read_to_string(&mut contents)
            .await
            .map_err(|_| LoadError::File)?;

        serde_json::from_str(&contents).map_err(|_| LoadError::Format)
    }

    pub fn print(&self) {
        let path = Self::path();
        println!("Path: {:#?}", path.display());
    }

    async fn save(self) -> Result<(), SaveError> {
        use async_std::prelude::*;
    
        let json = serde_json::to_string_pretty(&self)
            .map_err(|_| SaveError::Format)?;

        let path = Self::path();
        println!("Path:");
        println!("{:#?}", path);

        if let Some(dir) = path.parent() {
            async_std::fs::create_dir_all(dir)
                .await
                .map_err(|_| SaveError::File)?;
        }

        
        let mut file = async_std::fs::File::create(path)
            .await
            .map_err(|_| SaveError::File)?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|_| SaveError::Write)?;
        

        // This is a simple way to save at most once every couple seconds
        async_std::task::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct SavedState {
    farms: Vec<Farm>
}

impl SavedState {
    fn default() -> SavedState {
        SavedState { farms: vec![] }
    }
}

fn path_to_cache() -> std::path::PathBuf {
    let cur_dir = std::env::current_dir().unwrap_or_default();
    let cache_dir = match cur_dir.parent() {
        Some(p) => p,
        None => &cur_dir,
    }.join(".cache");
    cache_dir.join("early_warning.json")
}

#[tauri::command]
fn save(farms: Vec<Farm>) {
    let save_state = SavedState {farms};
    let output_text = serde_json::to_string_pretty(&save_state).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path_to_cache())
        .expect("Failed to open cache while saving");
    file.write_all(output_text.as_bytes()).expect("Failed to write data to file");
}

#[tauri::command]
fn load() -> Vec<Farm> {
    let cache_path = path_to_cache();
    DirBuilder::new()
        .recursive(true)
        .create(cache_path.parent().unwrap())
        .unwrap_or_default();
    let cache = OpenOptions::new()
        .read(true)
        .open(cache_path);
    let save_state: SavedState = match cache {
        Ok(file) => match serde_json::from_reader(file) {
            Ok(state) => state,
            Err(_) => SavedState::default(),
        },
        Err(_) => SavedState::default(),
    };
    save_state.farms
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![save, load])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
