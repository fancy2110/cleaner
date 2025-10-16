use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
    path::PathBuf,
    ptr::eq,
    sync::{Arc, RwLock},
    thread::panicking,
};

use crate::service::FileNode;

#[derive(Debug)]
pub struct Node {
    value: FileNode,                    //current file node info
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
    pub fn new(value: FileNode) -> Node {
        Node {
            value,
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
            .position(|item| item.read().unwrap().value.path == *key);

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

    pub fn get_value(&self) -> &FileNode {
        &self.value
    }

    pub fn get_name(&self) -> Result<String, String> {
        self.value
            .path
            .clone()
            .into_string()
            .map_err(|err| err.to_string_lossy().into_owned())
    }

    pub fn get_path(&self) -> PathBuf {
        let mut path = PathBuf::from(self.value.path.clone());

        let iter = RootIter {
            node: self.parent.clone(),
        };
        for node in iter {
            if let Ok(parent) = node.read() {
                let mut new_path = PathBuf::from(parent.value.path.clone());
                new_path.push(path);
                path = new_path;
            }
        }

        path
    }

    pub fn get_parent(&self) -> Option<NodeRef> {
        self.parent.as_ref().map(|item| item.clone())
    }

    pub fn clear(&mut self) {
        self.children.clear();
    }

    pub fn update<'a, F>(&mut self, mut modify: F)
    where
        F: FnMut(&mut FileNode),
    {
        let value = &mut self.value;
        modify(value);
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
        self.value.path == other.value.path
    }

    fn ne(&self, other: &Self) -> bool {
        return !eq(self, other);
    }
}

struct RootIter {
    node: Option<NodeRef>,
}

impl Iterator for RootIter {
    type Item = NodeRef;

    fn next(&mut self) -> Option<Self::Item> {
        let next = if let Some(cur) = &self.node {
            cur.read().unwrap().get_parent()
        } else {
            None
        };
        match &next {
            Some(next) => self.node = Some(next.clone()),
            None => self.node = None,
        }
        next
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
        let fileNode = FileNode::new(path.clone().into_os_string(), false, false);
        let node = Node::new(fileNode);
        assert_eq!(node.value.path, path);
        assert_eq!(node.get_name(), Ok(String::from("/test")));
        assert_eq!(node.count, 0);
    }

    #[test]
    fn test_node_insert() {
        let path = PathBuf::from("/root");
        let fileNode = FileNode::new(path.clone().into_os_string(), false, false);
        let mut node = Node::new(fileNode);

        let child_path = PathBuf::from("/root/child");
        let child_node = Node::new(FileNode::new(
            child_path.clone().into_os_string(),
            false,
            false,
        ));
        node.add_child(child_node);

        assert_eq!(node.count, 1);
    }

    //text multi nodes insert
    #[test]
    fn test_nodes_insert() {
        let path = PathBuf::from("/root");
        let fileNode = FileNode::new(path.clone().into_os_string(), false, false);
        let mut node = Node::new(fileNode);

        let files = vec!["file1.txt", "file2.txt", "file3.txt"];
        for name in files {
            let child_path = path.clone().join(name);
            let child_node = Node::new(FileNode::new(
                child_path.clone().into_os_string(),
                false,
                false,
            ));
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
        let fileNode = FileNode::new(root.clone().into_os_string(), false, false);
        let mut node = Node::new(fileNode);
        let files = vec!["file1.txt", "file2.txt", "file3.txt"];
        for name in files {
            let child_path = root.clone().join(name);
            let child_node = Node::new(FileNode::new(
                child_path.clone().into_os_string(),
                false,
                false,
            ));
            node.add_child(child_node);
        }

        root.push("dir1");
        let mut sub_node = Node::new(FileNode::new(root.clone().into_os_string(), false, false));
        let files = vec!["file1.bin", "file2.bin", "file3.bin"];
        for name in files {
            let child_path = root.clone().join(name);
            let child_node = Node::new(FileNode::new(
                child_path.clone().into_os_string(),
                false,
                false,
            ));
            sub_node.add_child(child_node);
        }
        node.add_child(sub_node);
        node
    }
}
