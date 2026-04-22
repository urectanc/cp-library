//! # References
//!
//! - [Disjoint Sparse Table に乗るのはやはりモノイドかもしれない。](https://noshi91.hatenablog.com/entry/2023/04/07/165310)

use std::ops::RangeBounds;

use algebra::Monoid;
use clamp_range::ClampRange;

pub struct DisjointSparseTable<M: Monoid> {
    len: usize,
    table: Vec<M::Elem>,
}

impl<M, T> From<T> for DisjointSparseTable<M>
where
    M: Monoid,
    T: AsRef<[M::Elem]>,
{
    fn from(value: T) -> Self {
        let a = value.as_ref();
        let len = a.len();
        let n = len + 2;
        let h = (len + 1).ilog2() as usize + 1;

        let mut table = vec![M::identity(); n * h];
        for (k, table) in table.chunks_mut(n).enumerate().skip(1) {
            let w = 1 << k;

            for m in (w..n).step_by(2 * w) {
                let l = m - w;
                for i in (l..m - 1).rev() {
                    // table[i] = prod(i..m - 1)
                    table[i] = M::op(&a[i], &table[i + 1]);
                }

                let r = (m + w).min(n);
                for i in m + 1..r {
                    // table[i] = prod(m - 1..i - 1)
                    table[i] = M::op(&table[i - 1], &a[i - 2]);
                }
            }
        }

        Self { len, table }
    }
}

impl<M: Monoid> DisjointSparseTable<M> {
    pub fn prod(&self, range: impl RangeBounds<usize>) -> M::Elem {
        let (l, r) = range.clamp(0, self.len);
        let r = r + 1;
        let offset = (self.len + 2) * (l ^ r).ilog2() as usize;
        let table = &self.table[offset..];
        M::op(&table[l], &table[r])
    }
}
