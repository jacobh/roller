use async_std::prelude::*;
use rustc_hash::FxHashMap;
use std::time::Instant;

use crate::color::Color;

#[derive(Debug, Clone, PartialEq)]
pub enum LightingEvent {
    UpdateMasterDimmer { dimmer: f64 },
    UpdateGroupDimmer { group_id: usize, dimmer: f64 },
    UpdateGlobalColor { color: Color },
    UpdateGlobalEffectIntensity(f64),
    TapTempo(Instant),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiControl {
    MasterDimmer,
    GroupDimmer { group_id: usize },
    GlobalEffectIntensity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiControlMapping {
    control_channel: u8,
    midi_control: MidiControl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiNoteAction {
    UpdateGlobalColor { color: Color },
    TapTempo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiNoteMapping {
    note: u8,
    on_action: MidiNoteAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiMapping {
    controls: FxHashMap<u8, MidiControlMapping>,
    notes: FxHashMap<u8, MidiNoteMapping>,
}
impl MidiMapping {
    fn new(controls: Vec<MidiControlMapping>, notes: Vec<MidiNoteMapping>) -> MidiMapping {
        MidiMapping {
            controls: controls
                .into_iter()
                .map(|mapping| (mapping.control_channel, mapping))
                .collect(),
            notes: notes
                .into_iter()
                .map(|mapping| (mapping.note, mapping))
                .collect(),
        }
    }
    fn try_midi_to_lighting_event(
        &self,
        midi_event: &MidiEvent,
    ) -> Result<LightingEvent, &'static str> {
        match midi_event {
            MidiEvent::ControlChange { control, value } => match self.controls.get(control) {
                Some(midi_control_mapping) => match midi_control_mapping.midi_control {
                    MidiControl::MasterDimmer => Ok(LightingEvent::UpdateMasterDimmer {
                        dimmer: 1.0 / 127.0 * (*value as f64),
                    }),
                    MidiControl::GroupDimmer { group_id } => Ok(LightingEvent::UpdateGroupDimmer {
                        group_id,
                        dimmer: 1.0 / 127.0 * (*value as f64),
                    }),
                    MidiControl::GlobalEffectIntensity => Ok(
                        LightingEvent::UpdateGlobalEffectIntensity(1.0 / 127.0 * (*value as f64)),
                    ),
                },
                None => Err("unknown control channel"),
            },
            MidiEvent::NoteOn { note, .. } => match self.notes.get(note) {
                Some(midi_note_mapping) => match &midi_note_mapping.on_action {
                    action => match action {
                        MidiNoteAction::UpdateGlobalColor { color } => {
                            Ok(LightingEvent::UpdateGlobalColor { color: *color })
                        }
                        MidiNoteAction::TapTempo => Ok(LightingEvent::TapTempo(Instant::now())),
                    },
                },
                None => Err("No mapping for this note"),
            },
            MidiEvent::NoteOff { .. } => Err("Not yet implemented"),
            MidiEvent::Other(_) => Err("unknown midi event type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MidiEvent {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8, velocity: u8 },
    ControlChange { control: u8, value: u8 },
    Other(rimd::Status),
}
impl From<rimd::MidiMessage> for MidiEvent {
    fn from(message: rimd::MidiMessage) -> MidiEvent {
        match message.status() {
            rimd::Status::NoteOn => MidiEvent::NoteOn {
                note: message.data(1),
                velocity: message.data(2),
            },
            rimd::Status::NoteOff => MidiEvent::NoteOff {
                note: message.data(1),
                velocity: message.data(2),
            },
            rimd::Status::ControlChange => MidiEvent::ControlChange {
                control: message.data(1),
                value: message.data(2),
            },
            _ => MidiEvent::Other(message.status()),
        }
    }
}

pub struct MidiController {
    _client: coremidi::Client,
    _source: coremidi::Source,
    destination: coremidi::Destination,
    _input_port: coremidi::InputPort,
    _output_port: coremidi::OutputPort,

    midi_mapping: MidiMapping,
    input_receiver: async_std::sync::Receiver<MidiEvent>,
}
impl MidiController {
    pub fn new_for_device_name(name: &str) -> Result<MidiController, ()> {
        let midi_client = coremidi::Client::new(&format!("roller-{}", name)).unwrap();

        let source = coremidi::Sources
            .into_iter()
            .find(|source| source.display_name() == Some(name.to_owned()))
            .unwrap();
        let destination = coremidi::Destinations
            .into_iter()
            .find(|dest| dest.display_name() == Some(name.to_owned()))
            .unwrap();

        let (input_sender, input_receiver) = async_std::sync::channel::<MidiEvent>(1024);
        let midi_input_port = midi_client
            .input_port(&format!("roller-input-{}", name), move |packet_list| {
                for packet in packet_list.iter() {
                    let midi_message = rimd::MidiMessage::from_bytes(packet.data().to_vec());
                    let midi_event = dbg!(MidiEvent::from(midi_message));
                    async_std::task::block_on(input_sender.send(midi_event));
                }
            })
            .unwrap();
        midi_input_port.connect_source(&source).unwrap();

        let midi_output_port = midi_client
            .output_port(&format!("roller-input-{}", name))
            .unwrap();

        Ok(MidiController {
            _client: midi_client,
            _source: source,
            destination: destination,
            _input_port: midi_input_port,
            _output_port: midi_output_port,
            midi_mapping: MidiMapping::new(
                vec![
                    MidiControlMapping {
                        control_channel: 48,
                        midi_control: MidiControl::GroupDimmer { group_id: 1 },
                    },
                    MidiControlMapping {
                        control_channel: 49,
                        midi_control: MidiControl::GroupDimmer { group_id: 2 },
                    },
                    MidiControlMapping {
                        control_channel: 55,
                        midi_control: MidiControl::GlobalEffectIntensity,
                    },
                    MidiControlMapping {
                        control_channel: 56,
                        midi_control: MidiControl::MasterDimmer,
                    },
                ],
                vec![
                    // Misc
                    MidiNoteMapping {
                        note: 98,
                        on_action: MidiNoteAction::TapTempo,
                    },
                    // Colours
                    MidiNoteMapping {
                        note: 56,
                        on_action: MidiNoteAction::UpdateGlobalColor {
                            color: Color::White,
                        },
                    },
                    MidiNoteMapping {
                        note: 48,
                        on_action: MidiNoteAction::UpdateGlobalColor {
                            color: Color::Yellow,
                        },
                    },
                    MidiNoteMapping {
                        note: 40,
                        on_action: MidiNoteAction::UpdateGlobalColor {
                            color: Color::DeepOrange,
                        },
                    },
                    MidiNoteMapping {
                        note: 32,
                        on_action: MidiNoteAction::UpdateGlobalColor { color: Color::Red },
                    },
                    MidiNoteMapping {
                        note: 24,
                        on_action: MidiNoteAction::UpdateGlobalColor {
                            color: Color::Violet,
                        },
                    },
                    MidiNoteMapping {
                        note: 16,
                        on_action: MidiNoteAction::UpdateGlobalColor {
                            color: Color::DarkBlue,
                        },
                    },
                    MidiNoteMapping {
                        note: 8,
                        on_action: MidiNoteAction::UpdateGlobalColor { color: Color::Teal },
                    },
                    MidiNoteMapping {
                        note: 0,
                        on_action: MidiNoteAction::UpdateGlobalColor {
                            color: Color::Green,
                        },
                    },
                ],
            ),
            input_receiver: input_receiver,
        })
    }
    fn midi_events(&self) -> impl Stream<Item = MidiEvent> {
        self.input_receiver.clone()
    }
    pub fn lighting_events(&self) -> impl Stream<Item = LightingEvent> {
        let mapping = self.midi_mapping.clone();
        self.midi_events()
            .map(move |midi_event| mapping.try_midi_to_lighting_event(&midi_event).ok())
            .filter(|lighting_event| lighting_event.is_some())
            .map(|lighting_event| lighting_event.unwrap())
    }
    pub fn send_packets(&self, packets: &coremidi::PacketList) -> Result<(), &'static str> {
        self._output_port
            .send(&self.destination, packets)
            .map_err(|_| "failed to send packets")
    }
}
