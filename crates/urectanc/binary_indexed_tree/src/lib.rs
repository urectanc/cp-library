use std::ops::RangeBounds;

use clamp_range::ClampRange;
use num_traits::PrimitiveInteger;

pub struct BinaryIndexedTree<T> {
    len: usize,
    tree: Vec<T>,
}

impl<T, A> From<A> for BinaryIndexedTree<T>
where
    T: PrimitiveInteger,
    A: AsRef<[T]>,
{
    fn from(a: A) -> Self {
        let a = a.as_ref();
        let len = a.len();
        let mut tree = vec![T::zero(); len + 1];
        tree[1..].copy_from_slice(a);

        for i in 1..len {
            let lsb = i & i.wrapping_neg();
            if i + lsb <= len {
                let add = tree[i];
                tree[i + lsb] += add;
            }
        }

        Self { len, tree }
    }
}

impl<T> BinaryIndexedTree<T>
where
    T: PrimitiveInteger,
{
    pub fn new(len: usize) -> Self {
        Self {
            len,
            tree: vec![T::zero(); len + 1],
        }
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut a = self.tree.clone();
        for i in (1..self.len).rev() {
            let lsb = i & i.wrapping_neg();
            if i + lsb <= self.len {
                let sub = a[i];
                a[i + lsb] -= sub;
            }
        }
        a[1..].to_owned()
    }

    pub fn get(&self, i: usize) -> T {
        self.sum(i..=i)
    }

    pub fn set(&mut self, i: usize, x: T) {
        self.add(i, x - self.get(i));
    }

    pub fn add(&mut self, i: usize, x: T) {
        let mut i = i + 1;
        while i <= self.len {
            self.tree[i] += x;
            i += i & i.wrapping_neg();
        }
    }

    pub fn sum(&self, range: impl RangeBounds<usize>) -> T {
        let (mut l, mut r) = range.clamp(0, self.len);

        let mut sum = T::zero();
        while l < r {
            sum += self.tree[r];
            r -= r & r.wrapping_neg();
        }
        while r < l {
            sum -= self.tree[l];
            l -= l & l.wrapping_neg();
        }

        sum
    }

    pub fn max_right(&self, f: impl Fn(T) -> bool) -> usize {
        let mut r = 0;
        let mut sum = T::zero();
        assert!(f(sum));
        let mut width = self.len.next_power_of_two();
        while width > 0 {
            if r + width <= self.len && f(sum + self.tree[r + width]) {
                sum += self.tree[r + width];
                r += width;
            }
            width >>= 1;
        }
        r
    }
}
