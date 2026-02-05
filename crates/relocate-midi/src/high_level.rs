use derive_more::{Debug, Deref};

use crate::file::midi::MIDIFile;

#[derive(Debug, Deref)]
pub struct MIDI(MIDIFile);
