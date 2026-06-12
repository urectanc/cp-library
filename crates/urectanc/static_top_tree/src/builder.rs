use std::collections::BinaryHeap;

use heavy_light_decomposition::HeavyLightDecomposition;

use super::*;

pub struct Builder<DP: TreeDP> {
    height: Vec<u32>,
    links: Vec<(NodeIndex, Link)>,
    path_tree: Vec<PathNode<DP>>,
    point_tree: Vec<PointNode<DP>>,
    vertex: Vec<VertexNode<DP>>,
}

impl<DP: TreeDP> Builder<DP> {
    pub fn new(vertex: Vec<DP::Vertex>) -> Self {
        let n = vertex.len();
        Self {
            height: vec![0; n],
            links: Vec::with_capacity(2 * n),
            vertex: vertex.into_iter().map(VertexNode::new).collect(),
            path_tree: vec![],
            point_tree: vec![],
        }
    }

    pub fn build(mut self, hld: &HeavyLightDecomposition) -> StaticTopTree<DP> {
        let child = hld.graph();

        for &v in hld.pre_order().iter().rev() {
            let mut heap: BinaryHeap<_> = child[v]
                .iter()
                .skip(1)
                .map(|&u| {
                    let path = self.build_path_tree(hld.heavy_path(u));
                    Cluster::new(path.index, DP::add_edge(&path.value), path.height)
                })
                .collect();

            for _ in 0..heap.len().saturating_sub(1) {
                let l = heap.pop().unwrap();
                let r = heap.pop().unwrap();
                let i = self.point_tree.len();
                self.links.push((l.index, Link::Point(i as u32 * 2)));
                self.links.push((r.index, Link::Point(i as u32 * 2 + 1)));
                heap.push(Cluster::new(
                    NodeIndex::Point(i),
                    DP::rake(&l.value, &r.value),
                    l.height.max(r.height) + 1,
                ));
                self.point_tree.push(PointNode::new(l.value, r.value));
            }

            if let Some(root) = heap.pop() {
                self.links.push((root.index, Link::Vertex(v as u32)));
                self.vertex[v].light_child = root.value;
                self.height[v] = root.height;
            }
        }

        let path = self.build_path_tree(hld.heavy_path(hld.root())).value;
        for &(index, link) in &self.links {
            match index {
                NodeIndex::Path(i) => self.path_tree[i].link = link,
                NodeIndex::Point(i) => self.point_tree[i].link = link,
                NodeIndex::Vertex(i) => self.vertex[i].link = link,
            }
        }

        StaticTopTree {
            path_tree: self.path_tree,
            point_tree: self.point_tree,
            vertex: self.vertex,
            dp: path,
        }
    }

    fn build_path_tree(&mut self, path: &[usize]) -> Cluster<DP::Path> {
        let mut path_cluster = PathTreeBuilder::<DP>::new(&mut self.path_tree, &mut self.links);
        for &v in path {
            path_cluster.add(
                NodeIndex::Vertex(v),
                DP::add_vertex(&self.vertex[v].light_child, &self.vertex[v].vertex),
                self.height[v],
            );
        }
        path_cluster.finish().unwrap()
    }
}

struct PathTreeBuilder<'a, DP: TreeDP> {
    tree: &'a mut Vec<PathNode<DP>>,
    links: &'a mut Vec<(NodeIndex, Link)>,
    stack: Vec<Cluster<DP::Path>>,
}

impl<'a, DP: TreeDP> PathTreeBuilder<'a, DP> {
    fn new(tree: &'a mut Vec<PathNode<DP>>, links: &'a mut Vec<(NodeIndex, Link)>) -> Self {
        Self {
            tree,
            links,
            stack: Vec::new(),
        }
    }

    fn add(&mut self, index: NodeIndex, value: DP::Path, height: u32) {
        self.stack.push(Cluster::new(index, value, height));
        loop {
            if let Some([l, m, r]) = self.stack.last_chunk::<3>()
                && (l == m || l >= r)
            {
                let last = self.stack.pop().unwrap();
                self.merge_last_two();
                self.stack.push(last);
            } else if let Some([l, r]) = self.stack.last_chunk::<2>()
                && l >= r
            {
                self.merge_last_two();
            } else {
                break;
            }
        }
    }

    fn finish(mut self) -> Option<Cluster<DP::Path>> {
        for _ in 0..self.stack.len().saturating_sub(1) {
            self.merge_last_two();
        }
        self.stack.pop()
    }

    fn merge_last_two(&mut self) {
        let r = self.stack.pop().unwrap();
        let l = self.stack.pop().unwrap();
        let i = self.tree.len();
        self.links.push((l.index, Link::Path(i as u32 * 2)));
        self.links.push((r.index, Link::Path(i as u32 * 2 + 1)));
        self.stack.push(Cluster::new(
            NodeIndex::Path(i),
            DP::compress(&l.value, &r.value),
            l.height.max(r.height) + 1,
        ));
        self.tree.push(PathNode::new(l.value, r.value));
    }
}

struct Cluster<T> {
    index: NodeIndex,
    value: T,
    height: u32,
}

impl<T> Cluster<T> {
    fn new(index: NodeIndex, value: T, height: u32) -> Self {
        Self {
            index,
            value,
            height,
        }
    }
}

impl<T> PartialOrd for Cluster<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Cluster<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.height.cmp(&self.height)
    }
}

impl<T> PartialEq for Cluster<T> {
    fn eq(&self, other: &Self) -> bool {
        self.height == other.height
    }
}

impl<T> Eq for Cluster<T> {}

#[derive(Clone, Copy)]
enum NodeIndex {
    Path(usize),
    Point(usize),
    Vertex(usize),
}
