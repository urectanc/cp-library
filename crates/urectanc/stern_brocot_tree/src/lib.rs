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

    pub fn binary_search(f: impl Fn(Rational<I>) -> bool, n: I) -> Self {
        assert!(n > I::zero());

        let go = |node: &Self, d: I, to_left: bool| {
            if to_left {
                node.nth_left(d)
            } else {
                node.nth_right(d)
            }
        };

        let over = |node: &Self, to_left: bool| {
            let v = node.val();
            v.num() > n || v.denom() > n || f(v) == to_left
        };

        let mut node = Self::root();
        let mut to_left = over(&node, false);
        loop {
            let (mut ok, mut ng) = (I::zero(), I::one());

            while !over(&go(&node, ng, to_left), to_left) {
                (ok, ng) = (ng, ng + ng);
            }

            while ng - ok > I::one() {
                let mid = ok.midpoint(ng);
                if over(&go(&node, mid, to_left), to_left) {
                    ng = mid;
                } else {
                    ok = mid;
                }
            }

            node = go(&node, ng, to_left);
            let v = node.val();
            if v.num() > n || v.denom() > n {
                return node;
            }

            to_left ^= true;
        }
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
