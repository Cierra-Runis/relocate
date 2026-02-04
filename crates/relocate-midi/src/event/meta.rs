use derive_more::Debug;
use pretty_hex::PrettyHex;

use crate::{description::track::EventKind, scanner::Scanner};

/// In the syntax descriptions for each of the meta-events a set of conventions
/// is used to describe parameters of the events. The FF which begins each
/// event, the type of each event, and the lengths of events which do not have a
/// variable amount of data are given directly in hexadecimal. A notation such
/// as dd or se, which consists of two lower-case letters, mnemonically
/// represents an 8-bit value. Four identical lower-case letters such as wwww
/// refer to a 16-bit value, stored most-significant-byte first. Six identical
/// lower-case letters such as tttttt refer to a 24-bit value, stored
/// most-significant-byte first. The notation len refers to the length portion
/// of the meta-event syntax, that is, a number, stored as a variable-length
/// quantity, which specifies how many data bytes follow it in the meta-event.
/// The notations text and data refer to however many bytes of (possibly text)
/// data were just specified by the length.
///
/// In general, meta-events in a track which occur at the same time may occur in
/// any order. If a copyright event is used, it should be placed as early as
/// possible in the file, so it will be noticed easily. Sequence Number and
/// Sequence/Track Name events, if present, must appear at time 0. An
/// end-of-track event must occur as the last event in the track.
#[derive(Debug)]
pub enum MetaEvent {
    /// This optional event, which must occur at the beginning of a track,
    /// before any nonzero delta-times, and before any transmittable MIDI
    /// events, specifies the number of a sequence. In a format 2 MIDI file,
    /// it is used to identify each "pattern" so that a "song"
    /// sequence using the Cue message to refer to the patterns. If the ID
    /// numbers are omitted, the sequences' locations in order in the file
    /// are used as defaults. In a format 0 or 1 MIDI file, which only
    /// contain one sequence, this number should be contained in the
    /// first (or only) track. If transfer of several multitrack sequences is
    /// required, this must be done as a group of format 1 files, each with
    /// a different sequence number.
    SequenceNumber(u16),

    /// Any amount of text describing anything. It is a good idea to put a text
    /// event right at the beginning of a track, with the name of the track,
    /// a description of its intended orchestration, and any other
    /// information which the user wants to put there. Text events may also
    /// occur at other times in a track, to be used as lyrics, or descriptions
    /// of cue points. The text in this event should be printable ASCII
    /// characters for maximum interchange. However, other character codes
    /// using the high-order bit may be used for interchange of files
    /// between different programs on the same computer which supports
    /// an extended character set. Programs on a computer which does not
    /// support non-ASCII characters should ignore those characters.
    TextEvent(String),

    /// Contains a copyright notice as printable ASCII text. The notice should
    /// contain the characters (C), the year of the copyright, and the owner
    /// of the copyright. If several pieces of music are in the same MIDI
    /// file, all of the copyright notices should be placed together in this
    /// event so that it will be at the beginning of the file. This event
    /// should be the first event in the first track chunk, at time 0.
    CopyrightNotice(String),

    /// If in a format 0 track, or the first track in a format 1 file, the name
    /// of the sequence. Otherwise, the name of the track.
    SequenceOrTrackName(String),

    /// A description of the type of instrumentation to be used in that track.
    /// May be used with the MIDI Prefix meta-event to specify which MIDI
    /// channel the description applies to, or the channel may be specified as
    /// text in the event itself.
    InstrumentName(String),

    /// A lyric to be sung. Generally, each syllable will be a separate lyric
    /// event which begins at the event's time.
    Lyric(String),

    /// Normally in a format 0 track, or the first track in a format 1 file. The
    /// name of that point in the sequence, such as a rehearsal letter or
    /// section name ("First Verse", etc.).
    Marker(String),

    /// A description of something happening on a film or video screen or stage
    /// at that point in the musical score ("Car crashes into house", "curtain
    /// opens", "she slaps his face", etc.)
    CuePoint(String),

    /// The MIDI channel (0-15) contained in this event may be used to associate
    /// a MIDI channel with all events which follow, including System Exclusive
    /// and meta-events. This channel is "effective" until the next normal
    /// MIDI event (which contains a channel) or the next MIDI Channel Prefix
    /// meta-event. If MIDI channels refer to "tracks", this message may help
    /// jam several tracks into a format 0 file, keeping their non-MIDI data
    /// associated with a track. This capability is also present in Yamaha's
    /// ESEQ file format.
    MIDIChannelPrefix(u8),

    /// This event is _not_ optional. It is included so that an exact ending
    /// point may be specified for the track, so that it has an exact length,
    /// which is necessary for tracks which are looped or concatenated.
    EndOfTrack,
}

#[derive(Debug)]
pub enum TryFromEventKindError {
    InvalidEventKind,
    InvalidNumber,
    InvalidData,
    InvalidTextEncoding,
    #[debug("\"{}\"", [*_0].hex_dump().to_string())]
    InvalidStatus(u8),
}

impl TryFrom<&EventKind> for MetaEvent {
    type Error = TryFromEventKindError;

    fn try_from(value: &EventKind) -> Result<Self, Self::Error> {
        match value {
            EventKind::Meta { status, data } => {
                macro_rules! text_event {
                    ($variant:ident) => {{
                        let text = std::str::from_utf8(data)
                            .map_err(|_| TryFromEventKindError::InvalidTextEncoding)?;
                        Ok(MetaEvent::$variant(text.to_string()))
                    }};
                }

                match status {
                    0x00 if data.len() == 2 => {
                        let mut scanner = Scanner::new(data);
                        let number = scanner
                            .eat_u16_be()
                            .ok_or(TryFromEventKindError::InvalidNumber)?;
                        Ok(MetaEvent::SequenceNumber(number))
                    }
                    0x00 => Err(TryFromEventKindError::InvalidData),

                    0x01 | 0x08..0x10 => text_event!(TextEvent),
                    0x02 => text_event!(CopyrightNotice),
                    0x03 => text_event!(SequenceOrTrackName),
                    0x04 => text_event!(InstrumentName),
                    0x05 => text_event!(Lyric),
                    0x06 => text_event!(Marker),
                    0x07 => text_event!(CuePoint),

                    0x20 if data.len() == 2 => {
                        let mut scanner = Scanner::new(data);
                        if scanner.eat() != Some(0x01) {
                            return Err(TryFromEventKindError::InvalidData);
                        }
                        let channel = scanner.eat().ok_or(TryFromEventKindError::InvalidData)?;
                        Ok(MetaEvent::MIDIChannelPrefix(channel))
                    }
                    0x20 => Err(TryFromEventKindError::InvalidData),

                    // According to the MIDI specification, `data` should be `[0x00]` here.
                    // However, some MIDI files omit this byte, so we will accept both.
                    0x2F => Ok(MetaEvent::EndOfTrack),

                    status => Err(TryFromEventKindError::InvalidStatus(*status)),
                }
            }
            _ => Err(TryFromEventKindError::InvalidEventKind),
        }
    }
}
