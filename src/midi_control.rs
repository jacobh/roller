use async_std::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum LightingEvent {
    UpdateMasterDimmer { dimmer: f64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiControl {
    MasterDimmer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiControlMapping {
    control_channel: u8,
    midi_control: MidiControl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiMapping {
    controls: HashMap<u8, MidiControlMapping>,
}
impl MidiMapping {
    fn new(controls: Vec<MidiControlMapping>) -> MidiMapping {
        MidiMapping {
            controls: controls
                .into_iter()
                .map(|mapping| (mapping.control_channel, mapping))
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
                },
                None => Err("unknown control channel"),
            },
            _ => Err("unknown midi event type"),
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

        let (input_sender, input_receiver) = async_std::sync::channel::<MidiEvent>(1024);
        let midi_input_port = midi_client
            .input_port(&format!("roller-input-{}", name), move |packet_list| {
                for packet in packet_list.iter() {
                    let midi_message = rimd::MidiMessage::from_bytes(packet.data().to_vec());
                    let midi_event = MidiEvent::from(midi_message);
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
            _input_port: midi_input_port,
            _output_port: midi_output_port,
            midi_mapping: MidiMapping::new(vec![MidiControlMapping {
                control_channel: 56,
                midi_control: MidiControl::MasterDimmer,
            }]),
            input_receiver: input_receiver,
        })
    }
    pub fn midi_events(&self) -> impl Stream<Item = MidiEvent> {
        self.input_receiver.clone()
    }
    pub fn lighting_events(&self) -> impl Stream<Item = LightingEvent> {
        let mapping = self.midi_mapping.clone();
        self.input_receiver
            .clone()
            .map(move |midi_event| mapping.try_midi_to_lighting_event(&midi_event).ok())
            .filter(|lighting_event| lighting_event.is_some())
            .map(|lighting_event| lighting_event.unwrap())
    }
}
