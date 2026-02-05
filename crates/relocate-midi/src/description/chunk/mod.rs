pub mod header;
pub mod track;

use derive_more::Debug;

use crate::file::chunk::{ChunkFile, header::HEADER_CHUNK_KIND, track::TRACK_CHUNK_KIND};

/// Each [chunk] has a 4-character [type] and a 32-bit [length], which is the
/// number of bytes in the [chunk].
///
/// This structure allows future [chunk type]s to be designed which may easily
/// be ignored if encountered by a program written before the [chunk type] is
/// introduced.
///
/// [chunk]: Chunk
/// [type]: Chunk::kind
/// [length]: Chunk::length
/// [chunk type]: ChunkKind
#[derive(Debug)]
pub struct Chunk {
    /// Each [chunk](Chunk) begins with a 4-character ASCII [type](Chunk::kind).
    pub kind: ChunkKind,

    /// The 4-character ASCII [type](Chunk::kind) is followed by a 32-bit
    /// [length](Chunk::length), most significant byte first (a length of 6 is
    /// stored as `00 00 00 06`).
    ///
    /// This [length](Chunk::length) refers to the number of bytes of
    /// [data](Chunk::data) which follow: the eight bytes of [type](Chunk::kind)
    /// and [length](Chunk::length) are not included.
    ///
    /// Therefore, a chunk with a [length](Chunk::length) of 6 would actually
    /// occupy 14 bytes in the disk file.
    pub length: u32,

    #[debug("{}", pretty_hex::pretty_hex(&data))]
    pub data: Vec<u8>,
}

impl From<ChunkFile<'_>> for Chunk {
    fn from(value: ChunkFile<'_>) -> Self {
        Chunk {
            kind: ChunkKind::from(&value.kind),
            length: value.length,
            data: value.data.to_vec(),
        }
    }
}

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
pub enum ChunkKind {
    /// A [header chunk](ChunkKind::Header) provides a minimal amount
    /// of information pertaining to the entire [MIDI File].
    ///
    /// [MIDI File]: crate::midi::MIDIFile
    Header,

    /// A [track chunk](ChunkKind::Track) contains a sequential stream
    /// of MIDI data which may contain information for up to 16 MIDI channels.
    Track,

    /// Your programs should _expect_ [alien chunk](ChunkKind::Alien)s
    /// and treat them as if they weren't there.
    Alien([u8; 4]),
}

impl From<&[u8; 4]> for ChunkKind {
    fn from(value: &[u8; 4]) -> Self {
        match value {
            HEADER_CHUNK_KIND => ChunkKind::Header,
            TRACK_CHUNK_KIND => ChunkKind::Track,
            other => ChunkKind::Alien(*other),
        }
    }
}
