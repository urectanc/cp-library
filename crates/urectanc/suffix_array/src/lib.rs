pub struct SuffixArray<'a, T> {
    s: &'a [T],
    sa: Vec<usize>,
}

impl<'a> From<&'a str> for SuffixArray<'a, u8> {
    fn from(value: &'a str) -> Self {
        let s = value.bytes().map(i32::from).collect::<Vec<_>>();
        let sa = suffix_array_i32(&s, 1 << 8);
        Self {
            s: value.as_bytes(),
            sa,
        }
    }
}

impl<'a, T> From<&'a [T]> for SuffixArray<'a, T>
where
    T: Ord,
{
    fn from(value: &'a [T]) -> Self {
        use std::collections::{BTreeMap, BTreeSet};

        let compressed: BTreeSet<_> = value.iter().collect();
        let sigma = compressed.len();
        let map: BTreeMap<_, _> = compressed
            .into_iter()
            .enumerate()
            .map(|(i, x)| (x, i as i32))
            .collect();
        let s = value.iter().map(|x| map[x]).collect::<Vec<_>>();
        let sa = suffix_array_i32(&s, sigma);
        Self { s: value, sa }
    }
}

impl<'a, T> SuffixArray<'a, T>
where
    T: Ord,
{
    pub fn as_slice(&self) -> &'_ [usize] {
        &self.sa
    }

    /// # References
    ///
    /// - [LCP配列の構築アルゴリズムたち #データ構造 - Qiita](https://qiita.com/kgoto/items/9e28e37b8a4b15ea7230)
    pub fn lcp_array(&self) -> Vec<usize> {
        let n = self.s.len();
        let mut phi = vec![!0; n];
        for i in 1..n {
            phi[self.sa[i]] = self.sa[i - 1];
        }

        let mut h = 0usize;
        for (i, phi) in phi.iter_mut().enumerate() {
            if *phi == !0 {
                continue;
            }
            while i + h < n && *phi + h < n && self.s[i + h] == self.s[*phi + h] {
                h += 1;
            }
            *phi = h;
            h = h.saturating_sub(1);
        }

        (1..n).map(|i| phi[self.sa[i]]).collect()
    }
}

struct Buckets<'a> {
    sa: &'a mut [i32],
    end: &'a mut [i32],
    cursor: &'a mut [i32],
}

impl<'a> std::ops::Index<usize> for Buckets<'a> {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        self.sa.get(index).unwrap()
    }
}

impl<'a> Buckets<'a> {
    fn new(sa: &'a mut [i32], end: &'a mut [i32], cursor: &'a mut [i32]) -> Self {
        assert_eq!(end.len(), cursor.len());
        Self { sa, end, cursor }
    }

    fn seek_front(&mut self) {
        self.cursor[0] = 0;
        self.cursor[1..].copy_from_slice(&self.end[..self.end.len() - 1]);
    }

    fn seek_back(&mut self) {
        self.cursor.copy_from_slice(self.end);
    }

    fn push_front(&mut self, bucket: i32, val: i32) {
        let cur = &mut self.cursor[bucket as usize];
        self.sa[*cur as usize] = val;
        *cur += 1;
    }

    fn push_back(&mut self, bucket: i32, val: i32) {
        let cur = &mut self.cursor[bucket as usize];
        *cur -= 1;
        self.sa[*cur as usize] = val;
    }
}

fn suffix_array_i32(s: &[i32], sigma: usize) -> Vec<usize> {
    let n = s.len();
    let mut sa = vec![-1; n];
    sa_is(s, &mut sa, sigma);
    sa.into_iter().map(|i| i as usize).collect()
}

fn sa_is(s: &[i32], sa: &mut [i32], sigma: usize) {
    let n = s.len();
    assert_eq!(sa.len(), n);

    if n == 0 {
        return;
    }

    if n == 1 {
        sa[0] = 0;
        return;
    }

    // 0: L, 1: S, 2: LMS
    let mut types = vec![0u8; s.len()];
    for (i, c) in s.windows(2).enumerate().rev() {
        types[i] = match c[0].cmp(&c[1]) {
            std::cmp::Ordering::Less => 1,
            std::cmp::Ordering::Equal => types[i + 1],
            std::cmp::Ordering::Greater => {
                types[i + 1] <<= 1;
                0
            }
        }
    }
    let mut lms = (0..n)
        .filter(|&i| types[i] == 2)
        .map(|i| i as i32)
        .collect::<Vec<_>>();

    let mut end = vec![0; sigma];
    let mut cursor = vec![0; sigma];

    for &ch in s {
        end[ch as usize] += 1;
    }

    for i in 1..sigma {
        end[i] += end[i - 1];
    }

    let mut induced_sort = |sa: &mut [i32], lms: &[i32]| {
        sa.fill(-1);
        let mut buckets = Buckets::new(sa, &mut end, &mut cursor);

        buckets.seek_back();
        for &i in lms.iter().rev() {
            buckets.push_back(s[i as usize], i);
        }

        buckets.seek_front();
        buckets.push_front(s[n - 1], n as i32 - 1);
        for i in 0..n {
            let target = buckets[i] - 1;
            if target >= 0 && types[target as usize] == 0 {
                buckets.push_front(s[target as usize], target);
            }
        }

        buckets.seek_back();
        for i in (0..n).rev() {
            let target = buckets[i] - 1;
            if target >= 0 && types[target as usize] != 0 {
                buckets.push_back(s[target as usize], target);
            }
        }
    };

    induced_sort(sa, &lms);

    if lms.len() <= 1 {
        return;
    }

    let mut p = 0;
    for i in 0..n {
        if types[sa[i] as usize] == 2 {
            sa[p] = sa[i];
            p += 1;
        }
    }
    let (sorted_lms, tmp) = sa.split_at_mut(lms.len());

    // The rightmost LMS substring is always unique
    let mut r = 0;
    for &l in lms.iter().rev() {
        tmp[l as usize >> 1] = r - l;
        r = l + 1;
    }

    let mut id = -1;
    let (mut pl, mut plen) = (n, 0);
    for &l in &*sorted_lms {
        let l = l as usize;
        let len = tmp[l >> 1] as usize;
        if len != plen || s[l..][..len] != s[pl..][..len] {
            (pl, plen) = (l, len);
            id += 1;
        }
        tmp[l >> 1] = id;
    }

    if ((id + 1) as usize) < lms.len() {
        for (compressed, &lms) in sorted_lms.iter_mut().zip(&lms) {
            *compressed = tmp[lms as usize >> 1];
        }
        let next_sa = &mut tmp[..lms.len()];

        sa_is(sorted_lms, next_sa, id as usize + 1);

        for (decompressed, &i) in sorted_lms.iter_mut().zip(&*next_sa) {
            *decompressed = lms[i as usize];
        }
    }
    lms.copy_from_slice(sorted_lms);

    induced_sort(sa, &lms);
}
