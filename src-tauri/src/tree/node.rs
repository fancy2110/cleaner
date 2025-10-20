use std::{
    collections::{HashMap, VecDeque},
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    path::PathBuf,
    ptr::eq,
    sync::{Arc, RwLock},
    thread::panicking,
};

#[derive(Debug)]
pub struct Node {
    pub path: OsString,
    pub size: usize,
    pub is_directory: bool,
    pub is_link: bool,
    pub modified: Option<u64>,
    pub created: Option<u64>,
    pub(crate) count: usize,            //total count of all sub nodes
    pub(crate) children: Vec<NodeRef>,  //all files and dirs in this node
    pub(crate) parent: Option<NodeRef>, //parent node reference
}

pub(crate) type NodeRef = Arc<RwLock<Node>>;

#[derive(Debug)]
pub enum UpdateMode {
    Child,
    Root,
}

impl Node {
    pub fn new(path: OsString, is_dir: bool, is_link: bool) -> Node {
        Node {
            path: path,
            size: 0,
            is_directory: is_dir,
            is_link: is_link,
            modified: None,
            created: None,
            count: 0, //self is the first one
            children: Vec::new(),
            parent: None,
        }
    }

    fn from(node: &Node) -> Node {
        Node {
            path: node.path.clone(),
            size: node.size,
            is_directory: node.is_directory,
            is_link: node.is_link,
            modified: node.modified,
            created: node.created,
            count: 0, //self is the first one
            children: Vec::new(),
            parent: None,
        }
    }

    pub fn add_child(&mut self, child: Node) -> NodeRef {
        self.count += child.count + 1; //include the inserted node
        let node = Arc::new(RwLock::new(child));
        self.children.push(node.clone());
        node
    }

    /**
     *  remove all children from this node and return all nodes contains in this subtree
     */
    pub fn remove_child(&mut self, key: &PathBuf) -> Vec<NodeRef> {
        let position = self
            .children
            .iter()
            .position(|item| item.read().unwrap().path == *key);

        println!("queue item position {:?}", position);
        let mut elems: Vec<NodeRef> = vec![];
        if let Some(position) = position {
            let elem = self.children.remove(position);
            let removed_count = elem.read().map_or(0, |node| node.count) + 1;
            self.count -= removed_count;
            let mut queue: VecDeque<NodeRef> = VecDeque::new();
            queue.push_back(elem);
            println!("queue size {}", queue.len());
            while let Some(node) = queue.pop_front() {
                let children: Option<Vec<NodeRef>> = node
                    .write()
                    .map(|mut node| node.children.drain(0..).collect())
                    .ok();
                if let Some(children) = children {
                    queue.extend(children);
                }
                elems.push(node.clone());
            }
        }

        elems
    }

    pub fn total_count(&self) -> usize {
        self.count + 1
    }

    pub fn get_name(&self) -> Result<String, String> {
        self.path
            .clone()
            .into_string()
            .map_err(|err| err.to_string_lossy().into_owned())
    }

    pub fn get_path(&self) -> PathBuf {
        PathBuf::from(self.path.clone())
    }

    pub fn get_parent(&self) -> Option<NodeRef> {
        self.parent.as_ref().map(|item| item.clone())
    }

    pub fn clear(&mut self) {
        self.children.clear();
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.children.drain(0..);
        self.parent.take();
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }

    fn ne(&self, other: &Self) -> bool {
        return !eq(self, other);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::NAN;
    use std::ffi::OsString;
    use std::path;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::sync::RwLock;

    #[test]
    fn test_node_creation() {
        let path = PathBuf::from("/test");
        let node = Node::new(path.clone().into_os_string(), false, false);
        assert_eq!(node.path, path);
        assert_eq!(node.get_name(), Ok(String::from("/test")));
        assert_eq!(node.count, 0);
    }

    #[test]
    fn test_node_insert() {
        let path = PathBuf::from("/root");
        let mut fileNode = Node::new(path.clone().into_os_string(), false, false);

        let child_path = PathBuf::from("/root/child");
        let child_node = Node::new(child_path.clone().into_os_string(), false, false);
        fileNode.add_child(child_node);

        assert_eq!(fileNode.count, 1);
    }

    //text multi nodes insert
    #[test]
    fn test_nodes_insert() {
        let path = PathBuf::from("/root");
        let mut node = Node::new(path.clone().into_os_string(), false, false);

        let files = vec!["file1.txt", "file2.txt", "file3.txt"];
        for name in files {
            let child_path = path.clone().join(name);
            let child_node = Node::new(child_path.clone().into_os_string(), false, false);
            node.add_child(child_node);
        }

        assert_eq!(node.count, 3);
    }

    #[test]
    fn test_node_remove() {
        let path = PathBuf::from("/root");
        let mut node = create_nodes(path.clone());
        assert_eq!(node.count, 7);

        let mut exist_path = path.clone();
        exist_path.push("file1.txt");
        let ret = node.remove_child(&exist_path);
        assert_eq!(ret.len(), 1);
        assert_eq!(node.count, 6);
    }

    fn create_nodes(mut root: PathBuf) -> Node {
        let mut node = Node::new(root.clone().into_os_string(), false, false);
        let files = vec!["file1.txt", "file2.txt", "file3.txt"];
        for name in files {
            let child_path = root.clone().join(name);
            let child_node = Node::new(child_path.clone().into_os_string(), false, false);
            node.add_child(child_node);
        }

        root.push("dir1");
        let mut sub_node = Node::new(root.clone().into_os_string(), false, false);
        let files = vec!["file1.bin", "file2.bin", "file3.bin"];
        for name in files {
            let child_path = root.clone().join(name);
            let child_node = Node::new(child_path.clone().into_os_string(), false, false);
            sub_node.add_child(child_node);
        }
        node.add_child(sub_node);
        node
    }
}
