use algebra::Group;

pub struct PotentializedUnionFind<G: Group> {
    parent_or_size: Vec<i32>,
    potential: Vec<G::Elem>,
}

impl<G> PotentializedUnionFind<G>
where
    G: Group,
    G::Elem: PartialEq,
{
    pub fn new(size: usize) -> Self {
        Self {
            parent_or_size: vec![-1; size],
            potential: vec![G::identity(); size],
        }
    }

    pub fn leader(&mut self, v: usize) -> (usize, G::Elem) {
        if self.parent_or_size[v] < 0 {
            return (v, self.potential[v].clone());
        }

        let p = self.parent_or_size[v] as usize;
        let (leader, potential) = self.leader(p);
        self.parent_or_size[v] = leader as i32;
        self.potential[v] = G::op(&potential, &self.potential[v]);

        (leader, self.potential[v].clone())
    }

    pub fn size(&mut self, v: usize) -> usize {
        let leader = self.leader(v).0;
        -self.parent_or_size[leader] as usize
    }

    pub fn same(&mut self, u: usize, v: usize) -> bool {
        self.diff(u, v).is_some()
    }

    pub fn diff(&mut self, v: usize, reference: usize) -> Option<G::Elem> {
        let (v, dv) = self.leader(v);
        let (r, dr) = self.leader(reference);
        (v == r).then_some(G::op(&G::inv(&dr), &dv))
    }

    pub fn merge(&mut self, v: usize, reference: usize, mut diff: G::Elem) -> bool {
        let (mut v, dv) = self.leader(v);
        let (mut r, dr) = self.leader(reference);

        if v == r {
            return G::op(&dr, &diff) == dv;
        }
        // lv = lr * dr * d * inv(dv)
        diff = G::op(&dr, &G::op(&diff, &G::inv(&dv)));

        if self.size(v) > self.size(r) {
            std::mem::swap(&mut v, &mut r);
            diff = G::inv(&diff);
        }

        self.parent_or_size[r] += std::mem::replace(&mut self.parent_or_size[v], r as i32);
        self.potential[v] = G::op(&self.potential[r], &diff);

        true
    }
}
