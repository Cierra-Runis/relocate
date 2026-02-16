pub mod division;
pub mod format;

use derive_more::{Debug, Display, Error};

use crate::{
    core::chunk::header::{division::Division, format::Format},
    file::chunk::header::HeaderChunkFile,
};

/// The [`HeaderChunk`] at the beginning of the file specifies some basic
/// information about the data in the file.
#[derive(Debug)]
pub struct HeaderChunk {
    /// Specifies the overall organization of the file.
    pub format: Format,

    /// The number of track chunks in the file.
    pub tracks_count: u16,

    /// Specifies the meaning of the delta-times.
    pub division: Division,
}

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    InvalidFormat,
    InvalidDivision,
    InvalidTracksCount,
}

impl TryFrom<&HeaderChunkFile<'_>> for HeaderChunk {
    type Error = TryFromError;

    fn try_from(value: &HeaderChunkFile) -> Result<Self, Self::Error> {
        let format = Format::try_from(value.format).map_err(|_| TryFromError::InvalidFormat)?;
        let tracks_count = u16::from_be_bytes(*value.tracks_count);
        let division =
            Division::try_from(*value.division).map_err(|_| TryFromError::InvalidDivision)?;

        if format == Format::SingleMultiChannelTrack && tracks_count != 1 {
            return Err(TryFromError::InvalidTracksCount);
        }

        Ok(HeaderChunk {
            format,
            tracks_count,
            division,
        })
    }
}
