use std::ops::RangeBounds;

use algebra::MapMonoid;
use clamp_range::ClampRange;

pub struct LazySegmentTree<M: MapMonoid> {
    len: usize,
    offset: usize,
    height: usize,
    tree: Vec<M::Elem>,
    lazy: Vec<M::Map>,
}

impl<M: MapMonoid> LazySegmentTree<M> {
    pub fn new(len: usize) -> Self {
        Self::from(vec![M::identity(); len])
    }

    pub fn to_vec(&mut self) -> Vec<M::Elem> {
        for i in 1..self.offset {
            self.push(i);
        }
        self.tree[self.offset..][..self.len].to_vec()
    }

    pub fn get(&mut self, mut i: usize) -> M::Elem {
        i += self.offset;
        for k in (1..=self.height).rev() {
            self.push(i >> k);
        }
        self.tree[i].clone()
    }

    pub fn set(&mut self, mut i: usize, val: M::Elem) {
        i += self.offset;
        for k in (1..=self.height).rev() {
            self.push(i >> k);
        }
        self.tree[i] = val;
        for k in 1..=self.height {
            self.update(i >> k);
        }
    }

    pub fn prod(&mut self, range: impl RangeBounds<usize>) -> M::Elem {
        let (mut l, mut r) = range.clamp(0, self.len);
        if (l, r) == (0, self.len) {
            return self.tree[1].clone();
        }

        (l, r) = (l + self.offset, r + self.offset);
        for k in (1..=self.height).rev() {
            if ((l >> k) << k) != l {
                self.push(l >> k);
            }
            if ((r >> k) << k) != r {
                self.push(r >> k);
            }
        }

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

    pub fn apply(&mut self, mut i: usize, f: M::Map) {
        i += self.offset;
        for k in (1..=self.height).rev() {
            self.push(i >> k);
        }
        self.tree[i] = M::apply(&self.tree[i], &f);
        for k in 1..=self.height {
            self.update(i >> k);
        }
    }

    pub fn apply_range(&mut self, range: impl RangeBounds<usize>, f: M::Map) {
        let (mut l, mut r) = range.clamp(0, self.len);
        (l, r) = (l + self.offset, r + self.offset);

        for k in (1..=self.height).rev() {
            if ((l >> k) << k) != l {
                self.push(l >> k);
            }
            if ((r >> k) << k) != r {
                self.push(r >> k);
            }
        }

        {
            let (mut l, mut r) = (l, r);
            while l < r {
                if l & 1 == 1 {
                    self.flush(l, &f);
                    l += 1;
                }
                if r & 1 == 1 {
                    r -= 1;
                    self.flush(r, &f);
                }
                (l, r) = (l >> 1, r >> 1);
            }
        }

        for k in 1..=self.height {
            if ((l >> k) << k) != l {
                self.update(l >> k);
            }
            if ((r >> k) << k) != r {
                self.update(r >> k);
            }
        }
    }

    pub fn max_right(&mut self, l: usize, f: impl Fn(&M::Elem) -> bool) -> usize {
        let mut r = l;
        let (mut i, mut width) = (r + self.offset, 1);
        let mut acc = M::identity();
        assert!(f(&acc));

        for k in (1..=self.height).rev() {
            self.push(i >> k);
        }

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
            self.push(i);
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

    pub fn min_left(&mut self, r: usize, f: impl Fn(&M::Elem) -> bool) -> usize {
        let mut l = r;
        let (mut i, mut width) = (l + self.offset, 1);
        let mut acc = M::identity();
        assert!(f(&acc));

        for k in (1..=self.height).rev() {
            self.push((i - 1) >> k);
        }

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
            self.push(i);
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

    fn push(&mut self, i: usize) {
        let f = std::mem::replace(&mut self.lazy[i], M::identity_map());
        self.flush(2 * i, &f);
        self.flush(2 * i + 1, &f);
    }

    fn flush(&mut self, i: usize, f: &M::Map) {
        self.tree[i] = M::apply(&self.tree[i], f);
        if i < self.offset {
            self.lazy[i] = M::compose(&self.lazy[i], f);
        }
    }
}

impl<M, T> From<T> for LazySegmentTree<M>
where
    M: MapMonoid,
    T: AsRef<[M::Elem]>,
{
    fn from(value: T) -> Self {
        let a = value.as_ref();
        let len = a.len();
        let offset = len.next_power_of_two();
        let height = offset.trailing_zeros() as usize;

        let mut tree = vec![M::identity(); 2 * offset];
        tree[offset..][..len].clone_from_slice(a);
        let lazy = vec![M::identity_map(); offset];

        let mut segtree = Self {
            len,
            offset,
            height,
            tree,
            lazy,
        };
        for i in (1..offset).rev() {
            segtree.update(i);
        }

        segtree
    }
}
