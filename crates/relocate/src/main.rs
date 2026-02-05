use relocate_midi::{
    core::chunk::{header::HeaderChunk, track::TrackChunk},
    file::{
        chunk::{
            ChunksFile,
            header::{HEADER_CHUNK_KIND, HeaderChunkFile},
            track::{TRACK_CHUNK_KIND, TrackChunkFile},
        },
        midi::MIDIFile,
    },
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "./assets/World Vanquisher.mid";
    let bytes = fs::read(path).expect("Failed to read MIDI file");
    let midi_file = MIDIFile::from(bytes);
    let chunks_file = ChunksFile::try_from(&midi_file)?;
    for chunk_file in chunks_file.iter() {
        match &chunk_file.kind {
            HEADER_CHUNK_KIND => {
                let chunk_file = HeaderChunkFile::try_from(chunk_file)?;
                let header_chunk = HeaderChunk::try_from(&chunk_file)?;
                println!("{:?}", header_chunk);
            }
            TRACK_CHUNK_KIND => {
                let chunk_file = TrackChunkFile::try_from(chunk_file)?;
                let track_chunk = TrackChunk::try_from(&chunk_file)?;
                println!("{:?}", track_chunk);
            }
            _ => {}
        }
    }
    Ok(())
}
