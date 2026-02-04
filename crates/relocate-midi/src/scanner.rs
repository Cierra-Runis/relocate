/// A byte scanner for efficiently reading bytes from a slice.
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

    /// The subslice after the cursor.
    #[inline]
    pub fn after(&self) -> &'a [u8] {
        // Safety: cursor is always in [0, bytes.len()].
        debug_assert!(self.cursor <= self.bytes.len());
        unsafe { self.bytes.get_unchecked(self.cursor..) }
    }
}

impl<'a> Scanner<'a> {
    /// Check if there are at least N bytes remaining.
    #[inline]
    pub fn has_bytes(&self, n: usize) -> bool {
        self.cursor + n <= self.bytes.len()
    }

    /// The byte right behind the cursor.
    #[inline]
    pub fn peek(&self) -> Option<u8> {
        self.after().first().cloned()
    }

    /// Consume and return the byte right behind the cursor.
    ///
    /// If there are no bytes left, returns `None` and does not advance the
    /// cursor.
    #[inline]
    pub fn eat(&mut self) -> Option<u8> {
        let peeked = self.peek()?;
        self.cursor += 1;
        Some(peeked)
    }

    /// Consume and return exactly N bytes as an array.
    #[inline]
    pub fn eat_bytes<const N: usize>(&mut self) -> Option<[u8; N]> {
        if !self.has_bytes(N) {
            return None;
        }
        let result = self.bytes[self.cursor..self.cursor + N].try_into().ok()?;
        self.cursor += N;
        Some(result)
    }

    /// Consume and return exactly N bytes as a Vec.
    #[inline]
    pub fn eat_vec(&mut self, n: usize) -> Option<Vec<u8>> {
        if !self.has_bytes(n) {
            return None;
        }
        let result = self.bytes[self.cursor..self.cursor + n].to_vec();
        self.cursor += n;
        Some(result)
    }

    /// Consume and return a u32 in big-endian format.
    #[inline]
    pub fn eat_u32_be(&mut self) -> Option<u32> {
        let bytes = self.eat_bytes::<4>()?;
        Some(u32::from_be_bytes(bytes))
    }

    /// Consume and return a u16 in big-endian format.
    #[inline]
    pub fn eat_u16_be(&mut self) -> Option<u16> {
        let bytes = self.eat_bytes::<2>()?;
        Some(u16::from_be_bytes(bytes))
    }

