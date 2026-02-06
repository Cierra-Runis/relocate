use derive_more::{Debug, Display, Error};

use crate::file::chunk::ChunkFile;

pub const HEADER_CHUNK_KIND: &[u8; 4] = b"MThd";
pub const HEADER_CHUNK_LENGTH: &u32 = &6;

#[derive(Debug)]
pub struct HeaderChunkFile<'a> {
    kind: &'static [u8; 4],
    length: &'static u32,
    pub format: &'a [u8; 2],
    pub tracks_count: &'a [u8; 2],
    pub division: &'a [u8; 2],
}

impl<'a> HeaderChunkFile<'a> {
    #[inline]
    pub fn kind(&self) -> &[u8; 4] {
        self.kind
    }

    #[inline]
    pub fn length(&self) -> &u32 {
        self.length
    }
}

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    InvalidKind,
    InvalidLength,
    CouldNotReadFormat,
    CouldNotReadTrackCount,
    CouldNotReadDivision,
    ScannerNotDone,
}

impl<'a> TryFrom<&ChunkFile<'a>> for HeaderChunkFile<'a> {
    type Error = TryFromError;

    fn try_from(value: &ChunkFile<'a>) -> Result<Self, Self::Error> {
        if value.kind != HEADER_CHUNK_KIND {
            return Err(TryFromError::InvalidKind);
        }
        if value.length != 6 {
            return Err(TryFromError::InvalidLength);
        }

        let mut scanner = crate::scanner::Scanner::new(value.data);
        let format = scanner
            .eat_bytes::<2>()
            .ok_or(TryFromError::CouldNotReadFormat)?;
        let tracks_count = scanner
            .eat_bytes::<2>()
            .ok_or(TryFromError::CouldNotReadTrackCount)?;
        let division = scanner
            .eat_bytes::<2>()
            .ok_or(TryFromError::CouldNotReadDivision)?;

        if !scanner.done() {
            return Err(TryFromError::ScannerNotDone);
        }

        Ok(HeaderChunkFile {
            kind: HEADER_CHUNK_KIND,
            length: HEADER_CHUNK_LENGTH,
            format,
            tracks_count,
            division,
        })
    }
}
