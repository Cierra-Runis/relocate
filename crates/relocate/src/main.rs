use relocate_midi::core::{chunk::Chunk, midi::MIDI};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read("./assets/World Vanquisher.mid").expect("Failed to read MIDI file");
    let midi = MIDI::try_from(bytes)?;

    for chunk in midi {
        match chunk {
            Chunk::Header(chunk) => println!("Header Chunk: {:?}", chunk),
            Chunk::Track(chunk) => {
                for event in chunk {
                    println!("Track Event: {:?}", event);
                }
            }
            Chunk::Alien(chunk_file) => println!("Alien Chunk: {:?}", chunk_file),
        }
    }

    Ok(())
}
