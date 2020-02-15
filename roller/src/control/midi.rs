use async_std::{prelude::*, sync::Arc};
use midi::{ControlChannel, MidiEvent, Note};
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};

use crate::{
    clock::{Beats, ClockOffset, ClockOffsetMode},
    color::Color,
    control::{
        button::{
            AkaiPadState, ButtonAction, ButtonGroupId, ButtonMapping, ButtonType, MetaButtonAction,
            MetaButtonMapping,
        },
        fader::{FaderType, MidiFaderMapping},
    },
    effect::{
        ColorEffect, ColorModulation, ColorModulator, DimmerEffect, DimmerModulator, Waveform,
    },
    lighting_engine::LightingEvent,
    project::FixtureGroupId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteState {
    On,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiMapping {
    faders: FxHashMap<ControlChannel, MidiFaderMapping>,
    pub buttons: FxHashMap<Note, ButtonMapping>,
    pub meta_buttons: FxHashMap<Note, MetaButtonMapping>,
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
    fn midi_to_lighting_event(&self, midi_event: &MidiEvent) -> Option<LightingEvent> {
        let now = Instant::now();

        match dbg!(midi_event) {
            MidiEvent::ControlChange { control, value } => self.faders.get(control).map(|fader| {
                fader
                    .fader_type
                    .lighting_event(1.0 / 127.0 * (*value as f64))
            }),
            MidiEvent::NoteOn { note, .. } => self
                .buttons
                .get(note)
                .map(|button| button.clone().into_lighting_event(NoteState::On, now))
                .or(self
                    .meta_buttons
                    .get(note)
                    .map(|meta_button| meta_button.lighting_event(now))),
            MidiEvent::NoteOff { note, .. } => self
                .buttons
                .get(note)
                .map(|button| button.clone().into_lighting_event(NoteState::Off, now)),
            MidiEvent::Other(_) => None,
        }
    }
    pub fn initial_pad_states(&self) -> FxHashMap<Note, AkaiPadState> {
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

    pub midi_mapping: Arc<MidiMapping>,
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
            midi_mapping: Arc::new(MidiMapping::new(
                vec![
                    MidiFaderMapping {
                        control_channel: ControlChannel::new(48),
                        fader_type: FaderType::GroupDimmer(FixtureGroupId::new(1)),
                    },
                    MidiFaderMapping {
                        control_channel: ControlChannel::new(49),
                        fader_type: FaderType::GroupDimmer(FixtureGroupId::new(2)),
                    },
                    MidiFaderMapping {
                        control_channel: ControlChannel::new(55),
                        fader_type: FaderType::GlobalEffectIntensity,
                    },
                    MidiFaderMapping {
                        control_channel: ControlChannel::new(56),
                        fader_type: FaderType::MasterDimmer,
                    },
                ],
                vec![
                    // Colours
                    ButtonMapping {
                        note: Note::new(56),
                        button_type: ButtonType::Switch,
                        group_id: Some(ButtonGroupId::new(1)),
                        on_action: ButtonAction::UpdateGlobalColor(Color::White),
                    },
                    ButtonMapping {
                        note: Note::new(48),
                        button_type: ButtonType::Switch,
                        group_id: Some(ButtonGroupId::new(1)),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Yellow),
                    },
                    ButtonMapping {
                        note: Note::new(40),
                        button_type: ButtonType::Switch,
                        group_id: Some(ButtonGroupId::new(1)),
                        on_action: ButtonAction::UpdateGlobalColor(Color::DeepOrange),
                    },
                    ButtonMapping {
                        note: Note::new(32),
                        button_type: ButtonType::Switch,
                        group_id: Some(ButtonGroupId::new(1)),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Red),
                    },
                    ButtonMapping {
                        note: Note::new(24),
                        button_type: ButtonType::Switch,
                        group_id: Some(ButtonGroupId::new(1)),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Violet),
                    },
                    ButtonMapping {
                        note: Note::new(16),
                        button_type: ButtonType::Switch,
                        group_id: Some(ButtonGroupId::new(1)),
                        on_action: ButtonAction::UpdateGlobalColor(Color::DarkBlue),
                    },
                    ButtonMapping {
                        note: Note::new(8),
                        button_type: ButtonType::Switch,
                        group_id: Some(ButtonGroupId::new(1)),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Teal),
                    },
                    ButtonMapping {
                        note: Note::new(0),
                        button_type: ButtonType::Switch,
                        group_id: Some(ButtonGroupId::new(1)),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Green),
                    },
                    // Dimmer Effects
                    ButtonMapping {
                        note: Note::new(63),
                        button_type: ButtonType::Toggle,
                        group_id: None,
                        on_action: ButtonAction::ActivateDimmerEffect(
                            DimmerModulator::new(Waveform::TriangleDown, Beats::new(1.0), 1.0)
                                .into(),
                        ),
                    },
                    ButtonMapping {
                        note: Note::new(55),
                        button_type: ButtonType::Toggle,
                        group_id: None,
                        on_action: ButtonAction::ActivateDimmerEffect(
                            DimmerModulator::new(Waveform::HalfSineUp, Beats::new(0.5), 1.0).into(),
                        ),
                    },
                    ButtonMapping {
                        note: Note::new(47),
                        button_type: ButtonType::Flash,
                        group_id: None,
                        on_action: ButtonAction::ActivateDimmerEffect(
                            DimmerModulator::new(Waveform::ShortSquarePulse, Beats::new(0.5), 1.0)
                                .into(),
                        ),
                    },
                    // Dimmer sequences
                    ButtonMapping {
                        note: Note::new(61),
                        button_type: ButtonType::Toggle,
                        group_id: None,
                        on_action: ButtonAction::ActivateDimmerEffect(DimmerEffect::new(
                            vec![
                                DimmerModulator::new(
                                    Waveform::ShortSquarePulse,
                                    Beats::new(1.0),
                                    1.0,
                                ),
                                DimmerModulator::new(Waveform::SineUp, Beats::new(1.0), (0.0, 0.7)),
                                DimmerModulator::new(
                                    Waveform::ShortSquarePulse,
                                    Beats::new(1.0),
                                    1.0,
                                ),
                                DimmerModulator::new(Waveform::Off, Beats::new(0.5), 1.0),
                                DimmerModulator::new(Waveform::SawUp, Beats::new(0.5), (0.0, 0.2)),
                            ],
                            Some(ClockOffset::new(ClockOffsetMode::GroupId, Beats::new(1.0))),
                        )),
                    },
                    // Color effects
                    ButtonMapping {
                        note: Note::new(58),
                        button_type: ButtonType::Toggle,
                        group_id: None,
                        on_action: ButtonAction::ActivateColorEffect(
                            ColorModulator::new(
                                ColorModulation::HueShift(120.0.into()),
                                Waveform::HalfSineUp,
                                Beats::new(2.0),
                                Some(ClockOffset::new(ClockOffsetMode::GroupId, Beats::new(1.0))),
                            )
                            .into(),
                        ),
                    },
                    ButtonMapping {
                        note: Note::new(50),
                        button_type: ButtonType::Toggle,
                        group_id: None,
                        on_action: ButtonAction::ActivateColorEffect(
                            ColorModulator::new(
                                ColorModulation::HueShift((-90.0).into()),
                                Waveform::HalfSineDown,
                                Beats::new(1.0),
                                Some(ClockOffset::new(ClockOffsetMode::Random, Beats::new(0.5))),
                            )
                            .into(),
                        ),
                    },
                    // Color sequences
                    ButtonMapping {
                        note: Note::new(34),
                        button_type: ButtonType::Toggle,
                        group_id: None,
                        on_action: ButtonAction::ActivateColorEffect(ColorEffect::new(
                            vec![
                                (ColorModulation::NoOp, Beats::new(1.0)).into(),
                                (ColorModulation::HueShift(30.0.into()), Beats::new(1.0)).into(),
                                (ColorModulation::HueShift((45.0).into()), Beats::new(1.0)).into(),
                                (ColorModulation::NoOp, Beats::new(1.0)).into(),
                                (ColorModulation::HueShift(30.0.into()), Beats::new(1.0)).into(),
                                (ColorModulation::HueShift(60.0.into()), Beats::new(1.0)).into(),
                            ],
                            Some(ClockOffset::new(ClockOffsetMode::GroupId, Beats::new(4.0))),
                        )),
                    },
                    ButtonMapping {
                        note: Note::new(26),
                        button_type: ButtonType::Toggle,
                        group_id: None,
                        on_action: ButtonAction::ActivateColorEffect(ColorEffect::new(
                            vec![
                                ColorModulator::new(
                                    ColorModulation::White,
                                    Waveform::ShortSquarePulse,
                                    Beats::new(1.0),
                                    None,
                                )
                                .into(),
                                (ColorModulation::NoOp, Beats::new(3.0)).into(),
                            ],
                            Some(ClockOffset::new(ClockOffsetMode::Random, Beats::new(0.5))),
                        )),
                    },
                ],
                vec![MetaButtonMapping {
                    note: Note::new(98),
                    on_action: MetaButtonAction::TapTempo,
                }],
            )),
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
            .map(move |midi_event| mapping.midi_to_lighting_event(&midi_event))
            .filter(|lighting_event| lighting_event.is_some())
            .map(|lighting_event| lighting_event.unwrap())
    }
    async fn send_packet(&self, packet: impl Into<Vec<u8>>) {
        self.output_sender.send(packet.into()).await
    }
    pub async fn set_pad_color(&self, note: Note, pad_color: AkaiPadState) {
        self.send_packet(vec![0x90, u8::from(note), pad_color.as_byte()])
            .await
    }
    pub async fn reset_pads(&self) {
        for i in 0..64 {
            self.set_pad_color(Note::new(i), AkaiPadState::Off).await;
        }
    }
}
