use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    ffi::{OsStr, OsString},
    fmt::Debug,
    fs::Metadata,
    path::{Component, PathBuf},
    sync::{
        Arc, RwLock,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::sync::Mutex;
use tokio::{
    fs,
    sync::mpsc::{self, Sender},
    task::JoinHandle,
};
use tracing::{debug, error, info, warn};

use crate::{model::FileDetails, tree::Tree, tree::node::Node};

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: OsString,
    pub size: usize,
    pub is_directory: bool,
    pub is_link: bool,
    pub modified: Option<u64>,
    pub created: Option<u64>,
}

impl FileNode {
    pub fn new(path: OsString, is_dir: bool, is_link: bool) -> FileNode {
        FileNode {
            path: path,
            size: 0,
            is_directory: is_dir,
            is_link: is_link,
            modified: None,
            created: None,
        }
    }

    fn from(node: &FileNode) -> FileNode {
        FileNode {
            path: node.path.clone(),
            size: node.size,
            is_directory: node.is_directory,
            is_link: node.is_link,
            modified: node.modified,
            created: node.created,
        }
    }
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

type FileTree = Arc<Tree>;
type TreeNode = Arc<RwLock<Node>>;

pub struct Scanner {
    /**
     *  waiting to scan item
     */
    queue: Arc<Mutex<VecDeque<TreeNode>>>,
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
            files: Arc::new(Tree::from_value(FileNode::new(
                OsString::from("/"),
                true,
                false,
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

    /**
     * begin scane path
     */
    pub async fn start(&mut self) -> mpsc::Receiver<ScanProgress> {
        let (tx, rx) = mpsc::channel(1000);
        // Clear existing workers
        self.workers.clear();

        // Process root node to provide initial queue items
        {
            let mut queue = self.queue.lock().await;
            let files = self.files.clone();
            if let Some(root) = files.root.as_ref() {
                let mut prog = self.progress.lock().await;
                if let Ok(node) = root.read() {
                    let file = node.get_value();
                    let path = node.get_path();
                    prog.current_path = Some(path);
                    prog.scaned_size = file.size;
                    prog.is_scanning = true;
                    queue.push_back(root.clone());
                } else {
                    return rx;
                }
            } else {
                return rx;
            };
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
                let _max_count = 100000;
                // let _max_count = i32::MAX;
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
                            debug!("Worker try to wait next job");
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
        item: TreeNode,
        queue: &Arc<Mutex<VecDeque<TreeNode>>>,
        tx: &Sender<ScanProgress>,
        progress: &Arc<Mutex<ScanProgress>>,
    ) -> Result<(), String> {
        let inserted = item;

        let path = inserted
            .read()
            .map(|node| node.get_value().path.clone())
            .map_err(|err| format!("Failed to read tree node: {}", err))?;

        let is_directory = inserted
            .read()
            .map(|node| node.get_value().is_directory)
            .map_err(|err| format!("Failed to read tree node: {}", err))?;

        if is_directory {
            Self::process_directory(inserted.clone(), queue).await?;
            Self::update_parent_size(inserted.clone());
            //just update directory
            if let Some(root) = Self::_get_file_node(tree, &PathBuf::from("/")).await {
                debug!("try to nofity files update");
                let _ = Self::notify_stats(root, &tx, &progress).await;
            }
        }

        debug!("process scan item, path: {:?}", path);
        Ok(())
    }

    async fn should_exit_scan(queue: &Arc<Mutex<VecDeque<TreeNode>>>) -> bool {
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
            path: path.into_os_string(),
            size: metadata.len() as usize,
            is_directory: metadata.is_dir(),
            is_link: metadata.is_symlink(),
            modified,
            created,
        }
    }

    async fn notify_stats(
        node: FileNode,
        tx: &Sender<ScanProgress>,
        progress: &Arc<Mutex<ScanProgress>>,
    ) -> std::io::Result<()> {
        // Update progress
        let mut prog = progress.lock().await;
        prog.scaned_files += 1;
        prog.scaned_size = node.size;
        let mut new_progress = prog.clone();
        new_progress.current_path = Some(PathBuf::from(node.path.clone()));

        // Send update through channel
        if let Err(e) = tx.send(new_progress).await {
            error!("Error sending stats update: {}", e);
        }
        debug!("notify files info, path: {:?}", node);

        Ok(())
    }

    async fn process_directory(
        dir_node: TreeNode,
        queue: &Arc<Mutex<VecDeque<TreeNode>>>,
    ) -> Result<(), String> {
        let dir_path = {
            dir_node
                .write()
                .map(|node| node.get_path().clone())
                .map_err(|err| err.to_string())
        }?;

        debug!("enter process_directory, path: {:?}", dir_path.display());

        let mut entries = match fs::read_dir(dir_path.clone()).await {
            Ok(entries) => entries,
            Err(e) => return Err(format!("{:?}", e)),
        };

        let mut children: Vec<TreeNode> = Vec::new();

        while let Ok(Some(entry)) = entries.next_entry().await {
            let metadata = entry.metadata().await;
            let file_type = entry.file_type().await;

            if let (Ok(file_type), Ok(metadata)) = (file_type, metadata) {
                let mut path = dir_path.clone();
                path.push(entry.path()); //full path for file node

                let file_node = Self::obtain_file_node(path.clone(), &metadata);

                let node = dir_node.write().map(|mut node| {
                    node.update(|node| node.size += file_node.size);
                    node.add_child(Node::new(file_node))
                });

                match (node, file_type.is_dir()) {
                    (Ok(node), true) => {
                        children.push(node.clone());
                    }
                    _ => {}
                }
            }
        }

        // Add all children to queue at once
        if !children.is_empty() {
            debug!(
                "add {} children to scan queue for {}",
                children.len(),
                dir_path.display()
            );
            let mut queue = queue.lock().await;
            queue.extend(children);
        }
        Ok(())
    }

    /**
     * update parent size from current node
     */
    fn update_parent_size(node: TreeNode) {
        if let Ok(node) = node.read() {
            let new_size = node.get_value().size;
            debug!(
                "update parent sizes, key:{:?}, size:{:?}",
                node.get_name(),
                new_size
            );
            let mut item = node.get_parent();
            while let Some(parent) = item {
                if let Ok(mut value) = parent.write() {
                    value.update(|node| node.size += new_size);
                    item = value.get_parent();
                } else {
                    item = None;
                }
            }
        } else {
            warn!("-------------update parent sizes lock failed---------------");
        }
    }

    async fn _get_file_node(tree: &FileTree, path: &PathBuf) -> Option<FileNode> {
        let result = tree.get_node(path).map_or(None, |node| {
            node.read()
                .map(|node| FileNode::from(node.get_value()))
                .ok()
        });

        debug!("get file node for {:?}", path.display());
        result
    }

    pub async fn get_file_node(&self, path: &PathBuf) -> Option<FileDetails> {
        debug!("enter get file node for {:?}", path.display());

        let node = self.files.get_node(path).or_else(|| {
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
        self.files = Arc::new(Tree::from_value(FileNode::new(
            PathBuf::from("/").into_os_string(),
            true,
            false,
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
