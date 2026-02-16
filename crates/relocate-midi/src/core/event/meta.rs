use derive_more::{Debug, Display, Error};

use crate::{file::event::track::MetaEventFile, scanner::Scanner};

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

    /// Many systems provide a number of separately addressable MIDI ports in
    /// order to get around bandwidth issues and the 16 MIDI channel limit. This
    /// optional event specifies the MIDI output port on which data within this
    /// MTrk chunk will be transmitted.
    ///
    /// Naturally, this event should be placed prior to any MIDI events that are
    /// to be affected. Usually it would be placed at time=0 (i.e. at the start
    /// of a track), however it is possible to place more than one such event in
    /// any MTrk chunk, should you wish to output data through a different port
    /// later in the track.
    ///
    /// See: <http://www.somascape.org/midi/tech/mfile.html>
    MIDIPort(u8),

    /// This event is _not_ optional. It is included so that an exact ending
    /// point may be specified for the track, so that it has an exact length,
    /// which is necessary for tracks which are looped or concatenated.
    EndOfTrack,

    /// This event indicates a tempo change.  Another way of putting
    /// "microseconds per quarter-note" is "24ths of a microsecond per MIDI
    /// clock".  Representing tempos as time per beat instead of beat per
    /// time allows absolutely exact long-term synchronization with
    /// a time-based sync protocol such as SMPTE time code or MIDI time code.
    /// This amount of accuracy provided by this tempo resolution allows a
    /// four-minute piece at 120 beats per minute to be accurate within 500
    /// usec at the end of the piece.  Ideally, these events should only
    /// occur where MIDI clocks would be located — this convention is intended
    /// to guarantee, or at least increase the likelihood, of compatibility
    /// with other synchronization devices so that a time signature/tempo
    /// map stored in this format may easily be transferred to another
    /// device.
    SetTempo(u32),

    // This event, if present, designates the SMPTE time at which the track chunk is supposed
    // to start. It should be present at the beginning of the track, that is, before any nonzero
    // delta-times, and before any transmittable MIDI events. The hour must be encoded with
    // the SMPTE format, just as it is in MIDI Time Code. In a format 1 file, the SMPTE
    // Offset must be stored with the tempo map, and has no meaning in any of the other tracks.
    // The ff field contains fractional frames, in 100ths of a frame, even in SMPTE
    // based tracks which specify a different frame subdivision for delta-times.
    SMPTEOffset {
        hours: u8,
        minutes: u8,
        seconds: u8,
        frames: u8,
        fractional_frames: u8,
    },

    /// The time signature is expressed as four numbers.
    ///
    /// nn and dd represent the numerator and denominator of the time signature
    /// as it would be notated.
    ///
    /// The denominator is a negative power of two: 2 represents
    /// a quarter-note, 3 represents an eighth-note, etc.
    ///
    /// The cc parameter expresses the number of MIDI clocks in a metronome
    /// click.
    ///
    /// The bb parameter expresses the number of notated 32nd-notes in what MIDI
    /// thinks of as a quarter-note (24 MIDI Clocks). This was added because
    /// there are already multiple programs which allow the user to specify
    /// that what MIDI thinks of as a quarter-note (24 clocks) is to be
    /// notated as, or related to in terms of, something else.
    ///
    /// Therefore, the complete event for 6/8 time, where the metronome clicks
    /// every three eighth-notes, but there are 24 clocks per quarter-note,
    /// 72 to the bar, would be (in hex):
    ///
    /// `FF 58 04 06 03 24 08`
    ///
    /// That is, 6/8 time (8 is 2 to the 3rd power, so this is 06 03), 36 MIDI
    /// clocks per dotted quarter (24 hex!), and eight notated 32nd-notes
    /// per MIDI quarter note.
    TimeSignature {
        numerator: u8,
        denominator: u8,
        midi_clocks_per_metronome_click: u8,
        thirty_second_notes_per_midi_quarter_note: u8,
    },

    KeySignature {
        sharps_flats: i8,
        major_minor: u8,
    },
}

