use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt::Debug,
    fs::Metadata,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};
use tokio::sync::Mutex;
use tokio::{
    fs,
    sync::mpsc::{self, Sender},
    task::JoinHandle,
};
use tracing::{debug, error, info};

use crate::{model::FileDetails, tree::node::Node, tree::Tree};

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub size: u64,
    pub is_directory: bool,
    pub modified: Option<u64>,
    pub created: Option<u64>,
}

impl FileNode {
    fn new(path: PathBuf) -> FileNode {
        FileNode {
            path: path.to_path_buf(),
            size: 0,
            is_directory: false,
            modified: None,
            created: None,
        }
    }

    fn from(node: &FileNode) -> FileNode {
        FileNode {
            path: node.path.clone(),
            size: node.size,
            is_directory: node.is_directory,
            modified: node.modified,
            created: node.created,
        }
    }
}

#[derive(Debug)]
struct ScanQueueItem {
    path: PathBuf,
    parent: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub total_files: usize,
    pub total_directories: usize,
    pub total_size: u64,
    pub current_path: Option<PathBuf>,
    pub is_scanning: bool,
}

type FileTree = Arc<Mutex<Tree<PathBuf, FileNode>>>;
type TreeNode = Arc<RwLock<Node<PathBuf, FileNode>>>;

pub struct Scanner {
    /**
     *  waiting to scan item
     */
    queue: Arc<Mutex<VecDeque<ScanQueueItem>>>,
    /**
     *  the root path
     */
    root: FileTree,
    workers: Vec<JoinHandle<()>>,
    concurrency: usize,
    progress: Arc<Mutex<ScanProgress>>,
}

