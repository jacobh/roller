use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

mod io;
pub use io::{MidiInput, MidiIoError, MidiOutput};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Note(u8);
impl Note {
    pub fn new(note: u8) -> Note {
        Note(note)
    }
}
impl From<Note> for u8 {
    fn from(note: Note) -> u8 {
        note.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ControlChannel(u8);
impl ControlChannel {
    pub fn new(control_channel: u8) -> ControlChannel {
        ControlChannel(control_channel)
    }
}

/// Borrowed from https://github.com/RustAudio/rimd/blob/54fd9bd2bd3caaa6fe1c31fbf71c0f3c6597fd1a/src/midi.rs#L51-L77
/// The status field of a midi message indicates what midi command it
/// represents and what channel it is on
#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive)]
pub enum Status {
    // voice
    NoteOff = 0x80,
    NoteOn = 0x90,
    PolyphonicAftertouch = 0xA0,
    ControlChange = 0xB0,
    ProgramChange = 0xC0,
    ChannelAftertouch = 0xD0,
    PitchBend = 0xE0,

    // sysex
    SysExStart = 0xF0,
    MIDITimeCodeQtrFrame = 0xF1,
    SongPositionPointer = 0xF2,
    SongSelect = 0xF3,
    TuneRequest = 0xF6, // F4 anf 5 are reserved and unused
    SysExEnd = 0xF7,
    TimingClock = 0xF8,
    Start = 0xFA,
    Continue = 0xFB,
    Stop = 0xFC,
    ActiveSensing = 0xFE, // FD also res/unused
    SystemReset = 0xFF,
}
impl Status {
    // None if SysEx
    pub fn data_bytes(&self) -> Option<usize> {
        match self {
            Status::NoteOff
            | Status::NoteOn
            | Status::PolyphonicAftertouch
            | Status::ControlChange
            | Status::PitchBend
            | Status::SongPositionPointer => Some(2),

            Status::SysExStart => None,

            Status::ProgramChange
            | Status::ChannelAftertouch
            | Status::MIDITimeCodeQtrFrame
            | Status::SongSelect => Some(1),

            Status::TuneRequest
            | Status::SysExEnd
            | Status::TimingClock
            | Status::Start
            | Status::Continue
            | Status::Stop
            | Status::ActiveSensing
            | Status::SystemReset => Some(0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MidiEvent {
    NoteOn { note: Note, velocity: u8 },
    NoteOff { note: Note, velocity: u8 },
    ControlChange { control: ControlChannel, value: u8 },
    Other(Status),
}
impl MidiEvent {
    pub fn from_bytes(bytes: &[u8]) -> MidiEvent {
        let (status_byte, data_bytes) = bytes.split_first().unwrap();
        let status = Status::from_u8(*status_byte).unwrap();
        MidiEvent::from_status_and_data(status, data_bytes)
    }
    pub fn from_status_and_data(status: Status, bytes: &[u8]) -> MidiEvent {
        match status {
            Status::NoteOn => MidiEvent::NoteOn {
                note: Note::new(bytes[0]),
                velocity: bytes[1],
            },
            Status::NoteOff => MidiEvent::NoteOff {
                note: Note::new(bytes[0]),
                velocity: bytes[1],
            },
            Status::ControlChange => MidiEvent::ControlChange {
                control: ControlChannel::new(bytes[0]),
                value: bytes[1],
            },
            _ => MidiEvent::Other(status),
        }
    }
    pub fn next_from_iter(iter: &mut impl Iterator<Item = u8>) -> Option<MidiEvent> {
        let status = Status::from_u8(iter.next()?).unwrap();
        // TODO SysEx messages aren't handled
        let data_bytes: Vec<u8> = iter.take(status.data_bytes()?).collect();

        Some(MidiEvent::from_status_and_data(status, &data_bytes))
    }
}
