pub mod format;

use derive_more::{Debug, Deref};

use crate::chunk::{Chunk, ChunkKind};
use crate::scanner::Scanner;

/// To any file system, a [MIDI File](MIDIFile)
/// is simply a [series of 8-bit bytes](Vec<u8>).
#[derive(Debug, Clone, Deref)]
#[debug("{}", pretty_hex::pretty_hex(_0))]
pub struct MIDIFile(Vec<u8>);

impl From<Vec<u8>> for MIDIFile {
    fn from(bytes: Vec<u8>) -> Self {
        MIDIFile(bytes)
    }
}

#[derive(Debug)]
pub enum TryFromMIDIFileError {
    IncompleteChunkPrefix,
    MalformedChunkKind,
    MalformedChunkLength,
    TruncatedChunkData,
}

impl TryFrom<&MIDIFile> for Vec<Chunk> {
    type Error = TryFromMIDIFileError;

    /// [MIDI File](MIDIFile)s are made up of [chunk](Chunk)s.
    fn try_from(midi_file: &MIDIFile) -> Result<Self, Self::Error> {
        let mut chunks = Vec::new();
        let mut scanner = Scanner::new(midi_file);

        while !scanner.done() {
            // Each chunk needs at least 8 bytes (4 for kind + 4 for length)
            if !scanner.has_bytes(8) {
                return Err(TryFromMIDIFileError::IncompleteChunkPrefix);
            }

            // Read the chunk kind (4 bytes)
            let kind_bytes = scanner
                .eat_bytes::<4>()
                .ok_or(TryFromMIDIFileError::MalformedChunkKind)?;
            let kind = ChunkKind::from(kind_bytes);

            // Read the chunk length (4 bytes, big-endian)
            let length = scanner
                .eat_u32_be()
                .ok_or(TryFromMIDIFileError::MalformedChunkLength)?;

            // Read the chunk data
            let data = scanner
                .eat_vec(length as usize)
                .ok_or(TryFromMIDIFileError::TruncatedChunkData)?;

            chunks.push(Chunk { kind, length, data });
        }

        Ok(chunks)
    }
}
