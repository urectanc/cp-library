use std::io::BufRead;

type TokenIter<'a> = std::iter::Peekable<std::str::SplitAsciiWhitespace<'a>>;

pub fn stdin() -> LineInput<std::io::StdinLock<'static>> {
    LineInput::new(std::io::stdin().lock())
}

pub fn stdout() -> std::io::BufWriter<std::io::StdoutLock<'static>> {
    std::io::BufWriter::new(std::io::stdout().lock())
}

pub trait Input {
    fn next_token(&mut self) -> &str;

    fn val<T: std::str::FromStr>(&mut self) -> T {
        let token = self.next_token();
        token.parse().unwrap_or_else(|_| {
            panic!(
                "Failed to parse token `{}` as {}",
                token,
                std::any::type_name::<T>(),
            )
        })
    }

    fn usize1(&mut self) -> usize {
        self.val::<usize>().checked_sub(1).unwrap()
    }

    fn vec<T: std::str::FromStr>(&mut self, len: usize) -> Vec<T> {
        (0..len).map(|_| self.val::<T>()).collect()
    }

    fn bytes(&mut self) -> Vec<u8> {
        self.val::<String>().into_bytes()
    }

    fn chars(&mut self) -> Vec<char> {
        self.val::<String>().chars().collect()
    }
}

pub struct LineInput<R> {
    reader: R,
    current_line: String,
    iter: TokenIter<'static>,
}

impl<R: BufRead> LineInput<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            current_line: String::new(),
            iter: "".split_ascii_whitespace().peekable(),
        }
    }

    fn read_line(&mut self) {
        self.current_line.clear();
        self.reader.read_line(&mut self.current_line).unwrap();
        self.iter = unsafe {
            std::mem::transmute::<TokenIter<'_>, TokenIter<'static>>(
                self.current_line
                    .as_str()
                    .split_ascii_whitespace()
                    .peekable(),
            )
        };
    }
}

impl<R: BufRead> Input for LineInput<R> {
    fn next_token(&mut self) -> &str {
        while self.iter.peek().is_none() {
            self.read_line();
        }

        self.iter.next().unwrap()
    }
}
