use derive_more::{Debug, Deref, IntoIterator};

use crate::{
    core::event::{Event, TryFromError},
    file::event::track::TrackEventsFile,
};

#[derive(Debug)]
pub struct TrackEvent {
    /// Represents the amount of time before the following event, stored as a
    /// variable-length quantity.
    ///
    /// If the first event in a track occurs at the very beginning of a track,
    /// or if two events occur simultaneously, a delta-time of zero is used.
    ///
    /// Delta-times are _always_ present. (_Not_ storing delta-times of 0
    /// requires at least two bytes for any other value, and most delta
    /// times _aren't_ zero.)
    ///
    /// Delta-time is in ticks as specified in the header chunk.
    pub delta_time: u32,

    pub kind: Event,
}

/// The track chunks (type MTrk) are where actual song data is stored.  Each
/// track chunk is simply a stream of MIDI events (and non-MIDI events),
/// preceded by delta-time values.
///
/// The format for Track Chunks (described below) is exactly the same for all
/// three formats (0, 1, and 2: see "Header Chunk" above) of MIDI Files.
#[derive(Debug, Deref, IntoIterator)]
pub struct TrackChunk(Vec<TrackEvent>);

impl<'a> TryFrom<&'a TrackEventsFile<'a>> for TrackChunk {
    type Error = TryFromError;

    fn try_from(value: &TrackEventsFile) -> Result<Self, Self::Error> {
        let mut track_events = Vec::new();
        for track_event_file in value.iter() {
            let delta_time = track_event_file.delta_time;
            let kind = Event::try_from(&track_event_file.event)?;
            track_events.push(TrackEvent { delta_time, kind });
        }
        Ok(TrackChunk(track_events))
    }
}
