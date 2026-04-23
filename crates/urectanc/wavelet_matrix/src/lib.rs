use std::ops::{Add, RangeBounds, Sub};

mod bit_vector;

use bit_vector::*;
use clamp_range::ClampRange;

pub trait StaticRangeSum {
    type Value: Clone + Copy + Default;

    fn new(w: &[Self::Value]) -> Self;

    fn range_sum(&self, l: usize, r: usize) -> Self::Value;
}

pub trait RangeSum: StaticRangeSum {
    fn update(&mut self, i: usize, val: Self::Value);
}

pub struct UnWeighted;

impl StaticRangeSum for UnWeighted {
    type Value = ();

    fn new(_w: &[Self::Value]) -> Self {
        Self
    }

    fn range_sum(&self, _l: usize, _r: usize) -> Self::Value {
        unimplemented!()
    }
}

pub struct WaveletMatrix<S = UnWeighted> {
    len: usize,
    bitvecs: Vec<BitVector>,
    cum: Vec<S>,
    offsets: Vec<usize>,
}

impl<S: StaticRangeSum> WaveletMatrix<S> {
    pub fn new(values: impl AsRef<[u32]>, weights: impl AsRef<[S::Value]>) -> Self {
        let values = values.as_ref();
        let weights = weights.as_ref();
        assert_eq!(values.len(), weights.len());

        let n = values.len();
        let max = values.iter().copied().max().unwrap();
        let height = (max + 2).next_power_of_two().trailing_zeros() as usize;

        let mut values = values.to_owned();
        let mut weights = weights.to_owned();
        let mut sorted_values = values.clone();
        let mut sorted_weights = weights.clone();
        let mut bitvecs = Vec::with_capacity(height);
        let mut cum = Vec::with_capacity(height);
        let mut offsets = Vec::with_capacity(height);

        for h in (0..height).rev() {
            let (mut l, mut r) = (0, n);
            let mut builder = BitVectorBuilder::new(n);
            for (i, (&a, &w)) in values.iter().zip(&weights).enumerate() {
                if a >> h & 1 == 0 {
                    sorted_values[l] = a;
                    sorted_weights[l] = w;
                    l += 1;
                } else {
                    r -= 1;
                    sorted_values[r] = a;
                    sorted_weights[r] = w;
                    builder.set(i);
                }
            }
            sorted_values[l..].reverse();
            sorted_weights[l..].reverse();

            std::mem::swap(&mut values, &mut sorted_values);
            std::mem::swap(&mut weights, &mut sorted_weights);

            bitvecs.push(builder.build());
            cum.push(S::new(&weights));
            offsets.push(l);
        }
        bitvecs.reverse();
        cum.reverse();
        offsets.reverse();

        Self {
            len: n,
            bitvecs,
            cum,
            offsets,
        }
    }

    pub fn update_weight(&mut self, i: usize, weight: S::Value)
    where
        S: RangeSum,
    {
        let Self {
            bitvecs,
            cum,
            offsets,
            ..
        } = self;
        let (mut l, mut r) = (i, i + 1);
        for ((bitvec, cum), &mut offset) in bitvecs.iter().zip(cum).zip(offsets).rev() {
            let l1 = bitvec.rank(l);
            let r1 = bitvec.rank(r);
            if l1 == r1 {
                l -= l1;
                r -= r1;
            } else {
                l = offset + l1;
                r = offset + r1;
            }
            cum.update(l, weight);
        }
    }

    pub fn range(&self, range: impl RangeBounds<usize>) -> Range<'_, S> {
        let (l, r) = range.clamp(0, self.len);
        Range { wm: self, l, r }
    }
}

pub struct Range<'a, S> {
    wm: &'a WaveletMatrix<S>,
    l: usize,
    r: usize,
}

impl<'a, S> Range<'a, S> {
    pub fn kth_smallest(&self, mut k: usize) -> u32 {
        let &Range {
            wm: WaveletMatrix {
                bitvecs, offsets, ..
            },
            mut l,
            mut r,
        } = self;
        let mut res = 0;
        for (h, (bitvec, &offset)) in bitvecs.iter().zip(offsets).enumerate().rev() {
            let l1 = bitvec.rank(l);
            let r1 = bitvec.rank(r);
            let zeros = (r - l) - (r1 - l1);
            if k < zeros {
                l -= l1;
                r -= r1;
            } else {
                k -= zeros;
                l = offset + l1;
                r = offset + r1;
                res |= 1 << h;
            }
        }
        res
    }

