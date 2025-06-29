use std::{
    fs,
    path::Path,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use crate::{model::Volumn, service::Scanner};
use sysinfo::{Disks, System};
use tauri::{command, State};
use tokio::sync::Mutex;
use tracing::debug;

pub enum FileType {
    File(String),
    Directory,
    Symlink,
}

/**
 * All element in file system, like : File, Link or Directory
 */
struct Element {
    /**
     * Path of current node
     */
    pub path: Box<Path>,
    /**
     * Total size of this directory or file
     */
    pub size: AtomicU64,
    /**
     * the type of urrent path
     */
    pub file_type: FileType,
    /**
     * create time of current path
     */
    created: Duration,
    /**
     * update time of current path
     */
    modified: Duration,
    /**
     *  ready permission of current path
     */
    pub readonly: bool,
    /**
     * if the file type is directory, then this field will be filled with the child elements of current path
     * else this field will be None
     */
    children: Mutex<Option<Vec<Element>>>,
}

impl Element {
    /**
     * load all element under this path
     */
    async fn load() -> Result<(), String> {
        Ok(())
    }

    /**
     *  get the whole node in this file tree
     *  @param path the path of the node to be fetched
     */
    async fn get_element(path: &Path) -> Option<Element> {
        None
    }

    // /**
    //  * fetch current path‘s sub directories
    //  */
    // async fn load_children<'a> (&self) -> Option<&[Element]> {
    //     if let FileType::Directory = self.file_type {
    //         let mut children = self.children.lock().await;
    //         if let Some(children) = &mut *children {
    //             return Some(children.as_slice());
    //         } else {
    //             let mut new_data :Vec<Element>= vec![];
    //             if let Ok(entries) = fs::read_dir(&self.path) {
    //                 for entry in entries {
    //                     let entry = entry.unwrap();
    //                     let path = entry.path();
    //                     let metadata = entry.metadata().unwrap();
    //                 }
    //             }

    //             *children = Some(new_data);
    //             return &*children.map(|item|  item.as_slice());
    //         }
    //     }  else {
    //         return None;
    //     }
    // }
}

#[command]
/**
 * Get the list of drivers provided by the operation
 */
pub async fn get_available_drivers(
    state: State<'_, Mutex<Scanner>>,
) -> Result<Vec<Volumn>, String> {
    let scanner = state.lock().await;

    // Please note that we use "new_all" to ensure that all lists of
    // CPUs and processes are filled!
    let mut sys = System::new_all();

    // First we update all information of our `System` struct.
    sys.refresh_all();

    println!("=> system:");
    // RAM and swap information:
    println!("total memory: {} bytes", sys.total_memory());
    println!("used memory : {} bytes", sys.used_memory());
    println!("total swap  : {} bytes", sys.total_swap());
    println!("used swap   : {} bytes", sys.used_swap());

    // Display system information:
    println!("System name:             {:?}", System::name());
    println!("System kernel version:   {:?}", System::kernel_version());
    println!("System OS version:       {:?}", System::os_version());
    println!("System host name:        {:?}", System::host_name());

    // We display all disks' information:
    let disks = Disks::new_with_refreshed_list();
    debug!("System disks size: {:?}", disks.list().len());
    let mut volumns: Vec<Volumn> = vec![];
    for disk in &disks {
        let full_path = disk.mount_point();

        let volumn = Volumn {
            name: disk.name().to_string_lossy().into_owned(),
            path: full_path.to_path_buf(),
            total_size: disk.total_space(),
            available_size: disk.available_space(),
        };

        debug!("full path {:?}, info:{:?}", full_path, volumn);

        volumns.push(volumn);
    }
    return Ok(volumns);
}
