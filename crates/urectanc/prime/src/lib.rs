pub fn miller_rabin(n: u64) -> bool {
    const WITNESS_3: [u64; 3] = [2, 7, 61];
    const WITNESS_7: [u64; 7] = [2, 325, 9375, 28178, 450775, 9780504, 1795265022];
    const THRESHOLD: u64 = 4_759_123_141;

    if n == 1 || n.is_multiple_of(2) {
        return n == 2;
    }

    let witness = if n < THRESHOLD {
        &WITNESS_3[..]
    } else {
        &WITNESS_7[..]
    };

    let m = Montgomery::new(n);
    let (one, minus_one) = (m.encode(1), m.encode(n - 1));
    let d = n >> (n - 1).trailing_zeros();

    for &a in witness {
        if n <= a {
            continue;
        }

        let mut d = d;
        let mut y = m.pow(m.encode(a), d);
        while d != n - 1 && !m.eq(y, one) && !m.eq(y, minus_one) {
            y = m.mul(y, y);
            d <<= 1;
        }
        if !m.eq(y, minus_one) && d & 1 == 0 {
            return false;
        }
    }

    true
}

/// # Reference
///
/// - [ポラード・ロー素因数分解法について #競技プログラミング - Qiita](https://qiita.com/t_fuki/items/7cd50de54d3c5d063b4a)
/// - [60bit整数を高速に素因数分解する - よーる](https://lpha-z.hatenablog.com/entry/2024/02/11/231500)
pub fn factorize(mut n: u64) -> Vec<u64> {
    assert!(n > 0);

    if n == 1 {
        return vec![];
    }

    let mut res = vec![];

    while let Some(p) = pollard_brent(n) {
        res.push(p);
        n /= p;
    }

    res.sort_unstable();
    res
}

fn pollard_brent(n: u64) -> Option<u64> {
    const M: usize = 512;

    if n <= 1 {
        return None;
    }

    if n.is_multiple_of(2) {
        return Some(2);
    }

    if miller_rabin(n) {
        return Some(n);
    }

    let montgomery = Montgomery::new(n);

    for c in 1..n {
        let f = |a: u64| montgomery.mul_add(a, a, c);
        let (mut x, mut y, mut ys) = (0, 0, 0);
        let (mut g, mut q, mut r, mut k) = (1, 1, 1, 0);

        while g == 1 {
            x = y;
            while k < r && g == 1 {
                ys = y;
                for _ in 0..M.min(r - k) {
                    y = f(y);
                    q = montgomery.mul(q, x.abs_diff(y));
                }
                g = gcd(q, n);
                k += M;
            }
            k = r;
            r <<= 1;
        }

        if g == n {
            g = 1;
            y = ys;
            while g == 1 {
                y = f(y);
                g = gcd(x.abs_diff(y), n);
            }
        }

        if g != n {
            return if miller_rabin(g) {
                Some(g)
            } else if miller_rabin(n / g) {
                Some(n / g)
            } else {
                pollard_brent(g)
            };
        }
    }

    unreachable!()
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    if a == 0 || b == 0 {
        return a + b;
    }

    let shift = (a | b).trailing_zeros();
    a >>= a.trailing_zeros();
    b >>= b.trailing_zeros();

    while a != b {
        if a > b {
            a -= b;
            a >>= a.trailing_zeros();
        } else {
            b -= a;
            b >>= b.trailing_zeros();
        }
    }

    a << shift
}

struct Montgomery {
    n: u64,
    n_inv: u64,
    r2: u64,
}

impl Montgomery {
    fn new(modulus: u64) -> Self {
        assert!(modulus < 1 << 61);
        assert!(modulus & 1 == 1);
        let n = modulus;
        let mut n_inv = 1u64;
        for _ in 0..6 {
            n_inv = n_inv.wrapping_mul(2u64.wrapping_sub(n.wrapping_mul(n_inv)));
        }
        let r2 = ((n as u128).wrapping_neg() % (n as u128)) as u64;
        Self { n, n_inv, r2 }
    }

    fn eq(&self, a: u64, b: u64) -> bool {
        let d = a.abs_diff(b);
        d == 0 || d == self.n
    }

    fn encode(&self, a: u64) -> u64 {
        self.mul(a, self.r2)
    }

    fn mul(&self, a: u64, b: u64) -> u64 {
        self.mul_add(a, b, 0)
    }

    fn mul_add(&self, a: u64, b: u64, c: u64) -> u64 {
        assert!(a < self.n * 2);
        assert!(b < self.n * 2);
        assert!(c < self.n);
        // a * b + c < (4n + 1)n < Rn
        let t = a as u128 * b as u128;
        let tc = ((t >> 64) as u64).wrapping_add(c);
        let m = self.n_inv.wrapping_mul(t as u64);
        let mn = ((m as u128 * self.n as u128) >> 64) as u64;
        tc.wrapping_sub(mn).wrapping_add(self.n)
    }

    pub fn pow(&self, a: u64, mut b: u64) -> u64 {
        let mut res = self.encode(1);
        let mut pow = a;
        while b > 0 {
            if b & 1 == 1 {
                res = self.mul(res, pow);
            }
            pow = self.mul(pow, pow);
            b >>= 1;
        }
        res
    }
}
