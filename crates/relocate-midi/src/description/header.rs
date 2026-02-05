use derive_more::Debug;

use crate::{
    description::chunk::{Chunk, ChunkKind},
    midi::format::MIDIFormat,
    scanner::Scanner,
};

/// The [header chunk] at the beginning of the file specifies some basic
/// information about the data in the file.
///
/// [header chunk]: HeaderChunk
#[derive(Debug)]
pub struct HeaderChunk {
    /// Specifies the overall organization of the file.
    pub format: MIDIFormat,

    /// The number of track chunks in the file.
    pub tracks_count: u16,

    /// Specifies the meaning of the delta-times.
    pub division: Division,
}

#[derive(Debug)]
pub enum Division {
    /// For metrical time.
    TicksPerQuarterNote(u16),

    /// For time-code-based time.
    TimeCode {
        frames_per_second: FramesPerSecond,
        ticks_per_frame: u8,
    },
}

#[derive(Debug)]
pub enum FramesPerSecond {
    FPS24 = -24,
    FPS25 = -25,
    FPS30Drop = -29,
    FPS30 = -30,
}

#[derive(Debug)]
pub enum TryFromU16ToDivisionError {
    InvalidFPS,
}

impl TryFrom<[u8; 2]> for Division {
    type Error = TryFromU16ToDivisionError;

    /// If bit 15 of `value` is a zero, the bits 14 thru 0 represent the number
    /// of delta-time "ticks" which make up a quarter-note.
    ///
    /// If bit 15 of `value` is a one, delta-times in a file correspond to
    /// subdivisions of a second, in a way consistent with SMPTE and MIDI
    /// time code.
    fn try_from(value: [u8; 2]) -> Result<Self, Self::Error> {
        let [high, low] = value;

        if (high & 0x80) == 0 {
            return Ok(Division::TicksPerQuarterNote(u16::from_be_bytes(value)));
        }

        let frames_per_second = match high as i8 {
            -24 => Ok(FramesPerSecond::FPS24),
            -25 => Ok(FramesPerSecond::FPS25),
            -29 => Ok(FramesPerSecond::FPS30Drop),
            -30 => Ok(FramesPerSecond::FPS30),
            _ => Err(TryFromU16ToDivisionError::InvalidFPS),
        }?;
        let ticks_per_frame = low;

        Ok(Division::TimeCode {
            frames_per_second,
            ticks_per_frame,
        })
    }
}

#[derive(Debug)]
pub enum TryFromChunkError {
    InvalidChunkKind,
    MalformedMIDIFormat,
    MalformedTracksCount,
    MalformedDivision,
}

impl TryFrom<&Chunk> for HeaderChunk {
    type Error = TryFromChunkError;

    fn try_from(chunk: &Chunk) -> Result<Self, Self::Error> {
        match &chunk.kind {
            ChunkKind::Header(_) => {
                let mut scanner = Scanner::new(&chunk.data);

                let format_bytes = scanner
                    .eat_array::<2>()
                    .ok_or(TryFromChunkError::MalformedMIDIFormat)?;
                let format = MIDIFormat::try_from(format_bytes)
                    .map_err(|_| TryFromChunkError::MalformedMIDIFormat)?;

                let tracks_count = scanner
                    .eat_u16_be()
                    .ok_or(TryFromChunkError::MalformedTracksCount)?;

                // It will always be `1` for [MIDIFormat::SingleMultiChannelTrack].
                match format {
                    MIDIFormat::SingleMultiChannelTrack if tracks_count != 1 => {
                        return Err(TryFromChunkError::MalformedTracksCount);
                    }
                    _ => {}
                }

                // Read division (2 bytes)
                let division_bytes = scanner
                    .eat_array::<2>()
                    .ok_or(TryFromChunkError::MalformedDivision)?;
                let division = Division::try_from(division_bytes)
                    .map_err(|_| TryFromChunkError::MalformedDivision)?;

                Ok(HeaderChunk {
                    format,
                    tracks_count,
                    division,
                })
            }
            _ => Err(TryFromChunkError::InvalidChunkKind),
        }
    }
}
