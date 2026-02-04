/// A byte scanner.
#[derive(Debug)]
pub struct Scanner<'a> {
    /// The byte slice to scan.
    bytes: &'a [u8],

    /// The index at which we currently are. To guarantee safety, it must always
    /// hold that: `0 <= cursor <= bytes.len()`.
    cursor: usize,
}

impl<'a> Scanner<'a> {
    /// Creates a new `Scanner` from the given byte slice.
    #[inline]
    pub fn new(bytes: &'a [u8]) -> Self {
        Scanner { bytes, cursor: 0 }
    }

    /// Whether the scanner has fully consumed the byte slice.
    #[inline]
    pub fn done(&self) -> bool {
        self.cursor == self.bytes.len()
    }

    /// The subslice before the cursor.
    #[inline]
    pub fn before(&self) -> &'a [u8] {
        // Safety: cursor is always in [0, bytes.len()].
        debug_assert!(self.cursor <= self.bytes.len());
        unsafe { self.bytes.get_unchecked(..self.cursor) }
    }

    /// The subslice after the cursor.
    #[inline]
    pub fn after(&self) -> &'a [u8] {
        // Safety: cursor is always in [0, bytes.len()].
        debug_assert!(self.cursor <= self.bytes.len());
        unsafe { self.bytes.get_unchecked(self.cursor..) }
    }

    /// The subslice before and after the cursor.
    #[inline]
    pub fn parts(&self) -> (&'a [u8], &'a [u8]) {
        (self.before(), self.after())
    }

    /// The byte right behind the cursor.
    #[inline]
    pub fn peek(&self) -> Option<u8> {
        self.after().first().cloned()
    }

    /// Consume and return the byte right behind the cursor.
    #[inline]
    pub fn eat(&mut self) -> Option<u8> {
        let peeked = self.peek();
        self.cursor += 1;
        peeked
    }
}

impl<'a> Scanner<'a> {
    pub fn eat_variable_length_quantity(&mut self) -> Option<u32> {
        let mut value: u32 = 0;
        for _ in 0..4 {
            let byte = self.eat()?;
            value = (value << 7) | (byte as u32 & 0x7F);
            if byte & 0x80 == 0 {
                return Some(value);
            }
        }
        // Eat one last byte without the continuation bit.
        let byte = self.eat()?;
        value = (value << 7) | (byte as u32 & 0x7F);
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eat_variable_length_quantity() {
        let test_cases: &[(&[u8], u32)] = &[
            (&[0x00], 0x00000000),
            (&[0x40], 0x00000040),
            (&[0x7F], 0x0000007F),
            (&[0x81, 0x00], 0x00000080),
            (&[0xC0, 0x00], 0x00002000),
            (&[0xFF, 0x7F], 0x00003FFF),
            (&[0x81, 0x80, 0x00], 0x00004000),
            (&[0xC0, 0x80, 0x00], 0x00100000),
            (&[0xFF, 0xFF, 0x7F], 0x001FFFFF),
            (&[0x81, 0x80, 0x80, 0x00], 0x00200000),
            (&[0xC0, 0x80, 0x80, 0x00], 0x08000000),
            (&[0xFF, 0xFF, 0xFF, 0x7F], 0x0FFFFFFF),
        ];
        for &(data, expected) in test_cases {
            let mut scanner = Scanner::new(data);
            let value = scanner.eat_variable_length_quantity();
            assert_eq!(value, Some(expected));
            assert!(scanner.done());
        }
    }
}
