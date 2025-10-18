use std::{
    ffi::OsString,
    fmt::Debug,
    path::{Component, PathBuf},
    sync::{Arc, RwLock},
};

use tracing::{debug, warn};
use tracing_futures::Instrument;

use crate::{
    driver::get_available_drivers,
    service::FileNode,
    tree::node::{Node, NodeRef, RootIter},
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

        let iter = RootIter {
            node: Some(new_node.clone()),
        };
        for node in iter {
            let parent = node.read().map_or(None, |node| node.parent.clone());
            if let Some(parent) = parent {
                match (parent.write(), new_node.read()) {
                    (Ok(mut parent_node), Ok(child_node)) => {
                        parent_node.count += child_node.total_count();
                    }
                    _ => (),
                }
            }
        }
        return Ok(new_node);
    }

    pub fn remove(&mut self, key: &PathBuf) -> Result<NodeRef, String> {
        return self
            .get_node(key)
            .map_or(Err(format!("{} not found", key.display())), |node| {
                if let Ok(node) = node.write()
                    && let Some(parent) = node.parent.as_ref()
                    && let Ok(mut parent) = parent.write()
                {
                    parent.children.retain(|child| {
                        child.read().map_or(true, |child| {
                            child.get_value().path != node.get_value().path
                        })
                    });
                }

                /*
                 * remove all cache node from search map
                 */
                Ok(node.clone())
            });
    }

    fn trace_to_root<'a, F>(&mut self, node: &NodeRef, mut modify: F)
    where
        F: FnMut(&NodeRef),
    {
        let mut iter = Some(node.clone());
        while let Some(value) = iter.as_ref() {
            modify(value);
            iter = value.read().map_or(None, |node| node.parent.clone());
        }
    }

    pub fn contains(&self, key: &PathBuf) -> bool {
        return self.get_node(key).is_some();
    }

    pub fn path_to_root(&self, key: &PathBuf) -> Vec<NodeRef> {
        let mut path = vec![];
        let mut node = self.get_node(key);
        while let Some(value) = node {
            path.push(value.clone());
            node = value.read().map_or(None, |node| node.parent.clone())
        }
        path
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
            if let Some(node) = current
                && let Ok(node) = node.read()
            {
                let mut target = None;
                for child in &node.children {
                    let is_ok = child
                        .read()
                        .map(|node| node.get_value().path.as_os_str() == path.as_os_str())
                        .is_ok();
                    if is_ok {
                        target = Some(child.clone());
                        break;
                    }
                }
                current = target;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::sync::RwLock;

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

    // 测试节点插入
    #[test]
    fn test_insert_node() {
        let path = PathBuf::from("/");
        let file = FileNode::new(path.clone().into_os_string(), true, false);
        // 从值创建树
        let mut tree: Tree = Tree::from_value(file);

        // 插入子节点
        let file1_path = PathBuf::from("/File2.txt");
        let file1 = FileNode::new(file1_path.clone().into_os_string(), false, false);
        let result = tree.insert(&path, file1);
        assert!(result.is_ok());
        assert_eq!(tree.size(), 2);
        assert!(tree.contains(&file1_path));

        // 插入到不存在的父节点
        let file2_path = PathBuf::from("/File3.txt");
        let file2 = FileNode::new(file2_path.clone().into_os_string(), false, false);
        let result = tree.insert(&PathBuf::from("nofile"), file2);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "parent not found, nofile".to_string());
        assert_eq!(tree.size(), 2);
    }

    // 测试节点删除
    // #[test]
    // fn test_remove_node() {
    //     let path = PathBuf::from("/");
    //     let root = FileNode::new(path.clone().into_os_string(), true, false);
    //     // 从值创建树
    //     let mut tree: Tree = Tree::from_value(root);

    //     tree.insert(&"root".to_string(), "child1".to_string(), 101)
    //         .unwrap();
    //     tree.insert(&"child1".to_string(), "grandchild1".to_string(), 102)
    //         .unwrap();

    //     // 删除不存在的节点
    //     let result = tree.remove(&"non_existent".to_string());
    //     assert!(result.is_ok());
    //     assert_eq!(result.unwrap().len(), 0);
    //     assert_eq!(tree.size(), 3);

    //     // 删除叶节点
    //     let result = tree.remove(&"grandchild1".to_string());
    //     assert!(result.is_ok());
    //     assert_eq!(result.unwrap().len(), 1);
    //     assert_eq!(tree.size(), 2);
    //     assert!(!tree.contains(&"grandchild1".to_string()));

    //     // 删除带有子节点的节点
    //     let result = tree.remove(&"child1".to_string());
    //     assert!(result.is_ok());
    //     assert_eq!(tree.size(), 1);
    //     assert!(!tree.contains(&"child1".to_string()));
    // }

    // 测试路径获取
    // #[test]
    // fn test_path_to_root() {
    //     let mut tree: Tree<String, i32> = Tree::from_value("root".to_string(), 100);
    //     tree.insert(&"root".to_string(), "child1".to_string(), 101)
    //         .unwrap();
    //     tree.insert(&"child1".to_string(), "grandchild1".to_string(), 102)
    //         .unwrap();

    //     // 获取从叶节点到根的路径
    //     let path = tree.path_to_root(&"grandchild1".to_string());
    //     assert_eq!(path.len(), 3);
    //     assert_eq!(path[0].read().unwrap().key, "grandchild1".to_string());
    //     assert_eq!(path[1].read().unwrap().key, "child1".to_string());
    //     assert_eq!(path[2].read().unwrap().key, "root".to_string());

    //     // 获取从根节点到根的路径
    //     let path = tree.path_to_root(&"root".to_string());
    //     assert_eq!(path.len(), 1);
    //     assert_eq!(path[0].read().unwrap().key, "root".to_string());

    //     // 获取不存在节点的路径
    //     let path = tree.path_to_root(&"non_existent".to_string());
    //     assert_eq!(path.len(), 0);
    // }

    // 测试构建深度为10且节点数少于100的树
    #[test]
    fn test_build_deep_tree() {
        // create root node
        let root_path = PathBuf::from("/");
        let root_file = FileNode::new(root_path.clone().into_os_string(), true, false);
        let mut tree: Tree = Tree::from_value(root_file);
        assert_eq!(tree.size(), 1);
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

        // let node = FileNode::new(item.as_os_str().to_string_lossy(), true, false);
        // tree.insert(node);
        assert_eq!(tree.size(), 111);

        // // 创建一个确定性的树结构，深度为10，节点数少于100
        // let mut total_nodes = 1; // 已经有一个根节点

        // // 创建主干路径 (一条深度为10的路径)
        // let mut current_path = root_path.clone();
        // // 每层的目录数和文件数，用于控制树的扩散程度
        // for depth in 1..=10 {
        //     if total_nodes >= 95 {
        //         break; // 确保不超过100个节点
        //     }
        //     let dir_name = format!("dir_{}", depth);
        //     let dir_node = FileNode::new(OsString::from(dir_name.clone()), true, false);
        //     let next_path = current_path.join(&dir_name);

        //     tree.insert(&current_path, dir_node)
        //         .expect("Failed to insert directory");
        //     total_nodes += 1;
        //     println!(
        //         "Added directory level_{}, total nodes: {}",
        //         depth, total_nodes
        //     );
        //     current_path = next_path;

        //     // 在每一层添加一些兄弟节点（文件和目录）
        //     let siblings_to_add = 12 - depth; // 越深层级添加越少的节点
        //     for i in 1..=siblings_to_add {
        //         // 添加一个目录
        //         let sibling_dir_name = format!("sibling_dir_{}_{}", depth, i);
        //         let sibling_dir_node =
        //             FileNode::new(OsString::from(sibling_dir_name.clone()), true, false);
        //         let sibling_path = current_path.parent().unwrap().join(&sibling_dir_name);

        //         tree.insert(
        //             &PathBuf::from(current_path.parent().unwrap()),
        //             sibling_dir_node,
        //         )
        //         .expect("Failed to insert sibling directory");
        //         total_nodes += 1;
        //         println!(
        //             "Added sibling directory {}_{}, total nodes: {}",
        //             depth, i, total_nodes
        //         );

        //         // 在每个兄弟目录中添加一些文件
        //         let files_to_add = 10 - depth;
        //         for j in 1..=files_to_add {
        //             if total_nodes >= 95 {
        //                 break; // 确保不超过100个节点
        //             }

        //             let file_name = format!("file_{}_{}_{}.txt", depth, i, j);
        //             let file_node = FileNode::new(OsString::from(file_name.clone()), false, false);

        //             tree.insert(&sibling_path, file_node)
        //                 .expect("Failed to insert file");
        //             total_nodes += 1;
        //             println!(
        //                 "Added file {}_{}_{}.txt, total nodes: {}",
        //                 depth, i, j, total_nodes
        //             );
        //         }
        //     }

        //     // 在主干路径的每一层也添加一些文件
        //     let files_in_main = 12 - depth;
        //     for j in 1..=files_in_main {
        //         if total_nodes >= 95 {
        //             break; // 确保不超过100个节点
        //         }

        //         let file_name = format!("main_file_{}_{}.txt", depth, j);
        //         let file_node = FileNode::new(OsString::from(file_name.clone()), false, false);

        //         tree.insert(&current_path, file_node)
        //             .expect("Failed to insert file in main path");
        //         total_nodes += 1;
        //         println!(
        //             "Added main file {}_{}.txt, total nodes: {}",
        //             depth, j, total_nodes
        //         );
        //     }

        //     if total_nodes >= 95 {
        //         break; // 确保不超过100个节点
        //     }
        // }

        // // 验证树的大小和性质
        // println!("Final node count from counter: {}", total_nodes);
        // println!("Final node count from tree.size(): {}", tree.size());
        // assert!(tree.size() < 100, "树的节点数应小于100");
        // assert!(tree.size() > 80, "树的节点数应接近100，至少大于80");

        // // 验证树的深度
        // let deepest_path = current_path.clone();
        // let path_to_root = tree.path_to_root(&deepest_path);

        // // 树的深度应该为11（1个根节点 + 10层）
        // assert_eq!(
        //     path_to_root.len(),
        //     11,
        //     "树的深度应该为11 (1个根节点 + 10层)"
        // );

        // println!(
        //     "构建的树有 {} 个节点，深度为 {}，最深路径为 {}",
        //     tree.size(),
        //     path_to_root.len(),
        //     deepest_path.display()
        // );
    }

    // // 测试获取节点
    // #[test]
    // fn test_get_node() {
    //     let mut tree: Tree<String, i32> = Tree::from_value("root".to_string(), 100);
    //     tree.insert(&"root".to_string(), "child1".to_string(), 101)
    //         .unwrap();

    //     // 获取存在的节点
    //     let node = tree.get_node(&"child1".to_string());
    //     assert!(node.is_some());
    //     assert_eq!(node.unwrap().read().unwrap().key, "child1".to_string());

    //     // 获取不存在的节点
    //     let node = tree.get_node(&"non_existent".to_string());
    //     assert!(node.is_none());
    // }

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

    // 测试节点父子关系
    // #[test]
    // fn test_node_parent_child_relationship() {
    //     let mut tree: Tree<String, i32> = Tree::from_value("root".to_string(), 100);
    //     let child = tree
    //         .insert(&"root".to_string(), "child1".to_string(), 101)
    //         .unwrap();

    //     // 验证父节点关系
    //     assert!(child.read().unwrap().get_parent().is_some());
    //     assert_eq!(
    //         child
    //             .read()
    //             .unwrap()
    //             .get_parent()
    //             .unwrap()
    //             .read()
    //             .unwrap()
    //             .key,
    //         "root".to_string()
    //     );
    // }
}
