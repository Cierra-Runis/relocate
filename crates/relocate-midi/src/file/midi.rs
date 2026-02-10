use derive_more::{Debug, Deref, Display};

/// To any file system, a [`MIDIFile`] is simply
/// a [series of 8-bit bytes](Vec<u8>).
#[derive(Debug, Display, Deref)]
#[debug("{}", pretty_hex::pretty_hex(_0))]
#[display("{}", pretty_hex::simple_hex(_0))]
pub struct MIDIFile(Vec<u8>);

impl From<Vec<u8>> for MIDIFile {
    fn from(bytes: Vec<u8>) -> Self {
        MIDIFile(bytes)
    }
}
