//! # References
//!
//! - [Making NTT convolution 10x faster with avx2](https://piskareviv.github.io/blog_aux_ntt_1/)
//! - [tayu-procon/number-theoretic-transform](https://github.com/tayu0110/tayu-procon/tree/master/number-theoretic-transform)
//! - [競プロ 数論変換 除数 一覧 ( FFT NTT MOD 一覧 )  |  Mathenachia](https://www.mathenachia.blog/ntt-mod-list-01/)

use modint::{Mod998244353, Modulus, StaticModInt};
use montgomery::{Montgomery, MontgomeryModInt};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::__m256i;
use std::mem::transmute;

#[cfg(target_arch = "x86_64")]
mod simd;

pub trait NTTFriendly: Montgomery {
    const PRIMITIVE_ROOT: u32;

    const BUTTERFLY_CACHE: ButterflyCache<Self> = ButterflyCache::new();
}

impl NTTFriendly for Mod998244353 {
    const PRIMITIVE_ROOT: u32 = 3;
}

macro_rules! define_ntt_friendly_modulus {
    ($name:ident, $modulus:expr, $primitive_root:expr) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        struct $name;
        impl Modulus for $name {
            const MOD: u32 = const {
                assert!($modulus < (1u32 << 31));
                $modulus
            };
        }

        impl Montgomery for $name {}

        impl NTTFriendly for $name {
            const PRIMITIVE_ROOT: u32 = $primitive_root;
        }
    };
}

define_ntt_friendly_modulus!(Mod167772161, 167772161, 3);
define_ntt_friendly_modulus!(Mod469762049, 469762049, 3);
define_ntt_friendly_modulus!(Mod754974721, 754974721, 11);

pub trait NumberTheoreticTransform<M, T: ?Sized> {
    fn ntt(&mut self);

    fn intt(&mut self);

    fn hadamard(&mut self, rhs: &T);
}

impl<M, T: ?Sized> NumberTheoreticTransform<M, T> for T
where
    T: AsRef<[StaticModInt<M>]> + AsMut<[StaticModInt<M>]>,
    M: NTTFriendly,
{
    fn ntt(&mut self) {
        let f = unsafe { transmute::<&mut [StaticModInt<M>], &mut [ModInt<M>]>(self.as_mut()) };
        assert!(f.len() <= (1 << (M::MOD - 1).trailing_zeros()));
        transform::<M>(f);
    }

    fn intt(&mut self) {
        let f = unsafe { transmute::<&mut [StaticModInt<M>], &mut [ModInt<M>]>(self.as_mut()) };
        assert!(f.len() <= (1 << (M::MOD - 1).trailing_zeros()));
        inverse_transform::<M>(f);
    }

    fn hadamard(&mut self, rhs: &T) {
        std::iter::zip(self.as_mut(), rhs.as_ref()).for_each(|(l, r)| *l *= r);
    }
}

pub fn convolve<M: NTTFriendly>(
    lhs: impl AsRef<[StaticModInt<M>]>,
    rhs: impl AsRef<[StaticModInt<M>]>,
) -> Vec<StaticModInt<M>> {
    let mut lhs = lhs.as_ref().to_owned();
    let mut rhs = rhs.as_ref().to_owned();
    let new_len = lhs.len() + rhs.len() - 1;
    let ntt_len = new_len.next_power_of_two();
    lhs.resize(ntt_len, 0.into());
    rhs.resize(ntt_len, 0.into());
    lhs.ntt();
    rhs.ntt();
    lhs.hadamard(&rhs);
    lhs.intt();
    lhs.truncate(new_len);
    lhs
}

