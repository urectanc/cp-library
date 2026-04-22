use std::ops::RangeBounds;

use clamp_range::ClampRange;

mod mersenne_modint;

type ModInt = mersenne_modint::ModIntM61;

pub struct RollingHash {
    pow_inv: Vec<ModInt>,
    cum: Vec<ModInt>,
    cum_rev: Vec<ModInt>,
}

impl RollingHash {
    pub fn new(s: &str, base: u64) -> Self {
        let base_inv = ModInt::new(base).inv();

        let n = s.len();
        let mut pow = vec![ModInt::new(1); n + 1];
        let mut pow_inv = vec![ModInt::new(1); n + 1];

        for i in 0..n {
            pow[i + 1] = pow[i] * ModInt::new(base);
            pow_inv[i + 1] = pow_inv[i] * base_inv;
        }

        let mut cum = vec![ModInt::new(0); n + 1];
        for (i, s) in s.bytes().enumerate() {
            cum[i + 1] = cum[i] + pow[i] * ModInt::new(s as u64);
        }

        let mut cum_rev = vec![ModInt::new(0); n + 1];
        for (i, s) in s.bytes().rev().enumerate() {
            cum_rev[i + 1] = cum_rev[i] + pow[i] * ModInt::new(s as u64);
        }

        Self {
            pow_inv,
            cum,
            cum_rev,
        }
    }

    pub fn random_base() -> u64 {
        use std::hash::{BuildHasher, Hasher};
        let rand = std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish();
        rand % ModInt::modulus()
    }

    pub fn hash(&self, range: impl RangeBounds<usize>) -> u64 {
        let n = self.cum.len() - 1;
        let (l, r) = range.clamp(0, n);
        ((self.cum[r] - self.cum[l]) * self.pow_inv[l]).as_u64()
    }

    pub fn rev_hash(&self, range: impl RangeBounds<usize>) -> u64 {
        let n = self.cum.len() - 1;
        let (l, r) = range.clamp(0, n);
        let (l, r) = (n - r, n - l);
        ((self.cum_rev[r] - self.cum_rev[l]) * self.pow_inv[l]).as_u64()
    }

    pub fn is_palindrome(&self, range: impl RangeBounds<usize> + Copy) -> bool {
        self.hash(range) == self.rev_hash(range)
    }
}
