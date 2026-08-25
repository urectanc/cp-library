use std::{
    collections::VecDeque,
    iter::{Product, Sum},
    ops::{Add, AddAssign, Div, Index, IndexMut, Mul, MulAssign, Rem, Shl, Shr, Sub, SubAssign},
};

use modint::{ModInt, Modulus};
use number_theoretic_transform::{NTTFriendly, convolve};

use super::Polynomial;

impl<M> Index<usize> for Polynomial<M> {
    type Output = ModInt<M>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.coeff[index]
    }
}

impl<M> IndexMut<usize> for Polynomial<M> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.coeff[index]
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

impl<M: NTTFriendly> Div for Polynomial<M> {
    type Output = Polynomial<M>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(mut self, mut rhs: Self) -> Self::Output {
        self.normalize();
        rhs.normalize();
        let Some(d) = self.deg().checked_sub(rhs.deg()).map(|d| d + 1) else {
            return Self::zero();
        };

        self.coeff.reverse();
        rhs.coeff.reverse();

        let mut quo = self * rhs.inv(d).unwrap();
        quo.coeff.truncate(d);
        quo.coeff.reverse();
        quo
    }
}

impl<M: NTTFriendly> Rem for Polynomial<M> {
    type Output = Polynomial<M>;

    fn rem(self, rhs: Self) -> Self::Output {
        let mut rem = self.clone() - self / rhs.clone() * rhs;
        rem.normalize();
        rem
    }
}

impl<M: NTTFriendly> Polynomial<M> {
    pub fn div_rem(self, rhs: Self) -> (Self, Self) {
        let quo = self.clone() / rhs.clone();
        let mut rem = self - &quo * &rhs;
        rem.normalize();
        (quo, rem)
    }
}

impl<M: Modulus> Shl<usize> for Polynomial<M> {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn shl(mut self, rhs: usize) -> Self::Output {
        self.coeff.resize(self.deg() + rhs, 0.into());
        self.coeff.rotate_right(rhs);
        self
    }
}

impl<M: Modulus> Shr<usize> for Polynomial<M> {
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
