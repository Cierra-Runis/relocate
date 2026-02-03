use std::{sync::Arc, time::Duration};

use derive_more::Debug;

use crate::chunk::{Chunk, ChunkKind};

/// The track chunks (type MTrk) are where actual song data is stored.  Each
/// track chunk is simply a stream of MIDI events (and non-MIDI events),
/// preceded by delta-time values.
///
/// The format for Track Chunks (described below) is exactly the same for all
/// three formats (0, 1, and 2: see "Header Chunk" above) of MIDI Files.
#[derive(Debug)]
pub struct TrackChunk(Arc<[TrackEvent]>);

#[derive(Debug)]
pub struct TrackEvent {
    pub delta_time: Duration,
}

#[derive(Debug)]
pub enum Event {
    MIDI,
    SystemExclude,
    Meta,
}

impl TryFrom<&Chunk> for TrackChunk {
    type Error = ();

    fn try_from(chunk: &Chunk) -> Result<Self, Self::Error> {
        match &chunk.kind {
            ChunkKind::Track(_) => {
                // TODO: Parse track events from chunk.data
                Ok(TrackChunk(Arc::new([])))
            }
            _ => Err(()),
        }
    }
}
