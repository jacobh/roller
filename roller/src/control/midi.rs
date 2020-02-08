use async_std::prelude::*;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::time::{Duration, Instant};

use crate::control::midi_event::MidiEvent;
use crate::{
    clock::Beats,
    color::Color,
    control::{
        button::{ButtonAction, ButtonMapping, ButtonType, MetaButtonAction, MetaButtonMapping},
        fader::{FaderType, MidiFaderMapping},
    },
    effect::{DimmerEffect, Effect},
    lighting_engine::LightingEvent,
    project::GroupId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteState {
    On,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiMapping {
    faders: FxHashMap<u8, MidiFaderMapping>,
    pub buttons: FxHashMap<u8, ButtonMapping>,
    pub meta_buttons: FxHashMap<u8, MetaButtonMapping>,
}
impl MidiMapping {
    fn new(
        faders: Vec<MidiFaderMapping>,
        buttons: Vec<ButtonMapping>,
        meta_buttons: Vec<MetaButtonMapping>,
    ) -> MidiMapping {
        MidiMapping {
            faders: faders
                .into_iter()
                .map(|mapping| (mapping.control_channel, mapping))
                .collect(),
            buttons: buttons
                .into_iter()
                .map(|mapping| (mapping.note, mapping))
                .collect(),
            meta_buttons: meta_buttons
                .into_iter()
                .map(|mapping| (mapping.note, mapping))
                .collect(),
        }
    }
    fn try_midi_to_lighting_event(
        &self,
        midi_event: &MidiEvent,
    ) -> Result<LightingEvent, &'static str> {
        let now = Instant::now();

        match dbg!(midi_event) {
            MidiEvent::ControlChange { control, value } => match self.faders.get(control) {
                Some(midi_fader_mapping) => Ok(midi_fader_mapping
                    .fader_type
                    .lighting_event(1.0 / 127.0 * (*value as f64))),
                None => Err("unknown control channel"),
            },
            MidiEvent::NoteOn { note, .. } => match self.buttons.get(note) {
                Some(button_mapping) => Ok(button_mapping
                    .clone()
                    .into_lighting_event(NoteState::On, now)),
                None => match self.meta_buttons.get(note) {
                    Some(meta_button_mapping) => Ok(meta_button_mapping.lighting_event(now)),
                    None => Err("No mapping for this note"),
                },
            },
            MidiEvent::NoteOff { note, .. } => match self.buttons.get(note) {
                Some(button_mapping) => Ok(button_mapping
                    .clone()
                    .into_lighting_event(NoteState::Off, now)),
                None => Err("No mapping for this note"),
            },
            MidiEvent::Other(_) => Err("unknown midi event type"),
        }
    }
    pub fn initial_pad_states(&self) -> FxHashMap<u8, AkaiPadState> {
        self.buttons
            .keys()
            .map(|note| (*note, AkaiPadState::Yellow))
            .collect()
    }
}

pub struct MidiController {
    _client: coremidi::Client,
    _source: coremidi::Source,
    _input_port: coremidi::InputPort,

    pub midi_mapping: MidiMapping,
    input_receiver: async_std::sync::Receiver<MidiEvent>,
    output_sender: async_std::sync::Sender<Vec<u8>>,
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
                    // multiple messages may appear in the same packet
                    for message_data in packet.data().chunks_exact(3) {
                        let midi_event = MidiEvent::from_bytes(message_data);
                        async_std::task::block_on(input_sender.send(midi_event));
                    }
                }
            })
            .unwrap();
        midi_input_port.connect_source(&source).unwrap();

        let (output_sender, mut output_receiver) = async_std::sync::channel::<Vec<u8>>(512);

        let destination = coremidi::Destinations
            .into_iter()
            .find(|dest| dest.display_name() == Some(name.to_owned()))
            .unwrap();

        let midi_output_port = midi_client
            .output_port(&format!("roller-input-{}", name))
            .unwrap();

        async_std::task::spawn(async move {
            while let Some(packet) = output_receiver.next().await {
                let packets = coremidi::PacketBuffer::new(0, &packet);
                midi_output_port
                    .send(&destination, &packets)
                    .map_err(|_| "failed to send packets")
                    .unwrap();
                async_std::task::sleep(Duration::from_millis(1)).await;
            }
        });

        Ok(MidiController {
            _client: midi_client,
            _source: source,
            _input_port: midi_input_port,
            midi_mapping: MidiMapping::new(
                vec![
                    MidiFaderMapping {
                        control_channel: 48,
                        fader_type: FaderType::GroupDimmer(GroupId::new(1)),
                    },
                    MidiFaderMapping {
                        control_channel: 49,
                        fader_type: FaderType::GroupDimmer(GroupId::new(2)),
                    },
                    MidiFaderMapping {
                        control_channel: 55,
                        fader_type: FaderType::GlobalEffectIntensity,
                    },
                    MidiFaderMapping {
                        control_channel: 56,
                        fader_type: FaderType::MasterDimmer,
                    },
                ],
                vec![
                    // Colours
                    ButtonMapping {
                        note: 56,
                        button_type: ButtonType::Switch,
                        group_id: Some(1),
                        on_action: ButtonAction::UpdateGlobalColor {
                            color: Color::White,
                        },
                    },
                    ButtonMapping {
                        note: 48,
                        button_type: ButtonType::Switch,
                        group_id: Some(1),
                        on_action: ButtonAction::UpdateGlobalColor {
                            color: Color::Yellow,
                        },
                    },
                    ButtonMapping {
                        note: 40,
                        button_type: ButtonType::Switch,
                        group_id: Some(1),
                        on_action: ButtonAction::UpdateGlobalColor {
                            color: Color::DeepOrange,
                        },
                    },
                    ButtonMapping {
                        note: 32,
                        button_type: ButtonType::Switch,
                        group_id: Some(1),
                        on_action: ButtonAction::UpdateGlobalColor { color: Color::Red },
                    },
                    ButtonMapping {
                        note: 24,
                        button_type: ButtonType::Switch,
                        group_id: Some(1),
                        on_action: ButtonAction::UpdateGlobalColor {
                            color: Color::Violet,
                        },
                    },
                    ButtonMapping {
                        note: 16,
                        button_type: ButtonType::Switch,
                        group_id: Some(1),
                        on_action: ButtonAction::UpdateGlobalColor {
                            color: Color::DarkBlue,
                        },
                    },
                    ButtonMapping {
                        note: 8,
                        button_type: ButtonType::Switch,
                        group_id: Some(1),
                        on_action: ButtonAction::UpdateGlobalColor { color: Color::Teal },
                    },
                    ButtonMapping {
                        note: 0,
                        button_type: ButtonType::Switch,
                        group_id: Some(1),
                        on_action: ButtonAction::UpdateGlobalColor {
                            color: Color::Green,
                        },
                    },
                    // Dimmer Effects
                    ButtonMapping {
                        note: 63,
                        button_type: ButtonType::Toggle,
                        group_id: None,
                        on_action: ButtonAction::ActivateDimmerEffect(DimmerEffect::new(
                            Effect::TriangleDown,
                            Beats::new(1.0),
                            1.0,
                        )),
                    },
                    ButtonMapping {
                        note: 55,
                        button_type: ButtonType::Toggle,
                        group_id: None,
                        on_action: ButtonAction::ActivateDimmerEffect(DimmerEffect::new(
                            Effect::SawUp,
                            Beats::new(0.5),
                            1.0,
                        )),
                    },
                    ButtonMapping {
                        note: 47,
                        button_type: ButtonType::Flash,
                        group_id: None,
                        on_action: ButtonAction::ActivateDimmerEffect(DimmerEffect::new(
                            Effect::ShortSquarePulse,
                            Beats::new(0.5),
                            1.0,
                        )),
                    },
                ],
                vec![MetaButtonMapping {
                    note: 98,
                    on_action: MetaButtonAction::TapTempo,
                }],
            ),
            input_receiver: input_receiver,
            output_sender: output_sender,
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
    async fn send_packet(&self, packet: impl Into<Vec<u8>>) {
        self.output_sender.send(packet.into()).await
    }
    pub async fn set_pad_color(&self, note: u8, pad_color: AkaiPadState) {
        self.send_packet(vec![0x90, note, pad_color.as_byte()])
            .await
    }
    pub async fn reset_pads(&self) {
        for i in 0..64 {
            self.set_pad_color(i, AkaiPadState::Off).await;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum AkaiPadState {
    Off,
    Green,
    GreenBlink,
    Red,
    RedBlink,
    Yellow,
    YellowBlink,
}
impl AkaiPadState {
    pub fn as_byte(self) -> u8 {
        match self {
            AkaiPadState::Off => 0,
            AkaiPadState::Green => 1,
            AkaiPadState::GreenBlink => 2,
            AkaiPadState::Red => 3,
            AkaiPadState::RedBlink => 4,
            AkaiPadState::Yellow => 5,
            AkaiPadState::YellowBlink => 6,
        }
    }
}
