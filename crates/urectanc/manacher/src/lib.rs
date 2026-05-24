pub fn manancher(s: &str) -> Vec<usize> {
    let n = s.len();
    let mut a = vec![b'.'; 2 * n + 3];
    a[0] = b'^';
    a[2 * n + 2] = b'$';
    for (i, s) in s.bytes().enumerate() {
        a[2 * (i + 1)] = s;
    }

    let n = 2 * n + 1;
    let mut p = vec![0; n + 2];
    let (mut l, mut r) = (1, 1);
    for i in 1..=n {
        p[i] = p[l + r - i].min(r - i);
        while a[i - p[i]] == a[i + p[i]] {
            p[i] += 1;
        }
        if i + p[i] > r {
            (l, r) = (i - p[i], i + p[i]);
        }
    }
    p[2..n].into_iter().map(|&p| p - 1).collect()
}
