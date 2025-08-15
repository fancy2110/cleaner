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
