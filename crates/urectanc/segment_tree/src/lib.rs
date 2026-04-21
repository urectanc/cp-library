//! # Reference
//! - [お手軽非再帰 Segment Tree の書き方 - HackMD](https://hackmd.io/@tatyam-prime/rkA5wJMdo)

use std::ops::RangeBounds;

use algebra::Monoid;
use clamp_range::ClampRange;

pub struct SegmentTree<M: Monoid> {
    len: usize,
    offset: usize,
    tree: Vec<M::Elem>,
}

impl<M: Monoid> SegmentTree<M> {
    pub fn new(len: usize) -> Self {
        let offset = len.next_power_of_two();
        let tree = vec![M::identity(); 2 * offset];
        Self { len, offset, tree }
    }

    pub fn to_vec(&self) -> Vec<M::Elem> {
        self.tree[self.offset..][..self.len].to_vec()
    }

    pub fn get(&self, i: usize) -> M::Elem {
        self.tree[i + self.offset].clone()
    }

    pub fn set(&mut self, mut i: usize, val: M::Elem) {
        i += self.offset;
        self.tree[i] = val;
        while i > 1 {
            i >>= 1;
            self.update(i);
        }
    }

    pub fn prod(&self, range: impl RangeBounds<usize>) -> M::Elem {
        let (mut l, mut r) = range.clamp(0, self.len);
        if (l, r) == (0, self.len) {
            return self.tree[1].clone();
        }

        (l, r) = (l + self.offset, r + self.offset);
        let mut acc_l = M::identity();
        let mut acc_r = M::identity();
        while l < r {
            if l & 1 == 1 {
                acc_l = M::op(&acc_l, &self.tree[l]);
                l += 1;
            }
            if r & 1 == 1 {
                r -= 1;
                acc_r = M::op(&self.tree[r], &acc_r);
            }
            (l, r) = (l >> 1, r >> 1);
        }

        M::op(&acc_l, &acc_r)
    }

    pub fn max_right(&self, l: usize, f: impl Fn(&M::Elem) -> bool) -> usize {
        let mut r = l;
        let (mut i, mut width) = (r + self.offset, 1);
        let mut acc = M::identity();
        assert!(f(&acc));

        while r + width <= self.len {
            if i & 1 == 1 {
                let next_acc = M::op(&acc, &self.tree[i]);
                if !f(&next_acc) {
                    break;
                }
                acc = next_acc;
                (r, i) = (r + width, i + 1);
            }
            (i, width) = (i >> 1, width << 1);
        }

        while width > 1 {
            (i, width) = (i << 1, width >> 1);
            if r + width <= self.len {
                let next_acc = M::op(&acc, &self.tree[i]);
                if f(&next_acc) {
                    acc = next_acc;
                    (r, i) = (r + width, i + 1);
                }
            }
        }

        r
    }

    pub fn min_left(&self, r: usize, f: impl Fn(&M::Elem) -> bool) -> usize {
        let mut l = r;
        let (mut i, mut width) = (l + self.offset, 1);
        let mut acc = M::identity();
        assert!(f(&acc));

        while l >= width {
            if i & 1 == 1 {
                let next_acc = M::op(&self.tree[i - 1], &acc);
                if !f(&next_acc) {
                    break;
                }
                acc = next_acc;
                (l, i) = (l - width, i - 1);
            }
            (i, width) = (i >> 1, width << 1);
        }

        while width > 1 {
            (i, width) = (i << 1, width >> 1);
            if l >= width {
                let next_acc = M::op(&self.tree[i - 1], &acc);
                if f(&next_acc) {
                    acc = next_acc;
                    (l, i) = (l - width, i - 1);
                }
            }
        }

        l
    }

    fn update(&mut self, i: usize) {
        self.tree[i] = M::op(&self.tree[2 * i], &self.tree[2 * i + 1]);
    }
}

impl<M, T> From<T> for SegmentTree<M>
where
    M: Monoid,
    T: AsRef<[M::Elem]>,
{
    fn from(value: T) -> Self {
        let a = value.as_ref();
        let len = a.len();
        let offset = len.next_power_of_two();

        let mut tree = vec![M::identity(); 2 * offset];
        tree[offset..][..len].clone_from_slice(a);

        let mut segtree = Self { len, offset, tree };
        for i in (1..offset).rev() {
            segtree.update(i);
        }

        segtree
    }
}
