use derive_more::{Debug, Display, Error};

use crate::file::event::track::EventFile;

pub mod meta;

#[derive(Debug)]
pub enum Event {
    Meta(meta::MetaEvent),
}

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    MetaEventFileToMetaEvent(meta::TryFromError),
}

impl<'a> TryFrom<&'a EventFile<'a>> for Event {
    type Error = TryFromError;

    fn try_from(value: &EventFile) -> Result<Self, Self::Error> {
        match value {
            EventFile::Meta(meta_event_file) => {
                let meta_event = meta::MetaEvent::try_from(meta_event_file)
                    .map_err(TryFromError::MetaEventFileToMetaEvent)?;
                Ok(Event::Meta(meta_event))
            }
            _ => todo!(), // TODO: Implement conversion for SysEx and MIDI events
        }
    }
}
