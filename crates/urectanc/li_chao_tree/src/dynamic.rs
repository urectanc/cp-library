use std::ops::Range;

use super::Function;

pub struct DynamicLiChaoTree<F, X> {
    nodes: Vec<Node<F>>,
    range: Range<X>,
}

impl<F> DynamicLiChaoTree<F, i64>
where
    F: Function<i64>,
{
    pub fn new(range: Range<i64>) -> Self {
        let root = Node::new(F::inf());
        Self {
            nodes: vec![root],
            range,
        }
    }

    fn new_node(&mut self, f: F) -> usize {
        self.nodes.push(Node::new(f));
        self.nodes.len() - 1
    }

    pub fn add_line(&mut self, f: F) {
        self.add_line_on(0, self.range.start, self.range.end, f);
    }

    fn add_line_on(&mut self, i: usize, l: i64, r: i64, mut cand: F) -> usize {
        // nodes[i] covers [l, r)
        let Some(&mut Node {
            ref mut f,
            lch,
            rch,
        }) = self.nodes.get_mut(i)
        else {
            return self.new_node(cand);
        };

        if f.eval(l) > cand.eval(l) {
            std::mem::swap(f, &mut cand);
        }

        let m = l.midpoint(r);
        if l == m || r == m || f.eval(r) <= cand.eval(r) {
            return i;
        }

        if f.eval(m) <= cand.eval(m) {
            self.nodes[i].rch = self.add_line_on(rch as usize, m, r, cand) as u32;
        } else {
            std::mem::swap(f, &mut cand);
            self.nodes[i].lch = self.add_line_on(lch as usize, l, m, cand) as u32;
        }

        i
    }

    pub fn add_segment(&mut self, l: i64, r: i64, f: F) {
        assert!(self.range.contains(&l) && self.range.contains(&r));
        self.add_segment_on(0, self.range.start, self.range.end, l, r, f);
    }

    fn add_segment_on(&mut self, mut i: usize, l: i64, r: i64, fl: i64, fr: i64, f: F) -> usize {
        if r <= fl || fr <= l {
            return i;
        }

        if fl <= l && r <= fr {
            return self.add_line_on(i, l, r, f);
        }

        if self.nodes.len() <= i {
            i = self.new_node(F::inf());
        }

        let Node { lch, rch, .. } = self.nodes[i];
        let m = l.midpoint(r);
        if fl < m {
            self.nodes[i].lch = self.add_segment_on(lch as usize, l, m, fl, fr, f) as u32;
        }
        if m <= fr {
            self.nodes[i].rch = self.add_segment_on(rch as usize, m, r, fl, fr, f) as u32;
        }

        i
    }

    pub fn min_at(&self, x: i64) -> Option<i64> {
        assert!(self.range.contains(&x));

        let (mut i, mut l, mut r) = (0, self.range.start, self.range.end);
        let mut res = i64::MAX;
        while let Some(node) = self.nodes.get(i) {
            res = res.min(node.f.eval(x));
            let m = l.midpoint(r);
            (i, l, r) = if x < m {
                (node.lch as usize, l, m)
            } else {
                (node.rch as usize, m, r)
            };
        }
        (res != i64::MAX).then_some(res)
    }
}

struct Node<F> {
    f: F,
    lch: u32,
    rch: u32,
}

impl<F> Node<F> {
    fn new(f: F) -> Self {
        Self {
            f,
            lch: !0,
            rch: !0,
        }
    }
}
