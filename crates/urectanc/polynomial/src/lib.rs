//! # References
//!
//! - [多項式/形式的冪級数ライブラリ | Nyaan’s Library](https://nyaannyaan.github.io/library/fps/formal-power-series.hpp)
//! - [[多項式・形式的べき級数] 高速に計算できるものたち | maspyのHP](https://maspypy.com/%e5%a4%9a%e9%a0%85%e5%bc%8f%e3%83%bb%e5%bd%a2%e5%bc%8f%e7%9a%84%e3%81%b9%e3%81%8d%e7%b4%9a%e6%95%b0-%e9%ab%98%e9%80%9f%e3%81%ab%e8%a8%88%e7%ae%97%e3%81%a7%e3%81%8d%e3%82%8b%e3%82%82%e3%81%ae#toc7)
//! - [FFT の回数を削減するテクニック集](https://noshi91.hatenablog.com/entry/2023/12/10/163348)

use std::{
    collections::VecDeque,
    iter::{Product, Sum},
    ops::{Add, AddAssign, Index, IndexMut, Mul, MulAssign, Shl, Shr, Sub, SubAssign},
};

use modint::{Modulus, StaticModInt};
use number_theoretic_transform::{NTTFriendly, NumberTheoreticTransform, convolve};

#[derive(Clone)]
pub struct Polynomial<M> {
    coeff: Vec<StaticModInt<M>>,
}

impl<M: Modulus> Polynomial<M> {
    pub fn zero() -> Self {
        vec![].into()
    }

    pub fn one() -> Self {
        vec![1.into()].into()
    }

    pub fn deg(&self) -> usize {
        self.coeff.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, StaticModInt<M>> {
        self.coeff.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, StaticModInt<M>> {
        self.coeff.iter_mut()
    }

    pub fn prefix(&self, len: usize) -> Self {
        self.coeff
            .iter()
            .copied()
            .chain(std::iter::repeat(0.into()))
            .take(len)
            .collect()
    }

    pub fn derivative(&self) -> Self {
        self.coeff
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, x)| x * i)
            .collect()
    }

    pub fn integral(&self) -> Self {
        if self.deg() == 0 {
            return Self::zero();
        }

        let mut inv = modinv_table::<M>(self.deg());
        inv[1..]
            .iter_mut()
            .zip(&self.coeff)
            .for_each(|(a, b)| *a *= b);
        inv.into()
    }
}

impl<M: NTTFriendly> Polynomial<M> {
    pub fn inv(&self, precision: usize) -> Option<Self> {
        (self[0] != 0.into()).then_some(())?;

        let mut inv = Self::from(vec![self[0].inv()]);
        while inv.deg() < precision {
            self.refine_inv(&mut inv);
        }

        inv.coeff.truncate(precision);
        Some(inv)
    }

    fn refine_inv(&self, inv: &mut Self) {
        // g_2n - g_n = -g_n(fg_n - 1)
        let n = inv.deg();
        let mut f = self.prefix(2 * n);
        let mut g = inv.prefix(2 * n);
        // cyclic convolution mod (x^2n - 1)
        f.coeff.ntt();
        g.coeff.ntt();
        f.coeff.hadamard(&g.coeff);
        f.coeff.intt();
        // clear [0, n) and [2n, 3n)
        f.coeff[..n].fill(0.into());
        f.coeff.ntt();
        f.coeff.hadamard(&g.coeff);
        f.coeff.intt();
        inv.coeff.extend(f.coeff[n..].iter().map(|&f| -f));
    }

    pub fn log(&self, precision: usize) -> Option<Self> {
        (self.deg() > 0 && self[0] == 1.into()).then_some(())?;
        let inv = self.inv(precision)?;
        Some((self.derivative() * inv).integral().prefix(precision))
    }