impl Scanner {
    pub fn new(concurrency: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            root: Arc::new(Mutex::new(Tree::from_value(
                PathBuf::from("/"),
                FileNode::new(PathBuf::from("/")),
            ))),
            workers: Vec::new(),
            concurrency: concurrency.max(1),
            progress: Arc::new(Mutex::new(ScanProgress {
                total_files: 0,
                total_directories: 0,
                total_size: 0,
                current_path: None,
                is_scanning: false,
            })),
        }
    }

    pub async fn add_to_queue(&mut self, path: PathBuf) {
        let mut nodes = self.root.lock().await;
        let node = match nodes.get_node(&path) {
            Some(node) => node.read().unwrap().key.clone(),
            None => {
                let node = FileNode::new(path.clone());
                nodes.insert(&path, path.clone(), node).unwrap();
                path
            }
        };

        let item = ScanQueueItem {
            path: node.clone(),
            parent: node.clone(),
        };
        self.queue.lock().await.push_back(item);
    }

    /**
     * begin scane path
     */
    pub async fn start(&mut self) -> mpsc::Receiver<FileNode> {
        let (tx, rx) = mpsc::channel(1000);
        let queue = Arc::clone(&self.queue);

        let item = queue.lock().await;
        debug!(
            "begin scan, path:{:?}, isScanning:{}",
            item.front(),
            self.workers.len()
        );

        // Clear existing workers
        self.workers.clear();
        let finish_count = Arc::new(AtomicUsize::new(self.concurrency));

        for worker_id in 0..self.concurrency {
            let queue = Arc::clone(&self.queue);
            let stats = Arc::clone(&self.root);
            let tx = tx.clone();
            let progress = Arc::clone(&self.progress);
            let finish_count = finish_count.clone();

            let worker = tokio::spawn(async move {
                debug!("Worker {} started", worker_id);

                loop {
                    let item = {
                        let mut queue = queue.lock().await;
                        queue.pop_front()
                    };

                    match item {
                        Some(item) => {
                            // Update current scanning path
                            {
                                let mut prog = progress.lock().await;
                                prog.current_path = Some(item.path.clone());
                                prog.is_scanning = true;
                            }

                            if let Err(e) =
                                Self::process_scan_item(item, &queue, &stats, &tx, &progress).await
                            {
                                error!("Error processing item: {}", e);
                            }
                        }
                        None => {
                            if Self::should_exit_scan(&queue).await {
                                debug!("Worker {} exiting", worker_id);
                                break;
                            }
                            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                        }
                    }
                }

                // Worker finished
                let mut prog = progress.lock().await;
                prog.is_scanning = false;
                prog.current_path = None;

                let count = finish_count.fetch_sub(1, Ordering::Relaxed);
                debug!("current runing workers {}", count - 1);
            });

            self.workers.push(worker);
        }

        rx
    }

    pub async fn stop_scanning(&mut self) {
        info!("Stopping scan...");

        // Clear the queue
        self.queue.lock().await.clear();

        // Abort all workers
        for worker in &self.workers {
            worker.abort();
        }
        self.workers.clear();

        // Reset progress
        let mut prog = self.progress.lock().await;
        prog.is_scanning = false;
        prog.current_path = None;
    }

    async fn process_scan_item(
        item: ScanQueueItem,
        queue: &Arc<Mutex<VecDeque<ScanQueueItem>>>,
        files: &FileTree,
        tx: &Sender<FileNode>,
        progress: &Arc<Mutex<ScanProgress>>,
    ) -> std::io::Result<()> {
        // Get metadata for the current item
        let metadata = fs::metadata(&item.path).await?;
        let new_node = Self::create_file_node(&item, &metadata);

        let parent = &item.parent;
        // Store stats and notify listeners
        // Self::store_and_notify_stats(&item.parent, &node, stats, tx, progress).await?;
        let mut files = files.lock().await;

        let new_node = files.insert(parent, new_node.path.clone(), new_node.clone());
        if let Ok(node) = new_node {
            Self::update_parent_sizes(node).await;
        }

        // Process directory contents if applicable
        if metadata.is_dir() {
            Self::process_directory(&item, queue).await?;
        }

        Ok(())
    }

    async fn should_exit_scan(queue: &Arc<Mutex<VecDeque<ScanQueueItem>>>) -> bool {
        let queue_size = queue.lock().await.len();
        debug!("should exit scan size:{}", queue_size);

        // Exit if queue is empty and we're not actively scanning
        queue_size == 0
    }

    fn create_file_node(item: &ScanQueueItem, metadata: &Metadata) -> FileNode {
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        let created = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        FileNode {
            path: item.path.clone(),
            size: if metadata.is_file() {
                metadata.len()
            } else {
                0
            },
            is_directory: metadata.is_dir(),
            modified,
            created,
        }
    }

    async fn store_and_notify_stats(
        parent: &PathBuf,
        node: &FileNode,
        stats: &Arc<Mutex<Tree<PathBuf, FileNode>>>,
        tx: &Sender<FileNode>,
        progress: &Arc<Mutex<ScanProgress>>,
    ) -> std::io::Result<()> {
        // Store stats
        {}

        // Update progress
        {
            let mut prog = progress.lock().await;
            if node.is_directory {
                prog.total_directories += 1;
            } else {
                prog.total_files += 1;
                prog.total_size += node.size;
            }
        }

        // Send update through channel
        if let Err(e) = tx.send(node.clone()).await {
            error!("Error sending stats update: {}", e);
        }

        Ok(())
    }

    async fn process_directory(
        item: &ScanQueueItem,
        queue: &Arc<Mutex<VecDeque<ScanQueueItem>>>,
    ) -> std::io::Result<()> {
        let mut entries = fs::read_dir(&item.path).await?;
        let mut children = Vec::new();

        while let Ok(Some(entry)) = entries.next_entry().await {
            children.push(ScanQueueItem {
                path: entry.path(),
                parent: item.path.clone(),
            });
        }

        // Add all children to queue at once
        if !children.is_empty() {
            let mut queue = queue.lock().await;
            queue.extend(children);
        }

        Ok(())
    }

    /**
     * update parent size from current node
     */
    async fn update_parent_sizes(node: TreeNode) {
        if let Ok(node) = node.read() {
            let new_value = node.get_value().size;
            let mut item = node.get_parent();
            while let Some(parent) = item {
                if let Ok(mut value) = parent.write() {
                    value.update(|node| node.size += new_value);
                    item = value.get_parent();
                } else {
                    item = None;
                }
            }
        }
    }

    // pub async fn update_directory_sizes(&self) {
    //     let stats_snapshot = self.nodes.lock().await.clone();

    //     // Build parent-child relationships
    //     let mut parent_children: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    //     for (path, file_stats) in &stats_snapshot {
    //         if let Some(parent) = &file_stats.parent {
    //             parent_children
    //                 .entry(parent.clone())
    //                 .or_insert_with(Vec::new)
    //                 .push(path.clone());
    //         }
    //     }

    //     // Calculate sizes recursively
    //     let mut updated_stats = stats_snapshot.clone();
    //     for (path, file_stats) in &stats_snapshot {
    //         if file_stats.is_directory {
    //             let size = Self::calculate_directory_size(&path, &stats_snapshot, &parent_children);
    //             if let Some(stats) = updated_stats.get_mut(path) {
    //                 stats.size = size;
    //             }
    //         }
    //     }

    //     // Update the main stats
    //     *self.nodes.lock().await = updated_stats;
    // }

    fn calculate_directory_size(
        dir_path: &Path,
        all_stats: &HashMap<PathBuf, FileNode>,
        parent_children: &HashMap<PathBuf, Vec<PathBuf>>,
    ) -> u64 {
        let mut total_size = 0;

        if let Some(children) = parent_children.get(dir_path) {
            for child_path in children {
                if let Some(child_stats) = all_stats.get(child_path) {
                    if child_stats.is_directory {
                        total_size +=
                            Self::calculate_directory_size(child_path, all_stats, parent_children);
                    } else {
                        total_size += child_stats.size;
                    }
                }
            }
        }

        total_size
    }

    pub async fn get_file_node(&self, path: &PathBuf) -> Option<FileNode> {
        let tree = self.root.lock().await;
        let node = tree.get_node(path).clone();
        match node {
            Some(node) => {
                if let Ok(node) = node.read() {
                    Some(FileNode::from(node.get_value()))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub async fn get_all_stats(&self) -> HashMap<PathBuf, FileNode> {
        HashMap::new()
    }

    pub async fn get_progress(&self) -> ScanProgress {
        self.progress.lock().await.clone()
    }

    pub async fn clear(&mut self) {
        self.stop_scanning().await;
        self.queue.lock().await.clear();
        self.root.lock().await.clear();
        *self.progress.lock().await = ScanProgress {
            total_files: 0,
            total_directories: 0,
            total_size: 0,
            current_path: None,
            is_scanning: false,
        };
    }

    pub async fn is_scanning(&self) -> bool {
        self.progress.lock().await.is_scanning
    }
}

impl Drop for Scanner {
    fn drop(&mut self) {
        for worker in &self.workers {
            worker.abort();
        }
    }
}