    pub fn prefix_count(&self, upper: u32) -> u32 {
        let &Range {
            wm: WaveletMatrix {
                bitvecs, offsets, ..
            },
            mut l,
            mut r,
        } = self;

        let mut cnt = 0;

        for (h, (bitvec, &offset)) in bitvecs.iter().zip(offsets).enumerate().rev() {
            let l1 = bitvec.rank(l);
            let r1 = bitvec.rank(r);
            if upper >> h & 1 == 0 {
                l -= l1;
                r -= r1;
            } else {
                cnt += (r - l) - (r1 - l1);
                l = offset + l1;
                r = offset + r1;
            }
        }

        cnt as _
    }

    pub fn count(&self, range: impl RangeBounds<u32>) -> u32 {
        let lo = match range.start_bound() {
            std::ops::Bound::Included(&lo) => lo,
            std::ops::Bound::Excluded(&lo) => lo + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let hi = match range.end_bound() {
            std::ops::Bound::Included(&hi) => hi + 1,
            std::ops::Bound::Excluded(&hi) => hi,
            std::ops::Bound::Unbounded => 1 << self.wm.bitvecs.len(),
        };
        self.prefix_count(hi) - self.prefix_count(lo)
    }
}

impl<'a, S> Range<'a, S>
where
    S: StaticRangeSum,
    S::Value: Add<Output = S::Value> + Sub<Output = S::Value>,
{
    pub fn count_sum(&self, upper: u32) -> (u32, S::Value) {
        let &Range {
            wm:
                WaveletMatrix {
                    bitvecs,
                    cum,
                    offsets,
                    ..
                },
            mut l,
            mut r,
        } = self;

        let mut cnt = 0;
        let mut sum = S::Value::default();

        for (h, ((bitvec, cum), &offset)) in bitvecs.iter().zip(cum).zip(offsets).enumerate().rev()
        {
            let l1 = bitvec.rank(l);
            let r1 = bitvec.rank(r);
            let l0 = l - l1;
            let r0 = r - r1;
            if upper >> h & 1 == 0 {
                l = l0;
                r = r0;
            } else {
                l = offset + l1;
                r = offset + r1;
                cnt += r0 - l0;
                sum = sum + cum.range_sum(l0, r0);
            }
        }

        (cnt as _, sum)
    }
}

pub struct RectangleSum<S> {
    wm: WaveletMatrix<S>,
    pos: Vec<usize>,
    x: Vec<u32>,
    y: Vec<u32>,
}

impl<S> RectangleSum<S>
where
    S: StaticRangeSum,
    S::Value: Add<Output = S::Value> + Sub<Output = S::Value>,
{
    pub fn new(points: &[(u32, u32, S::Value)]) -> Self {
        let n = points.len();
        let mut ord: Vec<_> = (0..n).collect();
        ord.sort_unstable_by_key(|&i| {
            let (x, y, _) = points[i];
            (x, y)
        });

        let mut pos = vec![0; n];
        for i in 0..n {
            pos[ord[i]] = i;
        }

        let ((x, mut y), w): ((Vec<_>, Vec<_>), Vec<_>) = ord
            .iter()
            .map(|&i| {
                let (x, y, w) = points[i];
                ((x, y), w)
            })
            .unzip();

        let mut a = y.clone();
        y.sort_unstable();
        y.dedup();

        a.iter_mut()
            .for_each(|a| *a = y.binary_search(a).unwrap() as u32);

        let wm = WaveletMatrix::new(&a, &w);

        Self { wm, pos, x, y }
    }

    pub fn update_weight(&mut self, i: usize, weight: S::Value)
    where
        S: RangeSum,
    {
        let i = self.pos[i];
        self.wm.update_weight(i, weight);
    }

    pub fn prefix_sum(&self, l: u32, r: u32, upper: u32) -> S::Value {
        let l = self.x.partition_point(|&x| x < l);
        let r = self.x.partition_point(|&x| x < r);
        let upper = self.y.partition_point(|&y| y < upper) as _;

        self.wm.range(l..r).count_sum(upper).1
    }

    pub fn sum(&self, l: u32, r: u32, lower: u32, upper: u32) -> S::Value {
        self.prefix_sum(l, r, upper) - self.prefix_sum(l, r, lower)
    }
}
