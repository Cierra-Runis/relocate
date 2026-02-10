pub mod header;
pub mod track;

use derive_more::{Debug, Display, Error};

use crate::{
    core::chunk::{header::HeaderChunk, track::TrackChunk},
    file::{
        chunk::{
            ChunkFile,
            header::{HEADER_CHUNK_KIND, HeaderChunkFile},
            track::{TRACK_CHUNK_KIND, TrackChunkFile},
        },
        event::track::TrackEventsFile,
    },
};

/// [`MIDI`](crate::core::midi::MIDI) contains three types of [`Chunk`]:
///
/// - [`Chunk::Header`] with [`HeaderChunk`] containing MIDI metadata
/// - [`Chunk::Track`] with [`TrackChunk`] containing MIDI event data
/// - [`Chunk::Alien`] with [`AlienChunk`] for unrecognized chunk types
#[derive(Debug)]
pub enum Chunk {
    /// Provides a minimal amount of information pertaining to the entire
    /// [MIDI](crate::core::midi::MIDI).
    Header(HeaderChunk),

    /// Contains a sequential stream of MIDI data which may contain information
    /// for up to 16 MIDI channels.
    Track(TrackChunk),

    /// Your programs should _expect_ [`Chunk::Alien`] and treat them as if they
    /// weren't there.
    Alien(AlienChunk),
}

/// An unrecognized chunk type, which your program should ignore.
/// It is simply the owned version of [`ChunkFile`].
#[derive(Debug)]
pub struct AlienChunk {
    pub kind: [u8; 4],
    pub length: u32,
    pub data: Vec<u8>,
}

impl<'a> From<ChunkFile<'a>> for AlienChunk {
    fn from(value: ChunkFile) -> Self {
        AlienChunk {
            kind: *value.kind,
            length: value.length,
            data: value.data.to_vec(),
        }
    }
}

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    ChunkFileToHeaderChunkFile(crate::file::chunk::header::TryFromError),
    HeaderChunkFileToHeaderChunk(crate::core::chunk::header::TryFromError),
    ChunkFileToTrackChunkFile(crate::file::chunk::track::TryFromError),
    TrackChunkFileToTrackEventsFile(crate::file::event::track::TryFromError),
    TrackEventsFileToTrackChunk(crate::core::chunk::track::TryFromError),
}

impl<'a> TryFrom<&'a ChunkFile<'a>> for Chunk {
    type Error = TryFromError;

    fn try_from(value: &'a ChunkFile<'a>) -> Result<Self, Self::Error> {
        match value.kind {
            HEADER_CHUNK_KIND => {
                let chunk_file = HeaderChunkFile::try_from(value)
                    .map_err(TryFromError::ChunkFileToHeaderChunkFile)?;
                let header_chunk = HeaderChunk::try_from(&chunk_file)
                    .map_err(TryFromError::HeaderChunkFileToHeaderChunk)?;
                Ok(Chunk::Header(header_chunk))
            }
            TRACK_CHUNK_KIND => {
                let chunk_file = TrackChunkFile::try_from(value)
                    .map_err(TryFromError::ChunkFileToTrackChunkFile)?;
                let events_file = TrackEventsFile::try_from(&chunk_file)
                    .map_err(TryFromError::TrackChunkFileToTrackEventsFile)?;
                let track_chunk = TrackChunk::try_from(&events_file)
                    .map_err(TryFromError::TrackEventsFileToTrackChunk)?;
                Ok(Chunk::Track(track_chunk))
            }
            _ => Ok(Chunk::Alien(AlienChunk::from(*value))),
        }
    }
}
