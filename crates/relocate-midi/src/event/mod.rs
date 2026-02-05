pub mod meta;
pub mod midi;
pub mod system_exclusive;

pub enum Event {
    Meta(meta::MetaEvent),
    MIDI(midi::MIDIEvent),
    SystemExclusive(system_exclusive::SystemExclusiveEvent),
}
