use std::fmt::Debug;

use num_traits::PrimitiveInteger;

#[derive(Clone, Copy)]
pub struct Rational<I> {
    num: I,
    denom: I,
}

impl<I: PrimitiveInteger> Rational<I> {
    pub fn new(num: I, denom: I) -> Self {
        Self { num, denom }
    }

    pub fn zero() -> Self {
        Self::new(I::zero(), I::one())
    }

    pub fn one() -> Self {
        Self::new(I::one(), I::one())
    }

    pub fn inf() -> Self {
        Self::new(I::one(), I::zero())
    }

    pub fn num(&self) -> I {
        self.num
    }

    pub fn denom(&self) -> I {
        self.denom
    }

    pub fn to_continued_fraction(&self) -> Vec<I> {
        assert!(self.denom() != I::zero());

        let (mut x, mut y) = (self.num(), self.denom());
        let mut res = Vec::new();
        while x > I::zero() && y > I::zero() {
            let (q, r) = (x / y, x % y);
            res.push(if r == I::zero() { q - I::one() } else { q });
            (x, y) = (y, r);
        }
        res
    }
}

impl<T: PrimitiveInteger> Ord for Rational<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.num * other.denom).cmp(&(other.num * self.denom))
    }
}

impl<T: PrimitiveInteger> PartialOrd for Rational<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: PrimitiveInteger> PartialEq for Rational<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<T: PrimitiveInteger> Eq for Rational<T> {}

impl<T: PrimitiveInteger> Debug for Rational<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}/{:?}", self.num, self.denom)
    }
}
