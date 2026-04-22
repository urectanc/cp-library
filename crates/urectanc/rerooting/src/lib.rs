use compressed_sparse_row::CSRArray;

pub trait TreeDP {
    type Value: Copy;

    type EdgeWeight: Copy + Default;

    fn identity(&self) -> Self::Value;

    fn merge(&self, x: &Self::Value, y: &Self::Value) -> Self::Value;

    fn add_edge(&self, x: &Self::Value, w: &Self::EdgeWeight) -> Self::Value;

    fn add_node(&self, x: &Self::Value, v: usize) -> Self::Value;
}

pub fn rerooting_dp<T: TreeDP>(
    edges: impl Iterator<Item = (usize, usize, T::EdgeWeight)>,
    tree_dp: T,
) -> Vec<T::Value> {
    let edges: Vec<_> = edges
        .into_iter()
        .flat_map(|(u, v, w)| [(u, (v, w)), (v, (u, w))])
        .collect();
    let n = edges.len() / 2 + 1;
    let graph = CSRArray::new(n, &edges);

    let root = 0;
    let mut bfs_order = Vec::with_capacity(n);
    bfs_order.push((root, !0));
    for i in 0..n {
        let (v, p) = bfs_order[i];
        for &(c, _) in graph[v].iter().filter(|&&(c, _)| c != p) {
            bfs_order.push((c, v));
        }
    }

    let mut dp = vec![tree_dp.identity(); n];
    let mut res = vec![tree_dp.identity(); n];

    for &(v, p) in bfs_order.iter().rev() {
        for &(c, ref w) in graph[v].iter().filter(|&&(c, _)| c != p) {
            dp[c] = tree_dp.add_edge(&tree_dp.add_node(&dp[c], c), w);
            dp[v] = tree_dp.merge(&dp[v], &dp[c]);
        }
    }

    let mut prefix = vec![];
    for &(v, p) in &bfs_order {
        let mut acc = res[v];
        for &(c, _) in graph[v].iter().filter(|&&(c, _)| c != p) {
            prefix.push(acc);
            acc = tree_dp.merge(&acc, &dp[c]);
        }
        res[v] = tree_dp.add_node(&acc, v);

        let mut racc = tree_dp.identity();
        for &(c, ref w) in graph[v].iter().filter(|&&(c, _)| c != p).rev() {
            let acc = prefix.pop().unwrap();
            res[c] = tree_dp.add_edge(&tree_dp.add_node(&tree_dp.merge(&acc, &racc), v), w);
            racc = tree_dp.merge(&dp[c], &racc);
        }
    }

    res
}
