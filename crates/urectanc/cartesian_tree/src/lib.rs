pub struct CartesianTree<const MAX: bool> {
    root: usize,
    left: Vec<usize>,
    right: Vec<usize>,
}

impl<const MAX: bool> CartesianTree<MAX> {
    pub fn new<T: Ord>(a: impl AsRef<[T]>) -> Self {
        let a = a.as_ref();
        assert!(!a.is_empty());
        let n = a.len();
        let mut left = vec![!0; n];
        let mut right = vec![!0; n];

        let mut stack = vec![];
        for (i, a_i) in a.iter().enumerate() {
            let mut p = !0;
            while let Some((j, _)) = stack.pop_if(|&mut (_, a_j)| (a_j < a_i) == MAX) {
                right[j] = p;
                p = j;
            }
            left[i] = p;
            stack.push((i, a_i));
        }

        for w in stack.windows(2) {
            right[w[0].0] = w[1].0;
        }

        let root = stack[0].0;
        Self { root, left, right }
    }

    pub fn root(&self) -> usize {
        self.root
    }

    pub fn left(&self, i: usize) -> Option<usize> {
        (self.left[i] != !0).then_some(self.left[i])
    }

    pub fn right(&self, i: usize) -> Option<usize> {
        (self.right[i] != !0).then_some(self.right[i])
    }

    pub fn dfs(&self, mut f: impl FnMut(usize, usize, usize)) {
        let n = self.left.len();
        let mut stack = vec![(0, n, self.root())];
        while let Some((l, r, m)) = stack.pop() {
            f(l, r, m);

            if let Some(x) = self.left(m) {
                stack.push((l, m, x));
            }

            if let Some(x) = self.right(m) {
                stack.push((m + 1, r, x));
            }
        }
    }
}
