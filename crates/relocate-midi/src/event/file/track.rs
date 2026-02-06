use derive_more::{Debug, Deref, Display, Error, IntoIterator};

use crate::{file::chunk::track::TrackChunkFile, scanner::Scanner};

pub const TRACK_EVENT_STATUS_META: &u8 = &0xFF;
pub const TRACK_EVENT_STATUS_SYS_EX_F0: &u8 = &0xF0;
pub const TRACK_EVENT_STATUS_SYS_EX_F7: &u8 = &0xF7;
pub const TRACK_EVENT_STATUS_MIDI_MIN: u8 = 0x80;
pub const TRACK_EVENT_STATUS_MIDI_MAX: u8 = 0xEF;

#[derive(Debug)]
pub enum TrackEventFile<'a> {
    Meta {
        status: &'static u8,
        kind: u8,
        length: u32,
        data: &'a [u8],
    },
    SysEx {
        status: &'static u8,
        length: u32,
        data: &'a [u8],
    },
    MIDI {
        status: u8,
        data: &'a [u8],
    },
}

#[derive(Debug, Deref, IntoIterator)]
pub struct TrackEventsFile<'a>(Vec<TrackEventFile<'a>>);

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    InvalidStatusByte,
    CouldNotReadStatus,
    CouldNotReadVLQ,
    CouldNotReadData,
}

impl<'a> TryFrom<&'a TrackChunkFile<'a>> for TrackEventsFile<'a> {
    type Error = TryFromError;

    fn try_from(value: &'a TrackChunkFile<'a>) -> Result<Self, Self::Error> {
        let mut events = Vec::new();
        let mut scanner = Scanner::new(value.track_events);
        let mut running_status: Option<u8> = None;

        while !scanner.done() {
            let event = match &scanner.peek().ok_or(TryFromError::CouldNotReadStatus)? {
                TRACK_EVENT_STATUS_META => {
                    scanner.eat().ok_or(TryFromError::CouldNotReadStatus)?;
                    running_status = None;

                    let kind = scanner.eat().ok_or(TryFromError::CouldNotReadStatus)?;
                    let length = scanner
                        .eat_variable_length_quantity()
                        .ok_or(TryFromError::CouldNotReadVLQ)?;
                    let data = scanner
                        .eat_slice(length as usize)
                        .ok_or(TryFromError::CouldNotReadData)?;

                    TrackEventFile::Meta {
                        status: TRACK_EVENT_STATUS_META,
                        kind,
                        length,
                        data,
                    }
                }
                TRACK_EVENT_STATUS_SYS_EX_F0 => {
                    scanner.eat().ok_or(TryFromError::CouldNotReadStatus)?;
                    running_status = None;

                    let length = scanner
                        .eat_variable_length_quantity()
                        .ok_or(TryFromError::CouldNotReadVLQ)?;
                    let data = scanner
                        .eat_slice(length as usize)
                        .ok_or(TryFromError::CouldNotReadData)?;

                    TrackEventFile::SysEx {
                        status: TRACK_EVENT_STATUS_SYS_EX_F0,
                        length,
                        data,
                    }
                }
                TRACK_EVENT_STATUS_SYS_EX_F7 => {
                    scanner.eat().ok_or(TryFromError::CouldNotReadStatus)?;
                    running_status = None;

                    let length = scanner
                        .eat_variable_length_quantity()
                        .ok_or(TryFromError::CouldNotReadVLQ)?;
                    let data = scanner
                        .eat_slice(length as usize)
                        .ok_or(TryFromError::CouldNotReadData)?;

                    TrackEventFile::SysEx {
                        status: TRACK_EVENT_STATUS_SYS_EX_F7,
                        length,
                        data,
                    }
                }
                TRACK_EVENT_STATUS_MIDI_MIN..=TRACK_EVENT_STATUS_MIDI_MAX => {
                    let status = scanner.eat().ok_or(TryFromError::CouldNotReadStatus)?;
                    running_status = Some(status);
                    todo!()
                }
                0x00..=0x7F => {
                    let status = running_status.ok_or(TryFromError::CouldNotReadStatus)?;
                    running_status = Some(status);
                    todo!()
                }
                // According to the SMF specification, System Common
                // (0xF1–0xF6) and System Real-Time (0xF8–0xFE) messages are
                // not valid events within a MIDI file. If such status bytes
                // appear, the file is non-conforming. In practice, many
                // parsers choose to ignore these bytes or treat them as
                // malformed data to maintain compatibility with legacy or
                // poorly generated files.
                0xF0..0xFF => Err(TryFromError::InvalidStatusByte)?,
            };
            events.push(event);
        }

        Ok(TrackEventsFile(events))
    }
}
