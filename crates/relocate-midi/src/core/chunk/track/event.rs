use derive_more::{Debug, PartialEq};

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

    pub kind: TrackEventKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrackEventKind {
    Meta {
        status: u8,
        data: Vec<u8>,
    },
    SystemExclusive {
        kind: SystemExclusiveEventKind,
        data: Vec<u8>,
    },
    MIDI {
        status: u8,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemExclusiveEventKind {
    F0,
    F7,
}
