use relocate_midi::{
    chunk::{Chunk, ChunkKind},
    description::{header::HeaderChunk, track::TrackChunk},
    midi::MIDIFile,
};
use std::fs;

fn main() {
    let path = "./assets/Lapis Lazuli.mid";
    let bytes = fs::read(path).expect("Failed to read MIDI file");
    let midi_file = MIDIFile::from(bytes);

    match Vec::<Chunk>::try_from(midi_file) {
        Ok(chunks) => {
            for chunk in chunks {
                match chunk.kind {
                    ChunkKind::Header(_) => match HeaderChunk::try_from(&chunk) {
                        Ok(chunk) => println!("Found a Header chunk: {:?}", chunk),
                        Err(e) => eprintln!("Error parsing Header chunk: {:?}", e),
                    },
                    ChunkKind::Track(_) => match TrackChunk::try_from(&chunk) {
                        Ok(chunk) => println!(
                            "Found a Track chunk with {:?} events",
                            chunk.track_events.len()
                        ),
                        Err(e) => eprintln!("Error parsing Track chunk: {:?}", e),
                    },
                    ChunkKind::Alien(_) => {
                        println!("Found an Alien chunk with length {}", chunk.length);
                    }
                }
            }
        }
        Err(e) => eprintln!("Error parsing MIDI file: {:?}", e),
    }
}
