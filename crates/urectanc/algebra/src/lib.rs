pub trait Monoid {
    type Elem: Clone;

    fn identity() -> Self::Elem;

    fn op(lhs: &Self::Elem, rhs: &Self::Elem) -> Self::Elem;
}

pub trait Group: Monoid {
    fn inv(elem: &Self::Elem) -> Self::Elem;
}

pub trait MapMonoid: Monoid {
    type Map: Clone;

    fn identity_map() -> Self::Map;

    fn apply(x: &Self::Elem, f: &Self::Map) -> Self::Elem;

    /// `g(f(x))`
    fn compose(f: &Self::Map, g: &Self::Map) -> Self::Map;
}

pub struct Reverse<M> {
    _phantom: std::marker::PhantomData<M>,
}

impl<M: Monoid> Monoid for Reverse<M> {
    type Elem = M::Elem;

    fn identity() -> Self::Elem {
        M::identity()
    }

    fn op(lhs: &Self::Elem, rhs: &Self::Elem) -> Self::Elem {
        M::op(rhs, lhs)
    }
}

pub struct Reversible<M: Monoid> {
    _phantom: std::marker::PhantomData<M>,
}

impl<M: Monoid> Monoid for Reversible<M> {
    type Elem = (M::Elem, M::Elem);

    #[inline]
    fn identity() -> Self::Elem {
        (M::identity(), M::identity())
    }

    #[inline]
    fn op(lhs: &Self::Elem, rhs: &Self::Elem) -> Self::Elem {
        (M::op(&lhs.0, &rhs.0), M::op(&rhs.1, &lhs.1))
    }
}

pub struct MonoidArray<M, const N: usize> {
    _phantom: std::marker::PhantomData<M>,
}

impl<M: Monoid, const N: usize> Monoid for MonoidArray<M, N> {
    type Elem = [M::Elem; N];

    #[inline]
    fn identity() -> Self::Elem {
        std::array::from_fn(|_| M::identity())
    }

    #[inline]
    fn op(lhs: &Self::Elem, rhs: &Self::Elem) -> Self::Elem {
        std::array::from_fn(|i| M::op(&lhs[i], &rhs[i]))
    }
}
