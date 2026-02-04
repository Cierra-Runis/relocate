use derive_more::Debug;

use crate::{
    chunk::{Chunk, ChunkKind},
    scanner::Scanner,
};

/// The track chunks (type MTrk) are where actual song data is stored.  Each
/// track chunk is simply a stream of MIDI events (and non-MIDI events),
/// preceded by delta-time values.
///
/// The format for Track Chunks (described below) is exactly the same for all
/// three formats (0, 1, and 2: see "Header Chunk" above) of MIDI Files.
#[derive(Debug)]
pub struct TrackChunk {
    pub track_events: Vec<TrackEvent>,
}

#[derive(Debug)]
pub struct TrackEvent {
    /// Represents the amount of time before the following event, stored as a
    /// variable-length quantity.
    ///
    /// If the first event in a track occurs at the very beginning of a track,
    /// or if two events occur simultaneously, a delta-time of zero is used.
    ///
    /// Delta-times are _always_ present. (_Not_ storing delta-times of 0
    /// requires at least two bytes for any other value, and most delta
    /// times _aren't_ zero.)
    ///
    /// Delta-time is in ticks as specified in the header chunk.
    pub delta_time: u32,

    pub event: Event,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    MIDI { status: u8, data: Vec<u8> },
    SystemExclude { data: Vec<u8> },
    Meta { kind: u8, data: Vec<u8> },
}

#[derive(Debug)]
pub enum TryFromChunkError {
    InvalidChunkType,
    MalformedRunningStatus,
}

impl TryFrom<&Chunk> for TrackChunk {
    type Error = TryFromChunkError;

    fn try_from(chunk: &Chunk) -> Result<Self, Self::Error> {
        match &chunk.kind {
            ChunkKind::Track(_) => {
                let mut track_events = Vec::new();
                let mut scanner = Scanner::new(&chunk.data);
                let mut running_status: Option<u8> = None;

                while !scanner.done() {
                    // Read variable-length delta time
                    let delta_time = scanner
                        .eat_variable_length_quantity()
                        .ok_or(TryFromChunkError::MalformedRunningStatus)?;

                    let status = scanner
                        .peek()
                        .ok_or(TryFromChunkError::MalformedRunningStatus)?;

                    let event = match status {
                        0xFF => {
                            scanner.eat(); // consume status byte

                            let meta_type = scanner
                                .eat()
                                .ok_or(TryFromChunkError::MalformedRunningStatus)?;
                            debug_assert!(meta_type < 128);

                            // Read VLQ length
                            let length = scanner
                                .eat_variable_length_quantity()
                                .ok_or(TryFromChunkError::MalformedRunningStatus)?;

                            let data = scanner
                                .eat_vec(length as usize)
                                .ok_or(TryFromChunkError::MalformedRunningStatus)?;

                            Event::Meta {
                                kind: meta_type,
                                data,
                            }
                        }

                        0xF0 | 0xF7 => {
                            scanner.eat(); // consume status byte

                            // TIPS: Event::SystemExclude will reset running status
                            running_status = None;

                            // Read VLQ length
                            let length = scanner
                                .eat_variable_length_quantity()
                                .ok_or(TryFromChunkError::MalformedRunningStatus)?;

                            let data = scanner
                                .eat_vec(length as usize)
                                .ok_or(TryFromChunkError::MalformedRunningStatus)?;

                            Event::SystemExclude { data }
                        }

                        status_byte if status_byte >= 0x80 => {
                            scanner.eat(); // consume status byte

                            // TIPS: MIDI channel event with explicit status
                            running_status = Some(status_byte);

                            let data_len = match status_byte & 0xF0 {
                                0xC0 | 0xD0 => 1, // Program Change, Channel Pressure
                                _ => 2,
                            };
                            let data = scanner
                                .eat_vec(data_len)
                                .ok_or(TryFromChunkError::MalformedRunningStatus)?;

                            Event::MIDI {
                                status: status_byte,
                                data,
                            }
                        }

                        _ => {
                            // MIDI channel event with running status
                            let status =
                                running_status.ok_or(TryFromChunkError::MalformedRunningStatus)?;

                            let data_len = match status & 0xF0 {
                                0xC0 | 0xD0 => 1,
                                _ => 2,
                            };
                            let data = scanner
                                .eat_vec(data_len)
                                .ok_or(TryFromChunkError::MalformedRunningStatus)?;

                            Event::MIDI { status, data }
                        }
                    };

                    track_events.push(TrackEvent { delta_time, event });
                }

                Ok(TrackChunk { track_events })
            }
            _ => Err(TryFromChunkError::InvalidChunkType),
        }
    }
}
