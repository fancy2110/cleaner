use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
    hash::Hash,
    ops::DerefMut,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use tracing::{debug, warn};

use crate::tree::node::{Node, NodeRef};

pub mod node;

#[derive(Debug)]
pub struct Tree<K, T>
where
    K: PartialEq + Hash + Clone,
{
    pub(crate) root: Option<NodeRef<K, T>>,
    pub(crate) nodes: HashMap<K, NodeRef<K, T>>,
}

impl<K, T> Tree<K, T>
where
    K: Eq + Hash + Clone,
{
    pub fn from_value(key: K, value: T) -> Tree<K, T> {
        let node = Node::new(key, value);
        Self::from_node(node)
    }

    pub fn from_node(node: Node<K, T>) -> Tree<K, T> {
        let key = node.key.clone();
        let node = Arc::new(RwLock::new(node));
        Tree {
            root: Some(node.clone()),
            nodes: HashMap::from([(key, node.clone())]),
        }
    }

    pub fn insert(&mut self, parent: &K, key: K, value: T) -> Result<NodeRef<K, T>, String> {
        self.insert_node(parent, Node::new(key, value))
    }

    pub fn insert_node(&mut self, parent: &K, value: Node<K, T>) -> Result<NodeRef<K, T>, String> {
        let node_key = value.key.clone();
        debug!("enter insert node, for parent");
        let node = match self.nodes.get_mut(parent) {
            Some(parent) => {
                let node = match parent.write() {
                    Ok(mut parent) => parent.add_child(value),
                    Err(e) => {
                        return Err(format!(
                            "insert node failed, parent write failed, err: {:?}",
                            e
                        ));
                    }
                };

                match node.write() {
                    Ok(mut node) => {
                        node.parent = Some(parent.clone());
                    }
                    Err(e) => {
                        return Err(format!(
                            "insert node failed, parent write failed, err: {:?}",
                            e
                        ));
                    }
                }
                node
            }
            None => {
                return Err("insert node failed, parent not found".to_string());
            }
        };

        debug!("enter insert node, for node");
        self.nodes.insert(node_key, node.clone());
        Ok(node)
    }

    pub fn remove(&mut self, key: &K) -> Result<Vec<NodeRef<K, T>>, String> {
        let nodes: Vec<NodeRef<K, T>> = match self.nodes.remove(key) {
            Some(value) => {
                let parent = value.write().unwrap().parent.take();
                if let Some(node_ref) = parent {
                    let mut node = node_ref.write().unwrap();
                    node.remove_child(key)
                } else {
                    vec![]
                }
            }
            None => vec![],
        };

        /*
         * remove all cache node from search map
         */
        for node in nodes.as_slice() {
            self.nodes.remove(&node.read().unwrap().key);
        }
        Ok(nodes)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.nodes.contains_key(key)
    }

    pub fn path_to_root(&self, key: &K) -> Vec<NodeRef<K, T>> {
        let mut path = vec![];
        let mut node = self.get_node(key);
        while let Some(value) = node {
            path.push(value.clone());
            node = match &value.read().unwrap().parent {
                Some(parent) => Some(parent.clone()),
                _ => break,
            }
        }
        path
    }

    pub fn get_node(&self, key: &K) -> Option<NodeRef<K, T>> {
        return self.nodes.get(key).cloned();
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }
}

