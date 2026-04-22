#![allow(clippy::missing_safety_doc)]
use std::arch::x86_64::{
    __m256i, _mm256_add_epi32, _mm256_add_epi64, _mm256_bsrli_epi128, _mm256_loadu_si256,
    _mm256_min_epu32, _mm256_mul_epu32, _mm256_or_si256, _mm256_set1_epi32, _mm256_storeu_si256,
    _mm256_sub_epi32,
};

use super::Montgomery;

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn load(ptr: *const u32) -> __m256i {
    unsafe { _mm256_loadu_si256(ptr as _) }
}

#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn store(dst: *mut u32, val: __m256i) {
    unsafe { _mm256_storeu_si256(dst.cast(), val) };
}

#[inline]
#[target_feature(enable = "avx2")]
pub fn broadcast(val: u32) -> __m256i {
    _mm256_set1_epi32(val as _)
}

#[inline]
#[target_feature(enable = "avx2")]
pub fn normalize<M: Montgomery>(val: __m256i) -> __m256i {
    _mm256_min_epu32(val, _mm256_sub_epi32(val, M::NX8))
}

#[inline]
#[target_feature(enable = "avx2")]
pub fn normalize2<M: Montgomery>(val: __m256i) -> __m256i {
    _mm256_min_epu32(val, _mm256_sub_epi32(val, M::N2X8))
}

#[inline]
#[target_feature(enable = "avx2")]
pub fn add2<M: Montgomery>(lhs: __m256i, rhs: __m256i) -> __m256i {
    normalize2::<M>(_mm256_add_epi32(lhs, rhs))
}

#[inline]
#[target_feature(enable = "avx2")]
pub fn sub2<M: Montgomery>(lhs: __m256i, rhs: __m256i) -> __m256i {
    normalize2::<M>(_mm256_sub_epi32(_mm256_add_epi32(lhs, M::N2X8), rhs))
}

#[inline]
#[target_feature(enable = "avx2")]
pub fn mul2<M: Montgomery>(lhs: __m256i, rhs: __m256i) -> __m256i {
    let lhs1 = _mm256_bsrli_epi128(lhs, 4);
    let rhs1 = _mm256_bsrli_epi128(rhs, 4);
    let t0 = _mm256_mul_epu32(lhs, rhs);
    let t1 = _mm256_mul_epu32(lhs1, rhs1);
    let m0 = _mm256_mul_epu32(t0, M::N_PRIMEX8);
    let m1 = _mm256_mul_epu32(t1, M::N_PRIMEX8);
    let res0 = _mm256_add_epi64(t0, _mm256_mul_epu32(m0, M::NX8));
    let res1 = _mm256_add_epi64(t1, _mm256_mul_epu32(m1, M::NX8));
    _mm256_or_si256(_mm256_bsrli_epi128(res0, 4), res1)
}
