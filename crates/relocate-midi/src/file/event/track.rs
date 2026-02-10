use derive_more::{Debug, Deref, Display, Error, IntoIterator};
use log::warn;

use crate::{file::chunk::track::TrackChunkFile, scanner::Scanner};

pub const TRACK_EVENT_DATA_00_MIN_MIDI_RUNNING: u8 = 0x00;
pub const TRACK_EVENT_DATA_7F_MAX_MIDI_RUNNING: u8 = 0x7F;

pub const TRACK_EVENT_STATUS_80_MIN_MIDI: u8 = 0x80;
pub const TRACK_EVENT_STATUS_EF_MAX_MIDI: u8 = 0xEF;

/// Start of System Exclusive
pub const TRACK_EVENT_STATUS_F0_SOX: u8 = 0xF0;

pub const TRACK_EVENT_STATUS_F1_MIN_SYS_COMMON: u8 = 0xF1;
pub const TRACK_EVENT_STATUS_F6_MAX_SYS_COMMON: u8 = 0xF6;

/// End of System Exclusive
pub const TRACK_EVENT_STATUS_F7_EOX: u8 = 0xF7;

pub const TRACK_EVENT_STATUS_F8_MIN_SYS_REALTIME: u8 = 0xF8;
pub const TRACK_EVENT_STATUS_FE_MAX_SYS_REALTIME: u8 = 0xFE;

pub const TRACK_EVENT_STATUS_FF_META: u8 = 0xFF;

#[derive(Debug)]
pub struct TrackEventFile<'a> {
    pub delta_time: u32,
    pub event: EventFile<'a>,
}

#[derive(Debug)]
pub enum EventFile<'a> {
    Meta(MetaEventFile<'a>),
    SysEx(SysExEventFile<'a>),
    Midi(MIDIEventFile<'a>),
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
                TRACK_EVENT_DATA_00_MIN_MIDI_RUNNING..=TRACK_EVENT_DATA_7F_MAX_MIDI_RUNNING => {
                    let status = running_status.ok_or(TryFromError::RunningStatusNotSet)?;
                    running_status = Some(status);
                    let data = scanner
                        .eat_data_bytes()
                        .ok_or(TryFromError::CouldNotReadData)?;
                    TrackEventFile {
                        delta_time,
                        event: EventFile::Midi(MIDIEventFile { status, data }),
                    }
                }

                TRACK_EVENT_STATUS_80_MIN_MIDI..=TRACK_EVENT_STATUS_EF_MAX_MIDI => {
                    let status = scanner.eat().ok_or(TryFromError::CouldNotReadStatus)?;
                    running_status = Some(status);
                    let data = scanner
                        .eat_data_bytes()
                        .ok_or(TryFromError::CouldNotReadData)?;
                    TrackEventFile {
                        delta_time,
                        event: EventFile::Midi(MIDIEventFile { status, data }),
                    }
                }

                TRACK_EVENT_STATUS_FF_META => {
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
                            status: &TRACK_EVENT_STATUS_FF_META,
                            kind,
                            length,
                            data,
                        }),
                    }
                }

                TRACK_EVENT_STATUS_F0_SOX => {
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
                            status: &TRACK_EVENT_STATUS_F0_SOX,
                            length,
                            data,
                        }),
                    }
                }

                TRACK_EVENT_STATUS_F7_EOX => {
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
                            status: &TRACK_EVENT_STATUS_F7_EOX,
                            length,
                            data,
                        }),
                    }
                }

                // According to the SMF specification, System Common
                // (0xF1–0xF6) and System Real-Time (0xF8–0xFE) messages are
                // not valid events within a MIDI file. If such status bytes
                // appear, the file is non-conforming. In practice, many
                // parsers choose to ignore these bytes or treat them as
                // malformed data to maintain compatibility with legacy or
                // poorly generated files.
                TRACK_EVENT_STATUS_F1_MIN_SYS_COMMON..=TRACK_EVENT_STATUS_F6_MAX_SYS_COMMON
                | TRACK_EVENT_STATUS_F8_MIN_SYS_REALTIME..=TRACK_EVENT_STATUS_FE_MAX_SYS_REALTIME =>
                {
                    scanner.eat().ok_or(TryFromError::CouldNotReadStatus)?;
                    running_status = None;
                    warn!(
                        "Encountered invalid status byte {:#X} in MIDI file. Skipping event.",
                        status_byte
                    );
                    continue;
                }
            };
            events.push(event);
        }

        Ok(TrackEventsFile(events))
    }
}
