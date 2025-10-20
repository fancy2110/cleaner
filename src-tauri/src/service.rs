use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    ffi::{OsStr, OsString},
    fmt::Debug,
    fs::Metadata,
    path::{Component, PathBuf},
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::{
    fs,
    sync::mpsc::{self, Sender},
    task::JoinHandle,
};
use tracing::{debug, error, info, warn};

use crate::{
    model::FileDetails,
    tree::{self, Tree, node::Node},
};

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

type FileTree = Arc<RwLock<Tree>>;
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
            files: Arc::new(RwLock::new(Tree::from_node(Node::new(
                OsString::from("/"),
                true,
                false,
            )))),
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
        if let Ok(files) = self.files.read()
            && let Ok(mut queue) = self.queue.lock()
        {
            if let Some(root) = files.root.as_ref() {
                if let Ok(node) = root.read()
                    && let Ok(mut prog) = self.progress.lock()
                {
                    let path = node.get_path();
                    prog.current_path = Some(path);
                    prog.scaned_size = node.size;
                    prog.is_scanning = true;
                    queue.push_back(root.clone());
                } else {
                    return rx;
                }
            } else {
                return rx;
            };
        }

        let counter = Arc::new(AtomicUsize::new(0));

        for worker_id in 0..self.concurrency {
            let queue = Arc::clone(&self.queue);
            let tree = self.files.clone();
            let tx = tx.clone();
            let counter = Arc::clone(&counter);
            let interval = tokio::time::Duration::from_millis(50);

            let worker = tokio::spawn(async move {
                debug!("Worker {} started", worker_id);
                let tree = tree.clone();

                loop {
                    // Create a new scope for queue lock to ensure it's released before processing
                    let item = queue.lock().map_or(None, |mut queue| queue.pop_front());
                    let pendding_size = counter.fetch_sub(1, Ordering::Relaxed);
                    if pendding_size > 20 {
                        debug!("pending size is too large, size: {}", pendding_size);
                    }

                    if let Some(item) = item
                        && let Some(children) = Self::process_scan_item(&item).await
                    {
                        let progress = Self::update_parent_size(&tree, &item).await;
                        if let Ok(progress) = progress {
                            // let _ = tx.send(progress).await;
                        } else {
                            warn!("update parent size failed");
                        }

                        counter.fetch_add(children.len(), Ordering::Relaxed);
                        let _ = queue.lock().map(|mut queue| {
                            queue.extend(children);
                        });
                    } else {
                        debug!("Worker try to wait next job");
                        tokio::time::sleep(interval).await;
                    }
                }
            });

            self.workers.push(worker);
        }

        rx
    }

    pub async fn stop_scanning(&mut self) {
        info!("Stopping scan...");

        // Clear the queue
        let _ = self.queue.lock().map(|mut queue| queue.clear());

        // Abort all workers
        for worker in &self.workers {
            worker.abort();
        }
        self.workers.clear();

        // Reset progress
        let _ = self.progress.lock().map(|mut prog| {
            prog.is_scanning = false;
            prog.current_path = None;
        });
    }

    async fn process_scan_item(item: &TreeNode) -> Option<Vec<TreeNode>> {
        let inserted = item;

        let is_directory = inserted.read().is_ok_and(|node| node.is_directory);

        if is_directory {
            let path = Tree::path_to_root(&inserted).ok()?;
            if path == PathBuf::from("//System/Volumes/Data") {
                None
            } else {
                let children = Self::process_directory(path, inserted).await;
                children.ok()
            }
        } else {
            None
        }
    }

    async fn should_exit_scan(queue: &Arc<Mutex<VecDeque<TreeNode>>>) -> bool {
        let queue_size = queue.lock().map_or(0, |queue| queue.len());
        debug!("should exit scan size:{}", queue_size);
        queue_size == 0
    }

    /**
     * obtain file node from scan queue item and metadata
     * @param item
     * @param metadata
     * @return
     */
    fn obtain_file_node(name: OsString, metadata: &Metadata) -> Node {
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

        Node {
            path: name,
            size: metadata.len() as usize,
            is_directory: metadata.is_dir(),
            is_link: metadata.is_symlink(),
            modified,
            created,
            count: 0, //self is the first one
            children: Vec::new(),
            parent: None,
        }
    }

    async fn process_directory(
        dir_path: PathBuf,
        dir_node: &TreeNode,
    ) -> Result<Vec<TreeNode>, String> {
        let mut entries = match fs::read_dir(dir_path).await {
            Ok(entries) => entries,
            Err(e) => return Err(format!("{:?}", e)),
        };

        let mut children: Vec<TreeNode> = Vec::new();

        while let Ok(Some(entry)) = entries.next_entry().await {
            let metadata = entry.metadata().await;
            let file_type = entry.file_type().await;

            if let (Ok(file_type), Ok(metadata)) = (file_type, metadata) {
                let file_node = Self::obtain_file_node(entry.file_name(), &metadata);

                let node = dir_node.write().map(|mut node| {
                    node.size += file_node.size;

                    let new_node = node.add_child(file_node);
                    let _ = new_node.write().map(|mut node| {
                        node.parent = Some(dir_node.clone());
                    });
                    new_node
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
        Ok(children)
    }

    /**
     * update parent size from current node
     */
    async fn update_parent_size(tree: &FileTree, node: &TreeNode) -> Result<ScanProgress, String> {
        let new_size = node
            .read()
            .map_or(Err("file node value fetch failed"), |node| Ok(node.size))?;

        tree.write()
            .map_or(Err("Tree not found".to_string()), |mut tree| {
                tree.trace_to_root(&node, |parent| {
                    let _ = parent.write().map(|mut node| node.size += new_size);
                });
                Ok(())
            })?;

        if let Some(root) = tree.read().map_or(None, |tree| tree.root.clone())
            && let Ok(root) = root.read()
            && let Ok(node) = node.read()
        {
            Ok(ScanProgress {
                scaned_files: root.count,
                scaned_size: root.size,
                current_path: Some(node.get_path()),
                is_scanning: true,
            })
        } else {
            Err("Root node not found".to_string())
        }
    }

    pub async fn get_file_node(&self, path: &PathBuf) -> Option<FileDetails> {
        debug!("enter get file node for {:?}", path.display());
        let node = self.files.read().map_or(None, |node| node.get_node(path))?;

        let ret = node
            .read()
            .and_then(|node| {
                let mut current = FileDetails::from(&node);
                let sub_files = node.children.iter();
                debug!("sub files count:{}", node.children.len());
                let mut childrens = sub_files
                    .filter_map(|node| node.read().map(|node| FileDetails::from(&node)).ok())
                    .collect::<Vec<_>>();

                childrens.sort_by(|a, b| b.size.cmp(&a.size));

                current.children = Some(childrens);
                Ok(current)
            })
            .ok();

        debug!("exit get file node for {:?}", path.display());
        ret
    }

    pub async fn get_progress(&self) -> Result<ScanProgress, String> {
        self.progress
            .lock()
            .map_or(Err("Progress not initialized".to_string()), |prog| {
                Ok(prog.clone())
            })
    }

    pub async fn clear(&mut self) {
        debug!("clear scaner data");
        self.stop_scanning().await;
        let _ = self.queue.lock().map(|mut node| node.clear());
        self.files = Arc::new(RwLock::new(Tree::from_node(Node::new(
            PathBuf::from("/").into_os_string(),
            true,
            false,
        ))));

        let _ = self.progress.lock().map(|mut prog| prog.reset());
    }

    pub async fn is_scanning(&self) -> bool {
        self.progress.lock().is_ok_and(|prog| prog.is_scanning)
    }
}

impl Drop for Scanner {
    fn drop(&mut self) {
        for worker in &self.workers {
            worker.abort();
        }
    }
}
