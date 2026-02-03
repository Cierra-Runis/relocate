use derive_more::Debug;

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
    #[debug("Header: {:?}", _0)]
    Header([u8; 4]),

    /// A [track chunk](ChunkKind::Track) contains a sequential stream
    /// of MIDI data which may contain information for up to 16 MIDI channels.
    #[debug("Track: {:?}", _0)]
    Track([u8; 4]),

    /// Your programs should _expect_ [alien chunk](ChunkKind::Alien)s
    /// and treat them as if they weren't there.
    #[debug("Alien: {:?}", _0)]
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
