use std::ops::{Bound, RangeBounds};

pub trait ClampRange: RangeBounds<usize> {
    fn clamp(&self, l: usize, r: usize) -> (usize, usize) {
        assert!(l <= r);

        let start = match self.start_bound() {
            Bound::Included(&l) => l,
            Bound::Excluded(&l) => l + 1,
            Bound::Unbounded => l,
        }
        .clamp(l, r);

        let end = match self.end_bound() {
            Bound::Included(&r) => r + 1,
            Bound::Excluded(&r) => r,
            Bound::Unbounded => r,
        }
        .clamp(l, r);

        (start.min(end), end)
    }
}

impl<T: ?Sized> ClampRange for T where T: RangeBounds<usize> {}
