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

/// [MIDI File]s contain two [types of chunk]s:
/// [header chunk]s and [track chunk]s.
///
/// The concepts of multiple tracks, multiple MIDI outputs, patterns, sequences,
/// and songs may all be implemented using several track [chunk]s.
///
/// [MIDI File]: crate::midi::MIDIFile
/// [types of chunk]: ChunkKind
/// [header chunk]: ChunkKind::Header
/// [track chunk]: ChunkKind::Track
/// [chunk]: crate::chunk::Chunk
#[derive(Debug)]
pub enum Chunk<'a> {
    /// A [header chunk](ChunkKind::Header) provides a minimal amount
    /// of information pertaining to the entire [MIDI File].
    ///
    /// [MIDI File]: crate::midi::MIDIFile
    Header(HeaderChunk),

    /// A [track chunk](ChunkKind::Track) contains a sequential stream
    /// of MIDI data which may contain information for up to 16 MIDI channels.
    Track(TrackChunk),

    /// Your programs should _expect_ [alien chunk](ChunkKind::Alien)s
    /// and treat them as if they weren't there.
    Alien(&'a ChunkFile<'a>),
}

/// TODO: Inner error variants should contain the original error.
#[derive(Debug, Display, Error)]
pub enum TryFromError {
    HeaderChunkFileConversionError,
    HeaderChunkConversionError,
    TrackChunkFileConversionError,
    TrackEventsFileConversionError(crate::file::event::track::TryFromError),
    TrackChunkConversionError,
}

impl<'a> TryFrom<&'a ChunkFile<'a>> for Chunk<'a> {
    type Error = TryFromError;

    fn try_from(value: &'a ChunkFile<'a>) -> Result<Self, Self::Error> {
        match value.kind {
            HEADER_CHUNK_KIND => {
                let chunk_file = HeaderChunkFile::try_from(value)
                    .map_err(|_| TryFromError::HeaderChunkFileConversionError)?;
                let header_chunk = HeaderChunk::try_from(&chunk_file)
                    .map_err(|_| TryFromError::HeaderChunkConversionError)?;
                Ok(Chunk::Header(header_chunk))
            }
            TRACK_CHUNK_KIND => {
                let chunk_file = TrackChunkFile::try_from(value)
                    .map_err(|_| TryFromError::TrackChunkFileConversionError)?;
                let events_file = TrackEventsFile::try_from(&chunk_file)
                    .map_err(TryFromError::TrackEventsFileConversionError)?;
                let track_chunk = TrackChunk::try_from(&events_file)
                    .map_err(|_| TryFromError::TrackChunkConversionError)?;
                Ok(Chunk::Track(track_chunk))
            }
            _ => Ok(Chunk::Alien(value)),
        }
    }
}
