use std::{fs::FileType, path::PathBuf};

use serde::Serialize;

use crate::service::FileNode;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDetails {
    pub name: String,
    pub path: PathBuf,
    pub size: usize,
    pub is_directory: bool,
    pub created: u64,
    pub modified: u64,
    pub readonly: bool,
    pub file_type: String,
    pub children: Option<Vec<FileDetails>>,
}

impl FileDetails {
    pub fn from(stat: FileNode) -> FileDetails {
        let path = stat.path;
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or("/".to_string());

        FileDetails {
            name: name,
            path: path,
            size: stat.size,
            is_directory: stat.is_directory,
            created: stat.created.unwrap_or_default(),
            modified: stat.modified.unwrap_or_default(),
            readonly: false,
            file_type: "file".to_string(),
            children: None,
        }
    }
}

impl Default for FileDetails {
    fn default() -> Self {
        Self {
            name: Default::default(),
            path: Default::default(),
            size: Default::default(),
            is_directory: Default::default(),
            created: Default::default(),
            modified: Default::default(),
            readonly: Default::default(),
            file_type: Default::default(),
            children: Default::default(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct Volumn {
    pub name: String,
    pub path: PathBuf,
    pub total_size: u64,
    pub available_size: u64,
}
