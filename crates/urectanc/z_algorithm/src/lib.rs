pub fn z_algorithm(s: &[impl Ord]) -> Vec<usize> {
    if s.is_empty() {
        return vec![];
    }

    let n = s.len();
    let mut z = vec![0; n];
    z[0] = n;
    let (mut l, mut r) = (0, 1usize);
    for i in 1..n {
        if z[i - l] < r.saturating_sub(i) {
            z[i] = z[i - l];
        } else {
            z[i] = r.saturating_sub(i).min(z[i - l]);
            while i + z[i] < n && s[z[i]] == s[i + z[i]] {
                z[i] += 1;
            }
            if r < i + z[i] {
                (l, r) = (i, i + z[i]);
            }
        }
    }

    z
}