impl<K, T> Drop for Tree<K, T>
where
    K: PartialEq + Hash + Clone,
{
    fn drop(&mut self) {
        println!("tree recycled");
        self.nodes.clear();
        self.root.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::RwLock;

    // 测试树的创建
    #[test]
    fn test_tree_creation() {
        // 从值创建树
        let tree: Tree<String, i32> = Tree::from_value("root".to_string(), 100);
        assert!(tree.root.is_some());
        assert_eq!(tree.size(), 1);
        assert!(tree.contains(&"root".to_string()));

        // 从节点创建树
        let node = Node::new("root2".to_string(), 200);
        let tree2 = Tree::from_node(node);
        assert!(tree2.root.is_some());
        assert_eq!(tree2.size(), 1);
        assert!(tree2.contains(&"root2".to_string()));
    }

    // 测试节点插入
    #[test]
    fn test_insert_node() {
        let mut tree: Tree<String, i32> = Tree::from_value("root".to_string(), 100);

        // 插入子节点
        let result = tree.insert(&"root".to_string(), "child1".to_string(), 101);
        assert!(result.is_ok());
        assert_eq!(tree.size(), 2);
        assert!(tree.contains(&"child1".to_string()));

        // 插入到不存在的父节点
        let result = tree.insert(&"non_existent".to_string(), "child2".to_string(), 102);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "insert node failed, parent not found".to_string()
        );
        assert_eq!(tree.size(), 2);
    }

    // 测试节点删除
    #[test]
    fn test_remove_node() {
        let mut tree: Tree<String, i32> = Tree::from_value("root".to_string(), 100);
        tree.insert(&"root".to_string(), "child1".to_string(), 101)
            .unwrap();
        tree.insert(&"child1".to_string(), "grandchild1".to_string(), 102)
            .unwrap();

        // 删除不存在的节点
        let result = tree.remove(&"non_existent".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
        assert_eq!(tree.size(), 3);

        // 删除叶节点
        let result = tree.remove(&"grandchild1".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
        assert_eq!(tree.size(), 2);
        assert!(!tree.contains(&"grandchild1".to_string()));

        // 删除带有子节点的节点
        let result = tree.remove(&"child1".to_string());
        assert!(result.is_ok());
        assert_eq!(tree.size(), 1);
        assert!(!tree.contains(&"child1".to_string()));
    }

    // 测试路径获取
    #[test]
    fn test_path_to_root() {
        let mut tree: Tree<String, i32> = Tree::from_value("root".to_string(), 100);
        tree.insert(&"root".to_string(), "child1".to_string(), 101)
            .unwrap();
        tree.insert(&"child1".to_string(), "grandchild1".to_string(), 102)
            .unwrap();

        // 获取从叶节点到根的路径
        let path = tree.path_to_root(&"grandchild1".to_string());
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].read().unwrap().key, "grandchild1".to_string());
        assert_eq!(path[1].read().unwrap().key, "child1".to_string());
        assert_eq!(path[2].read().unwrap().key, "root".to_string());

        // 获取从根节点到根的路径
        let path = tree.path_to_root(&"root".to_string());
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].read().unwrap().key, "root".to_string());

        // 获取不存在节点的路径
        let path = tree.path_to_root(&"non_existent".to_string());
        assert_eq!(path.len(), 0);
    }

    // 测试获取节点
    #[test]
    fn test_get_node() {
        let mut tree: Tree<String, i32> = Tree::from_value("root".to_string(), 100);
        tree.insert(&"root".to_string(), "child1".to_string(), 101)
            .unwrap();

        // 获取存在的节点
        let node = tree.get_node(&"child1".to_string());
        assert!(node.is_some());
        assert_eq!(node.unwrap().read().unwrap().key, "child1".to_string());

        // 获取不存在的节点
        let node = tree.get_node(&"non_existent".to_string());
        assert!(node.is_none());
    }

    // 测试节点值的访问
    #[test]
    fn test_node_value_access() {
        let node = Node::new("test".to_string(), 42);
        assert_eq!(node.get_value(), &42);
    }

    // 测试节点更新
    #[test]
    fn test_node_update() {
        let node = Node::new("test".to_string(), 42);
        let node_ref = Arc::new(RwLock::new(node));

        {
            let mut node = node_ref.write().unwrap();
            node.update(|value| *value = 100);
        }

        assert_eq!(node_ref.read().unwrap().get_value(), &100);
    }

    // 测试节点父子关系
    #[test]
    fn test_node_parent_child_relationship() {
        let mut tree: Tree<String, i32> = Tree::from_value("root".to_string(), 100);
        let child = tree
            .insert(&"root".to_string(), "child1".to_string(), 101)
            .unwrap();

        // 验证父节点关系
        assert!(child.read().unwrap().get_parent().is_some());
        assert_eq!(
            child
                .read()
                .unwrap()
                .get_parent()
                .unwrap()
                .read()
                .unwrap()
                .key,
            "root".to_string()
        );
    }
}
