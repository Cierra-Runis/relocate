use derive_more::{Debug, Deref, Display, Error, IntoIterator};

use crate::{
    core::chunk::Chunk,
    file::{chunk::ChunksFile, midi::MIDIFile},
};

#[derive(Debug, Deref, IntoIterator)]
pub struct MIDI(Vec<Chunk>);

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    CouldNotConvertChunk,
    CouldNotReadChunksFile,
}

impl TryFrom<Vec<u8>> for MIDI {
    type Error = TryFromError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        MIDI::try_from(&MIDIFile::from(value))
    }
}

impl<'a> TryFrom<&'a MIDIFile> for MIDI {
    type Error = TryFromError;

    fn try_from(value: &'a MIDIFile) -> Result<Self, Self::Error> {
        let mut chunks = Vec::new();

        let chunks_file =
            ChunksFile::try_from(value).map_err(|_| TryFromError::CouldNotReadChunksFile)?;
        for chunk_file in chunks_file {
            let chunk =
                Chunk::try_from(&chunk_file).map_err(|_| TryFromError::CouldNotConvertChunk)?;
            chunks.push(chunk);
        }

        Ok(MIDI(chunks))
    }
}
