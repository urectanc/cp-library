use std::fmt::Debug;

pub struct CSRArray<T> {
    n: usize,
    index: Vec<usize>,
    csr: Vec<T>,
}

impl<T> CSRArray<T> {
    pub fn new(n: usize, items: impl AsRef<[(usize, T)]>) -> Self
    where
        T: Copy + Default,
    {
        let items = items.as_ref();
        let mut index = vec![0; n + 1];
        for &(k, _) in items {
            index[k] += 1;
        }
        for i in 0..n {
            index[i + 1] += index[i];
        }

        let m = items.len();
        let mut csr = vec![T::default(); m];
        for &(k, v) in items.iter().rev() {
            index[k] -= 1;
            csr[index[k]] = v;
        }

        Self { n, index, csr }
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    pub fn get(&self, i: usize) -> Option<&[T]> {
        (i < self.n).then(|| &self.csr[self.index[i]..self.index[i + 1]])
    }

    pub fn iter(&'_ self) -> Row<'_, T> {
        Row {
            csr: self,
            index: 0,
        }
    }
}

impl<T> std::ops::Index<usize> for CSRArray<T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<'a, T> IntoIterator for &'a CSRArray<T> {
    type Item = &'a [T];
    type IntoIter = Row<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Debug> Debug for CSRArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_list();
        for row in self {
            f.entry(&row);
        }
        f.finish()
    }
}

pub struct Row<'a, T> {
    csr: &'a CSRArray<T>,
    index: usize,
}

impl<'a, T> Iterator for Row<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        let row = self.csr.get(self.index)?;
        self.index += 1;
        Some(row)
    }
}
