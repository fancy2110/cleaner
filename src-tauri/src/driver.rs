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
}

#[command]
/**
 * Get the list of drivers provided by the operation
 */
pub async fn get_available_drivers(_: State<'_, Mutex<Scanner>>) -> Result<Vec<Volumn>, String> {
    // Please note that we use "new_all" to ensure that all lists of
    // CPUs and processes are filled!
    let mut sys = System::new_all();

    // First we update all information of our `System` struct.
    sys.refresh_all();

    debug!("=> system:");
    // RAM and swap information:
    debug!("total memory: {} bytes", sys.total_memory());
    debug!("used memory : {} bytes", sys.used_memory());
    debug!("total swap  : {} bytes", sys.total_swap());
    debug!("used swap   : {} bytes", sys.used_swap());

    // Display system information:
    debug!("System name:             {:?}", System::name());
    debug!("System kernel version:   {:?}", System::kernel_version());
    debug!("System OS version:       {:?}", System::os_version());
    debug!("System host name:        {:?}", System::host_name());

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
