use derive_more::Debug;

use crate::{
    chunk::{Chunk, kind::ChunkKind},
    midi::format::MIDIFormat,
};

/// The [header chunk] at the beginning of the file specifies some basic
/// information about the data in the file.
///
/// [header chunk]: HeaderChunk
#[derive(Debug)]
pub struct HeaderChunk {
    /// The [format](HeaderChunk::format) specifies
    /// the overall organization of the file.
    pub format: MIDIFormat,

    /// The [tracks_count](HeaderChunk::tracks_count) is the number of track
    /// chunks in the file. It will always be `1` for
    /// [MIDIFormat::SingleMultiChannelTrack].
    pub tracks_count: u16,

    pub division: Division,
}

#[derive(Debug)]
pub enum Division {
    TicksPerQuarterNote(u16),
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
                let format_bytes: [u8; 2] = chunk.data[0..2]
                    .try_into()
                    .map_err(|_| TryFromChunkError::MalformedMIDIFormat)?;
                let format = MIDIFormat::try_from(format_bytes)
                    .map_err(|_| TryFromChunkError::MalformedMIDIFormat)?;

                let tracks_count_bytes: [u8; 2] = chunk.data[2..4]
                    .try_into()
                    .map_err(|_| TryFromChunkError::MalformedTracksCount)?;
                let tracks_count = u16::from_be_bytes(tracks_count_bytes);

                let division_bytes: [u8; 2] = chunk.data[4..6]
                    .try_into()
                    .map_err(|_| TryFromChunkError::MalformedDivision)?;
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
