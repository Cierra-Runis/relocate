use crate::description::chunk::header::division::fps::{FPS, TryFromError};

pub mod fps;

#[derive(Debug)]
pub enum Division {
    /// For metrical time.
    TicksPerQuarterNote(u16),

    /// For time-code-based time.
    TimeCode {
        frames_per_second: FPS,
        ticks_per_frame: u8,
    },
}

impl TryFrom<[u8; 2]> for Division {
    type Error = TryFromError;

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

        let fps = FPS::try_from(high).map_err(|_| TryFromError::InvalidFPS)?;

        Ok(Division::TimeCode {
            frames_per_second: fps,
            ticks_per_frame: low,
        })
    }
}
