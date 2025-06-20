use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
    hash::Hash,
    ops::DerefMut,
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub struct Node<K, T>
where
    K: PartialEq + Hash + Clone,
{
    pub(crate) key: K,
    value: T,
    pub(crate) children: Vec<NodeRef<K, T>>,
    pub(crate) parent: Option<NodeRef<K, T>>,
}

pub(crate) type NodeRef<K, T> = Arc<RwLock<Node<K, T>>>;

#[derive(Debug)]
pub enum UpdateMode {
    Child,
    Root,
}

impl<K, T> Node<K, T>
where
    K: PartialEq + Hash + Clone,
{
    pub fn new(key: K, value: T) -> Node<K, T> {
        Node {
            key,
            value,
            children: Vec::new(),
            parent: None,
        }
    }

    pub fn add_child(&mut self, child: Node<K, T>) -> NodeRef<K, T> {
        let node = Arc::new(RwLock::new(child));
        self.children.push(node.clone());
        node
    }

    /**
     *  remove all children from this node and return all nodes contains in this subtree
     */
    pub fn remove_child(&mut self, key: &K) -> Vec<NodeRef<K, T>> {
        let position = self
            .children
            .iter()
            .position(|item| item.read().unwrap().key == *key);

        println!("queue item position {:?}", position);
        let mut elems: Vec<NodeRef<K, T>> = vec![];
        if let Some(position) = position {
            let elem = self.children.remove(position);
            let mut queue: VecDeque<NodeRef<K, T>> = VecDeque::new();
            queue.push_back(elem);
            println!("queue size {}", queue.len());
            while let Some(node) = queue.pop_front() {
                let children: Vec<NodeRef<K, T>> =
                    node.write().unwrap().children.drain(0..).collect();
                queue.extend(children);
                elems.push(node.clone());
            }
        }

        elems
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }

    pub fn get_parent(&self) -> Option<NodeRef<K, T>> {
        self.parent.as_ref().map(|item| item.clone())
    }

    pub fn update<'a, F>(&mut self, mut modify: F)
    where
        F: FnMut(&mut T),
    {
        let value = &mut self.value;
        modify(value);
    }
}

impl<K, T> Drop for Node<K, T>
where
    K: PartialEq + Hash + Clone,
{
    fn drop(&mut self) {
        self.children.drain(0..);
        self.parent.take();
    }
}

impl<K, T> PartialEq for Node<K, T>
where
    K: PartialEq + Hash + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

struct RootIter<K, T>
where
    K: PartialEq + Hash + Clone,
{
    node: Option<NodeRef<K, T>>,
}

impl<K, T> Iterator for RootIter<K, T>
where
    K: PartialEq + Hash + Clone,
{
    type Item = NodeRef<K, T>;

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
