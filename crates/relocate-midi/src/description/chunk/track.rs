use derive_more::{Debug, Deref};

use crate::{
    description::chunk::{Chunk, ChunkKind},
    scanner::Scanner,
};

/// The track chunks (type MTrk) are where actual song data is stored.  Each
/// track chunk is simply a stream of MIDI events (and non-MIDI events),
/// preceded by delta-time values.
///
/// The format for Track Chunks (described below) is exactly the same for all
/// three formats (0, 1, and 2: see "Header Chunk" above) of MIDI Files.
#[derive(Debug, Deref)]
pub struct TrackChunk(Vec<TrackEvent>);

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

    pub kind: EventKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    Meta {
        status: u8,
        data: Vec<u8>,
    },
    SystemExclusive {
        kind: SystemExclusiveEventKind,
        data: Vec<u8>,
    },
    MIDI {
        status: u8,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemExclusiveEventKind {
    F0,
    F7,
}

#[derive(Debug)]
pub enum TryFromChunkError {
    InvalidChunkType,
    InvalidVLQ,
    InvalidStatusByte,
    InvalidData,
    InvalidRunningStatus,
}

impl TryFrom<&Chunk> for TrackChunk {
    type Error = TryFromChunkError;

    fn try_from(chunk: &Chunk) -> Result<Self, Self::Error> {
        match &chunk.kind {
            ChunkKind::Track => {
                let mut events = Vec::new();
                let mut scanner = Scanner::new(&chunk.data);

                // Running status is used: status bytes of MIDI events may be omitted
                // if the preceding event is a MIDI event with the same status.
                let mut running_status: Option<u8> = None;

                while !scanner.done() {
                    let event = parse_event(&mut scanner, &mut running_status)?;
                    events.push(event);
                }

                Ok(TrackChunk(events))
            }
            _ => Err(TryFromChunkError::InvalidChunkType),
        }
    }
}

/// Parses a single track event from the scanner, including delta time and event
/// data. Updates the running status as needed based on the event type.
fn parse_event(
    scanner: &mut Scanner,
    running_status: &mut Option<u8>,
) -> Result<TrackEvent, TryFromChunkError> {
    let delta_time = scanner
        .eat_variable_length_quantity()
        .ok_or(TryFromChunkError::InvalidVLQ)?;

    let kind_byte = scanner.peek().ok_or(TryFromChunkError::InvalidStatusByte)?;

    let kind = match kind_byte {
        0xFF => {
            scanner.eat();
            *running_status = None; // TIPS: Reset for not MIDI event
            parse_meta_event(scanner)?
        }
        0xF0 => {
            scanner.eat();
            *running_status = None; // TIPS: Reset for not MIDI event
            parse_system_exclusive_event(scanner, SystemExclusiveEventKind::F0)?
        }
        0xF7 => {
            scanner.eat();
            *running_status = None; // TIPS: Reset for not MIDI event
            parse_system_exclusive_event(scanner, SystemExclusiveEventKind::F7)?
        }
        status if status >= 0x80 => {
            scanner.eat();
            *running_status = Some(status); // TIPS: Set for MIDI event
            parse_midi_event(scanner, status)?
        }
        _ => {
            let status = running_status.ok_or(TryFromChunkError::InvalidRunningStatus)?; // TIPS: Use for MIDI event
            parse_midi_event(scanner, status)?
        }
    };

    Ok(TrackEvent { delta_time, kind })
}

/// Specifies non-MIDI information useful to this format or to sequencers, with
/// this syntax: `FF <type> <length> <bytes>`
fn parse_meta_event(scanner: &mut Scanner) -> Result<EventKind, TryFromChunkError> {
    let status = scanner.eat().ok_or(TryFromChunkError::InvalidStatusByte)?;
    debug_assert!(status < 0x80);

    let length = scanner
        .eat_variable_length_quantity()
        .ok_or(TryFromChunkError::InvalidVLQ)?;

    let data = scanner
        .eat_vec(length as usize)
        .ok_or(TryFromChunkError::InvalidData)?;

    debug_assert_eq!(data.len() as u32, length);

    Ok(EventKind::Meta { status, data })
}

fn parse_system_exclusive_event(
    scanner: &mut Scanner,
    kind: SystemExclusiveEventKind,
) -> Result<EventKind, TryFromChunkError> {
    let length = scanner
        .eat_variable_length_quantity()
        .ok_or(TryFromChunkError::InvalidVLQ)?;

    let data = scanner
        .eat_vec(length as usize)
        .ok_or(TryFromChunkError::InvalidData)?;

    debug_assert_eq!(data.len() as u32, length);

    Ok(EventKind::SystemExclusive { kind, data })
}

fn parse_midi_event(scanner: &mut Scanner, status: u8) -> Result<EventKind, TryFromChunkError> {
    // TODO: It's true?
    let data_len = match status & 0xF0 {
        0xC0 | 0xD0 => 1, // Program Change, Channel Pressure
        _ => 2,
    };
    let data = scanner
        .eat_vec(data_len)
        .ok_or(TryFromChunkError::InvalidData)?;

    Ok(EventKind::MIDI { status, data })
}
