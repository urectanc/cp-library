use super::{BeatsMonoid, SegmentTreeBeats};

const INF: i64 = 1 << 60;

pub enum Act {
    Chmin(i64),
    Chmax(i64),
    Add(i64),
}

#[derive(Clone)]
pub struct RangeChminChmaxAddRangeSum {
    min: i64,
    min_cnt: i64,
    min2: i64,
    max: i64,
    max_cnt: i64,
    max2: i64,
    sum: i64,
    len: i64,
    add: i64,
}

impl RangeChminChmaxAddRangeSum {
    pub fn new(x: i64) -> Self {
        Self {
            min: x,
            min_cnt: 1,
            min2: INF,
            max: x,
            max_cnt: 1,
            max2: -INF,
            sum: x,
            len: 1,
            add: 0,
        }
    }

    pub fn min(&self) -> i64 {
        self.min
    }

    pub fn max(&self) -> i64 {
        self.max
    }

    pub fn sum(&self) -> i64 {
        self.sum
    }

    pub fn chmin(&mut self, chmin: i64) {
        debug_assert!((self.max2 + 1..self.max).contains(&chmin));
        self.sum += (chmin - self.max) * self.max_cnt;
        if self.min == self.max {
            self.min = chmin;
        }
        if self.min2 == self.max {
            self.min2 = chmin;
        }
        self.max = chmin;
    }

    pub fn chmax(&mut self, chmax: i64) {
        debug_assert!((self.min + 1..self.min2).contains(&chmax));
        self.sum += (chmax - self.min) * self.min_cnt;
        if self.max == self.min {
            self.max = chmax;
        }
        if self.max2 == self.min {
            self.max2 = chmax;
        }
        self.min = chmax;
    }

    pub fn add(&mut self, add: i64) {
        self.min += add;
        self.max += add;
        self.sum += add * self.len;
        self.add += add;
        if self.min != self.max {
            self.min2 += add;
            self.max2 += add;
        }
    }
}

impl BeatsMonoid for RangeChminChmaxAddRangeSum {
    type Elem = Self;
    type Map = Act;

    fn identity() -> Self::Elem {
        RangeChminChmaxAddRangeSum {
            min: INF,
            min_cnt: 0,
            min2: INF,
            max: -INF,
            max_cnt: 0,
            max2: -INF,
            sum: 0,
            len: 0,
            add: 0,
        }
    }

    fn op(lhs: &Self::Elem, rhs: &Self::Elem) -> Self::Elem {
        use std::cmp::Ordering;

        let (max, max_cnt, max2) = match lhs.max.cmp(&rhs.max) {
            Ordering::Less => (rhs.max, rhs.max_cnt, rhs.max2.max(lhs.max)),
            Ordering::Equal => (lhs.max, lhs.max_cnt + rhs.max_cnt, lhs.max2.max(rhs.max2)),
            Ordering::Greater => (lhs.max, lhs.max_cnt, lhs.max2.max(rhs.max)),
        };

        let (min, min_cnt, min2) = match lhs.min.cmp(&rhs.min) {
            Ordering::Less => (lhs.min, lhs.min_cnt, lhs.min2.min(rhs.min)),
            Ordering::Equal => (lhs.min, lhs.min_cnt + rhs.min_cnt, lhs.min2.min(rhs.min2)),
            Ordering::Greater => (rhs.min, rhs.min_cnt, rhs.min2.min(lhs.min)),
        };

        RangeChminChmaxAddRangeSum {
            min,
            min_cnt,
            min2,
            max,
            max_cnt,
            max2,
            sum: lhs.sum + rhs.sum,
            len: lhs.len + rhs.len,
            add: 0,
        }
    }

    fn apply(x: &mut Self::Elem, f: &Self::Map) -> bool {
        match *f {
            Act::Chmin(chmin) => {
                if x.max <= chmin {
                    true
                } else if x.max2 < chmin {
                    x.chmin(chmin);
                    true
                } else {
                    false
                }
            }
            Act::Chmax(chmax) => {
                if x.min >= chmax {
                    true
                } else if x.min2 > chmax {
                    x.chmax(chmax);
                    true
                } else {
                    false
                }
            }
            Act::Add(add) => {
                x.add(add);
                true
            }
        }
    }

    fn resolve(parent: &Self::Elem, child: &mut Self::Elem) {
        if parent.add != 0 {
            child.add(parent.add);
        }

        if parent.max < child.max {
            child.chmin(parent.max);
        }

        if parent.min > child.min {
            child.chmax(parent.min);
        }
    }

    fn clear(x: &mut Self::Elem) {
        x.add = 0;
    }
}

impl FromIterator<i64> for SegmentTreeBeats<RangeChminChmaxAddRangeSum> {
    fn from_iter<I: IntoIterator<Item = i64>>(iter: I) -> Self {
        iter.into_iter()
            .map(RangeChminChmaxAddRangeSum::new)
            .collect()
    }
}
