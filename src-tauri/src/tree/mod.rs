use std::{
    ffi::OsString,
    fmt::Debug,
    path::{Component, PathBuf},
    sync::{Arc, RwLock, RwLockWriteGuard},
};

use tracing::{debug, warn};
use tracing_futures::Instrument;

use crate::{
    driver::get_available_drivers,
    service::FileNode,
    tree::node::{Node, NodeRef},
};

pub mod node;

#[derive(Debug)]
pub struct Tree {
    pub(crate) root: Option<NodeRef>,
}

impl Tree {
    pub fn from_value(value: FileNode) -> Tree {
        let node = Node::new(value);
        Self::from_node(node)
    }

    pub fn from_node(node: Node) -> Tree {
        let node = Arc::new(RwLock::new(node));
        Tree { root: Some(node) }
    }

    pub fn insert(&mut self, parent: &PathBuf, value: FileNode) -> Result<NodeRef, String> {
        self.insert_node(parent, Node::new(value))
    }

    pub fn insert_node(&mut self, parent: &PathBuf, value: Node) -> Result<NodeRef, String> {
        debug!("enter insert node, for parent");
        let parent_node = self
            .get_node(parent)
            .ok_or_else(|| format!("parent not found, {}", parent.display()))?;

        let new_node = {
            let mut parent = parent_node
                .write()
                .map_err(|err| format!("failed to write node, {}", err))?;

            let new_node = parent.add_child(value);

            let _ = new_node.write().map(|mut node| {
                node.parent = Some(parent_node.clone());
            });
            new_node
        };

        self.trace_to_root(&new_node, |parent| {
            if let Ok(mut parent) = parent.write()
                && let Ok(new_node) = new_node.read()
            {
                parent.count += new_node.total_count();
            }
        });

        return Ok(new_node);
    }

    pub fn remove(&mut self, key: &PathBuf) -> Result<NodeRef, String> {
        let target =
            self.get_node(key)
                .map_or(Err(format!("key:{} not found", key.display())), |node| {
                    println!(
                        "item on remove , target:{}",
                        node.read().unwrap().get_value().path.display()
                    );
                    Ok(node)
                })?;

        if let Ok(node) = target.read()
            && let Some(parent) = node.parent.as_ref()
            && let Ok(mut parent) = parent.write()
        {
            parent.children.retain(|child| {
                child.read().map_or(true, |child| {
                    child.get_value().path != node.get_value().path
                })
            });
        } else {
            return Err(format!("remove from parent failed"));
        }

        self.trace_to_root(&target, |parent| {
            if let Ok(mut parent) = parent.write()
                && let Ok(node) = target.read()
            {
                parent.count -= node.total_count();
            }
        });

        /*
         * remove all cache node from search map
         */
        Ok(target)
    }

    fn trace_to_root<'a, F>(&mut self, node: &NodeRef, mut modify: F)
    where
        F: FnMut(&NodeRef),
    {
        let iter = RootIter {
            node: Some(node.clone()),
        };
        for node in iter {
            let parent = node.read().map_or(None, |node| node.parent.clone());
            if let Some(parent) = parent {
                modify(&parent)
            }
        }
    }

    pub fn contains(&self, key: &PathBuf) -> bool {
        return self.get_node(key).is_some();
    }

    pub fn path_to_root(&self, node: &NodeRef) -> Result<PathBuf, String> {
        let mut path =
            node.read().map_or(
                Err("Node not found".to_string()),
                |node| Ok(node.get_path()),
            )?;

        let iter = RootIter {
            node: Some(node.clone()),
        };

        for node in iter {
            if let Ok(parent) = node.read() {
                let mut new_path = parent.get_path();
                new_path.push(path);
                path = new_path;
            }
        }

        Ok(path)
    }

    /***
     * find tree node with path
     */
    pub fn get_node(&self, key: &PathBuf) -> Option<NodeRef> {
        let mut paths = key.components();
        let mut current = self.root.clone();
        let root_of_path = paths.next()?;
        if root_of_path != Component::RootDir {
            return None;
        }

        for path in paths {
            println!("find node {:?}", path);
            if let Some(node) = current
                && let Ok(node) = node.read()
            {
                current = node
                    .children
                    .iter()
                    .find(|child| {
                        child
                            .read()
                            .map(|node| node.get_value().path == path.as_os_str())
                            .is_ok_and(|result| result)
                    })
                    .map(|node| node.clone());
            } else {
                current = None;
                break;
            }
        }
        return current;
    }

    pub fn size(&self) -> usize {
        self.root
            .as_ref()
            .map_or(0, |node| node.read().map_or(0, |node| node.count + 1))
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        println!("tree recycled");
        self.root.take();
    }
}

struct RootIter {
    pub(crate) node: Option<NodeRef>,
}

impl Iterator for RootIter {
    type Item = NodeRef;

