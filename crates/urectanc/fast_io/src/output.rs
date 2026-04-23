use std::io::Write;

const BUF_SIZE: usize = 1 << 18;
const MIN_WRITE_CAPACITY: usize = 50;

pub struct Output<W: Write> {
    buf: [u8; BUF_SIZE],
    pos: usize,
    inner: W,
}

impl Output<std::io::StdoutLock<'static>> {
    pub fn stdout() -> Self {
        Self::new(std::io::stdout().lock())
    }
}

impl<W: Write> Drop for Output<W> {
    fn drop(&mut self) {
        self.flush();
    }
}

impl<W: Write> Output<W> {
    pub fn new(inner: W) -> Self {
        Self {
            buf: [0; BUF_SIZE],
            pos: 0,
            inner,
        }
    }

    #[cold]
    pub fn flush(&mut self) {
        self.inner
            .write_all(&self.buf[..self.pos])
            .expect("flush failed");
        self.pos = 0;
    }

    pub fn write<T: Writable<W>>(&mut self, val: T) {
        self.ensure_capacity();
        unsafe {
            T::write_unchecked(self, val);
            self.write_byte_unchecked(b' ');
        }
    }

    pub fn writeln<T: Writable<W>>(&mut self, val: T) {
        self.ensure_capacity();
        unsafe {
            T::write_unchecked(self, val);
            self.write_byte_unchecked(b'\n');
        }
    }

    #[inline]
    fn spare_capacity(&self) -> usize {
        BUF_SIZE - self.pos
    }

    fn ensure_capacity(&mut self) {
        if self.spare_capacity() < MIN_WRITE_CAPACITY {
            self.flush();
        }
    }

    unsafe fn write_byte_unchecked(&mut self, byte: u8) {
        unsafe {
            let dst = self.buf.as_mut_ptr().add(self.pos);
            std::ptr::write_unaligned(dst, byte);
        }
        self.pos += 1;
    }

    unsafe fn write_digits_unchecked<const LZ: bool>(&mut self, n: usize) {
        static TABLE: [u8; 40_000] = {
            let mut table = [b'0'; 40_000];
            let mut i = 0;
            while i < 10_000 {
                table[4 * i] += (i / 1000) as u8;
                table[4 * i + 1] += (i / 100 % 10) as u8;
                table[4 * i + 2] += (i / 10 % 10) as u8;
                table[4 * i + 3] += (i % 10) as u8;
                i += 1;
            }
            table
        };
        let offset = if LZ {
            0
        } else {
            (n < 10) as usize + (n < 100) as usize + (n < 1000) as usize
        };
        unsafe {
            let src = TABLE.as_ptr().add(4 * n + offset) as *const u32;
            let dst = self.buf.as_mut_ptr().add(self.pos) as *mut u32;
            std::ptr::write_unaligned(dst, std::ptr::read_unaligned(src));
        }
        self.pos += 4 - offset;
    }
}

pub trait Writable<W: Write> {
    unsafe fn write_unchecked(output: &mut Output<W>, val: Self);
}

impl<W: Write> Writable<W> for u32 {
    unsafe fn write_unchecked(output: &mut Output<W>, val: Self) {
        unsafe {
            if val >= 1_0000_0000 {
                output.write_digits_unchecked::<false>((val / 10000 / 10000) as usize);
                output.write_digits_unchecked::<true>((val / 10000 % 10000) as usize);
                output.write_digits_unchecked::<true>((val % 10000) as usize);
            } else if val >= 1_0000 {
                output.write_digits_unchecked::<false>((val / 10000) as usize);
                output.write_digits_unchecked::<true>((val % 10000) as usize);
            } else {
                output.write_digits_unchecked::<false>(val as usize);
            }
        }
    }
}

impl<W: Write> Writable<W> for u64 {
    unsafe fn write_unchecked(output: &mut Output<W>, val: Self) {
        unsafe {
            if val >= 1_0000_0000_0000_0000 {
                output.write_digits_unchecked::<false>(
                    (val / 10000 / 10000 / 10000 / 10000) as usize,
                );
                output
                    .write_digits_unchecked::<true>((val / 10000 / 10000 / 10000 % 10000) as usize);
                output.write_digits_unchecked::<true>((val / 10000 / 10000 % 10000) as usize);
                output.write_digits_unchecked::<true>((val / 10000 % 10000) as usize);
                output.write_digits_unchecked::<true>((val % 10000) as usize);
            } else if val >= 1_0000_0000_0000 {
                output.write_digits_unchecked::<false>((val / 10000 / 10000 / 10000) as usize);
                output.write_digits_unchecked::<true>((val / 10000 / 10000 % 10000) as usize);
                output.write_digits_unchecked::<true>((val / 10000 % 10000) as usize);
                output.write_digits_unchecked::<true>((val % 10000) as usize);
            } else if val >= 1_0000_0000 {
                output.write_digits_unchecked::<false>((val / 10000 / 10000) as usize);
                output.write_digits_unchecked::<true>((val / 10000 % 10000) as usize);
                output.write_digits_unchecked::<true>((val % 10000) as usize);
            } else if val >= 1_0000 {
                output.write_digits_unchecked::<false>((val / 10000) as usize);
                output.write_digits_unchecked::<true>((val % 10000) as usize);
            } else {
                output.write_digits_unchecked::<false>(val as usize);
            }
        }
    }
}

impl<W: Write> Writable<W> for usize {
    unsafe fn write_unchecked(output: &mut Output<W>, val: Self) {
        unsafe {
            u64::write_unchecked(output, val as _);
        }
    }
}

impl<W: Write> Writable<W> for i32 {
    unsafe fn write_unchecked(output: &mut Output<W>, val: Self) {
        unsafe {
            if val < 0 {
                output.write_byte_unchecked(b'-');
            }
            u32::write_unchecked(output, val.unsigned_abs());
        }
    }
}

impl<W: Write> Writable<W> for i64 {
    unsafe fn write_unchecked(output: &mut Output<W>, val: Self) {
        unsafe {
            if val < 0 {
                output.write_byte_unchecked(b'-');
            }
            u64::write_unchecked(output, val.unsigned_abs());
        }
    }
}

impl<W: Write> Writable<W> for &str {
    unsafe fn write_unchecked(output: &mut Output<W>, val: Self) {
        let len = val.len();
        debug_assert!(len <= MIN_WRITE_CAPACITY);
        unsafe {
            let dst = output.buf.as_mut_ptr().add(output.pos);
            std::ptr::copy_nonoverlapping(val.as_ptr(), dst, len);
        }
        output.pos += len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut buf = vec![];
        {
            let mut output = Output::<_>::new(&mut buf);
            output.write(123usize);
            output.write(-998244353i32);
        }

        let actual = String::from_utf8(buf).unwrap();
        assert_eq!(actual, "123 -998244353 ");
    }
}
