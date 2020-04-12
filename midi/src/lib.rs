use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};

mod io;
pub use io::{MidiInput, MidiIoError, MidiOutput};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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

#[derive(Debug)]
pub enum MidiMessageError {
    BadStatusByte(u8),
    UnexpectedEof,
    MalformedPacket,
}

pub struct MidiMessageStream<'a> {
    data: &'a [u8],
}

impl<'a> MidiMessageStream<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        MidiMessageStream { data }
    }

    pub fn try_read_u8(&mut self) -> Option<u8> {
        if self.data.is_empty() {
            None
        } else {
            let byte = self.data[0];
            self.data = &self.data[1..];
            Some(byte)
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, MidiMessageError> {
        self.try_read_u8().ok_or(MidiMessageError::UnexpectedEof)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MidiEvent {
    NoteOn { note: Note, velocity: u8 },
    NoteOff { note: Note, velocity: u8 },
    ControlChange { control: ControlChannel, value: u8 },
    TimingClock,
}
impl MidiEvent {
    pub fn from_bytes(bytes: &[u8]) -> Result<Option<MidiEvent>, MidiMessageError> {
        MidiEvent::read_next(&mut MidiMessageStream::new(bytes))
    }

    pub fn read_next(
        stream: &mut MidiMessageStream,
    ) -> Result<Option<MidiEvent>, MidiMessageError> {
        let status = match stream.try_read_u8() {
            Some(status) => status,
            None => return Ok(None),
        };

        let status = Status::from_u8(status).ok_or(MidiMessageError::BadStatusByte(status))?;

        match status {
            Status::NoteOn => Ok(Some(MidiEvent::NoteOn {
                note: Note::new(stream.read_u8()?),
                velocity: stream.read_u8()?,
            })),
            Status::NoteOff => Ok(Some(MidiEvent::NoteOff {
                note: Note::new(stream.read_u8()?),
                velocity: stream.read_u8()?,
            })),
            Status::ControlChange => Ok(Some(MidiEvent::ControlChange {
                control: ControlChannel::new(stream.read_u8()?),
                value: stream.read_u8()?,
            })),
            Status::TimingClock => Ok(Some(MidiEvent::TimingClock)),
            _ => Err(MidiMessageError::MalformedPacket),
        }
    }
}
