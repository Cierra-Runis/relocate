use relocate_midi::file::midi::MIDIFile;
use std::fs;

fn main() {
    let path = "./assets/Lapis Lazuli.mid";
    let bytes = fs::read(path).expect("Failed to read MIDI file");
    let midi_file = MIDIFile::from(bytes);

    println!("{}", midi_file);
}

// fn print_track_chunk(chunk: &TrackChunk) {
//     for event in chunk.iter() {
//         match event.kind {
//             EventKind::Meta { .. } => match MetaEvent::try_from(&event.kind)
// {                 Ok(event) => {
//                     println!("Found a Meta event: {:?}", event)
//                 }
//                 Err(e) => {
//                     eprintln!("Error parsing Meta event: {:?}", e)
//                 }
//             },
//             EventKind::SystemExclusive { .. } => {
//                 println!("Found a System Exclusive event")
//             }
//             EventKind::MIDI { .. } => {}
//         }
//     }
// }
