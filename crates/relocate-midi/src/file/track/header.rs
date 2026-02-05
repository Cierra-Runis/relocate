use derive_more::{Debug, Display, Error};

use crate::file::chunk::ChunkFile;

const TRACK_HEADER_KIND: &[u8; 4] = b"MThd";

#[derive(Debug)]
pub struct TrackHeaderFile {
    kind: &'static [u8; 4],
    pub length: [u8; 4],
    pub format: [u8; 2],
    pub track_count: [u8; 2],
    pub division: [u8; 2],
}

impl TrackHeaderFile {
    #[inline]
    pub fn kind(&self) -> &[u8; 4] {
        self.kind
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

impl<'a> TryFrom<ChunkFile<'a>> for TrackHeaderFile {
    type Error = TryFromError;

    fn try_from(value: ChunkFile<'a>) -> Result<Self, Self::Error> {
        if &value.kind != TRACK_HEADER_KIND {
            return Err(TryFromError::InvalidKind);
        }
        if value.length != [0x00, 0x00, 0x00, 0x06] {
            return Err(TryFromError::InvalidLength);
        }

        let mut scanner = crate::scanner::Scanner::new(value.data);
        let format = scanner
            .eat_array::<2>()
            .ok_or(TryFromError::CouldNotReadFormat)?;
        let track_count = scanner
            .eat_array::<2>()
            .ok_or(TryFromError::CouldNotReadTrackCount)?;
        let division = scanner
            .eat_array::<2>()
            .ok_or(TryFromError::CouldNotReadDivision)?;

        if !scanner.done() {
            return Err(TryFromError::ScannerNotDone);
        }

        Ok(TrackHeaderFile {
            kind: TRACK_HEADER_KIND,
            length: value.length,
            format,
            track_count,
            division,
        })
    }
}
