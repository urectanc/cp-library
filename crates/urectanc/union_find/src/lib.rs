pub struct UnionFind {
    parent_or_size: Vec<i32>,
}

impl UnionFind {
    pub fn new(size: usize) -> Self {
        Self {
            parent_or_size: vec![-1; size],
        }
    }

    pub fn leader(&mut self, v: usize) -> usize {
        if self.parent_or_size[v] < 0 {
            return v;
        }

        let p = self.parent_or_size[v] as usize;
        let leader = self.leader(p);
        self.parent_or_size[v] = leader as i32;
        leader
    }

    pub fn size(&mut self, v: usize) -> usize {
        let leader = self.leader(v);
        -self.parent_or_size[leader] as usize
    }

    pub fn same(&mut self, u: usize, v: usize) -> bool {
        self.leader(u) == self.leader(v)
    }

    pub fn merge(&mut self, u: usize, v: usize) -> Option<(usize, usize)> {
        let mut u = self.leader(u);
        let mut v = self.leader(v);
        if u == v {
            return None;
        }

        if self.size(u) > self.size(v) {
            std::mem::swap(&mut u, &mut v);
        }

        self.parent_or_size[v] += std::mem::replace(&mut self.parent_or_size[u], v as i32);
        Some((v, u))
    }

    pub fn groups(&mut self) -> Vec<Vec<usize>> {
        let n = self.parent_or_size.len();
        let mut id = vec![!0; n];
        let mut groups = vec![];
        for v in 0..n {
            let leader = self.leader(v);
            if id[leader] == !0 {
                id[leader] = groups.len();
                groups.push(vec![]);
            }
            groups[id[leader]].push(v);
        }
        groups
    }
}
