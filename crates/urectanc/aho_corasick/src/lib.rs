mod small_set;

use std::ops::Index;

use small_set::{Mask, SmallSet};

pub struct AhoCorasick {
    nodes: Vec<Node>,
}

impl AhoCorasick {
    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    pub fn root(&self) -> usize {
        0
    }

    pub fn trans(&self, mut id: usize, c: u8) -> usize {
        loop {
            if let Some(next) = self.nodes[id].trans(c) {
                return next;
            }

            if id == self.root() {
                return self.root();
            }

            id = self[id].suffix();
        }
    }
}

impl Index<usize> for AhoCorasick {
    type Output = Node;
    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

pub struct Builder<const K: u8> {
    nodes: Vec<Node>,
}

impl<const K: u8> Builder<K> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        assert!(K as usize <= std::mem::size_of::<Mask>() * 8);
        let root = Node::new(!0);
        Self { nodes: vec![root] }
    }

    pub fn insert(&mut self, s: &[u8]) -> usize {
        let mut current = 0;

        for &c in s {
            current = self.nodes[current].trans(c).unwrap_or_else(|| {
                let next = self.nodes.len();
                self.nodes[current].next.insert(c, next as u32);
                self.nodes.push(Node::new(current));
                next
            });
        }

        current
    }

    pub fn build(self) -> AhoCorasick {
        let mut nodes = self.nodes;
        let root = 0;

        let mut que = std::collections::VecDeque::from([root]);
        while let Some(current) = que.pop_front() {
            let current_node = std::mem::take(&mut nodes[current]);
            for (c, next) in &current_node.next {
                let next = next as usize;
                let mut link = current_node.link as usize;
                while link != root && nodes[link].trans(c).is_none() {
                    link = nodes[link].link as usize;
                }
                nodes[next].link = nodes[link].trans(c).unwrap_or(root) as u32;
                que.push_back(next);
            }
            nodes[current] = current_node;
        }

        AhoCorasick { nodes }
    }
}

#[derive(Default)]
pub struct Node {
    parent: u32,
    next: SmallSet<u32>,
    link: u32,
}

impl Node {
    pub fn new(parent: usize) -> Self {
        Self {
            parent: parent as u32,
            next: SmallSet::new(),
            link: 0,
        }
    }

    pub fn prefix(&self) -> Option<usize> {
        Some(self.parent as usize).filter(|&p| p != !0)
    }

    pub fn suffix(&self) -> usize {
        self.link as usize
    }

    pub fn trans(&self, c: u8) -> Option<usize> {
        self.next.get(c).map(|next| next as usize)
    }

    pub fn next(&self) -> impl Iterator<Item = (u8, usize)> {
        self.next.iter().map(|(c, to)| (c, to as usize))
    }
}
