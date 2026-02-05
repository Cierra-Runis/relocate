pub mod event;

use derive_more::{Debug, Deref, Display, Error, IntoIterator};

use crate::{
    core::chunk::track::event::{SysExEventKind, TrackEvent, TrackEventKind},
    file::chunk::track::TrackChunkFile,
    scanner::Scanner,
};

/// The track chunks (type MTrk) are where actual song data is stored.  Each
/// track chunk is simply a stream of MIDI events (and non-MIDI events),
/// preceded by delta-time values.
///
/// The format for Track Chunks (described below) is exactly the same for all
/// three formats (0, 1, and 2: see "Header Chunk" above) of MIDI Files.
#[derive(Debug, Deref, IntoIterator)]
pub struct TrackChunk(Vec<TrackEvent>);

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    InvalidVLQ,
    InvalidStatusByte,
    InvalidData,
    InvalidRunningStatus,
}

impl<'a> TryFrom<&'a TrackChunkFile<'a>> for TrackChunk {
    type Error = TryFromError;

    fn try_from(value: &TrackChunkFile) -> Result<Self, Self::Error> {
        let mut events = Vec::new();
        let mut scanner = Scanner::new(value.track_events);

        // Running status is used: status bytes of MIDI events may be
        // omitted if the preceding event is a MIDI event with the
        // same status.
        let mut running_status: Option<u8> = None;

        while !scanner.done() {
            let event = parse_event(&mut scanner, &mut running_status)?;
            events.push(event);
        }

        Ok(TrackChunk(events))
    }
}

// Parses a single track event from the scanner, including delta time and event
// data. Updates the running status as needed based on the event type.
fn parse_event(
    scanner: &mut Scanner,
    running_status: &mut Option<u8>,
) -> Result<TrackEvent, TryFromError> {
    let delta_time = scanner
        .eat_variable_length_quantity()
        .ok_or(TryFromError::InvalidVLQ)?;

    let kind_byte = scanner.peek().ok_or(TryFromError::InvalidStatusByte)?;

    let kind = match kind_byte {
        0xFF => {
            scanner.eat();
            *running_status = None; // TIPS: Reset for not MIDI event
            parse_meta_event(scanner)?
        }
        0xF0 => {
            scanner.eat();
            *running_status = None; // TIPS: Reset for not MIDI event
            parse_system_exclusive_event(scanner, SysExEventKind::F0)?
        }
        0xF7 => {
            scanner.eat();
            *running_status = None; // TIPS: Reset for not MIDI event
            parse_system_exclusive_event(scanner, SysExEventKind::F7)?
        }
        status if status >= 0x80 => {
            scanner.eat();
            *running_status = Some(status); // TIPS: Set for MIDI event
            parse_midi_event(scanner, status)?
        }
        _ => {
            let status = running_status.ok_or(TryFromError::InvalidRunningStatus)?; // TIPS: Use for MIDI event
            parse_midi_event(scanner, status)?
        }
    };

    Ok(TrackEvent { delta_time, kind })
}

// Specifies non-MIDI information useful to this format or to sequencers, with
// this syntax: `FF <type> <length> <bytes>`
fn parse_meta_event(scanner: &mut Scanner) -> Result<TrackEventKind, TryFromError> {
    let status = scanner.eat().ok_or(TryFromError::InvalidStatusByte)?;
    debug_assert!(status < 0x80);

    let length = scanner
        .eat_variable_length_quantity()
        .ok_or(TryFromError::InvalidVLQ)?;

    let data = scanner
        .eat_vec(length as usize)
        .ok_or(TryFromError::InvalidData)?;

    debug_assert_eq!(data.len() as u32, length);

    Ok(TrackEventKind::Meta { status, data })
}

fn parse_system_exclusive_event(
    scanner: &mut Scanner,
    kind: SysExEventKind,
) -> Result<TrackEventKind, TryFromError> {
    let length = scanner
        .eat_variable_length_quantity()
        .ok_or(TryFromError::InvalidVLQ)?;

    let data = scanner
        .eat_vec(length as usize)
        .ok_or(TryFromError::InvalidData)?;

    debug_assert_eq!(data.len() as u32, length);

    Ok(TrackEventKind::SysEx { kind, data })
}

fn parse_midi_event(scanner: &mut Scanner, status: u8) -> Result<TrackEventKind, TryFromError> {
    // TODO: It's true?
    let data_len = match status & 0xF0 {
        0xC0 | 0xD0 => 1, // Program Change, Channel Pressure
        _ => 2,
    };
    let data = scanner.eat_vec(data_len).ok_or(TryFromError::InvalidData)?;

    Ok(TrackEventKind::MIDI { status, data })
}
