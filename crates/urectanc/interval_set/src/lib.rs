use clamp_range::ClampRange;
use std::{
    collections::BTreeMap,
    ops::{Range, RangeBounds},
};

pub struct IntervalSet {
    intervals: BTreeMap<usize, usize>,
    len: usize,
}

impl IntervalSet {
    pub fn new() -> Self {
        Self {
            intervals: BTreeMap::new(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn find(&self, x: usize) -> Option<Range<usize>> {
        self.intervals
            .range(..=x)
            .last()
            .map(|(&l, &r)| l..r)
            .filter(|range| range.contains(&x))
    }

    pub fn contains(&self, x: usize) -> bool {
        self.find(x).is_some()
    }

    pub fn insert(&mut self, range: impl RangeBounds<usize>) {
        let (mut l, mut r) = range.clamp(usize::MIN, usize::MAX);

        if let Some((&s, &t)) = self.intervals.range(..l).last()
            && l <= t
        {
            self.intervals.remove(&s);
            self.len -= t - s;
            l = s;
            r = r.max(t);
        }

        while let Some((&s, &t)) = self.intervals.range(l..).next()
            && s <= r
        {
            self.intervals.remove(&s);
            self.len -= t - s;
            r = r.max(t);
        }

        self.intervals.insert(l, r);
        self.len += r - l;
    }

    pub fn remove(&mut self, range: impl RangeBounds<usize>) {
        let (l, r) = range.clamp(usize::MIN, usize::MAX);
        self.insert(l..r);

        let (&s, &t) = self.intervals.range(..=l).last().unwrap();
        self.intervals.remove(&s);

        if s < l {
            self.intervals.insert(s, l);
        }
        if r < t {
            self.intervals.insert(r, t);
        }

        self.len -= r - l;
    }

    pub fn iter(&self) -> std::collections::btree_map::Iter<'_, usize, usize> {
        self.intervals.iter()
    }
}

impl Default for IntervalSet {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for IntervalSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set()
            .entries(self.intervals.iter().map(|(&l, &r)| l..r))
            .finish()
    }
}