#[derive(Debug, Display, Error)]
pub enum TryFromError {
    InvalidEventKind,
    InvalidNumber,
    InvalidData,
    InvalidScannerState,
    #[debug("{:X}", _0)]
    InvalidStatus(#[error(ignore)] u8),
}

impl<'a> TryFrom<&'a MetaEventFile<'a>> for MetaEvent {
    type Error = TryFromError;

    fn try_from(value: &MetaEventFile) -> Result<Self, Self::Error> {
        let status = value.status;
        let data = value.data;
        macro_rules! text_event {
            ($variant:ident) => {
                Ok(MetaEvent::$variant(
                    String::from_utf8_lossy(data).to_string(),
                ))
            };
        }

        match status {
            0x00 => {
                let mut scanner = Scanner::new(data);
                let number = scanner.eat_u16_be().ok_or(TryFromError::InvalidNumber)?;
                if !scanner.done() {
                    return Err(TryFromError::InvalidScannerState);
                }
                Ok(MetaEvent::SequenceNumber(number))
            }

            0x01 | 0x08..0x10 => text_event!(TextEvent),
            0x02 => text_event!(CopyrightNotice),
            0x03 => text_event!(SequenceOrTrackName),
            0x04 => text_event!(InstrumentName),
            0x05 => text_event!(Lyric),
            0x06 => text_event!(Marker),
            0x07 => text_event!(CuePoint),

            0x20 => {
                let mut scanner = Scanner::new(data);
                let channel = *scanner.eat().ok_or(TryFromError::InvalidData)?;
                if !scanner.done() {
                    return Err(TryFromError::InvalidScannerState);
                }
                Ok(MetaEvent::MIDIChannelPrefix(channel))
            }

            0x21 => {
                let mut scanner = Scanner::new(data);
                let port = *scanner.eat().ok_or(TryFromError::InvalidData)?;
                if !scanner.done() {
                    return Err(TryFromError::InvalidScannerState);
                }
                Ok(MetaEvent::MIDIPort(port))
            }

            0x2F => Ok(MetaEvent::EndOfTrack),

            0x51 => {
                let mut scanner = Scanner::new(data);
                let [t1, t2, t3] = *scanner.eat_bytes::<3>().ok_or(TryFromError::InvalidData)?;
                let tempo = u32::from_be_bytes([0x00, t1, t2, t3]);
                if !scanner.done() {
                    return Err(TryFromError::InvalidScannerState);
                }
                Ok(MetaEvent::SetTempo(tempo))
            }

            0x54 => {
                let mut scanner = Scanner::new(data);
                let [hours, minutes, seconds, frames, fractional_frames] =
                    *scanner.eat_bytes::<5>().ok_or(TryFromError::InvalidData)?;
                if !scanner.done() {
                    return Err(TryFromError::InvalidScannerState);
                }
                Ok(MetaEvent::SMPTEOffset {
                    hours,
                    minutes,
                    seconds,
                    frames,
                    fractional_frames,
                })
            }

            0x58 => {
                let mut scanner = Scanner::new(data);
                let [numerator, denominator, cc, bb] =
                    *scanner.eat_bytes::<4>().ok_or(TryFromError::InvalidData)?;
                if !scanner.done() {
                    return Err(TryFromError::InvalidScannerState);
                }
                Ok(MetaEvent::TimeSignature {
                    numerator,
                    denominator,
                    midi_clocks_per_metronome_click: cc,
                    thirty_second_notes_per_midi_quarter_note: bb,
                })
            }

            0x59 => {
                let mut scanner = Scanner::new(data);
                let sharps_flats = *scanner.eat().ok_or(TryFromError::InvalidData)? as i8;
                let major_minor = *scanner.eat().ok_or(TryFromError::InvalidData)?;
                if !scanner.done() {
                    return Err(TryFromError::InvalidScannerState);
                }
                Ok(MetaEvent::KeySignature {
                    sharps_flats,
                    major_minor,
                })
            }

            status => Err(TryFromError::InvalidStatus(*status)),
        }
    }
}
