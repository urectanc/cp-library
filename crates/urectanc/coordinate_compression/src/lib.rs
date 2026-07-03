pub struct VecCompress<T> {
    value: Vec<T>,
}

impl<T> VecCompress<T>
where
    T: Copy + Ord,
{
    pub fn size(&self) -> usize {
        self.value.len()
    }

    pub fn values(&self) -> &[T] {
        &self.value
    }

    pub fn value(&self, compressed: usize) -> T {
        self.value[compressed]
    }

    pub fn compress(&self, val: T) -> usize {
        self.value.partition_point(|&x| x < val)
    }
}

impl<T> FromIterator<T> for VecCompress<T>
where
    T: Copy + Ord,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut raw_val: Vec<_> = iter.into_iter().collect();
        raw_val.sort_unstable();
        raw_val.dedup();
        Self { value: raw_val }
    }
}

pub struct InPlaceCompress<T> {
    value: Vec<T>,
}

impl<T> InPlaceCompress<T>
where
    T: Copy + TryInto<usize>,
{
    pub fn size(&self) -> usize {
        self.value.len()
    }

    pub fn values(&self) -> &[T] {
        &self.value
    }

    pub fn value(&self, compressed: T) -> T {
        self.value[T::try_into(compressed).ok().unwrap()]
    }
}

impl<'a, T> FromIterator<&'a mut T> for InPlaceCompress<T>
where
    T: Copy + Ord + TryFrom<usize>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a mut T>,
    {
        let mut refs: Vec<_> = iter.into_iter().collect();
        refs.sort_unstable();
        let raw_val = refs
            .chunk_by_mut(|a, b| a == b)
            .enumerate()
            .map(|(i, chunk)| {
                let raw_val = *chunk[0];
                let compressed = T::try_from(i).ok().unwrap();
                chunk.iter_mut().for_each(|v| **v = compressed);
                raw_val
            })
            .collect();
        Self { value: raw_val }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inplace_compress_round_trip() {
        let a: Vec<i32> = vec![3, 1, 4, 1, 5, 9];

        let mut compressed = a.clone();
        let compress: InPlaceCompress<_> = compressed.iter_mut().collect();
        assert_eq!(compressed, vec![1, 0, 2, 0, 3, 4]);

        let decompressed: Vec<_> = compressed.iter().map(|&x| compress.value(x)).collect();
        assert_eq!(a, decompressed);
    }
}
