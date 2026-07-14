//! A modular integer with a compile-time 31-bit modulus.

use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

pub trait Modulus: 'static + Clone + Copy + Debug + Default + PartialEq + Eq {
    const MOD: u32;
}

macro_rules! define_modulus {
    ($name:ident, $modulus:expr) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct $name;
        impl Modulus for $name {
            const MOD: u32 = const {
                assert!($modulus < (1u32 << 31));
                $modulus
            };
        }
    };
}

define_modulus!(Mod998244353, 998244353);
define_modulus!(Mod1000000007, 1000000007);

pub type ModInt998244353 = ModInt<Mod998244353>;
pub type ModInt1000000007 = ModInt<Mod1000000007>;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ModInt<M> {
    val: u32,
    _phantom: std::marker::PhantomData<fn() -> M>,
}

impl<M: Modulus> ModInt<M> {
    pub fn modulus() -> u32 {
        M::MOD
    }

    pub fn new(val: u32) -> Self {
        Self::from(val)
    }

    pub fn new_unchecked(val: u32) -> Self {
        Self {
            val,
            _phantom: std::marker::PhantomData,
        }
    }

    #[deprecated]
    pub fn raw(val: u32) -> Self {
        Self::new_unchecked(val)
    }

    pub fn zero() -> Self {
        Self::new_unchecked(0)
    }

    pub fn one() -> Self {
        Self::new_unchecked(1)
    }

    pub fn val(&self) -> u32 {
        self.val
    }

    pub fn pow(&self, mut exp: usize) -> Self {
        let mut res = Self::one();
        let mut pow = *self;
        while exp > 0 {
            if exp & 1 == 1 {
                res *= pow;
            }
            pow *= pow;
            exp >>= 1;
        }
        res
    }

    pub fn inv(&self) -> Self {
        self.checked_inv().unwrap_or_else(|| {
            panic!(
                "inverse not exist for {} (mod {})",
                self.val(),
                Self::modulus()
            )
        })
    }

    pub fn checked_inv(&self) -> Option<Self> {
        let x = self.val() as i32;
        let m = Self::modulus() as i32;

        // invariant: t.0 = t.1 * x (mod m) for t = u,v
        let mut u = (m, 0);
        let mut v = (x, 1);
        while v.0 != 0 {
            let q = u.0 / v.0;
            (u, v) = (v, (u.0 % v.0, u.1 - q * v.1));
        }

        (u.0 == 1).then_some(Self::new_unchecked(
            if u.1 < 0 { u.1 + m } else { u.1 } as u32
        ))
    }

    // https://en.wikipedia.org/wiki/Rational_reconstruction_(mathematics)
    fn to_rational(self) -> (i64, i64) {
        let m = Self::modulus() as i64;
        let mut u = (m, 0);
        let mut v = (self.val() as i64, 1);

        // invariant: x.0 = x.1 * val (mod m) for x = u,v
        while v.0 * v.0 * 2 > m {
            let q = u.0.div_euclid(v.0);
            let w = (u.0 - q * v.0, u.1 - q * v.1);
            (u, v) = (v, w);
        }
        if v.1 < 0 { (-v.0, -v.1) } else { v }
    }
}

impl<M> std::fmt::Display for ModInt<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
}

impl<M: Modulus> std::fmt::Debug for ModInt<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (num, denom) = self.to_rational();
        if denom == 1 {
            write!(f, "{num}")
        } else {
            write!(f, "{num}/{denom}")
        }
    }
}

impl<M: Modulus> std::str::FromStr for ModInt<M> {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u32>()?;
        Ok(Self::new(value))
    }
}

macro_rules! impl_from_integer {
    ( $( $ty:tt ),* ) => { $(
        impl<M: Modulus> From<$ty> for ModInt<M> {
            fn from(value: $ty) -> ModInt<M> {
                Self::new_unchecked(value.rem_euclid(Self::modulus() as $ty) as u32)
            }
        }
    )* };
}
impl_from_integer!(u32, u64, usize, i32, i64, isize);

impl<M: Modulus> std::ops::Neg for ModInt<M> {
    type Output = Self;
    fn neg(mut self) -> Self::Output {
        if self.val > 0 {
            self.val = Self::modulus() - self.val;
        }
        self
    }
}