    pub fn eat_variable_length_quantity(&mut self) -> Option<u32> {
        let mut value: u32 = 0;
        for _ in 0..4 {
            let byte = self.eat()?;
            value = (value << 7) | (byte as u32 & 0x7F);
            if byte & 0x80 == 0 {
                return Some(value);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestCase = (&'static [u8], Option<u32>, usize);

    #[test]
    fn test_eat_variable_length_quantity() {
        const TEST_CASES: &[TestCase] = &[
            // Valid VLQ from Specification
            (&[0x00], Some(0x00000000), 1),
            (&[0x00, 0xFF, 0xFF, 0xFF, 0xFF], Some(0x00000000), 1), // with extra bytes
            // Valid VLQ from Specification
            (&[0x40], Some(0x00000040), 1),
            (&[0x40, 0xFF, 0xFF, 0xFF, 0xFF], Some(0x00000040), 1), // with extra bytes
            // Valid VLQ from Specification
            (&[0x7F], Some(0x0000007F), 1),
            (&[0x7F, 0xFF, 0xFF, 0xFF, 0xFF], Some(0x0000007F), 1), // with extra bytes
            // Valid VLQ from Specification
            (&[0x81, 0x00], Some(0x00000080), 2),
            (&[0x81, 0x00, 0xFF, 0xFF, 0xFF], Some(0x00000080), 2), // with extra bytes
            // Valid VLQ from Specification
            (&[0xC0, 0x00], Some(0x00002000), 2),
            (&[0xC0, 0x00, 0xFF, 0xFF, 0xFF], Some(0x00002000), 2), // with extra bytes
            // Valid VLQ from Specification
            (&[0xFF, 0x7F], Some(0x00003FFF), 2),
            (&[0xFF, 0x7F, 0xFF, 0xFF, 0xFF], Some(0x00003FFF), 2), // with extra bytes
            // Valid VLQ from Specification
            (&[0x81, 0x80, 0x00], Some(0x00004000), 3),
            (&[0x81, 0x80, 0x00, 0xFF, 0xFF], Some(0x00004000), 3), // with extra bytes
            // Valid VLQ from Specification
            (&[0xC0, 0x80, 0x00], Some(0x00100000), 3),
            (&[0xC0, 0x80, 0x00, 0xFF, 0xFF], Some(0x00100000), 3), // with extra bytes
            // Valid VLQ from Specification
            (&[0xFF, 0xFF, 0x7F], Some(0x001FFFFF), 3),
            (&[0xFF, 0xFF, 0x7F, 0xFF, 0xFF], Some(0x001FFFFF), 3), // with extra bytes
            // Valid VLQ from Specification
            (&[0x81, 0x80, 0x80, 0x00], Some(0x00200000), 4),
            (&[0x81, 0x80, 0x80, 0x00, 0xFF], Some(0x00200000), 4), // with extra bytes
            // Valid VLQ from Specification
            (&[0xC0, 0x80, 0x80, 0x00], Some(0x08000000), 4),
            (&[0xC0, 0x80, 0x80, 0x00, 0xFF], Some(0x08000000), 4), // with extra bytes
            // Max Valid VLQ from Specification
            (&[0xFF, 0xFF, 0xFF, 0x7F], Some(0x0FFFFFFF), 4),
            (&[0xFF, 0xFF, 0xFF, 0x7F, 0xFF], Some(0x0FFFFFFF), 4), // with extra bytes
        ];
        for &(data, expected, expected_cursor) in TEST_CASES {
            let mut scanner = Scanner::new(data);
            let value = scanner.eat_variable_length_quantity();
            assert_eq!(value, expected);
            assert_eq!(scanner.cursor, expected_cursor);
        }
    }

    #[test]
    fn test_eat_variable_length_quantity_malformed() {
        const TEST_CASES: &[TestCase] = &[
            // Incomplete VLQ
            (&[0x80], None, 1),
            (&[0x80, 0xFF, 0xFF, 0xFF, 0xFF], None, 4), // with extra bytes
            // TIPS: Undefined Behavior but valid VLQ
            // TODO: Maybe we should not accept this or treat beginning `0x80` as `0x81`?
            (&[0x80, 0x0F, 0xFF, 0xFF, 0xFF], Some(0x0F), 2),
            // Incomplete VLQ
            (&[0xFF], None, 1),
            (&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF], None, 4), // with extra bytes
            // Incomplete VLQ
            (&[0xFF, 0xFF], None, 2),
            (&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF], None, 4), // with extra bytes
            // Incomplete VLQ
            (&[0x81, 0x80, 0x80, 0x80], None, 4),
            (&[0x81, 0x80, 0x80, 0x80, 0xFF], None, 4), // with extra bytes
            // Exceeds max VLQ size
            (&[0xFF, 0xFF, 0xFF, 0x80], None, 4),
            (&[0xFF, 0xFF, 0xFF, 0x80, 0xFF], None, 4), // with extra bytes
            // Exceeds max VLQ size
            (&[0xFF, 0xFF, 0xFF, 0xFF], None, 4),
            (&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF], None, 4), // with extra bytes
        ];
        for &(data, expected, expected_cursor) in TEST_CASES {
            let mut scanner = Scanner::new(data);
            let value = scanner.eat_variable_length_quantity();
            assert_eq!(value, expected);
            assert_eq!(scanner.cursor, expected_cursor);
        }
    }
}
