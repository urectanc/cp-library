pub mod dynamic;

pub trait Function<X>: Copy {
    fn inf() -> Self;
    fn eval(&self, x: X) -> i64;
}

#[derive(Clone, Copy)]
pub struct Line(pub i64, pub i64);

impl Function<i64> for Line {
    fn inf() -> Self {
        Self(0, i64::MAX)
    }

    fn eval(&self, x: i64) -> i64 {
        self.0 * x + self.1
    }
}

pub struct LiChaoTree<'a, F, X> {
    n: usize,
    f: Vec<F>,
    x: &'a [X],
}

impl<'a, F, X> LiChaoTree<'a, F, X>
where
    F: Function<X>,
    X: Copy,
{
    pub fn new(x: &'a [X]) -> Self {
        let n = x.len();
        let f = vec![F::inf(); 2 * n];
        Self { n, f, x }
    }

    pub fn add_line(&mut self, f: F) {
        self.add_segment(0, self.n, f);
    }

    fn add_line_at(&mut self, mut i: usize, k: usize, mut cand: F) {
        let &mut Self { ref mut f, x, .. } = self;
        let mut l = (i << k) - self.n;
        let mut r = l + (1 << k);

        loop {
            let f = &mut f[i];
            if f.eval(x[l]) > cand.eval(x[l]) {
                std::mem::swap(f, &mut cand);
            }

            if l + 1 == r || f.eval(x[r - 1]) <= cand.eval(x[r - 1]) {
                return;
            }

            let m = l.midpoint(r);
            (i, l, r) = if f.eval(x[m]) <= cand.eval(x[m]) {
                (2 * i + 1, m, r)
            } else {
                std::mem::swap(f, &mut cand);
                (2 * i, l, m)
            };
        }
    }

    pub fn add_segment(&mut self, l: usize, r: usize, f: F) {
        // open range (l, r)
        let (l, r) = (self.n + l - 1, self.n + r);
        let mask = (1 << (l ^ r).ilog2()) - 1;

        // add at x^1 if x is left child
        let mut bits = !l & mask;
        while bits > 0 {
            let i = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            self.add_line_at(l >> i ^ 1, i, f);
        }

        // add at x^1 if x is right child
        let mut bits = r & mask;
        while bits > 0 {
            let i = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            self.add_line_at(r >> i ^ 1, i, f);
        }
    }

    pub fn min_at(&self, i: usize) -> Option<i64> {
        let x = self.x[i];
        std::iter::successors(Some(i + self.n), |&i| (i > 0).then_some(i >> 1))
            .map(|i| self.f[i].eval(x))
            .min()
            .filter(|&x| x != i64::MAX)
    }
}
