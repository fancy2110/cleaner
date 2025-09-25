use std::{fs::DirEntry, path::Path};

use objc2_foundation::{NSFileManager, NSString};

pub(super) fn read_dir(path: &Path) -> Option<Vec<DirEntry>> {
    let mut vec = vec![];

    unsafe {
        let file_manager = NSFileManager::defaultManager();
        if let Some(str) = path.as_os_str().to_str() {
            let path = NSString::from_str(str);
            match file_manager.contentsOfDirectoryAtPath_error(&path) {
                Ok(entries) => {
                    for entry in entries {
                        // vec.push(DirEntry(entry));
                    }
                }
                Err(_) => return None,
            }
        }
    };
    Some(vec)
}
