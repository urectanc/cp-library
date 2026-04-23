type Word = u64;
type Rank = u32;

const WORD_SIZE: usize = std::mem::size_of::<Word>() * 8;

pub struct BitVector {
    bits: Vec<Word>,
    cum: Vec<Rank>,
}

impl From<&[bool]> for BitVector {
    fn from(a: &[bool]) -> Self {
        let mut builder = BitVectorBuilder::new(a.len());
        for (i, &a) in a.iter().enumerate() {
            if a {
                builder.set(i);
            }
        }
        builder.build()
    }
}

impl BitVector {
    pub fn rank(&self, k: usize) -> usize {
        let bits = self.bits[k / WORD_SIZE];
        let cum = self.cum[k / WORD_SIZE];
        let mask = (1 << (k & (WORD_SIZE - 1))) - 1;
        cum as usize + (bits & mask).count_ones() as usize
    }
}

pub struct BitVectorBuilder {
    bits: Vec<Word>,
}

impl BitVectorBuilder {
    pub fn new(n: usize) -> Self {
        let n = n.div_ceil(WORD_SIZE);
        Self {
            bits: vec![0; n + 1],
        }
    }

    pub fn set(&mut self, i: usize) {
        self.bits[i / WORD_SIZE] |= 1 << (i & (WORD_SIZE - 1));
    }

    pub fn build(self) -> BitVector {
        let n = self.bits.len() - 1;
        let mut cum = vec![0; n + 1];
        for i in 0..n {
            cum[i + 1] = cum[i] + self.bits[i].count_ones();
        }
        BitVector {
            bits: self.bits,
            cum,
        }
    }
}
