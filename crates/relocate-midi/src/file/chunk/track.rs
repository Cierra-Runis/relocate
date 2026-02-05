use derive_more::{Debug, Display, Error};

use crate::file::chunk::ChunkFile;

pub const TRACK_CHUNK_KIND: &[u8; 4] = b"MTrk";

#[derive(Debug)]
pub struct TrackChunkFile<'a> {
    kind: &'static [u8; 4],
    length: u32,
    pub track_events: &'a [u8],
}

impl TrackChunkFile<'_> {
    #[inline]
    pub fn kind(&self) -> &[u8; 4] {
        self.kind
    }

    #[inline]
    pub fn length(&self) -> u32 {
        self.length
    }
}

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    InvalidKind,
    CouldNotReadVLQ,
    CouldNotReadTrackEvents,
    ScannerNotDone,
}

impl<'a> TryFrom<&ChunkFile<'a>> for TrackChunkFile<'a> {
    type Error = TryFromError;

    fn try_from(value: &ChunkFile<'a>) -> Result<Self, Self::Error> {
        if &value.kind != TRACK_CHUNK_KIND {
            return Err(TryFromError::InvalidKind);
        }

        let mut scanner = crate::scanner::Scanner::new(value.data);
        let track_events = scanner
            .eat_slice(value.length as usize)
            .ok_or(TryFromError::CouldNotReadVLQ)?;

        if !scanner.done() {
            return Err(TryFromError::ScannerNotDone);
        }

        Ok(TrackChunkFile {
            kind: TRACK_CHUNK_KIND,
            length: value.length,
            track_events,
        })
    }
}
