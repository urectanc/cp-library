const PARENT_FLAG: usize = 1 << 30;

pub struct HeavyLightDecomposition {
    head_or_parent: Vec<usize>,
    index: Vec<usize>,
    pre_order: Vec<usize>,
    subtree_size: Vec<usize>,
}

impl HeavyLightDecomposition {
    pub fn from_edges<I>(edges: I, root: usize) -> Self
    where
        I: ExactSizeIterator<Item = (usize, usize)>,
    {
        let n = edges.len() + 1;

        let mut adj = vec![0; n];
        let mut deg = vec![0; n];
        for (u, v) in edges {
            adj[u] ^= v;
            adj[v] ^= u;
            deg[u] += 1;
            deg[v] += 1;
        }
        deg[root] = 0;

        let mut subtree_size = vec![1; n];
        let mut order = Vec::with_capacity(n);
        for mut v in 0..n {
            while deg[v] == 1 {
                let p = adj[v];
                adj[p] ^= v;
                deg[v] = 0;
                deg[p] -= 1;
                subtree_size[p] += subtree_size[v];
                order.push(v);
                v = p;
            }
        }
        order.push(root);

        let mut head_or_parent = adj;
        head_or_parent[root] = !0;
        let mut index = vec![0; n];
        let mut offset = vec![1; n];
        for &v in order.iter().rev().skip(1) {
            let p = head_or_parent[v];
            if offset[p] == 1 {
                let head = head_or_parent[p];
                head_or_parent[v] = if head & PARENT_FLAG == 0 { head } else { p };
            } else {
                head_or_parent[v] |= PARENT_FLAG;
            }
            index[v] = index[p] + offset[p];
            offset[p] += subtree_size[v];
        }

        for (v, &i) in index.iter().enumerate() {
            order[i] = v;
        }

        Self {
            head_or_parent,
            index,
            pre_order: order,
            subtree_size,
        }
    }

    pub fn pre_order(&self) -> &'_ [usize] {
        &self.pre_order
    }

    fn head(&self, v: usize) -> usize {
        let head = self.head_or_parent[v];
        if head & PARENT_FLAG == 0 { head } else { v }
    }

    pub fn parent(&self, v: usize) -> Option<usize> {
        let head = self.head_or_parent[v];
        if head == !0 {
            None
        } else if head & PARENT_FLAG == 0 {
            Some(self.pre_order[self.index(v) - 1])
        } else {
            Some(head ^ PARENT_FLAG)
        }
    }

    pub fn index(&self, v: usize) -> usize {
        self.index[v]
    }

    pub fn edge_index(&self, u: usize, v: usize) -> usize {
        self.index(u).max(self.index(v))
    }

    pub fn subtree_range(&self, v: usize) -> (usize, usize) {
        let l = self.index(v);
        (l, l + self.subtree_size[v])
    }

    pub fn is_ancestor(&self, ancestor: usize, descendant: usize) -> bool {
        let (l, r) = self.subtree_range(ancestor);
        (l..r).contains(&self.index(descendant))
    }

    pub fn la(&self, v: usize, mut d: usize) -> Option<usize> {
        let mut la = Some(v);

        while let Some(v) = la {
            let head = self.head(v);
            if self.index(v) - self.index(head) >= d {
                return Some(self.pre_order[self.index(v) - d]);
            }
            d -= self.index(v) - self.index(head) + 1;
            la = self.parent(head);
        }

        la
    }

    pub fn lca(&self, mut u: usize, mut v: usize) -> usize {
        if self.index(u) > self.index(v) {
            std::mem::swap(&mut u, &mut v);
        }

        if self.is_ancestor(u, v) {
            return u;
        }

        while self.index(u) < self.index(v) {
            v = self.parent(self.head(v)).unwrap();
        }

        v
    }

    pub fn dist(&self, u: usize, v: usize) -> usize {
        self.path_edges(u, v).map(|(l, r, _)| r - l).sum()
    }

    pub fn path_vertices(
        &self,
        u: usize,
        v: usize,
    ) -> impl Iterator<Item = (usize, usize, bool)> + '_ {
        self.path(u, v)
            .map(|(u, v, topdown, _)| (self.index(u), self.index(v) + 1, topdown))
    }

    pub fn path_edges(
        &self,
        u: usize,
        v: usize,
    ) -> impl Iterator<Item = (usize, usize, bool)> + '_ {
        self.path(u, v).map(|(u, v, topdown, last)| {
            (
                self.index(u) + usize::from(last),
                self.index(v) + 1,
                topdown,
            )
        })
    }

    fn path(&self, u: usize, v: usize) -> PathSegments<'_> {
        PathSegments {
            hld: self,
            u,
            v,
            exhausted: false,
        }
    }
}

pub struct PathSegments<'a> {
    hld: &'a HeavyLightDecomposition,
    u: usize,
    v: usize,
    exhausted: bool,
}

impl Iterator for PathSegments<'_> {
    // (u, v, topdown, last)
    // index(u) < index(v)
    type Item = (usize, usize, bool, bool);
    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }
        let Self { hld, u, v, .. } = *self;
        if hld.head(u) == hld.head(v) {
            self.exhausted = true;
            if hld.index(u) < hld.index(v) {
                Some((u, v, true, true))
            } else {
                Some((v, u, false, true))
            }
        } else {
            if hld.index(u) < hld.index(v) {
                let head = hld.head(v);
                self.v = hld.parent(head).unwrap();
                Some((head, v, true, false))
            } else {
                let head = hld.head(u);
                self.u = hld.parent(head).unwrap();
                Some((head, u, false, false))
            }
        }
    }
}
