pub type Mask = u32;

pub struct SmallSet<T> {
    mask: Mask,
    inner: Vec<T>,
}

impl<T: Copy> SmallSet<T> {
    pub fn new() -> Self {
        Self {
            mask: 0,
            inner: Vec::new(),
        }
    }

    pub fn get(&self, i: u8) -> Option<T> {
        (self.mask >> i & 1 == 1).then(|| self.inner[self.pos(i)])
    }

    pub fn insert(&mut self, i: u8, v: T) {
        self.inner.insert(self.pos(i), v);
        self.mask |= 1 << i;
    }

    fn pos(&self, i: u8) -> usize {
        (self.mask & ((1 << i) - 1)).count_ones() as usize
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            set: self,
            mask: self.mask,
        }
    }
}

impl<'a, T: Copy> IntoIterator for &'a SmallSet<T> {
    type Item = <Iter<'a, T> as Iterator>::Item;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Copy> Default for SmallSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Iter<'a, T> {
    set: &'a SmallSet<T>,
    mask: u32,
}

impl<'a, T: Copy> Iterator for Iter<'a, T> {
    type Item = (u8, T);

    fn next(&mut self) -> Option<Self::Item> {
        (self.mask != 0).then(|| {
            let i = self.mask.trailing_zeros() as u8;
            self.mask &= self.mask - 1;
            (i, self.set.get(i).unwrap())
        })
    }
}
