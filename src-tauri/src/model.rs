use std::time::SystemTime;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FileDetails {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub created: u64,
    pub modified: u64,
    pub readonly: bool,
    pub file_type: String,
    pub children: Option<Vec<FileDetails>>
}

#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub files: Vec<FileDetails>,
    pub total_size: u64,
    pub total_files: usize,
    pub total_dirs: usize,
}

/**
 * Volumn Information
 * */
#[derive(Debug, Serialize)]
pub struct Volumn {
    pub name: String,
    pub path: String,
    pub total_size: u64,
    pub available_size: u64,
}
