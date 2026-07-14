use modint::{ModInt, Modulus};

#[allow(private_bounds)]
pub trait PreCalc<M: Modulus>: Sealed {
    fn combinations(max: usize) -> Combination<M> {
        Combination::new(max)
    }

    fn inverses(max: usize) -> Vec<ModInt<M>> {
        let mut inv = vec![ModInt::one(); max + 1];
        let m = ModInt::<M>::modulus() as usize;
        for i in 2..=max {
            inv[i] = -inv[m % i] * (m / i);
        }
        inv
    }
}

trait Sealed {}

impl<M: Modulus> Sealed for ModInt<M> {}
impl<M: Modulus> PreCalc<M> for ModInt<M> {}

pub struct Combination<M> {
    fact: Vec<ModInt<M>>,
    finv: Vec<ModInt<M>>,
}

impl<M: Modulus> Combination<M> {
    pub fn new(max: usize) -> Self {
        let mut fact = vec![ModInt::one(); max + 1];
        let mut finv = vec![ModInt::one(); max + 1];
        for i in 2..=max {
            fact[i] = fact[i - 1] * i;
        }
        finv[max] = fact[max].inv();
        for i in (2..max).rev() {
            finv[i] = finv[i + 1] * (i + 1);
        }
        Self { fact, finv }
    }

    pub fn fact(&self, n: usize) -> ModInt<M> {
        self.fact[n]
    }

    pub fn finv(&self, n: usize) -> ModInt<M> {
        self.finv[n]
    }

    pub fn perm(&self, n: usize, k: usize) -> ModInt<M> {
        if n < k {
            ModInt::zero()
        } else {
            self.fact(n) * self.finv(n - k)
        }
    }

    pub fn binom(&self, n: usize, k: usize) -> ModInt<M> {
        self.perm(n, k) * self.finv(k)
    }

    // [x^n] 1/(1-x)^k = multi(k, n)
    pub fn multi(&self, n: usize, k: usize) -> ModInt<M> {
        self.binom(n + k - 1, k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use modint::ModInt998244353;

    type M = ModInt998244353;

    #[test]
    fn combinations() {
        let comb = M::combinations(10);
        assert_eq!(comb.fact(5), M::new(120));
        assert_eq!(comb.finv(5), M::new(120).inv());
        assert_eq!(comb.perm(5, 3), M::new(60));
        assert_eq!(comb.binom(5, 3), M::new(10));
        assert_eq!(comb.multi(5, 3), M::new(35));
    }

    #[test]
    fn inverses() {
        let inv = M::inverses(10);
        for i in 1..=10 {
            assert_eq!(inv[i], M::from(i).inv());
        }
    }
}
