use compressed_sparse_row::CSRArray;

pub struct LowLink {
    parent: Vec<usize>,
    dfs_order: Vec<usize>,
    index: Vec<usize>,
    low: Vec<usize>,
}

impl LowLink {
    pub fn new(n: usize, edges: &[(usize, usize)]) -> Self {
        let graph = {
            let edges: Vec<_> = edges.iter().flat_map(|&(u, v)| [(u, v), (v, u)]).collect();
            CSRArray::new(n, &edges)
        };

        let mut parent = vec![!0; n];
        let mut dfs_order = Vec::with_capacity(n);
        let mut index = vec![!0; n];
        let mut stack = vec![];

        for root in 0..n {
            if parent[root] != !0 {
                continue;
            }

            stack.push((root, !0));
            while let Some((current, prev)) = stack.pop() {
                if index[current] != !0 {
                    continue;
                }

                parent[current] = prev;
                index[current] = dfs_order.len();
                dfs_order.push(current);

                for &next in &graph[current] {
                    if next != prev {
                        stack.push((next, current));
                    }
                }
            }
        }

        let mut used = vec![false; n];
        let mut low = index.clone();
        for &(mut u, mut v) in edges {
            if index[u] < index[v] {
                std::mem::swap(&mut u, &mut v);
            }
            if parent[u] == v && !used[u] {
                used[u] = true;
            } else {
                low[u] = low[u].min(index[v]);
            }
        }

        for &v in dfs_order.iter().rev() {
            let p = parent[v];
            if p != !0 {
                low[p] = low[p].min(low[v]);
            }
        }

        Self {
            parent,
            dfs_order,
            index,
            low,
        }
    }

    pub fn is_bridge(&self, mut u: usize, mut v: usize) -> bool {
        if self.index[u] < self.index[v] {
            std::mem::swap(&mut u, &mut v);
        }
        self.index[v] < self.low[u]
    }

    pub fn two_edge_connected_components(&self) -> CSRArray<usize> {
        let n = self.dfs_order.len();
        let mut id = vec![!0; n];
        let mut current_id = 0;

        for &v in &self.dfs_order {
            if self.low[v] == self.index[v] {
                id[v] = current_id;
                current_id += 1;
            } else {
                id[v] = id[self.parent[v]];
            }
        }

        let components = id
            .into_iter()
            .enumerate()
            .map(|(v, id)| (id, v))
            .collect::<Vec<_>>();

        CSRArray::new(current_id, &components)
    }

    pub fn biconnected_components(&self) -> CSRArray<usize> {
        let n = self.dfs_order.len();
        let mut id = vec![!0; n];
        let mut current_id = 0;
        let mut components = vec![];
        let mut seen = vec![false; n];

        for &v in &self.dfs_order {
            let p = self.parent[v];
            if p == !0 {
                continue;
            }
            if self.index[p] <= self.low[v] {
                id[v] = current_id;
                components.push((current_id, p));
                seen[p] = true;
                current_id += 1;
            } else {
                id[v] = id[p];
            }
            components.push((id[v], v));
            seen[v] = true;
        }

        for (v, &seen) in seen.iter().enumerate() {
            if !seen {
                components.push((current_id, v));
                current_id += 1;
            }
        }

        CSRArray::new(current_id, &components)
    }
}
