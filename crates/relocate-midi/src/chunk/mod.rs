pub mod header;
pub mod kind;

use derive_more::{Debug, Display};

use crate::chunk::kind::ChunkKind;

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
