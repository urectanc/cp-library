use num_traits::PrimitiveInteger;
use rational::Rational;

#[derive(Clone, Copy)]
pub struct SternBrocotTree<T> {
    left: Rational<T>,
    right: Rational<T>,
}

impl<I: PrimitiveInteger> SternBrocotTree<I> {
    pub fn new(r: Rational<I>) -> Self {
        assert!(r.num() >= I::one() && r.denom() >= I::one());
        let path = r.to_continued_fraction();
        Self::from(path)
    }

    pub fn root() -> Self {
        Self {
            left: Rational::zero(),
            right: Rational::inf(),
        }
    }

    pub fn val(&self) -> Rational<I> {
        Rational::new(
            self.left.num() + self.right.num(),
            self.left.denom() + self.right.denom(),
        )
    }

    pub fn lower_bound(&self) -> Rational<I> {
        self.left
    }

    pub fn upper_bound(&self) -> Rational<I> {
        self.right
    }

    pub fn path(&self) -> Vec<I> {
        self.val().to_continued_fraction()
    }

    pub fn nth_left(&self, n: I) -> Self {
        assert!(n >= I::zero());
        let right = Rational::new(
            self.right.num() + self.left.num() * n,
            self.right.denom() + self.left.denom() * n,
        );
        Self {
            left: self.left,
            right,
        }
    }

    pub fn nth_right(&self, n: I) -> Self {
        assert!(n >= I::zero());
        let left = Rational::new(
            self.left.num() + self.right.num() * n,
            self.left.denom() + self.right.denom() * n,
        );
        Self {
            left,
            right: self.right,
        }
    }

    /// # Reference
    /// - [[Library Checker] Rational Approximation | maspyのHP](https://maspypy.com/library-checker-rational-approximation)
    pub fn binary_search(f: impl Fn(Rational<I>) -> bool, lim: I) -> Self {
        assert!(lim > I::zero());

        let check = |x: Rational<I>, ok: bool| x.num() <= lim && x.denom() <= lim && f(x) == ok;
        let step = |a: Rational<I>, b: Rational<I>, ok: bool| {
            if a.num() > lim || a.denom() > lim {
                return a;
            }
            let go = |n: I| Rational::new(a.num() + n * b.num(), a.denom() + n * b.denom());
            let (mut l, mut w) = (I::zero(), I::one());
            while check(go(l + w), ok) {
                (l, w) = (l + w, w + w);
            }
            while w > I::one() {
                w = w.midpoint(I::zero());
                if check(go(l + w), ok) {
                    l += w;
                }
            }
            go(l)
        };

        let mut node = Self::root();
        while node.val().num() <= lim && node.val().denom() <= lim {
            node.left = step(node.left, node.right, true);
            node.right = step(node.right, node.left, false);
        }
        node
    }
}

impl<I, T> From<T> for SternBrocotTree<I>
where
    T: AsRef<[I]>,
    I: PrimitiveInteger,
{
    fn from(value: T) -> Self {
        let path = value.as_ref();
        let mut node = Self::root();
        for (i, &a) in path.iter().enumerate() {
            node = if i % 2 == 0 {
                node.nth_right(a)
            } else {
                node.nth_left(a)
            };
        }
        node
    }
}
