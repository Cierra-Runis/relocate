pub mod header;
pub mod track;

use derive_more::{Debug, Deref, Display, Error};

use crate::{file::midi::MIDIFile, scanner::Scanner};

#[derive(Debug)]
pub struct ChunkFile<'a> {
    #[debug("{:?}", String::from_utf8_lossy(kind))]
    pub kind: [u8; 4],
    #[debug("{:?}", u32::from_be_bytes(*length))]
    pub length: [u8; 4],
    #[debug(skip)]
    pub data: &'a [u8],
}

#[derive(Debug, Deref)]
pub struct ChunksFile<'a>(Vec<ChunkFile<'a>>);

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    CouldNotReadKind,
    CouldNotReadLength,
    CouldNotReadData,
    ScannerNotDone,
}

impl<'a> TryFrom<&'a MIDIFile> for ChunksFile<'a> {
    type Error = TryFromError;

    fn try_from(value: &'a MIDIFile) -> Result<Self, Self::Error> {
        let mut files = Vec::new();
        let mut scanner = Scanner::new(value);

        while !scanner.done() {
            let kind = scanner
                .eat_array::<4>()
                .ok_or(TryFromError::CouldNotReadKind)?;
            let length = scanner
                .eat_array::<4>()
                .ok_or(TryFromError::CouldNotReadLength)?;
            let data = scanner
                .eat_slice(u32::from_be_bytes(length) as usize)
                .ok_or(TryFromError::CouldNotReadData)?;

            files.push(ChunkFile { kind, length, data });
        }

        Ok(ChunksFile(files))
    }
}
