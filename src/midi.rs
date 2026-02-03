use derive_more::{Debug, Display, Error};

/// To any file system, a [MIDI File](MIDIFile) is simply a [series of 8-bit
/// bytes](Vec<u8>).
#[derive(Debug, Display, Clone)]
#[display("{}", pretty_hex::pretty_hex(_0))]
pub struct MIDIFile(Vec<u8>);

impl From<Vec<u8>> for MIDIFile {
    fn from(bytes: Vec<u8>) -> Self {
        MIDIFile(bytes)
    }
}

#[derive(Debug, derive_more::Display, Error)]
pub enum TryFromMIDIFileError {
    #[debug("Incomplete chunk prefix: file ended before reading 8-byte prefix")]
    IncompleteChunkPrefix,
    #[debug("Malformed chunk type: failed to parse 4-byte type identifier")]
    MalformedChunkType,
    #[debug("Malformed chunk length: failed to parse 4-byte length field")]
    MalformedChunkLength,
    #[debug("Truncated chunk data: declared length exceeds remaining file size")]
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
                .map_err(|_| TryFromMIDIFileError::MalformedChunkType)?;
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
#[derive(Debug, Display)]
#[debug("Chunk: kind={:?}, length={:?}, data={}", kind, length, pretty_hex::pretty_hex(&data))]
#[display("Chunk: kind={}, length={}", kind, length)]
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

    pub data: Vec<u8>,
}

/// [MIDI File](MIDIFile)s contain two [types of chunk](ChunkKind)s:
/// [header chunk](ChunkKind::Header)s and [track chunk](ChunkKind::Track)s.
///
/// The concepts of multiple tracks, multiple MIDI outputs, patterns, sequences,
/// and songs may all be implemented using several track [chunk](Chunk)s.
#[derive(Debug, Display)]
pub enum ChunkKind {
    /// A [header chunk](ChunkKind::Header) provides a minimal amount
    /// of information pertaining to the entire [`MIDIFile`].
    #[debug("Header: {:?}", _0)]
    #[display("Header")]
    Header([u8; 4]),

    /// A [track chunk](ChunkKind::Track) contains a sequential stream
    /// of MIDI data which may contain information for up to 16 MIDI channels.
    #[debug("Track: {:?}", _0)]
    #[display("Track")]
    Track([u8; 4]),

    /// Your programs should _expect_ [alien chunk](ChunkKind::Alien)s
    /// and treat them as if they weren't there.
    #[debug("Alien: {:?}", _0)]
    #[display("Alien")]
    Alien([u8; 4]),
}

impl From<[u8; 4]> for ChunkKind {
    fn from(bytes: [u8; 4]) -> Self {
        match &bytes {
            b"MThd" => ChunkKind::Header(bytes),
            b"MTrk" => ChunkKind::Track(bytes),
            _ => ChunkKind::Alien(bytes),
        }
    }
}

impl From<ChunkKind> for [u8; 4] {
    fn from(val: ChunkKind) -> Self {
        match val {
            ChunkKind::Header(bytes) => bytes,
            ChunkKind::Track(bytes) => bytes,
            ChunkKind::Alien(bytes) => bytes,
        }
    }
}
