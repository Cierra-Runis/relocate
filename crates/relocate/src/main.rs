use relocate_midi::{
    core::chunk::Chunk,
    file::{chunk::ChunksFile, midi::MIDIFile},
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read("./assets/World Vanquisher.mid").expect("Failed to read MIDI file");
    let midi_file = MIDIFile::from(bytes);

    for chunk_file in ChunksFile::try_from(&midi_file)? {
        let chunk = Chunk::try_from(&chunk_file)?;
        match chunk {
            Chunk::Header(chunk) => println!("Header Chunk: {:?}", chunk),
            Chunk::Track(chunk) => println!("Track Chunk: {} events", chunk.len()),
            Chunk::Alien(chunk_file) => println!("Alien Chunk: {:?}", chunk_file),
        }
    }

    Ok(())
}
