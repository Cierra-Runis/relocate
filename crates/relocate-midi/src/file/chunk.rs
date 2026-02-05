use derive_more::{Debug, Deref, Display, Error};

use crate::{file::midi::MIDIFile, scanner::Scanner};

#[derive(Debug)]
pub struct ChunkFile<'a> {
    pub r#type: [u8; 4],
    pub length: [u8; 4],
    pub data: &'a [u8],
}

#[derive(Debug, Deref)]
pub struct ChunksFile<'a>(Vec<ChunkFile<'a>>);

#[derive(Debug, Display, Error)]
pub enum ChunksFileTryFromError {
    CouldNotReadType,
    CouldNotReadLength,
    CouldNotReadData,
}

impl<'a> TryFrom<&'a MIDIFile> for ChunksFile<'a> {
    type Error = ChunksFileTryFromError;

    fn try_from(value: &'a MIDIFile) -> Result<Self, Self::Error> {
        let mut chunk_files = Vec::new();
        let mut scanner = Scanner::new(value);

        while !scanner.done() {
            let r#type = scanner
                .eat_array::<4>()
                .ok_or(ChunksFileTryFromError::CouldNotReadType)?;
            let length = scanner
                .eat_array::<4>()
                .ok_or(ChunksFileTryFromError::CouldNotReadLength)?;
            let data = scanner
                .eat_slice(u32::from_be_bytes(length) as usize)
                .ok_or(ChunksFileTryFromError::CouldNotReadData)?;
            chunk_files.push(ChunkFile {
                r#type,
                length,
                data,
            });
        }

        Ok(ChunksFile(chunk_files))
    }
}
