use derive_more::{Debug, Deref};

use crate::{
    chunk::Chunk,
    midi::{MIDIFile, TryFromMIDIFileError},
};

#[derive(Debug, Deref)]
pub struct MIDI(MIDIFile);

impl MIDIFile {
    pub fn chunks(&self) -> Result<Vec<Chunk>, TryFromMIDIFileError> {
        Vec::try_from(self)
    }
}
