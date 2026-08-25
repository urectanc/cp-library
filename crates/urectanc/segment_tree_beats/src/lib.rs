use std::ops::RangeBounds;

use clamp_range::ClampRange;

pub mod range_chmin_chmax_add_range_sum;

pub trait BeatsMonoid {
    type Elem: Clone;
    type Map;

    fn identity() -> Self::Elem;
    fn op(lhs: &Self::Elem, rhs: &Self::Elem) -> Self::Elem;

    fn apply(x: &mut Self::Elem, f: &Self::Map) -> bool;
    fn resolve(parent: &Self::Elem, child: &mut Self::Elem);
    fn clear(x: &mut Self::Elem);
}

pub struct SegmentTreeBeats<M: BeatsMonoid> {
    #[allow(unused)]
    n: usize,
    offset: usize,
    tree: Vec<M::Elem>,
}

impl<M: BeatsMonoid> SegmentTreeBeats<M> {
    pub fn new(n: usize) -> Self {
        std::iter::repeat_n(M::identity(), n).collect()
    }

    pub fn fold(&mut self, range: impl RangeBounds<usize>) -> M::Elem {
        let (l, r) = range.clamp(0, self.n);
        let l = l + self.offset;
        let r = r + self.offset;
        let height = self.offset.trailing_zeros();

        for k in (1..=height).rev() {
            if ((l >> k) << k) != l {
                self.push(l >> k);
            }
            if ((r >> k) << k) != r {
                self.push(r >> k);
            }
        }

        let mut acc = M::identity();
        let l = l - 1;
        for k in (0..(l ^ r).ilog2()).rev() {
            if l >> k & 1 == 0 {
                acc = M::op(&self.tree[l >> k ^ 1], &acc);
            }
            if r >> k & 1 == 1 {
                acc = M::op(&acc, &self.tree[r >> k ^ 1]);
            }
        }
        acc
    }

    pub fn apply(&mut self, range: impl RangeBounds<usize>, f: M::Map) {
        let (l, r) = range.clamp(0, self.n);
        let (l, r) = (l + self.offset, r + self.offset);
        let height = self.offset.trailing_zeros();

        for k in (1..=height).rev() {
            if ((l >> k) << k) != l {
                self.push(l >> k);
            }
            if ((r >> k) << k) != r {
                self.push(r >> k);
            }
        }

        {
            let (mut l, mut r) = (l - 1, r);
            for _ in 0..(l ^ r).ilog2() {
                if l & 1 == 0 {
                    self.apply_subtree(l ^ 1, &f);
                }
                if r & 1 == 1 {
                    self.apply_subtree(r ^ 1, &f);
                }
                l >>= 1;
                r >>= 1;
            }
        }

        for k in 1..=height {
            if ((l >> k) << k) != l {
                self.update(l >> k);
            }
            if ((r >> k) << k) != r {
                self.update(r >> k);
            }
        }
    }

    fn update(&mut self, i: usize) {
        self.tree[i] = M::op(&self.tree[2 * i], &self.tree[2 * i + 1]);
    }

    fn apply_subtree(&mut self, i: usize, f: &M::Map) {
        if !M::apply(&mut self.tree[i], f) {
            assert!(i < self.offset);
            self.push(i);
            self.apply_subtree(2 * i, f);
            self.apply_subtree(2 * i + 1, f);
            self.update(i);
        }
    }

    fn push(&mut self, i: usize) {
        debug_assert!(i > 0);
        let [p, l, r] = self.tree.get_disjoint_mut([i, 2 * i, 2 * i + 1]).unwrap();
        M::resolve(p, l);
        M::resolve(p, r);
        M::clear(p);
    }
}

impl<M, T> From<T> for SegmentTreeBeats<M>
where
    M: BeatsMonoid,
    T: AsRef<[M::Elem]>,
{
    fn from(value: T) -> Self {
        let a = value.as_ref();
        let n = a.len();
        let offset = n.next_power_of_two();
        let mut tree = vec![M::identity(); 2 * offset];
        tree[offset..][..n].clone_from_slice(a);
        let mut beats = Self { n, offset, tree };
        for i in (1..offset).rev() {
            beats.update(i);
        }
        beats
    }
}

impl<M> FromIterator<M::Elem> for SegmentTreeBeats<M>
where
    M: BeatsMonoid,
{
    fn from_iter<I: IntoIterator<Item = M::Elem>>(iter: I) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}
