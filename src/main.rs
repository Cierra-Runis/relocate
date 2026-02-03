mod midi;

use crate::midi::MIDIFile;
use std::fs;

fn main() {
    let path = "./assets/Lapis Lazuli.mid";
    let bytes = fs::read(path).expect("Failed to read MIDI file");
    let midi_file = MIDIFile::new(bytes);
    match midi_file.chunks() {
        Ok(chunks) => {
            for chunk in chunks {
                println!("{:?}", chunk);
            }
        }
        Err(e) => {
            eprintln!("Error parsing MIDI file: {}", e);
        }
    }
}
