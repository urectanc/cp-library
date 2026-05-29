use modint::{Modulus, StaticModInt};

use super::{Polynomial, modinv_table};

pub struct SparsePolynomial<M> {
    coeff: Vec<(usize, StaticModInt<M>)>,
}

impl<M: Modulus> SparsePolynomial<M> {
    pub fn inv(self, precision: usize) -> Option<Polynomial<M>> {
        let c0 = self
            .coeff
            .first()
            .and_then(|&(i, c)| (i == 0).then_some(c))?;
        let scale = c0.inv();

        if precision == 0 {
            return Some(Polynomial::zero().prefix(precision));
        }

        let mut g = vec![Default::default(); precision];
        g[0] = scale;
        for i in 1..precision {
            let mut gi = StaticModInt::<M>::zero();
            for &(j, fj) in self.coeff.iter().take_while(|&&(j, _)| j <= i) {
                gi -= fj * g[i - j];
            }
            g[i] = gi * scale;
        }

        Some(g.into())
    }

    pub fn log(self, precision: usize) -> Option<Polynomial<M>> {
        self.coeff
            .first()
            .and_then(|&(i, c)| (i == 0 && c == 1.into()).then_some(()))?;

        if precision == 0 {
            return Some(Polynomial::zero().prefix(precision));
        }

        let inv = modinv_table::<M>(precision);
        let mut g = vec![Default::default(); precision];
        for &(i, fi) in self.coeff[1..].iter().take_while(|&&(i, _)| i < precision) {
            g[i] = fi;
        }
        for i in 1..precision {
            let mut gi = g[i] * i;
            for &(j, fj) in self.coeff[1..].iter().take_while(|&&(j, _)| j < i) {
                gi -= fj * g[i - j] * (i - j);
            }
            g[i] = gi * inv[i];
        }

        Some(g.into())
    }

    pub fn exp(self, precision: usize) -> Option<Polynomial<M>> {
        if self.coeff.is_empty() {
            return Some(Polynomial::one().prefix(precision));
        }
        self.coeff
            .first()
            .and_then(|&(i, _)| (i > 0).then_some(()))?;

        let inv = modinv_table::<M>(precision);
        let mut g = vec![Default::default(); precision];
        g[0] = StaticModInt::<M>::one();
        for i in 1..precision {
            let mut ci = StaticModInt::<M>::zero();
            for &(j, fj) in self.coeff.iter().take_while(|&&(j, _)| j <= i) {
                ci += fj * g[i - j] * j;
            }
            g[i] = ci * inv[i];
        }
        Some(g.into())
    }

    pub fn pow(mut self, exp: usize, precision: usize) -> Polynomial<M> {
        if exp == 0 {
            return Polynomial::one().prefix(precision);
        }

        let Some(&(shift, f0)) = self.coeff.first() else {
            return Polynomial::zero().prefix(precision);
        };

        self.coeff.iter_mut().for_each(|(i, _)| *i -= shift);
        let offset = shift.saturating_mul(exp).min(precision);
        let precision = precision - offset;

        if precision == 0 {
            return Polynomial::zero().prefix(offset + precision);
        }

        let inv = modinv_table::<M>(precision);
        let mut g = vec![Default::default(); precision];
        g[0] = f0.pow(exp as _);
        let scale = f0.inv();
        let e1 = StaticModInt::<M>::one() + exp;
        for i in 1..precision {
            let mut gi = StaticModInt::<M>::zero();
            for &(j, fj) in self.coeff[1..].iter().take_while(|&&(j, _)| j <= i) {
                gi += fj * g[i - j] * (e1 * j - i);
            }
            g[i] = gi * inv[i] * scale;
        }

        std::iter::repeat_n(StaticModInt::<M>::zero(), offset)
            .chain(g)
            .collect()
    }
}

impl<M: Modulus> From<Polynomial<M>> for SparsePolynomial<M> {
    fn from(f: Polynomial<M>) -> Self {
        f.into_iter().enumerate().collect()
    }
}

impl<M: Modulus> FromIterator<(usize, StaticModInt<M>)> for SparsePolynomial<M> {
    fn from_iter<T: IntoIterator<Item = (usize, StaticModInt<M>)>>(iter: T) -> Self {
        let mut coeff: Vec<_> = iter
            .into_iter()
            .filter(|&(_, c)| c != StaticModInt::<M>::zero())
            .collect();
        coeff.sort_unstable_by_key(|&(i, _)| i);
        Self { coeff }
    }
}

impl<M: Modulus> From<SparsePolynomial<M>> for Polynomial<M> {
    fn from(f: SparsePolynomial<M>) -> Self {
        let Some(deg) = f.coeff.iter().map(|&(i, _)| i).max() else {
            return Polynomial::zero();
        };
        let mut coeff = vec![StaticModInt::<M>::zero(); deg + 1];
        for &(i, c) in &f.coeff {
            coeff[i] = c;
        }
        coeff.into()
    }
}