impl<M: Modulus, T: Into<ModInt<M>>> AddAssign<T> for ModInt<M> {
    fn add_assign(&mut self, rhs: T) {
        self.val += rhs.into().val;
        if self.val >= Self::modulus() {
            self.val -= Self::modulus();
        }
    }
}

impl<M: Modulus, T: Into<ModInt<M>>> SubAssign<T> for ModInt<M> {
    fn sub_assign(&mut self, rhs: T) {
        self.val = self.val.wrapping_sub(rhs.into().val);
        if self.val > Self::modulus() {
            self.val = self.val.wrapping_add(Self::modulus());
        }
    }
}

impl<M: Modulus, T: Into<ModInt<M>>> MulAssign<T> for ModInt<M> {
    fn mul_assign(&mut self, rhs: T) {
        self.val = ((self.val as u64 * rhs.into().val as u64) % Self::modulus() as u64) as u32;
    }
}

impl<M: Modulus, T: Into<ModInt<M>>> DivAssign<T> for ModInt<M> {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn div_assign(&mut self, rhs: T) {
        *self *= rhs.into().inv();
    }
}

macro_rules! forward_ops {
    ($op: ident, $fn: ident, $op_assign: ident, $fn_assign: ident) => {
        impl<M: Modulus, T: Into<ModInt<M>>> $op<T> for ModInt<M> {
            type Output = ModInt<M>;
            fn $fn(mut self, rhs: T) -> ModInt<M> {
                self.$fn_assign(rhs);
                self
            }
        }

        impl<M: Modulus> $op<&ModInt<M>> for ModInt<M> {
            type Output = ModInt<M>;
            fn $fn(self, rhs: &ModInt<M>) -> ModInt<M> {
                self.$fn(*rhs)
            }
        }

        impl<M: Modulus, T: Into<ModInt<M>>> $op<T> for &ModInt<M> {
            type Output = ModInt<M>;
            fn $fn(self, rhs: T) -> ModInt<M> {
                (*self).$fn(rhs)
            }
        }

        impl<M: Modulus> $op<&ModInt<M>> for &ModInt<M> {
            type Output = ModInt<M>;
            fn $fn(self, rhs: &ModInt<M>) -> ModInt<M> {
                (*self).$fn(*rhs)
            }
        }

        impl<M: Modulus> $op_assign<&ModInt<M>> for ModInt<M> {
            fn $fn_assign(&mut self, rhs: &ModInt<M>) {
                *self = self.$fn(*rhs);
            }
        }
    };
}

forward_ops!(Add, add, AddAssign, add_assign);
forward_ops!(Sub, sub, SubAssign, sub_assign);
forward_ops!(Mul, mul, MulAssign, mul_assign);
forward_ops!(Div, div, DivAssign, div_assign);

impl<M: Modulus> std::iter::Sum for ModInt<M> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

impl<'a, M: Modulus> std::iter::Sum<&'a ModInt<M>> for ModInt<M> {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

impl<M: Modulus> std::iter::Product for ModInt<M> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), Mul::mul)
    }
}

impl<'a, M: Modulus> std::iter::Product<&'a ModInt<M>> for ModInt<M> {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::one(), Mul::mul)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type M = ModInt998244353;

    #[test]
    fn round_trip() {
        assert_eq!(M::new(1).val(), 1);
        assert_eq!(M::new(998244353).val(), 0);
    }

    #[test]
    fn arithmetrics() {
        let zero = M::zero();
        let one = M::one();
        let neg_one = M::new(998244352);
        let two = M::new(2);
        let half = M::new(499122177);
        assert_eq!(zero + one, one);
        assert_eq!(half + &half, one);
        assert_eq!(&two - one, one);
        assert_eq!(&zero - &one, neg_one);
        assert_eq!(one * two, two);
        assert_eq!(half * neg_one, -half);
        assert_eq!(two.pow(998244351), half);
    }

    #[test]
    fn inv() {
        assert!(M::zero().checked_inv().is_none());
        for i in 1..10 {
            let j = M::modulus() - i;
            assert_eq!(M::new(i).inv() * i, M::one());
            assert_eq!(M::new(j).inv() * j, M::one());
        }
    }

    #[test]
    fn to_rational() {
        let r = M::new(12345) / 6789;
        let (num, denom) = r.to_rational();
        assert_eq!(M::from(num) / denom, r);
    }
}
