#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::__m256i;
use std::ops::{Add, Mul, Sub};

use modint::{Mod998244353, Modulus};

#[cfg(target_arch = "x86_64")]
pub mod simd;

pub trait Montgomery: Modulus {
    const N: u32 = {
        assert!(Self::MOD & 1 == 1);
        assert!(Self::MOD < 1 << 30);
        Self::MOD
    };
    const N2: u32 = Self::N * 2;
    const N_PRIME: u32 = {
        // Nm_i + 1 = 0 (mod 2^i)
        // N(Nm_i + 2)m_i + 1 = 0 (mod 2^{i+1})
        let mut m = 1u32;
        m = m.wrapping_mul(2 + m.wrapping_mul(Self::N));
        m = m.wrapping_mul(2 + m.wrapping_mul(Self::N));
        m = m.wrapping_mul(2 + m.wrapping_mul(Self::N));
        m = m.wrapping_mul(2 + m.wrapping_mul(Self::N));
        m = m.wrapping_mul(2 + m.wrapping_mul(Self::N));
        assert!(Self::N.wrapping_mul(m) == !0);
        m
    };
    const R: u32 = ((1u64 << 32) % Self::N as u64) as u32;
    const RR: u32 = ((Self::R as u64 * Self::R as u64) % Self::N as u64) as u32;

    #[cfg(target_arch = "x86_64")]
    const NX8: __m256i = unsafe { std::mem::transmute::<_, _>([Self::N; 8]) };
    #[cfg(target_arch = "x86_64")]
    const N2X8: __m256i = unsafe { std::mem::transmute::<_, _>([Self::N2; 8]) };
    #[cfg(target_arch = "x86_64")]
    const N_PRIMEX8: __m256i = unsafe { std::mem::transmute::<_, _>([Self::N_PRIME; 8]) };
}

impl Montgomery for Mod998244353 {}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct MontgomeryModInt<M: Montgomery> {
    pub val: u32,
    _phantom: std::marker::PhantomData<M>,
}

impl<M: Montgomery> MontgomeryModInt<M> {
    pub const fn new(val: u32) -> Self {
        Self::raw(val).mul2(Self::raw(M::RR))
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
        Self::raw(M::R)
    }

    pub const fn normalize(self) -> Self {
        let (sub, borrow) = self.val.overflowing_sub(M::N);
        Self::raw(if borrow { self.val } else { sub })
    }

    pub const fn normalize2(self) -> Self {
        let (sub, borrow) = self.val.overflowing_sub(M::N2);
        Self::raw(if borrow { self.val } else { sub })
    }

    pub const fn add2(self, rhs: Self) -> Self {
        Self::raw(self.val + rhs.val).normalize2()
    }

    pub const fn sub2(self, rhs: Self) -> Self {
        Self::raw(self.val + M::N2 - rhs.val).normalize2()
    }

    pub const fn mul2(self, rhs: Self) -> Self {
        let t = self.val as u64 * rhs.val as u64;
        let m = (t as u32).wrapping_mul(M::N_PRIME);
        Self::raw(((t + (m as u64 * M::N as u64)) >> 32) as u32)
    }

    pub const fn pow(self, mut e: u32) -> Self {
        let mut result = Self::one();
        let mut base = self;
        while e > 0 {
            if e & 1 == 1 {
                result = result.mul2(base);
            }
            base = base.mul2(base);
            e >>= 1;
        }
        result.normalize()
    }

    pub const fn inv(self) -> Self {
        self.pow(M::N - 2)
    }
}

impl<M: Montgomery> Add for MontgomeryModInt<M> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.add2(rhs)
    }
}

impl<M: Montgomery> Sub for MontgomeryModInt<M> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.sub2(rhs)
    }
}

impl<M: Montgomery> Mul for MontgomeryModInt<M> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mul2(rhs)
    }
}
