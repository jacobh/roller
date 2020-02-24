use async_std::{prelude::*, sync::Arc};
use midi::{ControlChannel, MidiEvent, Note};
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};

use crate::{
    clock::{Beats, ClockOffset, ClockOffsetMode, Rate},
    color::Color,
    control::{
        button::{
            AkaiPadState, ButtonAction, ButtonGroup, ButtonGroupId, ButtonMapping, ButtonType,
            MetaButtonAction, MetaButtonMapping, PadMapping,
        },
        fader::{FaderCurve, FaderType, MidiFaderMapping},
    },
    effect::{
        ColorEffect, ColorModulation, ColorModulator, DimmerEffect, DimmerModulator, Waveform,
    },
    lighting_engine::{LightingEvent, SceneId},
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
    pub button_groups: Vec<ButtonGroup>,
    pub meta_buttons: FxHashMap<Note, MetaButtonMapping>,
}
impl MidiMapping {
    fn new(
        faders: Vec<MidiFaderMapping>,
        button_groups: Vec<ButtonGroup>,
        meta_buttons: Vec<MetaButtonMapping>,
    ) -> MidiMapping {
        MidiMapping {
            faders: faders
                .into_iter()
                .map(|mapping| (mapping.control_channel, mapping))
                .collect(),
            button_groups,
            meta_buttons: meta_buttons
                .into_iter()
                .map(|mapping| (mapping.note, mapping))
                .collect(),
        }
    }
    fn group_buttons(&self) -> impl Iterator<Item = (&'_ ButtonGroup, &'_ ButtonMapping)> {
        self.button_groups
            .iter()
            .flat_map(|group| group.buttons().map(move |button| (group, button)))
    }
    fn midi_to_lighting_event(&self, midi_event: &MidiEvent) -> Option<LightingEvent> {
        let now = Instant::now();

        match dbg!(midi_event) {
            MidiEvent::ControlChange { control, value } => self
                .faders
                .get(control)
                .map(|fader| fader.lighting_event(1.0 / 127.0 * (*value as f64))),
            MidiEvent::NoteOn { note, .. } => self
                .group_buttons()
                .find(|(_, button)| button.note == *note)
                .map(|(group, button)| {
                    button
                        .clone()
                        .into_lighting_event(group.clone(), NoteState::On, now)
                })
                .or_else(|| {
                    self.meta_buttons
                        .get(note)
                        .map(|meta_button| meta_button.lighting_event(now))
                }),
            MidiEvent::NoteOff { note, .. } => self
                .group_buttons()
                .find(|(_, button)| button.note == *note)
                .map(|(group, button)| {
                    button
                        .clone()
                        .into_lighting_event(group.clone(), NoteState::Off, now)
                }),
            MidiEvent::Other(_) => None,
        }
    }
    pub fn pad_mappings(&self) -> impl Iterator<Item = PadMapping<'_>> {
        self.group_buttons()
            .map(PadMapping::from)
            .chain(self.meta_buttons.values().map(PadMapping::from))
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
                        fader_curve: FaderCurve::root(1.25),
                    },
                    MidiFaderMapping {
                        control_channel: ControlChannel::new(49),
                        fader_type: FaderType::GroupDimmer(FixtureGroupId::new(2)),
                        fader_curve: FaderCurve::root(1.25),
                    },
                    MidiFaderMapping {
                        control_channel: ControlChannel::new(54),
                        fader_type: FaderType::ColorEffectIntensity,
                        fader_curve: FaderCurve::linear(),
                    },
                    MidiFaderMapping {
                        control_channel: ControlChannel::new(55),
                        fader_type: FaderType::DimmerEffectIntensity,
                        fader_curve: FaderCurve::sigmoid(0.75),
                    },
                    MidiFaderMapping {
                        control_channel: ControlChannel::new(56),
                        fader_type: FaderType::MasterDimmer,
                        fader_curve: FaderCurve::root(1.25),
                    },
                ],
                vec![
                    // Colours
                    ButtonGroup::new(
                        ButtonType::Switch,
                        vec![
                            ButtonMapping {
                                note: Note::new(56),
                                on_action: ButtonAction::UpdateGlobalColor(Color::White),
                            },
                            ButtonMapping {
                                note: Note::new(48),
                                on_action: ButtonAction::UpdateGlobalColor(Color::Yellow),
                            },
                            ButtonMapping {
                                note: Note::new(40),
                                on_action: ButtonAction::UpdateGlobalColor(Color::DeepOrange),
                            },
                            ButtonMapping {
                                note: Note::new(32),
                                on_action: ButtonAction::UpdateGlobalColor(Color::Red),
                            },
                            ButtonMapping {
                                note: Note::new(24),
                                on_action: ButtonAction::UpdateGlobalColor(Color::Violet),
                            },
                            ButtonMapping {
                                note: Note::new(16),
                                on_action: ButtonAction::UpdateGlobalColor(Color::DarkBlue),
                            },
                            ButtonMapping {
                                note: Note::new(8),
                                on_action: ButtonAction::UpdateGlobalColor(Color::Teal),
                            },
                            ButtonMapping {
                                note: Note::new(0),
                                on_action: ButtonAction::UpdateGlobalColor(Color::Green),
                            },
                        ],
                    ),
                    // Secondary
                    ButtonGroup::new(
                        ButtonType::Toggle,
                        vec![
                            ButtonMapping {
                                note: Note::new(57),
                                on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::White),
                            },
                            ButtonMapping {
                                note: Note::new(49),
                                on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Yellow),
                            },
                            ButtonMapping {
                                note: Note::new(41),
                                on_action: ButtonAction::UpdateGlobalSecondaryColor(
                                    Color::DeepOrange,
                                ),
                            },
                            ButtonMapping {
                                note: Note::new(33),
                                on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Red),
                            },
                            ButtonMapping {
                                note: Note::new(25),
                                on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Violet),
                            },
                            ButtonMapping {
                                note: Note::new(17),
                                on_action: ButtonAction::UpdateGlobalSecondaryColor(
                                    Color::DarkBlue,
                                ),
                            },
                            ButtonMapping {
                                note: Note::new(9),
                                on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Teal),
                            },
                            ButtonMapping {
                                note: Note::new(1),
                                on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Green),
                            },
                        ],
                    ),
                    // Dimmer Effects
                    ButtonMapping {
                        note: Note::new(63),
                        on_action: ButtonAction::ActivateDimmerEffect(
                            DimmerModulator::new(Waveform::TriangleDown, Beats::new(1.0), 1.0)
                                .into(),
                        ),
                    }
                    .into_group(ButtonType::Toggle),
                    ButtonMapping {
                        note: Note::new(55),
                        on_action: ButtonAction::ActivateDimmerEffect(
                            DimmerModulator::new(Waveform::HalfSineUp, Beats::new(0.5), 1.0).into(),
                        ),
                    }
                    .into_group(ButtonType::Toggle),
                    ButtonMapping {
                        note: Note::new(47),
                        on_action: ButtonAction::ActivateDimmerEffect(
                            DimmerModulator::new(Waveform::ShortSquarePulse, Beats::new(0.5), 1.0)
                                .into(),
                        ),
                    }
                    .into_group(ButtonType::Flash),
                    ButtonMapping {
                        note: Note::new(39),
                        on_action: ButtonAction::ActivateDimmerEffect(
                            DimmerModulator::new(Waveform::ShortSquarePulse, Beats::new(1.0), 1.0)
                                .into(),
                        ),
                    }
                    .into_group(ButtonType::Toggle),
                    // Dimmer sequences
                    ButtonMapping {
                        note: Note::new(61),
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
                    }
                    .into_group(ButtonType::Toggle),
                    ButtonMapping {
                        note: Note::new(53),
                        on_action: ButtonAction::ActivateDimmerEffect(DimmerEffect::new(
                            vec![
                                DimmerModulator::new(
                                    Waveform::ShortSquarePulse,
                                    Beats::new(1.0),
                                    1.0,
                                ),
                                DimmerModulator::new(Waveform::SineUp, Beats::new(1.0), 1.0),
                            ],
                            Some(ClockOffset::new(ClockOffsetMode::GroupId, Beats::new(1.0))),
                        )),
                    }
                    .into_group(ButtonType::Toggle),
                    // Color effects
                    ButtonMapping {
                        note: Note::new(58),
                        on_action: ButtonAction::ActivateColorEffect(ColorEffect::new(
                            vec![ColorModulator::new(
                                ColorModulation::HueShift(120.0.into()),
                                Waveform::HalfSineUp,
                                Beats::new(2.0),
                            )],
                            Some(ClockOffset::new(ClockOffsetMode::GroupId, Beats::new(1.0))),
                        )),
                    }
                    .into_group(ButtonType::Toggle),
                    ButtonMapping {
                        note: Note::new(50),
                        on_action: ButtonAction::ActivateColorEffect(ColorEffect::new(
                            vec![ColorModulator::new(
                                ColorModulation::HueShift((-90.0).into()),
                                Waveform::HalfSineDown,
                                Beats::new(1.0),
                            )],
                            Some(ClockOffset::new(ClockOffsetMode::Random, Beats::new(0.5))),
                        )),
                    }
                    .into_group(ButtonType::Toggle),
                    // Color sequences
                    ButtonMapping {
                        note: Note::new(34),
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
                    }
                    .into_group(ButtonType::Toggle),
                    ButtonMapping {
                        note: Note::new(26),
                        on_action: ButtonAction::ActivateColorEffect(ColorEffect::new(
                            vec![
                                ColorModulator::new(
                                    ColorModulation::White,
                                    Waveform::ShortSquarePulse,
                                    Beats::new(1.0),
                                ),
                                (ColorModulation::NoOp, Beats::new(3.0)).into(),
                            ],
                            Some(ClockOffset::new(ClockOffsetMode::Random, Beats::new(0.5))),
                        )),
                    }
                    .into_group(ButtonType::Toggle),
                ],
                vec![
                    MetaButtonMapping {
                        note: Note::new(98),
                        on_action: MetaButtonAction::TapTempo,
                    },
                    MetaButtonMapping {
                        note: Note::new(64),
                        on_action: MetaButtonAction::ActivateScene(SceneId::new(1)),
                    },
                    MetaButtonMapping {
                        note: Note::new(65),
                        on_action: MetaButtonAction::ActivateScene(SceneId::new(2)),
                    },
                    MetaButtonMapping {
                        note: Note::new(66),
                        on_action: MetaButtonAction::ActivateScene(SceneId::new(3)),
                    },
                    MetaButtonMapping {
                        note: Note::new(67),
                        on_action: MetaButtonAction::ActivateScene(SceneId::new(4)),
                    },
                    MetaButtonMapping {
                        note: Note::new(82),
                        on_action: MetaButtonAction::UpdateClockRate(Rate::new(0.333_333)),
                    },
                    MetaButtonMapping {
                        note: Note::new(83),
                        on_action: MetaButtonAction::UpdateClockRate(Rate::new(0.5)),
                    },
                    MetaButtonMapping {
                        note: Note::new(84),
                        on_action: MetaButtonAction::UpdateClockRate(Rate::new(0.666_667)),
                    },
                    MetaButtonMapping {
                        note: Note::new(85),
                        on_action: MetaButtonAction::UpdateClockRate(Rate::new(0.75)),
                    },
                    MetaButtonMapping {
                        note: Note::new(86),
                        on_action: MetaButtonAction::UpdateClockRate(Rate::new(1.0)),
                    },
                    MetaButtonMapping {
                        note: Note::new(87),
                        on_action: MetaButtonAction::UpdateClockRate(Rate::new(1.5)),
                    },
                    MetaButtonMapping {
                        note: Note::new(88),
                        on_action: MetaButtonAction::UpdateClockRate(Rate::new(2.0)),
                    },
                    MetaButtonMapping {
                        note: Note::new(89),
                        on_action: MetaButtonAction::UpdateClockRate(Rate::new(3.0)),
                    },
                ],
            )),
            input_receiver,
            output_sender,
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
    pub async fn set_pad_colors(&self, pad_colors: impl IntoIterator<Item = (Note, AkaiPadState)>) {
        for (note, pad_color) in pad_colors {
            self.set_pad_color(note, pad_color).await
        }
    }
    pub async fn reset_pads(&self) {
        for i in 0..64 {
            self.set_pad_color(Note::new(i), AkaiPadState::Off).await;
        }
    }
    pub async fn run_pad_startup(&self) {
        for i in 0..64 {
            self.set_pad_color(Note::new(i), AkaiPadState::Green).await;
            async_std::task::sleep(Duration::from_millis(10)).await;
        }
        async_std::task::sleep(Duration::from_millis(150)).await;
        self.reset_pads().await;
    }
}
