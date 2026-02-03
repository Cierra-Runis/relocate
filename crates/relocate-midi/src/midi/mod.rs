pub mod format;

use derive_more::Debug;

use crate::chunk::{Chunk, kind::ChunkKind};

/// To any file system, a [MIDI File](MIDIFile)
/// is simply a [series of 8-bit bytes](Vec<u8>).
#[derive(Debug, Clone)]
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

impl TryFrom<MIDIFile> for Vec<Chunk> {
    type Error = TryFromMIDIFileError;

    /// [MIDI File](MIDIFile)s are made up of [chunk](Chunk)s.
    fn try_from(midi_file: MIDIFile) -> Result<Self, Self::Error> {
        let mut chunks = Vec::new();
        let mut i = 0;

        while i < midi_file.0.len() {
            if i + 8 > midi_file.0.len() {
                return Err(TryFromMIDIFileError::IncompleteChunkPrefix);
            }

            let kind_bytes: [u8; 4] = midi_file.0[i..i + 4]
                .try_into()
                .map_err(|_| TryFromMIDIFileError::MalformedChunkKind)?;
            let kind = ChunkKind::from(kind_bytes);

            let length_bytes: [u8; 4] = midi_file.0[i + 4..i + 8]
                .try_into()
                .map_err(|_| TryFromMIDIFileError::MalformedChunkLength)?;
            let length = u32::from_be_bytes(length_bytes);

            let data_start = i + 8;
            let data_end = data_start + length as usize;

            if data_end > midi_file.0.len() {
                return Err(TryFromMIDIFileError::TruncatedChunkData);
            }

            let data = midi_file.0[data_start..data_end].to_vec();

            chunks.push(Chunk { kind, length, data });

            i = data_end;
        }

        Ok(chunks)
    }
}
