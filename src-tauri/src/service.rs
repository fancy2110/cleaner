use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt::Debug,
    fs::Metadata,
    io,
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
use tracing::{debug, error, info, warn};

use crate::{model::FileDetails, tree::node::Node, tree::Tree};

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub size: usize,
    pub is_directory: bool,
    pub modified: Option<u64>,
    pub created: Option<u64>,
}

impl FileNode {
    fn new(path: PathBuf, is_dir: bool) -> FileNode {
        FileNode {
            path: path.to_path_buf(),
            size: 0,
            is_directory: is_dir,
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
    pub scaned_files: usize,
    pub scaned_size: usize,
    pub current_path: Option<PathBuf>,
    pub is_scanning: bool,
}
impl ScanProgress {
    fn reset(&mut self) {
        self.scaned_files = 0;
        self.scaned_size = 0;
        self.current_path = None;
        self.is_scanning = false;
    }
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
    files: FileTree,
    workers: Vec<JoinHandle<()>>,
    concurrency: usize,
    progress: Arc<Mutex<ScanProgress>>,
}

impl Scanner {
    pub fn new(concurrency: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            files: Arc::new(Mutex::new(Tree::from_value(
                PathBuf::from("/"),
                FileNode::new(PathBuf::from("/"), true),
            ))),
            workers: Vec::new(),
            concurrency: concurrency,
            progress: Arc::new(Mutex::new(ScanProgress {
                scaned_files: 0,
                scaned_size: 0,
                current_path: None,
                is_scanning: false,
            })),
        }
    }

    pub async fn add_to_queue(&mut self, path: PathBuf) {
        let mut nodes = self.files.lock().await;
        let node = match nodes.get_node(&path) {
            Some(node) => node.read().unwrap().key.clone(),
            None => {
                let node = FileNode::new(path.clone(), true);
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
    pub async fn start(&mut self) -> mpsc::Receiver<ScanProgress> {
        let (tx, rx) = mpsc::channel(1000);
        // Clear existing workers
        self.workers.clear();

        // Process root node to provide initial queue items
        let queue = self.queue.clone();
        let files = self.files.lock().await;

        {
            let path = if let Some(root) = files.root.as_ref() {
                let mut prog = self.progress.lock().await;
                if let Ok(node) = root.read() {
                    let file = node.get_value();
                    prog.current_path = Some(file.path.clone());
                    prog.scaned_size = file.size;
                    prog.is_scanning = true;
                    file.path.clone()
                } else {
                    return rx;
                }
            } else {
                return rx;
            };

            let _ = Self::process_directory(path, &queue).await;
        }

        let finish_count = Arc::new(AtomicUsize::new(self.concurrency));

        for worker_id in 0..self.concurrency {
            let queue = Arc::clone(&self.queue);
            let tree = Arc::clone(&self.files);
            let tx = tx.clone();
            let progress = Arc::clone(&self.progress);
            let finish_count = finish_count.clone();

            let worker = tokio::spawn(async move {
                debug!("Worker {} started", worker_id);
                let _max_count = 10000;
                let mut count = 0;

                loop {
                    if count >= _max_count {
                        break;
                    }

                    count += 1;

                    // Create a new scope for queue lock to ensure it's released before processing
                    let item = {
                        let mut queue = queue.lock().await;
                        queue.pop_front()
                    };

                    match item {
                        Some(item) => {
                            // Update current scanning path in a separate scope
                            {
                                let mut prog = progress.lock().await;
                                prog.current_path = Some(item.path.clone());
                            }

                            // Process the item with all locks released
                            if let Err(e) =
                                Self::process_scan_item(&tree, item, &queue, &tx, &progress).await
                            {
                                warn!("Error processing item: {}", e);
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

                // Worker finished - update progress in a separate scope
                {
                    let mut prog = progress.lock().await;
                    prog.is_scanning = false;
                    prog.current_path = None;
                }

                let count = finish_count.fetch_sub(1, Ordering::Relaxed);
                debug!("current running workers {}", count - 1);
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
        tree: &FileTree,
        item: ScanQueueItem,
        queue: &Arc<Mutex<VecDeque<ScanQueueItem>>>,
        tx: &Sender<ScanProgress>,
        progress: &Arc<Mutex<ScanProgress>>,
    ) -> Result<(), String> {
        let path = &item.path;
        // Get metadata for the current item
        let node = match fs::metadata(&path).await {
            io::Result::Ok(metadata) => Self::obtain_file_node(path.clone(), &metadata),
            io::Result::Err(msg) => return Err(format!("{:?}", msg)),
        };

        let parent = &item.parent;
        // Store stats and notify listeners
        // Self::store_and_notify_stats(&item.parent, &node, stats, tx, progress).await?;
        debug!(
            "before insert process scan item, path: {:?}, parent: {:?}",
            node.path, parent
        );
        let new_node = {
            let mut files = tree.lock().await;
            files.insert(parent, node.path.clone(), node.clone())
        };

        if let Ok(node) = new_node {
            Self::update_parent_size(node).await;
            if let Some(root) = Self::_get_file_node(tree, &PathBuf::from("/")).await {
                debug!("try to nofity files update");
                let _ = Self::notify_stats(root, &tx, &progress).await;
            }
        }

        debug!(
            "after insert process scan item, path: {:?}, parent: {:?}",
            node.path, parent
        );
        // Process directory contents if applicable
        if node.is_directory {
            Self::process_directory(item.path, queue).await?;
        }

        Ok(())
    }

    async fn should_exit_scan(queue: &Arc<Mutex<VecDeque<ScanQueueItem>>>) -> bool {
        let queue_size = queue.lock().await.len();
        debug!("should exit scan size:{}", queue_size);

        // Exit if queue is empty and we're not actively scanning
        queue_size == 0
    }

    /**
     * obtain file node from scan queue item and metadata
     * @param item
     * @param metadata
     * @return
     */
    fn obtain_file_node(path: PathBuf, metadata: &Metadata) -> FileNode {
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
            path: path,
            size: metadata.len() as usize,
            is_directory: metadata.is_dir(),
            modified,
            created,
        }
    }

    async fn notify_stats(
        node: FileNode,
        tx: &Sender<ScanProgress>,
        progress: &Arc<Mutex<ScanProgress>>,
    ) -> std::io::Result<()> {
        debug!("before notify files info, path: {:?}", node);
        // Update progress
        let mut prog = progress.lock().await;
        prog.scaned_files += 1;
        prog.scaned_size = node.size;
        let mut new_progress = prog.clone();
        new_progress.current_path = Some(node.path.clone());

        // Send update through channel
        if let Err(e) = tx.send(new_progress).await {
            error!("Error sending stats update: {}", e);
        }
        debug!("after notify files info, path: {:?}", node);

        Ok(())
    }

    async fn process_directory(
        item: PathBuf,
        queue: &Arc<Mutex<VecDeque<ScanQueueItem>>>,
    ) -> Result<(), String> {
        debug!("enter process_directory, path: {:?}", item);
        let mut entries = match fs::read_dir(&item).await {
            Ok(entries) => entries,
            Err(e) => return Err(format!("{:?}", e)),
        };
        let mut children = Vec::new();

        while let Ok(Some(entry)) = entries.next_entry().await {
            children.push(ScanQueueItem {
                path: entry.path(),
                parent: item.clone(),
            });
        }

        debug!(
            "before add {} children to scan queue for {}",
            children.len(),
            item.display()
        );
        // Add all children to queue at once
        if !children.is_empty() {
            let mut queue = queue.lock().await;
            queue.extend(children);
        }

        debug!("after add children to scan queue for {}", item.display());
        Ok(())
    }

    /**
     * update parent size from current node
     */
    async fn update_parent_size(node: TreeNode) {
        if let Ok(node) = node.read() {
            let new_size = node.get_value().size;
            debug!(
                "begain update parent sizes, key:{:?}, size:{:?}",
                node.key, new_size
            );
            let mut item = node.get_parent();
            while let Some(parent) = item {
                if let Ok(mut value) = parent.write() {
                    value.update(|node| node.size += new_size);
                    debug!(
                        "try to update parent sizes, key:{:?}, size:{:?}",
                        value.key,
                        value.get_value().size
                    );
                    item = value.get_parent();
                } else {
                    item = None;
                }
            }
            debug!("end update parent sizes, key:{:?}", node.key);
        } else {
            warn!("-------------update parent sizes lock failed---------------");
        }
    }

    async fn _get_file_node(tree: &FileTree, path: &PathBuf) -> Option<FileNode> {
        debug!("enter get file node for {:?}", path.display());
        let tree = tree.lock().await;
        let result = tree.get_node(path).map_or(None, |node| {
            node.read()
                .map(|node| FileNode::from(node.get_value()))
                .ok()
        });

        debug!("exit get file node for {:?}", path.display());
        result
    }

    pub async fn get_file_node(&self, path: &PathBuf) -> Option<FileDetails> {
        let tree = &self.files.lock().await;
        debug!("enter get file node for {:?}", path.display());

        let node = tree.get_node(path).or_else(|| {
            debug!("get file node failed, node not found");
            None
        })?;

        let ret = node
            .read()
            .and_then(|node| {
                let mut current = FileDetails::from(node.get_value().clone());
                let sub_files = node.children.iter();
                debug!("sub files count:{}", node.children.len());
                let mut childrens = sub_files
                    .filter_map(|node| {
                        node.read()
                            .map(|node| FileDetails::from(node.get_value().clone()))
                            .ok()
                    })
                    .collect::<Vec<_>>();

                childrens.sort_by(|a, b| b.size.cmp(&a.size));

                current.children = Some(childrens);
                Ok(current)
            })
            .ok();

        debug!("exit get file node for {:?}", path.display());
        ret
    }

    pub async fn get_progress(&self) -> ScanProgress {
        self.progress.lock().await.clone()
    }

    pub async fn clear(&mut self) {
        debug!("clear scaner data");
        self.stop_scanning().await;
        self.queue.lock().await.clear();
        self.files = Arc::new(Mutex::new(Tree::from_value(
            PathBuf::from("/"),
            FileNode::new(PathBuf::from("/"), true),
        )));

        let mut progress = self.progress.lock().await;
        progress.reset();
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