    pub fn exp(&self, precision: usize) -> Option<Self> {
        if self.deg() == 0 {
            return Some(Self::one());
        }
        (self[0] == 0.into()).then_some(())?;

        let modinv = modinv_table::<M>(precision);
        let mut exp = Self::one();
        let mut exp_inv = Self::one();
        while exp.deg() < precision {
            let n = exp.deg();
            let mut f = exp.prefix(2 * n);
            let mut g = exp_inv.prefix(2 * n);
            f.coeff.ntt();
            g.coeff.ntt();

            let mut h = self.prefix(n);
            h.coeff.iter_mut().enumerate().for_each(|(i, w)| *w *= i);
            h.coeff.ntt();
            // MAGIC: here we may use ntt(exp[..2n])[..n] instead of ntt(exp[..n]).
            h.coeff[..].hadamard(&f.coeff[..n]);
            h.coeff.intt();

            // x(f' - fh') mod (x^n - 1)
            let mut s = exp.prefix(2 * n);
            s.coeff[..n]
                .iter_mut()
                .enumerate()
                .for_each(|(i, w)| *w *= i);
            s -= h;

            s.coeff.ntt();
            s.coeff.hadamard(&g.coeff);
            s.coeff.intt();

            let mut u = self.prefix(2 * n);
            u.coeff[..n].fill(0.into());
            u.iter_mut()
                .zip(&modinv)
                .skip(n)
                .zip(s.coeff)
                .for_each(|((u, inv), s)| *u -= s * inv);
            u.coeff.ntt();
            u.coeff.hadamard(&f.coeff);
            u.coeff.intt();

            exp.coeff.extend_from_slice(&u.coeff[n..]);

            if exp.deg() < precision {
                exp.refine_inv(&mut exp_inv);
            }
        }

        exp.coeff.truncate(precision);
        Some(exp)
    }

    pub fn pow(&self, exp: usize, precision: usize) -> Self {
        if exp == 0 {
            return Self::one().prefix(precision);
        }
        let Some(shift) = self.iter().position(|&c| c != 0.into()) else {
            return Self::zero().prefix(precision);
        };

        let c = self.coeff[shift];
        let scale = c.inv();

        let f: Polynomial<_> = self.iter().skip(shift).map(|a| a * scale).collect();
        let mut f = f.log(precision).unwrap();
        f.iter_mut().for_each(|a| *a *= exp);
        let pow = f.exp(precision).unwrap();

        let scale = c.pow(exp as u64);
        std::iter::repeat_n(0.into(), shift.saturating_mul(exp))
            .chain(pow.into_iter().map(|a| a * scale))
            .take(precision)
            .collect()
    }

    /// # References
    ///
    /// - [線形漸化的数列のN項目の計算 #アルゴリズム - Qiita](https://qiita.com/ryuhe1/items/da5acbcce4ac1911f47a)
    pub fn bostan_mori(num: &Self, denom: &Self, mut k: usize) -> StaticModInt<M> {
        assert!(num.deg() < denom.deg());
        let ntt_len = (2 * denom.deg() - 1).next_power_of_two();
        let mut p = num.prefix(ntt_len);
        let mut q = denom.prefix(ntt_len);

        while k > 0 {
            p.coeff.ntt();
            q.coeff.ntt();
            let mut r = q.clone();
            for i in (0..ntt_len).step_by(2) {
                r.coeff.swap(i, i + 1);
            }
            p.coeff.hadamard(&r.coeff);
            q.coeff.hadamard(&r.coeff);
            p.coeff.intt();
            q.coeff.intt();
            p = p.into_iter().skip(k & 1).step_by(2).collect();
            q = q.into_iter().step_by(2).collect();
            p.coeff.resize(ntt_len, 0.into());
            q.coeff.resize(ntt_len, 0.into());
            k >>= 1;
        }

        p[0] / q[0]
    }
}

impl<M, T> From<T> for Polynomial<M>
where
    M: Modulus,
    T: AsRef<[StaticModInt<M>]>,
{
    fn from(value: T) -> Self {
        Self {
            coeff: value.as_ref().to_owned(),
        }
    }
}

impl<M, S> FromIterator<S> for Polynomial<M>
where
    M: Modulus,
    S: Into<StaticModInt<M>>,
{
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self::from(iter.into_iter().map(Into::into).collect::<Vec<_>>())
    }
}

impl<M: Modulus> IntoIterator for Polynomial<M> {
    type Item = StaticModInt<M>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.coeff.into_iter()
    }
}

impl<'a, M: Modulus> IntoIterator for &'a Polynomial<M> {
    type Item = &'a StaticModInt<M>;
    type IntoIter = std::slice::Iter<'a, StaticModInt<M>>;

    fn into_iter(self) -> Self::IntoIter {
        self.coeff.iter()
    }
}

impl<'a, M: Modulus> IntoIterator for &'a mut Polynomial<M> {
    type Item = &'a mut StaticModInt<M>;
    type IntoIter = std::slice::IterMut<'a, StaticModInt<M>>;