    fn next(&mut self) -> Option<Self::Item> {
        let next = if let Some(cur) = &self.node
            && let Ok(node) = cur.read()
        {
            node.parent.clone()
        } else {
            None
        };
        self.node = next.clone();
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::sync::RwLock;

    fn build_test_tree() -> Tree {
        // create root node
        let root_path = PathBuf::from("/");
        let root_file = FileNode::new(root_path.clone().into_os_string(), true, false);
        let mut tree: Tree = Tree::from_value(root_file);
        //
        let paths: Vec<String> = (0..10).map(|i| format!("dir{}", i)).collect();
        let mut current_path = root_path.clone();
        for dir_name in &paths {
            println!("current:{}, new:{}", current_path.display(), dir_name);
            let dir_node = FileNode::new(OsString::from(dir_name.clone()), true, false);

            for i in 0..10 {
                let file_name = format!("file{}", i);
                let file_node = FileNode::new(OsString::from(file_name), false, false);
                let _ = tree.insert(&current_path, file_node);
            }

            let _ = tree.insert(&current_path, dir_node);
            current_path.push(dir_name.clone());
        }
        tree
    }

    // 测试树的创建
    #[test]
    fn test_tree_creation() {
        let path = PathBuf::from("/");
        let file = FileNode::new(path.clone().into_os_string(), true, false);
        // 从值创建树
        let mut tree: Tree = Tree::from_value(file);
        assert!(tree.root.is_some());
        assert_eq!(tree.size(), 1);
        assert!(tree.contains(&path));

        // 从节点创建树
        let file_path1 = PathBuf::from("File1.txt");
        let file1 = FileNode::new(file_path1.clone().into_os_string(), false, false);
        let _ = tree.insert(&path, file1);
        assert_eq!(tree.size(), 2);
        assert!(tree.contains(&PathBuf::from("/File1.txt")));
    }

    // 测试构建深度为10且节点数少于100的树
    #[test]
    fn test_build_deep_tree() {
        let tree = build_test_tree();
        assert_eq!(tree.size(), 111);
    }

    // 测试节点插入
    #[test]
    fn test_find_node() {
        let tree = build_test_tree();
        assert!(
            tree.get_node(&PathBuf::from("/dir0/dir1/dir2/file1"))
                .is_some()
        );
    }

    // 测试节点插入
    #[test]
    fn test_insert_node() {
        let mut tree = build_test_tree();

        let before_count = tree.size();
        // 插入子节点
        let file1_path = PathBuf::from("File2.txt");
        let file1 = FileNode::new(file1_path.clone().into_os_string(), false, false);
        let result = tree.insert(&PathBuf::from("/"), file1);
        assert!(result.is_ok());
        assert_eq!(tree.size(), before_count + 1);
        assert!(tree.contains(&PathBuf::from("/File2.txt")));

        let before_count = tree.size();
        // 插入到不存在的父节点
        let file2_path = PathBuf::from("File3.txt");
        let file2 = FileNode::new(file2_path.clone().into_os_string(), false, false);
        let result = tree.insert(&PathBuf::from("nofile"), file2);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "parent not found, nofile".to_string());
        assert_eq!(tree.size(), before_count);
    }

    ///测试节点删除
    #[test]
    fn test_remove_node() {
        let path = PathBuf::from("/");
        // 从值创建树
        let mut tree: Tree = build_test_tree();
        let before_size = tree.size();
        // 删除不存在的节点
        let result = tree.remove(&PathBuf::from("non_existent"));
        assert!(result.is_err());
        assert_eq!(tree.size(), before_size);

        let before_size = tree.size();
        // 删除叶节点
        let target_path = PathBuf::from("/dir0/dir1/dir2/file1");
        let result = tree.remove(&target_path);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().read().unwrap().total_count(),
            before_size - tree.size()
        );
        assert!(!tree.contains(&target_path));

        // 删除带有子节点的节点
        let before_size = tree.size();
        let target_path = PathBuf::from("/dir0/dir1/dir2/dir3");
        let result = tree.remove(&target_path);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().read().unwrap().total_count(),
            before_size - tree.size()
        );
        assert!(!tree.contains(&target_path));
    }

    ///测试路径获取
    #[test]
    fn test_path_to_root() {
        let tree = build_test_tree();

        let target_path = PathBuf::from("/dir0/dir1/dir2/dir3/file3");
        // 获取从叶节点到根的路径
        let node = tree.get_node(&target_path);
        assert!(node.is_some());

        let path = tree.path_to_root(&node.unwrap()).unwrap();
        assert_eq!(path, target_path);
    }

    // // 测试节点值的访问
    // #[test]
    // fn test_node_value_access() {
    //     let node = Node::new("test".to_string(), 42);
    //     assert_eq!(node.get_value(), &42);
    // }

    // // 测试节点更新
    // #[test]
    // fn test_node_update() {
    //     let node = Node::new("test".to_string(), 42);
    //     let node_ref = Arc::new(RwLock::new(node));

    //     {
    //         let mut node = node_ref.write().unwrap();
    //         node.update(|value| *value = 100);
    //     }

    //     assert_eq!(node_ref.read().unwrap().get_value(), &100);
    // }
}
