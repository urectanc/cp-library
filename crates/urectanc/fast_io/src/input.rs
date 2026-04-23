use std::{io::Read, os::fd::FromRawFd};

// sys/mman.h
mod mman {
    use std::ffi::{c_int, c_void};

    pub const PROT_READ: c_int = 1;
    pub const MAP_PRIVATE: c_int = 2;

    #[link(name = "c")]
    unsafe extern "C" {
        pub unsafe fn mmap(
            addr: *mut c_void,
            len: usize,
            prot: c_int,
            flags: c_int,
            fd: c_int,
            offset: isize,
        ) -> *mut c_void;
    }
}

pub struct Input {
    cursor: *const u8,
}

impl Input {
    pub fn new(buf: &[u8]) -> Self {
        Self {
            cursor: buf.as_ptr(),
        }
    }

    pub fn stdin() -> Self {
        use mman::*;

        let mut stdin = unsafe { std::fs::File::from_raw_fd(0) };
        let buf = match stdin.metadata() {
            Ok(metadata) if metadata.is_file() => {
                let len = metadata.len() as usize;
                unsafe { mmap(std::ptr::null_mut(), len, PROT_READ, MAP_PRIVATE, 0, 0) as _ }
            }
            _ => {
                let mut buf = Vec::new();
                stdin.read_to_end(&mut buf).unwrap();
                Box::leak(buf.into_boxed_slice()).as_ptr()
            }
        };

        Self { cursor: buf }
    }

    fn seek(&mut self, offset: usize) {
        self.cursor = unsafe { self.cursor.add(offset) };
    }

    fn peek<T>(&self) -> T {
        let ptr = self.cursor as *const T;
        // FIXME: mmapした領域終端がページ末尾付近の場合が危険
        // 領域外参照かつページをまたぐとSIGBUS/SIGSEGVで落ちる
        // e.g. library-checker-problems/data_structure/staticrmq/in/small_08.in
        unsafe { std::ptr::read_unaligned(ptr) }
    }

    fn next<T>(&mut self) -> T {
        let val = self.peek();
        self.seek(std::mem::size_of::<T>());
        val
    }

    fn skip_whitespace(&mut self) {
        while self.peek::<u8>().is_ascii_whitespace() {
            self.seek(1);
        }
    }

    fn parse_neg(&mut self) -> bool {
        let neg = self.peek::<u8>() == b'-';
        self.seek(neg as usize);
        neg
    }

    fn parse_digits(&mut self, mut val: u64) -> u64 {
        loop {
            let c = self.next::<u8>();
            if c.is_ascii_whitespace() {
                break;
            }
            val = val * 10 + (c - b'0') as u64;
        }
        val
    }

    fn parse_8digits(&mut self) -> Option<u64> {
        let mut val = self.peek::<u64>() ^ 0x3030303030303030;
        if val & 0xf0f0f0f0f0f0f0f0 != 0 {
            return None;
        }
        self.seek(8);
        val = val.wrapping_mul((10 << 8) + 1) >> 8 & 0x00ff00ff00ff00ff;
        val = val.wrapping_mul((100 << 16) + 1) >> 16 & 0x0000ffff0000ffff;
        val = val.wrapping_mul((10000 << 32) + 1) >> 32;
        Some(val)
    }

    pub fn val<T: Readable>(&mut self) -> T {
        self.skip_whitespace();
        T::read(self)
    }

    pub fn vec<T: Readable>(&mut self, len: usize) -> Vec<T> {
        (0..len).map(|_| self.val()).collect()
    }

    pub fn bytes(&mut self) -> &[u8] {
        self.skip_whitespace();
        let start = self.cursor;
        while !self.peek::<u8>().is_ascii_whitespace() {
            self.seek(1);
        }
        unsafe {
            let len = self.cursor.offset_from(start) as usize;
            std::slice::from_raw_parts(start, len)
        }
    }
}

pub trait Readable {
    fn read(input: &mut Input) -> Self;
}

impl Readable for u8 {
    fn read(input: &mut Input) -> Self {
        input.parse_digits(0) as _
    }
}

impl Readable for u16 {
    fn read(input: &mut Input) -> Self {
        input.parse_digits(0) as _
    }
}

impl Readable for u32 {
    fn read(input: &mut Input) -> Self {
        let val = input.parse_8digits().unwrap_or(0);
        input.parse_digits(val) as _
    }
}

impl Readable for u64 {
    fn read(input: &mut Input) -> Self {
        let val = input.parse_8digits().map_or(0, |x| {
            input.parse_8digits().map_or(x, |y| x * 100_000_000 + y)
        });
        input.parse_digits(val)
    }
}

impl Readable for usize {
    fn read(input: &mut Input) -> Self {
        u64::read(input) as _
    }
}

macro_rules! impl_readable_signed {
    ($signed:ty, $unsigned:ty) => {
        impl Readable for $signed {
            fn read(input: &mut Input) -> Self {
                let neg = input.parse_neg();
                let val = <$unsigned>::read(input) as Self;
                if neg { -val } else { val }
            }
        }
    };
}

impl_readable_signed!(i8, u8);
impl_readable_signed!(i16, u16);
impl_readable_signed!(i32, u32);
impl_readable_signed!(i64, u64);
impl_readable_signed!(isize, usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_u64() {
        let input = "    123456789012345678  -998877665544332211 1234567890    -1010           ";
        let mut input = Input::new(input.as_bytes());
        assert_eq!(input.val::<u64>(), 123456789012345678);
        assert_eq!(input.val::<i64>(), -998877665544332211);
        assert_eq!(input.val::<u64>(), 1234567890);
        assert_eq!(input.val::<i64>(), -1010);
    }
}
