//! # Reference
//! - [Foldable-Queue(SWAG) およびそのDeque版について #アルゴリズム - Qiita](https://qiita.com/Shirotsume/items/4a2837b5895ef9a7aeb1)

use algebra::Monoid;

pub struct SWAGDeque<M: Monoid> {
    front: Vec<M::Elem>,
    back: Vec<M::Elem>,
    acc_front: Vec<M::Elem>,
    acc_back: Vec<M::Elem>,
}

impl<M: Monoid> SWAGDeque<M> {
    pub fn new() -> Self {
        Self {
            front: vec![],
            back: vec![],
            acc_front: vec![M::identity()],
            acc_back: vec![M::identity()],
        }
    }

    fn rebuild(&mut self) {
        self.acc_front.truncate(1);
        self.acc_front
            .extend(self.front.iter().scan(M::identity(), |acc, x| {
                *acc = M::op(x, acc);
                Some(acc.clone())
            }));

        self.acc_back.truncate(1);
        self.acc_back
            .extend(self.back.iter().scan(M::identity(), |acc, x| {
                *acc = M::op(acc, x);
                Some(acc.clone())
            }));
    }

    pub fn push_front(&mut self, x: M::Elem) {
        let acc = self.acc_front.last().unwrap();
        self.acc_front.push(M::op(&x, acc));
        self.front.push(x);
    }

    pub fn push_back(&mut self, x: M::Elem) {
        let acc = self.acc_back.last().unwrap();
        self.acc_back.push(M::op(acc, &x));
        self.back.push(x);
    }

    pub fn pop_front(&mut self) -> Option<M::Elem> {
        if self.front.is_empty() {
            balance(&mut self.back, &mut self.front);
            self.rebuild();
        }
        self.front.pop().inspect(|_| {
            self.acc_front.pop();
        })
    }

    pub fn pop_back(&mut self) -> Option<M::Elem> {
        if self.back.is_empty() {
            balance(&mut self.front, &mut self.back);
            self.rebuild();
        }
        self.back.pop().inspect(|_| {
            self.acc_back.pop();
        })
    }

    pub fn prod(&self) -> M::Elem {
        M::op(
            self.acc_front.last().unwrap(),
            self.acc_back.last().unwrap(),
        )
    }
}

impl<M: Monoid> Default for SWAGDeque<M> {
    fn default() -> Self {
        Self::new()
    }
}

fn balance<T>(src: &mut Vec<T>, dst: &mut Vec<T>) {
    let mid = src.len().div_ceil(2);
    dst.append(&mut src.split_off(mid));
    std::mem::swap(src, dst);
    dst.reverse();
    debug_assert!(!dst.is_empty() || src.is_empty());
}
