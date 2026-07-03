type Line = (i64, i64);

pub struct ConvexHullTrick<const MONOTONE: bool> {
    lines: Vec<Line>,
    head: usize,
    last_query: i64,
}

impl<const MONOTONE: bool> ConvexHullTrick<MONOTONE> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            head: 0,
            last_query: i64::MIN,
        }
    }

    pub fn add(&mut self, f: Line) {
        if let Some(g) = self.lines.last()
            && f.0 == g.0
        {
            if f.1 < g.1 {
                self.lines.pop();
            } else {
                return;
            }
        }
        assert!(self.lines.last().is_none_or(|g| f.0 < g.0));

        while !self.is_convex_hull(&f) {
            self.lines.pop();
        }
        self.lines.push(f);
    }

    fn is_convex_hull(&mut self, f: &Line) -> bool {
        // https://noshi91.hatenablog.com/entry/2021/03/23/200810
        self.lines[self.head..]
            .split_last_chunk::<2>()
            .is_none_or(|(_, &[h, g])| {
                (g.1 - h.1).div_euclid(h.0 - g.0) < (f.1 - g.1).div_euclid(g.0 - f.0)
            })
    }

    fn eval(&self, i: usize, x: i64) -> i64 {
        let f = self.lines[i];
        f.0 * x + f.1
    }

    pub fn min_at(&mut self, x: i64) -> i64 {
        assert!(!self.lines.is_empty());

        let i = if MONOTONE {
            assert!(self.last_query <= x);
            self.last_query = x;
            while self.head + 1 < self.lines.len()
                && self.eval(self.head, x) >= self.eval(self.head + 1, x)
            {
                self.head += 1;
            }
            self.head
        } else {
            let (mut l, mut r) = (0, self.lines.len());
            while r - l > 1 {
                let m = l.midpoint(r);
                if self.eval(m - 1, x) >= self.eval(m, x) {
                    l = m;
                } else {
                    r = m;
                }
            }
            l
        };
        self.eval(i, x)
    }
}
