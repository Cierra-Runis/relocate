/// A byte scanner for efficiently reading bytes from a slice.
#[derive(Debug)]
pub struct Scanner<'a> {
    /// The byte slice to scan.
    bytes: &'a [u8],

    /// The index at which we currently are. To guarantee safety, it must always
    /// hold that cursor in `[0, bytes.len()]`.
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
    fn after(&self) -> &'a [u8] {
        // SAFETY: cursor is always in `[0, bytes.len()]`.
        debug_assert!(self.cursor <= self.bytes.len());
        unsafe { self.bytes.get_unchecked(self.cursor..) }
    }
}

impl<'a> Scanner<'a> {
    /// Peek at the byte right behind the cursor without consuming it.
    ///
    /// If there are no bytes left, returns `None`.
    #[inline]
    pub fn peek(&self) -> Option<&'a u8> {
        self.after().first()
    }

    /// Consume and return the byte right behind the cursor.
    ///
    /// If there are no bytes left, returns `None` and does not advance the
    /// cursor.
    #[inline]
    pub fn eat(&mut self) -> Option<&'a u8> {
        let peeked = self.peek()?;
        self.cursor += 1;
        Some(peeked)
    }

    /// Consume and return exactly `n` bytes as a borrowed slice.
    #[inline]
    pub fn eat_slice(&mut self, n: usize) -> Option<&'a [u8]> {
        let start_cursor = self.cursor;
        let end_cursor = start_cursor.checked_add(n)?;
        if end_cursor > self.bytes.len() {
            return None;
        }
        let result = &self.bytes[start_cursor..end_cursor];
        self.cursor = end_cursor;
        Some(result)
    }

    /// Consume and return exactly N bytes as a borrowed array.
    #[inline]
    pub fn eat_bytes<const N: usize>(&mut self) -> Option<&'a [u8; N]> {
        self.eat_slice(N)?.try_into().ok()
    }
}

impl<'a> Scanner<'a> {
    /// Consume and return a u16 in big-endian format.
    #[inline]
    pub fn eat_u16_be(&mut self) -> Option<u16> {
        let bytes = self.eat_bytes::<2>()?;
        Some(u16::from_be_bytes(*bytes))
    }

    /// Consume and return a u32 in big-endian format.
    #[inline]
    pub fn eat_u32_be(&mut self) -> Option<u32> {
        let bytes = self.eat_bytes::<4>()?;
        Some(u32::from_be_bytes(*bytes))
    }

    /// Consume and return a variable-length quantity value as defined in the
    /// MIDI Specification.
    ///
    /// If the variable-length quantity is malformed (e.g., incomplete or
    /// exceeds the maximum size), returns `None`.
    pub fn eat_variable_length_quantity(&mut self) -> Option<u32> {
        let mut value: u32 = 0;
        for _ in 0..4 {
            let byte = self.eat()?;
            value = (value << 7) | u32::from(byte & 0x7F);
            if byte & 0x80 == 0 {
                return Some(value);
            }
        }
        None
    }

    /// Consume bytes until a byte with the high bit set is found, returning
    /// the consumed bytes as a slice (not including the high-bit byte).
    ///
    /// This is used to read MIDI data bytes (which have high bit = 0) until
    /// we encounter a status byte (high bit = 1), which we do NOT consume.
    ///
    /// Returns the slice of consumed data bytes, or None if we reach the end
    /// without finding a high-bit byte.
    pub fn eat_data_bytes(&mut self) -> Option<&'a [u8]> {
        let start_cursor = self.cursor;
        while let Some(byte) = self.peek() {
            if byte & 0x80 != 0 {
                // Found a byte with high bit set, stop without consuming it
                let end_cursor = self.cursor;
                return Some(&self.bytes[start_cursor..end_cursor]);
            }
            // Consume the byte with high bit = 0
            self.eat();
        }
        // Reached the end of the slice, return the consumed bytes
        Some(&self.bytes[start_cursor..self.cursor])
    }
}
