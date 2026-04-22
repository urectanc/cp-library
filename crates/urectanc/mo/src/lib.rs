pub trait Mo {
    type Result: Clone + Default;

    fn increment_x(&mut self, x: usize);

    fn decrement_x(&mut self, x: usize);

    fn increment_y(&mut self, y: usize);

    fn decrement_y(&mut self, y: usize);

    fn query(&self) -> Self::Result;
}

pub fn solve<M: Mo>(mut data: M, queries: &[(usize, usize)]) -> Vec<M::Result> {
    let q = queries.len();
    let x_max = queries.iter().map(|&(x, _)| x).max().unwrap() as f64;
    let y_max = queries.iter().map(|&(_, y)| y).max().unwrap() as f64;
    let is_segment = queries.iter().all(|&(x, y)| x <= y);

    let coeff = if is_segment { 2.0 } else { 1.0 } / 3.0;
    let bucket_num = (coeff * q as f64 * x_max / y_max).sqrt();
    let bucket_width = (x_max / bucket_num).ceil() as usize;

    let mut sorted = queries.iter().copied().enumerate().collect::<Vec<_>>();
    sorted.sort_unstable_by_key(|&(_, (x, y))| (x / bucket_width, y));
    sorted
        .chunk_by_mut(|&(_, (x0, _)), &(_, (x1, _))| x0 / bucket_width == x1 / bucket_width)
        .skip(1)
        .step_by(2)
        .for_each(|bucket| bucket.reverse());

    let mut ans = vec![M::Result::default(); q];
    let (mut x, mut y) = (0, 0);
    for (i, (qx, qy)) in sorted {
        while qx < x {
            x -= 1;
            data.decrement_x(x);
        }
        while y < qy {
            data.increment_y(y);
            y += 1;
        }
        while x < qx {
            data.increment_x(x);
            x += 1;
        }
        while qy < y {
            y -= 1;
            data.decrement_y(y);
        }

        ans[i] = data.query();
    }

    ans
}
