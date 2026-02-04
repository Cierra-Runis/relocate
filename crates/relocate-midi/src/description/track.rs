use derive_more::Debug;

use crate::chunk::{Chunk, ChunkKind};

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
                let mut i = 0;
                let mut running_status: Option<u8> = None;

                while i < chunk.data.len() {
                    let mut delta_time: u32 = 0;

                    loop {
                        let byte = chunk.data[i];
                        i += 1;

                        delta_time = (delta_time << 7) | ((byte & 0x7F) as u32);

                        if (byte & 0x80) == 0 {
                            break;
                        }
                    }

                    let status = chunk.data[i];
                    let event = match status {
                        0xFF => {
                            i += 1; // consume status byte

                            let meta_type = chunk.data[i];
                            debug_assert!(meta_type < 128);
                            i += 1; // consume meta type

                            // read VLQ length
                            let mut length = 0u32;
                            loop {
                                let b = chunk.data[i];
                                i += 1;
                                length = (length << 7) | (b & 0x7F) as u32;
                                if b & 0x80 == 0 {
                                    break;
                                }
                            }

                            let data = &chunk.data[i..i + length as usize];
                            i += length as usize;

                            Event::Meta {
                                kind: meta_type,
                                data: data.to_vec(),
                            }
                        }

                        0xF0 | 0xF7 => {
                            i += 1; // consume status byte

                            // TIPS: Event::SystemExclude will reset running status
                            running_status = None;

                            let mut length = 0u32;
                            loop {
                                let b = chunk.data[i];
                                i += 1;
                                length = (length << 7) | (b & 0x7F) as u32;
                                if b & 0x80 == 0 {
                                    break;
                                }
                            }

                            let data = &chunk.data[i..i + length as usize];
                            i += length as usize;

                            Event::SystemExclude {
                                data: data.to_vec(),
                            }
                        }

                        status_byte if status_byte >= 0x80 => {
                            i += 1; // consume status byte

                            // TIPS: MIDI channel event with explicit status
                            running_status = Some(status_byte);

                            let data_len = match status_byte & 0xF0 {
                                0xC0 | 0xD0 => 1, // Program Change, Channel Pressure
                                _ => 2,
                            };
                            let data = &chunk.data[i..i + data_len];
                            i += data_len;

                            Event::MIDI {
                                status: status_byte,
                                data: data.to_vec(),
                            }
                        }

                        _ => {
                            i += 0; // do not consume byte since it's part of data
                            // MIDI channel event with running status
                            let status =
                                running_status.ok_or(TryFromChunkError::MalformedRunningStatus)?;

                            let data_len = match status & 0xF0 {
                                0xC0 | 0xD0 => 1,
                                _ => 2,
                            };
                            let data = &chunk.data[i..i + data_len];
                            i += data_len;

                            Event::MIDI {
                                status,
                                data: data.to_vec(),
                            }
                        }
                    };

                    track_events.push(TrackEvent { delta_time, event });
                }

                debug_assert_eq!(i, chunk.data.len());
                Ok(TrackChunk { track_events })
            }
            _ => Err(TryFromChunkError::InvalidChunkType),
        }
    }
}