    fn into_iter(self) -> Self::IntoIter {
        self.coeff.iter_mut()
    }
}

impl<M> Index<usize> for Polynomial<M> {
    type Output = StaticModInt<M>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.coeff[index]
    }
}

impl<M> IndexMut<usize> for Polynomial<M> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.coeff[index]
    }
}

impl<M: Modulus> std::fmt::Debug for Polynomial<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.coeff).finish()
    }
}

impl<M: Modulus> Add for Polynomial<M> {
    type Output = Polynomial<M>;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<M: Modulus> AddAssign for Polynomial<M> {
    fn add_assign(&mut self, mut rhs: Self) {
        if self.deg() < rhs.deg() {
            std::mem::swap(self, &mut rhs);
        }
        self.iter_mut().zip(rhs).for_each(|(l, r)| *l += r);
    }
}

impl<M: Modulus> Sub for Polynomial<M> {
    type Output = Polynomial<M>;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<M: Modulus> SubAssign for Polynomial<M> {
    fn sub_assign(&mut self, mut rhs: Self) {
        if self.deg() < rhs.deg() {
            std::mem::swap(self, &mut rhs);
            self.iter_mut().for_each(|l| *l = -*l);
            *self += rhs;
        } else {
            self.iter_mut().zip(rhs).for_each(|(l, r)| *l -= r);
        }
    }
}

impl<M: NTTFriendly> Mul for Polynomial<M> {
    type Output = Polynomial<M>;

    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}

impl<M: NTTFriendly> Mul for &Polynomial<M> {
    type Output = Polynomial<M>;

    fn mul(self, rhs: Self) -> Self::Output {
        convolve(&self.coeff, &rhs.coeff).into()
    }
}

impl<M: NTTFriendly> MulAssign for Polynomial<M> {
    fn mul_assign(&mut self, rhs: Self) {
        self.coeff = convolve(&self.coeff, &rhs.coeff);
    }
}

impl<M: NTTFriendly> Shl<usize> for Polynomial<M> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn shl(mut self, rhs: usize) -> Self::Output {
        self.coeff.resize(self.deg() + rhs, 0.into());
        self.coeff.rotate_right(rhs);
        self
    }
}

impl<M: NTTFriendly> Shr<usize> for Polynomial<M> {
    type Output = Self;

    fn shr(mut self, rhs: usize) -> Self::Output {
        self.coeff.rotate_left(rhs);
        self.coeff.truncate(self.deg().saturating_sub(rhs));
        self
    }
}

impl<M: Modulus> Sum for Polynomial<M> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item)
    }
}

impl<M: NTTFriendly> Product for Polynomial<M> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut que: VecDeque<_> = iter.collect();
        while que.len() > 1 {
            let f = que.pop_front().unwrap();
            let g = que.pop_front().unwrap();
            que.push_back(f * g);
        }
        que.pop_front().unwrap_or(Self::one())
    }
}

pub fn berlekamp_massey<M: Modulus>(a: &[StaticModInt<M>]) -> Polynomial<M> {
    let mut b = vec![-StaticModInt::raw(1)];
    let mut c = vec![-StaticModInt::raw(1)];
    let mut y = StaticModInt::raw(1);
    let mut shift = 0;
    for i in 0..a.len() {
        shift += 1;
        let x = a[..=i]
            .iter()
            .rev()
            .zip(&c)
            .map(|(a, c)| a * c)
            .sum::<StaticModInt<M>>();
        if x == 0.into() {
            continue;
        }
        let r = x / y;
        if c.len() < b.len() + shift {
            let old_c = c.clone();
            c.resize(b.len() + shift, 0.into());
            c[shift..]
                .iter_mut()
                .zip(std::mem::replace(&mut b, old_c))
                .for_each(|(c, b)| *c -= r * b);
            y = x;
            shift = 0;
        } else {
            c[shift..].iter_mut().zip(&b).for_each(|(c, b)| *c -= r * b);
        }
    }
    c.into_iter().skip(1).collect()
}

fn modinv_table<M: Modulus>(n: usize) -> Vec<StaticModInt<M>> {
    let mut inv = vec![StaticModInt::raw(0); n + 1];
    if n > 0 {
        inv[1] = 1.into();
    }
    let m = M::MOD as usize;
    for i in 2..=n {
        inv[i] = -inv[m % i] * (m / i);
    }
    inv
}
