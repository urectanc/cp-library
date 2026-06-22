pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    if a == 0 || b == 0 {
        return a + b;
    }

    let shift = (a | b).trailing_zeros();
    a >>= a.trailing_zeros();
    b >>= b.trailing_zeros();

    while a != b {
        if a > b {
            a -= b;
            a >>= a.trailing_zeros();
        } else {
            b -= a;
            b >>= b.trailing_zeros();
        }
    }

    a << shift
}
