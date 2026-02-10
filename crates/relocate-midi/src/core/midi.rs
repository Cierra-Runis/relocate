use derive_more::{Debug, Deref, Display, Error, IntoIterator};

use crate::{
    core::chunk::Chunk,
    file::{chunk::ChunksFile, midi::MIDIFile},
};

/// Above of [`MIDIFile`], a [`MIDI`] is a [series of chunks](Vec<Chunk>).
#[derive(Debug, Deref, IntoIterator)]
pub struct MIDI(Vec<Chunk>);

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    MIDIFileToChunksFile(crate::file::chunk::TryFromError),
    ChunkFileToChunk(crate::core::chunk::TryFromError),
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
            ChunksFile::try_from(value).map_err(TryFromError::MIDIFileToChunksFile)?;
        for chunk_file in chunks_file {
            let chunk = Chunk::try_from(&chunk_file).map_err(TryFromError::ChunkFileToChunk)?;
            chunks.push(chunk);
        }

        Ok(MIDI(chunks))
    }
}
