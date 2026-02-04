use relocate_midi::{
    chunk::{Chunk, ChunkKind},
    description::{
        header::HeaderChunk,
        track::{EventKind, TrackChunk},
    },
    event::meta::MetaEvent,
    midi::MIDIFile,
};
use std::fs;

fn main() {
    let path = "./assets/Lapis Lazuli.mid";
    let bytes = fs::read(path).expect("Failed to read MIDI file");
    let midi_file = MIDIFile::from(bytes);

    match Vec::<Chunk>::try_from(&midi_file) {
        Ok(chunks) => {
            for chunk in chunks {
                println!();
                match chunk.kind {
                    ChunkKind::Header(_) => match HeaderChunk::try_from(&chunk) {
                        Ok(chunk) => println!("Found a Header chunk: {:?}", chunk),
                        Err(e) => eprintln!("Error parsing Header chunk: {:?}", e),
                    },
                    ChunkKind::Track(_) => match TrackChunk::try_from(&chunk) {
                        Ok(chunk) => {
                            for event in chunk.iter() {
                                match event.kind {
                                    EventKind::Meta { .. } => {
                                        match MetaEvent::try_from(&event.kind) {
                                            Ok(event) => {
                                                println!("Found a Meta event: {:?}", event)
                                            }
                                            Err(e) => {
                                                eprintln!("Error parsing Meta event: {:?}", e)
                                            }
                                        }
                                    }
                                    EventKind::SystemExclusive { .. } => {
                                        println!("Found a System Exclusive event")
                                    }
                                    EventKind::MIDI { .. } => {}
                                }
                            }
                        }
                        Err(e) => eprintln!("Error parsing Track chunk: {:?}", e),
                    },
                    ChunkKind::Alien(_) => {
                        println!("Found an Alien chunk with length {}", chunk.length)
                    }
                }
            }
        }
        Err(e) => eprintln!("Error parsing MIDI file: {:?}", e),
    }
}
