use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

const fn gcd_inv(a: i64, b: i64) -> (i64, i64) {
    let a = a.rem_euclid(b);
    if a == 0 {
        return (b, 0);
    }

    // invariant: x.0 = x.1 * a (mod b) for x = u,v
    let mut u = (b, 0);
    let mut v = (a, 1);
    while v.0 != 0 {
        let q = u.0.div_euclid(v.0);
        u.0 -= q * v.0;
        u.1 -= q * v.1;
        (u, v) = (v, u);
    }

    if u.1 < 0 {
        u.1 += b.div_euclid(u.0);
    }

    u
}

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

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct StaticModInt<M> {
    val: u32,
    _phantom: std::marker::PhantomData<fn() -> M>,
}

impl<M: Modulus> StaticModInt<M> {
    pub const fn modulus() -> u32 {
        M::MOD
    }

    pub const fn new(val: u32) -> Self {
        Self {
            val: val.rem_euclid(Self::modulus()),
            _phantom: std::marker::PhantomData,
        }
    }

    pub const fn raw(val: u32) -> Self {
        Self {
            val,
            _phantom: std::marker::PhantomData,
        }
    }

    pub const fn zero() -> Self {
        Self::raw(0)
    }

    pub const fn one() -> Self {
        Self::raw(1)
    }

    pub const fn val(&self) -> u32 {
        self.val
    }

    pub const fn pow(self, mut exp: u64) -> Self {
        let modulus = Self::modulus() as u64;
        let mut base = self.val() as u64;
        let mut acc = 1u64;

        while exp > 0 {
            if exp & 1 == 1 {
                acc = acc * base % modulus;
            }
            base = base * base % modulus;
            exp >>= 1;
        }

        Self::raw(acc as u32)
    }

    pub const fn inv(self) -> Self {
        self.checked_inv().expect("the inverse does not exist")
    }

    pub const fn checked_inv(self) -> Option<Self> {
        let (gcd, inv) = gcd_inv(self.val() as i64, Self::modulus() as i64);
        if gcd == 1 {
            Some(Self::raw(inv as u32))
        } else {
            None
        }
    }

    // https://en.wikipedia.org/wiki/Rational_reconstruction_(mathematics)
    fn to_rational(self) -> (i64, i64) {
        let m = Self::modulus() as i64;
        let mut u = (m, 0i64);
        let mut v = (self.val() as i64, 1i64);

        // invariant: x.0 = x.1 * val (mod m) for x = u,v
        while v.0 * v.0 * 2 > m {
            let q = u.0.div_euclid(v.0);
            let w = (u.0 - q * v.0, u.1 - q * v.1);
            (u, v) = (v, w);
        }
        if v.1 < 0 { (-v.0, -v.1) } else { v }
    }
}

impl<M: Modulus> std::fmt::Display for StaticModInt<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
}

impl<M: Modulus> std::fmt::Debug for StaticModInt<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (num, denom) = self.to_rational();
        if denom == 1 {
            write!(f, "{num}")
        } else {
            write!(f, "{num}/{denom}")
        }
    }
}

impl<M: Modulus> std::str::FromStr for StaticModInt<M> {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u32>()?;
        Ok(value.into())
    }
}

macro_rules! impl_from_integer {
    ( $( $ty:tt ),* ) => { $(
        impl<M: Modulus> From<$ty> for StaticModInt<M> {
            fn from(value: $ty) -> StaticModInt<M> {
                Self::raw((value as $ty).rem_euclid(Self::modulus() as $ty) as u32)
            }
        }
    )* };
}
impl_from_integer!(u32, u64, usize, i32, i64, isize);

impl<M: Modulus> std::ops::Neg for StaticModInt<M> {
    type Output = Self;
    fn neg(mut self) -> Self::Output {
        if self.val > 0 {
            self.val = Self::modulus() - self.val;
        }
        self
    }
}

impl<M: Modulus, T: Into<StaticModInt<M>>> AddAssign<T> for StaticModInt<M> {
    fn add_assign(&mut self, rhs: T) {
        self.val += rhs.into().val;
        if self.val >= Self::modulus() {
            self.val -= Self::modulus();
        }
    }
}

impl<M: Modulus, T: Into<StaticModInt<M>>> SubAssign<T> for StaticModInt<M> {
    fn sub_assign(&mut self, rhs: T) {
        self.val = self.val.wrapping_sub(rhs.into().val);
        if self.val > Self::modulus() {
            self.val = self.val.wrapping_add(Self::modulus());
        }
    }
}

impl<M: Modulus, T: Into<StaticModInt<M>>> MulAssign<T> for StaticModInt<M> {
    fn mul_assign(&mut self, rhs: T) {
        self.val = ((self.val as u64 * rhs.into().val as u64) % Self::modulus() as u64) as u32;
    }
}

impl<M: Modulus, T: Into<StaticModInt<M>>> DivAssign<T> for StaticModInt<M> {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn div_assign(&mut self, rhs: T) {
        *self *= rhs.into().inv();
    }
}

macro_rules! impl_binnary_operators {
    ($op: ident, $op_assign: ident, $fn: ident, $fn_assign: ident) => {
        impl<M: Modulus, T: Into<StaticModInt<M>>> std::ops::$op<T> for StaticModInt<M> {
            type Output = StaticModInt<M>;
            fn $fn(mut self, rhs: T) -> StaticModInt<M> {
                self.$fn_assign(rhs.into());
                self
            }
        }

        impl<M: Modulus> std::ops::$op<&StaticModInt<M>> for StaticModInt<M> {
            type Output = StaticModInt<M>;
            fn $fn(self, rhs: &StaticModInt<M>) -> StaticModInt<M> {
                self.$fn(*rhs)
            }
        }

        impl<M: Modulus, T: Into<StaticModInt<M>>> std::ops::$op<T> for &StaticModInt<M> {
            type Output = StaticModInt<M>;
            fn $fn(self, rhs: T) -> StaticModInt<M> {
                (*self).$fn(rhs.into())
            }
        }

        impl<M: Modulus> std::ops::$op<&StaticModInt<M>> for &StaticModInt<M> {
            type Output = StaticModInt<M>;
            fn $fn(self, rhs: &StaticModInt<M>) -> StaticModInt<M> {
                (*self).$fn(*rhs)
            }
        }

        impl<M: Modulus> std::ops::$op_assign<&StaticModInt<M>> for StaticModInt<M> {
            fn $fn_assign(&mut self, rhs: &StaticModInt<M>) {
                *self = self.$fn(*rhs);
            }
        }
    };
}

impl_binnary_operators!(Add, AddAssign, add, add_assign);
impl_binnary_operators!(Sub, SubAssign, sub, sub_assign);
impl_binnary_operators!(Mul, MulAssign, mul, mul_assign);
impl_binnary_operators!(Div, DivAssign, div, div_assign);

impl<M: Modulus> std::iter::Sum for StaticModInt<M> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

impl<'a, M: Modulus> std::iter::Sum<&'a StaticModInt<M>> for StaticModInt<M> {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

impl<M: Modulus> std::iter::Product for StaticModInt<M> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), Mul::mul)
    }
}

impl<'a, M: Modulus> std::iter::Product<&'a StaticModInt<M>> for StaticModInt<M> {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::one(), Mul::mul)
    }
}

pub type ModInt998244353 = StaticModInt<Mod998244353>;
pub type ModInt1000000007 = StaticModInt<Mod1000000007>;
