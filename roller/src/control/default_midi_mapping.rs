use midi::{ControlChannel, Note};

use crate::{
    clock::{Beats, ClockOffset, ClockOffsetMode, Rate},
    color::Color,
    control::{
        button::{
            ButtonAction, ButtonGroup, ButtonMapping, ButtonType, MetaButtonAction,
            MetaButtonMapping,
        },
        fader::{FaderCurve, FaderType, MidiFaderMapping},
        midi::MidiMapping,
    },
    effect::{
        ColorEffect, ColorModulation, ColorModulator, DimmerEffect, DimmerModulator,
        EffectDirection, PixelEffect, PixelModulator, PositionEffect, PositionModulator, Waveform,
    },
    lighting_engine::SceneId,
    position::BasePositionMode,
    project::FixtureGroupId,
};

pub fn default_midi_mapping() -> MidiMapping {
    MidiMapping::new(
        vec![
            MidiFaderMapping {
                control_channel: ControlChannel::new(48),
                fader_type: FaderType::GroupDimmer(FixtureGroupId::new(1)),
                fader_curve: FaderCurve::root(0.8),
            },
            MidiFaderMapping {
                control_channel: ControlChannel::new(49),
                fader_type: FaderType::GroupDimmer(FixtureGroupId::new(2)),
                fader_curve: FaderCurve::root(0.8),
            },
            MidiFaderMapping {
                control_channel: ControlChannel::new(50),
                fader_type: FaderType::GroupDimmer(FixtureGroupId::new(3)),
                fader_curve: FaderCurve::root(0.8),
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
                fader_curve: FaderCurve::root(0.8),
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
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::DeepOrange),
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
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::DarkBlue),
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
                    DimmerModulator::new(Waveform::TriangleDown, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                note: Note::new(55),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::SineDown, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                note: Note::new(47),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::HalfSineUp, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                note: Note::new(39),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::SawUp, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                note: Note::new(31),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::HalfSineDown, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                note: Note::new(23),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::SawDown, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                note: Note::new(15),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::ShortSquarePulse, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            // Dimmer sequences
            ButtonMapping {
                note: Note::new(62),
                on_action: ButtonAction::ActivateDimmerEffect(DimmerEffect::new(
                    vec![
                        DimmerModulator::new(Waveform::ShortSquarePulse, Beats::new(1.0), 1.0),
                        DimmerModulator::new(Waveform::SineUp, Beats::new(1.0), (0.0, 0.7)),
                        DimmerModulator::new(Waveform::ShortSquarePulse, Beats::new(1.0), 1.0),
                        DimmerModulator::new(Waveform::Off, Beats::new(0.5), 1.0),
                        DimmerModulator::new(Waveform::SawUp, Beats::new(0.5), (0.0, 0.2)),
                    ],
                    Some(ClockOffset::new(
                        ClockOffsetMode::Location(EffectDirection::BottomToTop),
                        Beats::new(1.0),
                    )),
                )),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                note: Note::new(54),
                on_action: ButtonAction::ActivateDimmerEffect(DimmerEffect::new(
                    vec![
                        DimmerModulator::new(Waveform::ShortSquarePulse, Beats::new(1.0), 1.0),
                        DimmerModulator::new(Waveform::SineUp, Beats::new(1.0), (0.0, 0.7)),
                    ],
                    Some(ClockOffset::new(
                        ClockOffsetMode::Location(EffectDirection::BottomToTop),
                        Beats::new(1.0),
                    )),
                )),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                note: Note::new(46),
                on_action: ButtonAction::ActivateDimmerEffect(DimmerEffect::new(
                    vec![
                        DimmerModulator::new(Waveform::HalfSineDown, Beats::new(1.0), 1.0),
                        DimmerModulator::new(Waveform::HalfSineUp, Beats::new(1.0), (0.0, 0.8)),
                        DimmerModulator::new(Waveform::HalfSineUp, Beats::new(1.0), (0.0, 0.9)),
                        DimmerModulator::new(Waveform::HalfSineUp, Beats::new(1.0), 1.0),
                    ],
                    None,
                )),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                note: Note::new(38),
                on_action: ButtonAction::ActivateDimmerEffect(DimmerEffect::new(
                    vec![DimmerModulator::new(
                        Waveform::SawDown,
                        Beats::new(4.0),
                        1.0,
                    )],
                    Some(ClockOffset::new(
                        ClockOffsetMode::Location(EffectDirection::LeftToRight),
                        Beats::new(1.0),
                    )),
                )),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                note: Note::new(30),
                on_action: ButtonAction::ActivateDimmerEffect(DimmerEffect::new(
                    vec![DimmerModulator::new(Waveform::SineUp, Beats::new(4.0), 1.0)],
                    Some(ClockOffset::new(
                        ClockOffsetMode::Location(EffectDirection::FromCenter),
                        Beats::new(1.0),
                    )),
                )),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                note: Note::new(22),
                on_action: ButtonAction::ActivateDimmerEffect(DimmerEffect::new(
                    vec![
                        DimmerModulator::new(Waveform::SawDown, Beats::new(2.0), 1.0),
                        DimmerModulator::new(Waveform::Off, Beats::new(1.0), 1.0),
                        DimmerModulator::new(Waveform::Off, Beats::new(1.0), 1.0),
                        DimmerModulator::new(Waveform::Off, Beats::new(1.0), 1.0),
                        DimmerModulator::new(Waveform::SawDown, Beats::new(2.0), 1.0),
                        DimmerModulator::new(Waveform::Off, Beats::new(1.0), 1.0),
                        DimmerModulator::new(Waveform::Off, Beats::new(1.0), 1.0),
                    ],
                    Some(ClockOffset::new(
                        ClockOffsetMode::FixtureIndex,
                        Beats::new(2.0),
                    )),
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
            ButtonMapping {
                note: Note::new(18),
                on_action: ButtonAction::ActivateColorEffect(ColorEffect::new(
                    vec![
                        ColorModulator::new(
                            ColorModulation::HueShift(180.0.into()),
                            Waveform::HalfRootUp,
                            Beats::new(1.0),
                        ),
                        ColorModulator::new(
                            ColorModulation::HueShift(180.0.into()),
                            Waveform::HalfRootDown,
                            Beats::new(0.5),
                        ),
                        ColorModulator::new(
                            ColorModulation::HueShift(180.0.into()),
                            Waveform::HalfRootUp,
                            Beats::new(1.5),
                        ),
                        ColorModulator::new(
                            ColorModulation::HueShift(180.0.into()),
                            Waveform::HalfRootDown,
                            Beats::new(1.0),
                        ),
                    ],
                    Some(ClockOffset::new(ClockOffsetMode::Random, Beats::new(2.0))),
                )),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                note: Note::new(10),
                on_action: ButtonAction::ActivateColorEffect(ColorEffect::new(
                    vec![ColorModulator::new(
                        ColorModulation::ToSecondaryColor,
                        Waveform::SineUp,
                        Beats::new(2.0),
                    )],
                    None,
                )),
            }
            .into_group(ButtonType::Toggle),
            // Pixel effects
            ButtonGroup::new(
                ButtonType::Toggle,
                vec![
                    ButtonMapping {
                        note: Note::new(60),
                        on_action: ButtonAction::ActivatePixelEffect(PixelEffect::new(
                            vec![
                                PixelModulator::new(
                                    Waveform::SawDown,
                                    Beats::new(1.0),
                                    EffectDirection::ToCenter,
                                ),
                                PixelModulator::new(
                                    Waveform::SawDown,
                                    Beats::new(1.0),
                                    EffectDirection::ToCenter,
                                ),
                                PixelModulator::new(
                                    Waveform::SawDown,
                                    Beats::new(1.0),
                                    EffectDirection::ToCenter,
                                ),
                                PixelModulator::new(
                                    Waveform::HalfRootUp,
                                    Beats::new(1.0),
                                    EffectDirection::ToCenter,
                                ),
                            ],
                            None,
                        )),
                    },
                    ButtonMapping {
                        note: Note::new(52),
                        on_action: ButtonAction::ActivatePixelEffect(
                            PixelModulator::new(
                                Waveform::SineDown,
                                Beats::new(2.0),
                                EffectDirection::FromCenter,
                            )
                            .into(),
                        ),
                    },
                    ButtonMapping {
                        note: Note::new(44),
                        on_action: ButtonAction::ActivatePixelEffect(PixelEffect::new(
                            vec![PixelModulator::new(
                                Waveform::SigmoidWaveDown,
                                Beats::new(2.0),
                                EffectDirection::BottomToTop,
                            )],
                            Some(ClockOffset::new(
                                ClockOffsetMode::FixtureIndex,
                                Beats::new(1.0),
                            )),
                        )),
                    },
                    ButtonMapping {
                        note: Note::new(36),
                        on_action: ButtonAction::ActivatePixelEffect(
                            PixelModulator::new(
                                Waveform::OnePointFiveRootDown,
                                Beats::new(1.0),
                                EffectDirection::FromCenter,
                            )
                            .into(),
                        ),
                    },
                    ButtonMapping {
                        note: Note::new(28),
                        on_action: ButtonAction::ActivatePixelEffect(PixelEffect::new(
                            vec![
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootDown,
                                    Beats::new(1.0),
                                    EffectDirection::BottomToTop,
                                ),
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootUp,
                                    Beats::new(1.0),
                                    EffectDirection::BottomToTop,
                                ),
                            ],
                            Some(ClockOffset::new(
                                ClockOffsetMode::FixtureIndex,
                                Beats::new(1.0),
                            )),
                        )),
                    },
                    ButtonMapping {
                        note: Note::new(20),
                        on_action: ButtonAction::ActivatePixelEffect(PixelEffect::new(
                            vec![
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootUp,
                                    Beats::new(1.0),
                                    EffectDirection::BottomToTop,
                                ),
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootUp,
                                    Beats::new(1.0),
                                    EffectDirection::BottomToTop,
                                ),
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootDown,
                                    Beats::new(1.0),
                                    EffectDirection::BottomToTop,
                                ),
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootDown,
                                    Beats::new(1.0),
                                    EffectDirection::BottomToTop,
                                ),
                            ],
                            Some(ClockOffset::new(
                                ClockOffsetMode::FixtureIndex,
                                Beats::new(2.0),
                            )),
                        )),
                    },
                    ButtonMapping {
                        note: Note::new(12),
                        on_action: ButtonAction::ActivatePixelEffect(PixelEffect::new(
                            vec![PixelModulator::new(
                                Waveform::SigmoidWaveDown,
                                Beats::new(2.0),
                                EffectDirection::BottomToTop,
                            )],
                            Some(ClockOffset::new(
                                ClockOffsetMode::FixtureIndex,
                                Beats::new(0.667),
                            )),
                        )),
                    },
                    ButtonMapping {
                        note: Note::new(4),
                        on_action: ButtonAction::ActivatePixelEffect(PixelEffect::new(
                            vec![
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootUp,
                                    Beats::new(0.5),
                                    EffectDirection::BottomToTop,
                                ),
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootUp,
                                    Beats::new(0.5),
                                    EffectDirection::BottomToTop,
                                ),
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootUp,
                                    Beats::new(0.5),
                                    EffectDirection::BottomToTop,
                                ),
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootUp,
                                    Beats::new(0.5),
                                    EffectDirection::BottomToTop,
                                ),
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootDown,
                                    Beats::new(0.5),
                                    EffectDirection::BottomToTop,
                                ),
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootDown,
                                    Beats::new(0.5),
                                    EffectDirection::BottomToTop,
                                ),
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootDown,
                                    Beats::new(0.5),
                                    EffectDirection::BottomToTop,
                                ),
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootDown,
                                    Beats::new(0.5),
                                    EffectDirection::BottomToTop,
                                ),
                            ],
                            Some(ClockOffset::new(
                                ClockOffsetMode::FixtureIndex,
                                Beats::new(0.25),
                            )),
                        )),
                    },
                ],
            ),
            // Positions
            ButtonGroup::new(
                ButtonType::Switch,
                vec![
                    ButtonMapping {
                        note: Note::new(59),
                        on_action: ButtonAction::UpdateBasePosition((0.0, 0.0).into()),
                    },
                    ButtonMapping {
                        note: Note::new(51),
                        on_action: ButtonAction::UpdateBasePosition(
                            ((-15.0, -30.0), BasePositionMode::MirrorPan).into(),
                        ),
                    },
                    ButtonMapping {
                        note: Note::new(43),
                        on_action: ButtonAction::UpdateBasePosition(
                            ((30.0, -30.0), BasePositionMode::MirrorPan).into(),
                        ),
                    },
                    ButtonMapping {
                        note: Note::new(35),
                        on_action: ButtonAction::UpdateBasePosition(
                            ((50.0, -75.0), BasePositionMode::MirrorPan).into(),
                        ),
                    },
                ],
            ),
            ButtonGroup::new(
                ButtonType::Toggle,
                vec![
                    ButtonMapping {
                        note: Note::new(3),
                        on_action: ButtonAction::ActivatePositionEffect(PositionEffect::new(
                            Some(PositionModulator::new(
                                Waveform::TriangleDown,
                                Beats::new(8.0),
                                240.0,
                            )),
                            Some(PositionModulator::new(
                                Waveform::SigmoidWaveUp,
                                Beats::new(4.0),
                                50.0,
                            )),
                            Some(ClockOffset::new(
                                ClockOffsetMode::FixtureIndex,
                                Beats::new(4.0),
                            )),
                        )),
                    },
                    ButtonMapping {
                        note: Note::new(11),
                        on_action: ButtonAction::ActivatePositionEffect(PositionEffect::new(
                            Some(PositionModulator::new(
                                Waveform::TriangleDown,
                                Beats::new(8.0),
                                180.0,
                            )),
                            Some(PositionModulator::new(
                                Waveform::TriangleDown,
                                Beats::new(4.0),
                                180.0,
                            )),
                            Some(ClockOffset::new(
                                ClockOffsetMode::FixtureIndex,
                                Beats::new(2.0),
                            )),
                        )),
                    },
                    ButtonMapping {
                        note: Note::new(19),
                        on_action: ButtonAction::ActivatePositionEffect(PositionEffect::new(
                            Some(PositionModulator::new(
                                Waveform::TriangleDown,
                                Beats::new(8.0),
                                270.0,
                            )),
                            Some(PositionModulator::new(
                                Waveform::TriangleDown,
                                Beats::new(16.0),
                                180.0,
                            )),
                            Some(ClockOffset::new(
                                ClockOffsetMode::FixtureIndex,
                                Beats::new(8.0),
                            )),
                        )),
                    },
                ],
            ),
        ],
        vec![
            MetaButtonMapping {
                note: Note::new(98),
                on_action: MetaButtonAction::TapTempo,
            },
            MetaButtonMapping {
                note: Note::new(64),
                on_action: MetaButtonAction::ToggleFixtureGroupControl(FixtureGroupId::new(1)),
            },
            MetaButtonMapping {
                note: Note::new(65),
                on_action: MetaButtonAction::ToggleFixtureGroupControl(FixtureGroupId::new(2)),
            },
            MetaButtonMapping {
                note: Note::new(66),
                on_action: MetaButtonAction::ToggleFixtureGroupControl(FixtureGroupId::new(3)),
            },
            MetaButtonMapping {
                note: Note::new(68),
                on_action: MetaButtonAction::ActivateScene(SceneId::new(1)),
            },
            MetaButtonMapping {
                note: Note::new(69),
                on_action: MetaButtonAction::ActivateScene(SceneId::new(2)),
            },
            MetaButtonMapping {
                note: Note::new(70),
                on_action: MetaButtonAction::ActivateScene(SceneId::new(3)),
            },
            MetaButtonMapping {
                note: Note::new(71),
                on_action: MetaButtonAction::ActivateScene(SceneId::new(4)),
            },
            MetaButtonMapping {
                note: Note::new(82),
                on_action: MetaButtonAction::UpdateClockRate(Rate::new(1.0 / 3.0)),
            },
            MetaButtonMapping {
                note: Note::new(83),
                on_action: MetaButtonAction::UpdateClockRate(Rate::new(1.0 / 2.0)),
            },
            MetaButtonMapping {
                note: Note::new(84),
                on_action: MetaButtonAction::UpdateClockRate(Rate::new(1.0)),
            },
            MetaButtonMapping {
                note: Note::new(85),
                on_action: MetaButtonAction::UpdateClockRate(Rate::new(2.0)),
            },
            MetaButtonMapping {
                note: Note::new(86),
                on_action: MetaButtonAction::UpdateClockRate(Rate::new(3.0)),
            },
        ],
    )
}
