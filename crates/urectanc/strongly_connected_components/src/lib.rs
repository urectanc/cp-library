use compressed_sparse_row::CSRArray;

/// Reference:
/// - [【SCC編】AtCoder Library 解読 〜Pythonでの実装まで〜](https://qiita.com/AkariLuminous/items/a2c789cebdd098dcb503)
/// - [実装コラム　非再帰 DFS](https://nachiavivias.github.io/cp-library/column/2022/01.html)
pub fn strongly_connected_components(
    n: usize,
    edges: impl AsRef<[(usize, usize)]>,
) -> CSRArray<usize> {
    let graph = CSRArray::new(n, edges.as_ref());

    let mut low = vec![!0; n];
    let mut stack = vec![];
    let mut pending = vec![];
    let mut id = 0;
    let mut scc_id = n;

    for v in 0..n {
        if low[v] != !0 {
            continue;
        }
        stack.push((v, 0, true));

        while let Some((current, edge_idx, scc_root)) = stack.last_mut() {
            if *edge_idx == 0 {
                low[*current] = id;
                id += 1;
            }

            if let Some(&next) = graph[*current].get(*edge_idx) {
                *edge_idx += 1;

                if low[next] == !0 {
                    stack.push((next, 0, true));
                } else {
                    if low[next] < low[*current] {
                        low[*current] = low[next];
                        *scc_root = false;
                    }
                }
            } else {
                let (current, _, scc_root) = stack.pop().unwrap();

                if let Some((parent, _, scc_root)) = stack.last_mut()
                    && low[*parent] > low[current]
                {
                    low[*parent] = low[current];
                    *scc_root = false;
                }

                if scc_root {
                    while let Some(v) = pending.pop_if(|&mut v| low[current] <= low[v]) {
                        low[v] = scc_id;
                    }
                    low[current] = scc_id;
                    scc_id += 1;
                } else {
                    pending.push(current);
                }
            }
        }
    }

    let scc = (0..n).map(|v| (scc_id - 1 - low[v], v)).collect::<Vec<_>>();
    CSRArray::new(scc_id - n, scc)
}
