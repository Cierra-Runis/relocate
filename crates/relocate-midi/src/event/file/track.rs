use derive_more::{Debug, Deref, Display, Error, IntoIterator};

use crate::{file::chunk::track::TrackChunkFile, scanner::Scanner};

pub const TRACK_EVENT_STATUS_META: u8 = 0xFF;
pub const TRACK_EVENT_STATUS_SYS_EX_F0: u8 = 0xF0;
pub const TRACK_EVENT_STATUS_SYS_EX_F7: u8 = 0xF7;
pub const TRACK_EVENT_STATUS_MIDI_MIN: u8 = 0x80;
pub const TRACK_EVENT_STATUS_MIDI_MAX: u8 = 0xEF;

#[derive(Debug)]
pub struct TrackEventFile<'a> {
    pub delta_time: u32,
    pub event: EventFile<'a>,
}

#[derive(Debug)]
pub enum EventFile<'a> {
    Meta(MetaEventFile<'a>),
    SysEx(SysExEventFile<'a>),
    MIDI(MIDIEventFile<'a>),
}

#[derive(Debug)]
pub struct MetaEventFile<'a> {
    pub status: &'static u8,
    pub kind: &'a u8,
    pub length: u32,
    pub data: &'a [u8],
}

#[derive(Debug)]
pub struct SysExEventFile<'a> {
    pub status: &'static u8,
    pub length: u32,
    pub data: &'a [u8],
}

#[derive(Debug)]
pub struct MIDIEventFile<'a> {
    pub status: &'a u8,
    pub data: &'a [u8],
}

#[derive(Debug, Deref, IntoIterator)]
pub struct TrackEventsFile<'a>(Vec<TrackEventFile<'a>>);

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    InvalidStatusByte,
    CouldNotReadStatus,
    CouldNotReadVLQ,
    CouldNotReadData,
    RunningStatusNotSet,
}

impl<'a> TryFrom<&'a TrackChunkFile<'a>> for TrackEventsFile<'a> {
    type Error = TryFromError;

    fn try_from(value: &'a TrackChunkFile<'a>) -> Result<Self, Self::Error> {
        let mut events = Vec::new();
        let mut scanner = Scanner::new(value.track_events);
        let mut running_status: Option<&'a u8> = None;

        while !scanner.done() {
            let delta_time = scanner
                .eat_variable_length_quantity()
                .ok_or(TryFromError::CouldNotReadVLQ)?;

            let status_byte = *scanner.peek().ok_or(TryFromError::CouldNotReadStatus)?;

            let event = match status_byte {
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

                    TrackEventFile {
                        delta_time,
                        event: EventFile::Meta(MetaEventFile {
                            status: &TRACK_EVENT_STATUS_META,
                            kind,
                            length,
                            data,
                        }),
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

                    TrackEventFile {
                        delta_time,
                        event: EventFile::SysEx(SysExEventFile {
                            status: &TRACK_EVENT_STATUS_SYS_EX_F0,
                            length,
                            data,
                        }),
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

                    TrackEventFile {
                        delta_time,
                        event: EventFile::SysEx(SysExEventFile {
                            status: &TRACK_EVENT_STATUS_SYS_EX_F7,
                            length,
                            data,
                        }),
                    }
                }
                TRACK_EVENT_STATUS_MIDI_MIN..=TRACK_EVENT_STATUS_MIDI_MAX => {
                    let status = scanner.eat().ok_or(TryFromError::CouldNotReadStatus)?;
                    running_status = Some(status);

                    let data = scanner
                        .eat_until_high_bit_is_one()
                        .ok_or(TryFromError::CouldNotReadData)?;
                    TrackEventFile {
                        delta_time,
                        event: EventFile::MIDI(MIDIEventFile { status, data }),
                    }
                }
                0x00..=0x7F => {
                    let status = running_status.ok_or(TryFromError::RunningStatusNotSet)?;
                    running_status = Some(status);
                    let data = scanner
                        .eat_until_high_bit_is_one()
                        .ok_or(TryFromError::CouldNotReadData)?;
                    TrackEventFile {
                        delta_time,
                        event: EventFile::MIDI(MIDIEventFile { status, data }),
                    }
                }
                // According to the SMF specification, System Common
                // (0xF1–0xF6) and System Real-Time (0xF8–0xFE) messages are
                // not valid events within a MIDI file. If such status bytes
                // appear, the file is non-conforming. In practice, many
                // parsers choose to ignore these bytes or treat them as
                // malformed data to maintain compatibility with legacy or
                // poorly generated files.
                0xF1..0xFF => Err(TryFromError::InvalidStatusByte)?,
            };
            events.push(event);
        }

        Ok(TrackEventsFile(events))
    }
}
