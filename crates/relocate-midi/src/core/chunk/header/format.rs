use derive_more::{Debug, Display, Eq, Error, PartialEq};

/// We may decide to define other format IDs to support other structures.
///
/// A program encountering an unknown format ID may still read other MTrk
/// chunks it finds from the file, as format 1 or 2, if its user can make
/// sense of them and arrange them into some other structure if appropriate.
///
/// Also, more parameters may be added to the MThd chunk in the future: it
/// is important to read and honor the length, even if it is longer than 6.
#[derive(Debug, Display, PartialEq, Eq)]
#[non_exhaustive]
pub enum Format {
    /// The file contains a single multi-channel track.
    SingleMultiChannelTrack,
    /// The file contains one or more simultaneous tracks (or MIDI outputs) of a
    /// sequence.
    SimultaneousTracks,
    /// The file contains one or more sequentially independent single-track
    /// patterns.
    SequentiallyIndependentSingleTrackPatterns,
}

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    UnknownFormatBytes,
}

impl TryFrom<&[u8; 2]> for Format {
    type Error = TryFromError;

    fn try_from(bytes: &[u8; 2]) -> Result<Self, Self::Error> {
        match bytes {
            [0x00, 0x00] => Ok(Format::SingleMultiChannelTrack),
            [0x00, 0x01] => Ok(Format::SimultaneousTracks),
            [0x00, 0x02] => Ok(Format::SequentiallyIndependentSingleTrackPatterns),
            _ => Err(TryFromError::UnknownFormatBytes),
        }
    }
}
