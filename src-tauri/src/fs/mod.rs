mod linux;
mod macos;
mod windows;

use std::fs::DirEntry;
use std::path::Path;

/**
 *  read dirs with native os api
 *  @returns {Option<Vec<DirEntry>>}
*/
pub fn read_dir(path: &Path) -> Option<Vec<DirEntry>> {
    let entries: Option<Vec<DirEntry>>;
    #[cfg(target_os = "cygwin")]
    {
        entries = windows::read_dir(path);
    }
    #[cfg(target_os = "linux")]
    {
        entries = linux::read_dir(path);
    }
    #[cfg(target_os = "macos")]
    {
        entries = macos::read_dir(path);
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        // 默认实现，适用于其他操作系统
        entries = match std::fs::read_dir(path) {
            Ok(entries) => Some(entries),
            Err(_) => return None,
        };
    }

    return entries;
}