pub fn convolve_mod_arbitrary(
    lhs: impl AsRef<[u32]>,
    rhs: impl AsRef<[u32]>,
    modulus: u32,
) -> Vec<u32> {
    // lhs.len() + rhs.len() - 1 <= 2^24
    // l = min(lhs.len(), rhs.len()) <= 2^23
    // lm^2 <= m^2 * 2^23 < M1M2M3
    assert!(modulus <= 2663300486);

    const M1: u64 = Mod167772161::MOD as _;
    const M2: u64 = Mod469762049::MOD as _;

    let m = modulus as u64;
    let m1m2 = (M1 * M2) % m;
    let inv_m1 = StaticModInt::<Mod469762049>::from(M1).inv();
    let inv_m1m2 = StaticModInt::<Mod754974721>::from(M1 * M2).inv();

    let (lhs, rhs) = (lhs.as_ref(), rhs.as_ref());
    let r1 = convolve::<Mod167772161>(
        lhs.iter().copied().map(Into::into).collect::<Vec<_>>(),
        rhs.iter().copied().map(Into::into).collect::<Vec<_>>(),
    );
    let r2 = convolve::<Mod469762049>(
        lhs.iter().copied().map(Into::into).collect::<Vec<_>>(),
        rhs.iter().copied().map(Into::into).collect::<Vec<_>>(),
    );
    let r3 = convolve::<Mod754974721>(
        lhs.iter().copied().map(Into::into).collect::<Vec<_>>(),
        rhs.iter().copied().map(Into::into).collect::<Vec<_>>(),
    );
    r1.iter()
        .zip(&r2)
        .zip(&r3)
        .map(|((r1, r2), r3)| {
            let c1 = r1.val() as u64;
            let c2 = ((r2 - c1) * inv_m1).val() as u64;
            let c3 = ((r3 - c1 - c2 * M1) * inv_m1m2).val() as u64;
            // M1 + M2M1 + M3m < 2^64
            ((c1 + c2 * M1 + c3 * m1m2) % m) as u32
        })
        .collect()
}

type ModInt<M> = MontgomeryModInt<M>;

pub struct ButterflyCache<M: NTTFriendly> {
    imag: ModInt<M>,
    iimag: ModInt<M>,
    rate1: [ModInt<M>; 30],
    irate1: [ModInt<M>; 30],
    rate2: [ModInt<M>; 30],
    irate2: [ModInt<M>; 30],
    #[cfg(target_arch = "x86_64")]
    w1230: __m256i,
    #[cfg(target_arch = "x86_64")]
    iw1230: __m256i,
    #[cfg(target_arch = "x86_64")]
    rate1230: [__m256i; 30],
    #[cfg(target_arch = "x86_64")]
    irate1230: [__m256i; 30],
}

impl<M: NTTFriendly> ButterflyCache<M> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        let lg = (M::MOD - 1).trailing_zeros() as usize;

        // (2^lg)-th root of M
        let mut r = ModInt::<M>::new(M::PRIMITIVE_ROOT).pow((M::MOD - 1) >> lg);
        let mut ir = r.inv();

        // root[i] = r^(bitrev(2^i))
        let mut root = [ModInt::<M>::zero(); 30];
        let mut iroot = [ModInt::<M>::zero(); 30];
        // rate1[i] = r^(bitrev(k+1)) / r^(bitrev(k)) where k.trailing_ones() == i
        let mut rate1 = [ModInt::<M>::zero(); 30];
        let mut irate1 = [ModInt::<M>::zero(); 30];
        // rate2[i] = r^(bitrev(k+2)) / r^(bitrev(k)) where k.trailing_ones() == i
        let mut rate2 = [ModInt::<M>::zero(); 30];
        let mut irate2 = [ModInt::<M>::zero(); 30];

        let mut i = lg;
        while i > 0 {
            i -= 1;
            root[i] = r.normalize();
            iroot[i] = ir.normalize();
            r = r.mul2(r);
            ir = ir.mul2(ir);
        }

        let one = ModInt::<M>::one();
        let mut rate1230 = [[ModInt::<M>::zero(); 8]; 30];
        let mut irate1230 = [[ModInt::<M>::zero(); 8]; 30];
        let mut acc = ModInt::<M>::one();
        let mut iacc = ModInt::<M>::one();
        let mut i = 0;
        while i < lg - 1 {
            let r3 = root[i + 3].mul2(iacc).normalize();
            let ir3 = iroot[i + 3].mul2(acc).normalize();
            let r2 = r3.mul2(r3).normalize();
            let ir2 = ir3.mul2(ir3).normalize();
            let r1 = r2.mul2(r2).normalize();
            let ir1 = ir2.mul2(ir2).normalize();
            rate1[i] = r1;
            irate1[i] = ir1;
            rate2[i] = r2;
            irate2[i] = ir2;
            rate1230[i] = [r3, r3, r3, r3, r2, r2, r1, one];
            irate1230[i] = [ir3, ir3, ir3, ir3, ir2, ir2, ir1, one];
            acc = root[i + 3].mul2(acc);
            iacc = iroot[i + 3].mul2(iacc);
            i += 1;
        }

        let (deg90, deg45, deg135) = (root[1], root[2], root[1].mul2(root[2]).normalize());
        let (deg270, deg315, deg225) = (iroot[1], iroot[2], iroot[1].mul2(iroot[2]).normalize());
        let w1230 = [one, deg90, deg45, deg135, one, deg90, one, one];
        let iw1230 = [one, deg270, deg315, deg225, one, deg270, one, one];

        Self {
            imag: deg90,
            iimag: deg270,
            rate1,
            irate1,
            rate2,
            irate2,
            #[cfg(target_arch = "x86_64")]
            w1230: unsafe { transmute::<[ModInt<M>; 8], __m256i>(w1230) },
            #[cfg(target_arch = "x86_64")]
            iw1230: unsafe { transmute::<[ModInt<M>; 8], __m256i>(iw1230) },
            #[cfg(target_arch = "x86_64")]
            rate1230: unsafe { transmute::<[[ModInt<M>; 8]; 30], [__m256i; 30]>(rate1230) },
            #[cfg(target_arch = "x86_64")]
            irate1230: unsafe { transmute::<[[ModInt<M>; 8]; 30], [__m256i; 30]>(irate1230) },
        }
    }
}

