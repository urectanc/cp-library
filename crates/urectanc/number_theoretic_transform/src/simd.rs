use montgomery::simd::*;
use std::arch::x86_64::{
    _mm256_add_epi32, _mm256_blend_epi32, _mm256_permute2x128_si256, _mm256_permutevar8x32_epi32,
    _mm256_setr_epi32, _mm256_shuffle_epi32, _mm256_sub_epi32,
};

use super::{ButterflyCache, ModInt, NTTFriendly};

#[target_feature(enable = "avx2")]
pub unsafe fn transform_avx2<M: NTTFriendly>(f: &mut [ModInt<M>]) {
    let &ButterflyCache {
        ref rate1,
        w1230,
        ref rate1230,
        ..
    } = &M::BUTTERFLY_CACHE;

    let n = f.len();
    assert!(n.is_power_of_two() && n >= 8);

    let log = n.trailing_zeros();
    for k in (3..log).rev() {
        let block_size = 2 << k;
        let offset = block_size >> 1;

        transform_block::<M, true, true>(&mut f[..], offset, ModInt::one());
        let mut w = rate1[0];
        for (i, block) in f.chunks_exact_mut(block_size).enumerate().skip(1) {
            transform_block::<M, true, false>(block, offset, w);
            w = w * rate1[i.trailing_ones() as usize];
        }
    }

    let mut wx8 = w1230;
    for (i, chunk) in f.chunks_exact_mut(8).enumerate() {
        let w0x8 = _mm256_permutevar8x32_epi32(wx8, _mm256_setr_epi32(7, 0, 7, 1, 7, 2, 7, 3));
        let w1x8 = _mm256_permutevar8x32_epi32(wx8, _mm256_setr_epi32(7, 7, 4, 4, 7, 7, 5, 5));
        let w2x8 = _mm256_permutevar8x32_epi32(wx8, _mm256_setr_epi32(7, 7, 7, 7, 6, 6, 6, 6));
        let head = chunk.as_mut_ptr().cast();
        let mut vec = unsafe { load(head) };

        vec = mul2::<M>(vec, w2x8);
        let a_negb = _mm256_blend_epi32::<0b11110000>(vec, _mm256_sub_epi32(M::N2X8, vec));
        let b_a = _mm256_permute2x128_si256::<1>(vec, vec);
        // omit normalize2()
        vec = _mm256_add_epi32(a_negb, b_a);

        vec = mul2::<M>(vec, w1x8);
        let a_negb = _mm256_blend_epi32::<0b11001100>(vec, _mm256_sub_epi32(M::N2X8, vec));
        let b_a = _mm256_shuffle_epi32::<0b01_00_11_10>(vec);
        vec = _mm256_add_epi32(a_negb, b_a);

        vec = mul2::<M>(vec, w0x8);
        let a_negb = _mm256_blend_epi32::<0b10101010>(vec, _mm256_sub_epi32(M::N2X8, vec));
        let b_a = _mm256_shuffle_epi32::<0b10_11_00_01>(vec);
        vec = add2::<M>(a_negb, b_a);

        unsafe { store(head, vec) };
        wx8 = normalize::<M>(mul2::<M>(wx8, rate1230[i.trailing_ones() as usize]));
    }
}

#[target_feature(enable = "avx2")]
pub unsafe fn inverse_transform_avx2<M: NTTFriendly>(f: &mut [ModInt<M>]) {
    let &ButterflyCache {
        ref irate1,
        iw1230,
        ref irate1230,
        ..
    } = &M::BUTTERFLY_CACHE;

    let n = f.len();
    assert!(n.is_power_of_two() && n >= 8);

    let mut wx8 = iw1230;
    for (i, chunk) in f.chunks_exact_mut(8).enumerate() {
        let w0x8 = _mm256_permutevar8x32_epi32(wx8, _mm256_setr_epi32(7, 0, 7, 1, 7, 2, 7, 3));
        let w1x8 = _mm256_permutevar8x32_epi32(wx8, _mm256_setr_epi32(7, 7, 4, 4, 7, 7, 5, 5));
        let w2x8 = _mm256_permutevar8x32_epi32(wx8, _mm256_setr_epi32(7, 7, 7, 7, 6, 6, 6, 6));
        let head = chunk.as_mut_ptr().cast();
        let mut vec = unsafe { load(head) };

        let a_negb = _mm256_blend_epi32::<0b10101010>(vec, _mm256_sub_epi32(M::N2X8, vec));
        let b_a = _mm256_shuffle_epi32::<0b10_11_00_01>(vec);
        vec = mul2::<M>(_mm256_add_epi32(a_negb, b_a), w0x8);

        let a_negb = _mm256_blend_epi32::<0b11001100>(vec, _mm256_sub_epi32(M::N2X8, vec));
        let b_a = _mm256_shuffle_epi32::<0b01_00_11_10>(vec);
        vec = mul2::<M>(_mm256_add_epi32(a_negb, b_a), w1x8);

        let a_negb = _mm256_blend_epi32::<0b11110000>(vec, _mm256_sub_epi32(M::N2X8, vec));
        let b_a = _mm256_permute2x128_si256::<1>(vec, vec);
        vec = mul2::<M>(_mm256_add_epi32(a_negb, b_a), w2x8);

        unsafe { store(head, vec) };
        wx8 = normalize::<M>(mul2::<M>(wx8, irate1230[i.trailing_ones() as usize]));
    }

    let log = n.trailing_zeros();
    for k in 3..log {
        let block_size = 2 << k;
        let offset = block_size >> 1;

        transform_block::<M, false, true>(&mut f[..], offset, ModInt::one());
        let mut w = irate1[0];
        for (i, block) in f.chunks_exact_mut(block_size).enumerate().skip(1) {
            transform_block::<M, false, false>(block, offset, w);
            w = w * irate1[i.trailing_ones() as usize];
        }
    }

    let inv_n = ModInt::<M>::new(n as u32).inv();
    let inv_nx8 = broadcast(inv_n.val);
    let ptr = f.as_mut_ptr();
    for i in (0..n).step_by(8) {
        let ptr = unsafe { ptr.add(i) };
        let mut a = unsafe { load(ptr.cast()) };
        a = normalize::<M>(mul2::<M>(a, inv_nx8));
        unsafe { store(ptr.cast(), a) };
    }
}

#[inline]
#[target_feature(enable = "avx2")]
fn transform_block<M: NTTFriendly, const FORWARD: bool, const TRIVIAL: bool>(
    block: &mut [ModInt<M>],
    offset: usize,
    w: ModInt<M>,
) {
    let head: *mut u32 = block.as_mut_ptr().cast();
    let wx8 = broadcast(w.val);
    for i in (0..offset).step_by(8) {
        let (ptr_a, ptr_b) = unsafe { (head.add(i), head.add(offset + i)) };
        let (mut a, mut b) = unsafe { (load(ptr_a), load(ptr_b)) };
        if FORWARD && !TRIVIAL {
            b = mul2::<M>(b, wx8);
        }
        (a, b) = (add2::<M>(a, b), sub2::<M>(a, b));
        if !FORWARD && !TRIVIAL {
            b = mul2::<M>(b, wx8);
        }
        unsafe {
            store(ptr_a, a);
            store(ptr_b, b);
        }
    }
}
