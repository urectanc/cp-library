use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModIntM61(u64);

impl ModIntM61 {
    const MOD: u64 = (1 << 61) - 1;

    pub fn new(val: u64) -> Self {
        Self(val)
    }

    pub fn modulus() -> u64 {
        Self::MOD
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn pow(self, mut exp: u64) -> Self {
        let mut base = self;
        let mut acc = Self::new(1);
        while exp > 0 {
            if exp & 1 == 1 {
                acc = acc * base;
            }
            base = base * base;
            exp >>= 1;
        }
        acc
    }

    pub fn inv(self) -> Self {
        self.pow(Self::MOD - 2)
    }
}

impl Add for ModIntM61 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let sum = self.0 + rhs.0;
        if sum >= Self::MOD {
            Self::new(sum - Self::MOD)
        } else {
            Self::new(sum)
        }
    }
}

impl Sub for ModIntM61 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let (sub, borrow) = self.0.overflowing_sub(rhs.0);
        if borrow {
            Self::new(sub.wrapping_add(Self::MOD))
        } else {
            Self::new(sub)
        }
    }
}

impl Mul for ModIntM61 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let prod = self.0 as u128 * rhs.0 as u128;
        let (hi, lo) = ((prod >> 61) as u64, prod as u64 & Self::MOD);
        // hi < MOD, lo <= MOD
        Self::new(hi) + Self::new(lo)
    }
}