fn transform<M: NTTFriendly>(f: &mut [ModInt<M>]) {
    if f.len() >= 8 && is_x86_feature_detected!("avx") {
        unsafe { simd::transform_avx2(f) };
        return;
    }

    let &ButterflyCache {
        imag, ref rate2, ..
    } = &M::BUTTERFLY_CACHE;

    let n = f.len();
    assert!(n.is_power_of_two());

    let log = n.trailing_zeros();
    if log % 2 == 1 {
        let (b0, b1) = f.split_at_mut(n / 2);
        for (x0, x1) in std::iter::zip(b0, b1) {
            (*x0, *x1) = (*x0 + *x1, *x0 - *x1);
        }
    }

    for k in (0..log).step_by(2).rev() {
        let block_size = 1 << k;
        let mut w = ModInt::<M>::one();
        for (i, chunk) in f.chunks_exact_mut(4 * block_size).enumerate() {
            let w2 = w * w;
            let w3 = w2 * w;
            let (b01, b23) = chunk.split_at_mut(2 * block_size);
            let (b0, b1) = b01.split_at_mut(block_size);
            let (b2, b3) = b23.split_at_mut(block_size);
            for (((x0, x1), x2), x3) in b0.iter_mut().zip(b1).zip(b2).zip(b3) {
                let (y0, y1, y2, y3) = (*x0, *x1 * w, *x2 * w2, *x3 * w3);
                let (z0, z1, z2, z3) = (y0 + y2, y1 + y3, y0 - y2, (y1 - y3) * imag);
                (*x0, *x1, *x2, *x3) = (z0 + z1, z0 - z1, z2 + z3, z2 - z3);
            }
            w = w * rate2[i.trailing_ones() as usize];
        }
    }
}

fn inverse_transform<M: NTTFriendly>(f: &mut [ModInt<M>]) {
    if f.len() >= 8 && is_x86_feature_detected!("avx") {
        unsafe { simd::inverse_transform_avx2(f) };
        return;
    }

    let &ButterflyCache {
        iimag, ref irate2, ..
    } = &M::BUTTERFLY_CACHE;

    let n = f.len();
    assert!(n.is_power_of_two());

    let log = n.trailing_zeros();
    for k in (0..log).step_by(2) {
        let block_size = 1 << k;
        let mut w = ModInt::<M>::one();
        for (i, chunk) in f.chunks_exact_mut(4 * block_size).enumerate() {
            let (b01, b23) = chunk.split_at_mut(2 * block_size);
            let (b0, b1) = b01.split_at_mut(block_size);
            let (b2, b3) = b23.split_at_mut(block_size);

            let w2 = w * w;
            let w3 = w2 * w;
            for (((x0, x1), x2), x3) in b0.iter_mut().zip(b1).zip(b2).zip(b3) {
                let (y0, y1, y2, y3) = (*x0, *x1, *x2, *x3);
                let (z0, z1, z2, z3) = (y0 + y1, y0 - y1, y2 + y3, (y2 - y3) * iimag);
                (*x0, *x1, *x2, *x3) = (z0 + z2, (z1 + z3) * w, (z0 - z2) * w2, (z1 - z3) * w3);
            }
            w = w * irate2[i.trailing_ones() as usize];
        }
    }

    if log % 2 == 1 {
        let (b0, b1) = f.split_at_mut(n / 2);
        for (x0, x1) in std::iter::zip(b0, b1) {
            (*x0, *x1) = (*x0 + *x1, *x0 - *x1);
        }
    }

    let inv_n = ModInt::<M>::new(n as u32).inv();
    f.iter_mut().for_each(|x| *x = (*x * inv_n).normalize());
}
