use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

pub trait PrimitiveInteger:
    'static
    + Copy
    + Ord
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + RemAssign
{
    fn rem_euclid(self, rhs: Self) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn min_value() -> Self;
    fn max_value() -> Self;
}

macro_rules! impl_primitive_integer {
    ($($ty:ty),*) => { $(
        impl PrimitiveInteger for $ty {
            fn rem_euclid(self, rhs: Self) -> Self {
                self.rem_euclid(rhs)
            }
            fn zero() -> Self { 0 }
            fn one() -> Self { 1 }
            fn min_value() -> Self { Self::MIN }
            fn max_value() -> Self { Self::MAX }
        }
    )* };
}

impl_primitive_integer!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
