pub mod event;

use derive_more::{Debug, Deref, Display, Error, IntoIterator};

use crate::{
    core::chunk::track::event::TrackEvent,
    event::file::track::{TrackEventFile, TrackEventsFile},
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

impl<'a> TryFrom<&'a TrackEventsFile<'a>> for TrackChunk {
    type Error = TryFromError;

    fn try_from(value: &TrackEventsFile) -> Result<Self, Self::Error> {
        let mut events = Vec::new();
        for track_event_file in value.iter() {
            let delta_time = track_event_file.delta_time;
            todo!();
        }
        Ok(TrackChunk(events))
    }
}
