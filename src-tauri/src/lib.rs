// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tauri::{command, State};
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;
use tracing::{debug, info};

mod driver;
mod model;
mod service;
mod tree;
use service::{ScanProgress, Scanner};

use driver::get_available_drivers;

use model::{FileDetails, ScanResult};

#[command]
async fn scan_directory(path: String) -> Result<ScanResult, String> {
    let mut result = ScanResult {
        files: Vec::new(),
        total_size: 0,
        total_files: 0,
        total_dirs: 0,
    };

    match scan_dir_recursive(&path, &mut result) {
        Ok(_) => Ok(result),
        Err(e) => Err(format!("扫描目录出错: {}", e)),
    }
}

fn scan_dir_recursive(path: &str, result: &mut ScanResult) -> Result<(), String> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            let entry = entry.map_err(|e| format!("读取目录条目时出错: {}", e))?;
            let path = entry.path();
            let metadata = entry
                .metadata()
                .map_err(|e| format!("无法获取元数据: {}", e))?;

            let name = path
                .file_name()
                .ok_or_else(|| "无法获取文件名".to_string())?
                .to_string_lossy()
                .into_owned();

            let created = metadata
                .created()
                .unwrap_or_else(|_| SystemTime::now())
                .elapsed()
                .unwrap_or_default();

            let modified = metadata
                .modified()
                .unwrap_or_else(|_| SystemTime::now())
                .elapsed()
                .unwrap_or_default();

            let path_str = path;

            let file_details = FileDetails {
                name,
                path: path_str.clone(),
                size: metadata.len(),
                is_dir: metadata.is_dir(),
                created: created.as_secs(),
                modified: modified.as_secs(),
                readonly: metadata.permissions().readonly(),
                file_type: if metadata.is_dir() {
                    "directory".to_string()
                } else {
                    if metadata.file_type().is_file() {
                        "file".to_string()
                    } else if metadata.file_type().is_symlink() {
                        "symlink".to_string()
                    } else {
                        "unknown".to_string()
                    }
                },
                children: None,
            };

            if metadata.is_dir() {
                result.total_dirs += 1;
                result.files.push(file_details);
                // scan_dir_recursive(&path_str, result)?;
            } else {
                result.total_files += 1;
                result.total_size += metadata.len();
                result.files.push(file_details);
            }
        }
    }
    Ok(())
}

#[command]
async fn start_scan(
    state: State<'_, Mutex<Scanner>>,
    path: &str,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let path = PathBuf::from(path);
    debug!(
        "start_folder_scan called with path: {:?}",
        path.to_string_lossy()
    );

    let mut scanner = state.lock().await;

    // Clear previous scan data
    scanner.clear().await;

    // Start scanning and get receiver
    let mut rx = scanner.start().await;

    // Spawn task to handle file updates
    tokio::spawn(async move {
        while let Some(stats) = rx.recv().await {
            // Emit update event to frontend
            let _ = app_handle.emit("folder-scan-update", &FileDetails::from(stats));
        }

        debug!("all scan job finished");
        // Emit completion event
        let _ = app_handle.emit("folder-scan-complete", "Scan completed");
    });

    Ok(())
}

#[command]
async fn get_folder_stats(
    path: String,
    state: State<'_, Mutex<Scanner>>,
) -> Result<Option<FileDetails>, String> {
    let scanner = state.lock().await;
    let stats = scanner
        .get_file_node(&PathBuf::from(path))
        .await;
    Ok(stats)
}

#[command]
async fn get_scan_progress(state: State<'_, Mutex<Scanner>>) -> Result<ScanProgress, String> {
    let scanner = state.lock().await;
    Ok(scanner.get_progress().await)
}

#[command]
async fn stop_folder_scan(state: State<'_, Mutex<Scanner>>) -> Result<(), String> {
    let mut scanner = state.lock().await;
    scanner.stop_scanning().await;
    Ok(())
}

#[command]
async fn is_scanning(state: State<'_, Mutex<Scanner>>) -> Result<bool, String> {
    let scanner = state.lock().await;
    Ok(scanner.is_scanning().await)
}

#[command]
async fn clear_folder_scan(state: State<'_, Mutex<Scanner>>) -> Result<(), String> {
    let mut scanner = state.lock().await;
    debug!("clear folder scan has been called");
    scanner.clear().await;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scanner = Scanner::new(1); // 3 concurrent workers

    let app = tauri::Builder::default()
        .manage(Mutex::new(scanner))
        .setup(|app| {
            let resolver = app.handle().path();
            info!(
                "app steup with resources path: {:?}",
                resolver.config_dir().unwrap()
            );

            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_scan,
            get_folder_stats,
            get_scan_progress,
            stop_folder_scan,
            is_scanning,
            clear_folder_scan,
            get_available_drivers
        ])
        .build(tauri::generate_context!());
    if let Ok(app) = app {
        app.run(|_, event| {});
    } else {
        panic!("error while running tauri application")
    }
}
