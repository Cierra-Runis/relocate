mod midi;

use crate::midi::{Chunk, MIDIFile};
use std::fs;

fn main() {
    let path = "./assets/Lapis Lazuli.mid";
    let bytes = fs::read(path).expect("Failed to read MIDI file");
    let midi_file = MIDIFile::from(bytes);

    match Vec::<Chunk>::try_from(midi_file) {
        Ok(chunks) => {
            for chunk in chunks {
                println!("{chunk}");
            }
        }
        Err(e) => eprintln!("Error parsing MIDI file: {}", e),
    }
}
