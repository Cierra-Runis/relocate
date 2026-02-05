use relocate_midi::{
    core::chunk::Chunk,
    file::{chunk::ChunksFile, midi::MIDIFile},
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "./assets/World Vanquisher.mid";
    let bytes = fs::read(path).expect("Failed to read MIDI file");
    let midi_file = MIDIFile::from(bytes);
    let chunks_file = ChunksFile::try_from(&midi_file)?;
    for chunk_file in chunks_file {
        let chunk = Chunk::try_from(&chunk_file)?;
        println!("{:?}", chunk);
    }
    Ok(())
}
