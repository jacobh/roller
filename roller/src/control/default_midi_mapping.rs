use roller_protocol::{ButtonCoordinate, ButtonGridLocation, FaderId};

use crate::{
    clock::{Beats, ClockOffset, ClockOffsetMode, Rate},
    color::Color,
    control::{
        button::{
            ButtonAction, ButtonGroup, ButtonMapping, ButtonType, MetaButtonAction,
            MetaButtonMapping,
        },
        fader::{FaderControlMapping, FaderCurve, FaderType},
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
            FaderControlMapping {
                id: FaderId::new(0),
                fader_type: FaderType::GroupDimmer(FixtureGroupId::new(1)),
                fader_curve: FaderCurve::root(0.8),
            },
            FaderControlMapping {
                id: FaderId::new(1),
                fader_type: FaderType::GroupDimmer(FixtureGroupId::new(2)),
                fader_curve: FaderCurve::root(0.8),
            },
            FaderControlMapping {
                id: FaderId::new(2),
                fader_type: FaderType::GroupDimmer(FixtureGroupId::new(3)),
                fader_curve: FaderCurve::root(0.8),
            },
            FaderControlMapping {
                id: FaderId::new(6),
                fader_type: FaderType::ColorEffectIntensity,
                fader_curve: FaderCurve::linear(),
            },
            FaderControlMapping {
                id: FaderId::new(7),
                fader_type: FaderType::DimmerEffectIntensity,
                fader_curve: FaderCurve::sigmoid(0.75),
            },
            FaderControlMapping {
                id: FaderId::new(8),
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
                        coordinate: ButtonCoordinate::new(0, 7),
                        on_action: ButtonAction::UpdateGlobalColor(Color::White),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(0, 6),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Yellow),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(0, 5),
                        on_action: ButtonAction::UpdateGlobalColor(Color::DeepOrange),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(0, 4),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Red),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(0, 3),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Violet),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(0, 2),
                        on_action: ButtonAction::UpdateGlobalColor(Color::DarkBlue),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(0, 1),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Teal),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(0, 0),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Green),
                    },
                ],
            ),
            // Secondary
            ButtonGroup::new(
                ButtonType::Toggle,
                vec![
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(1, 7),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::White),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(1, 6),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Yellow),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(1, 5),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::DeepOrange),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(1, 4),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Red),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(1, 3),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Violet),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(1, 2),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::DarkBlue),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(1, 1),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Teal),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(1, 0),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Green),
                    },
                ],
            ),
            // Dimmer Effects
            ButtonMapping {
                coordinate: ButtonCoordinate::new(7, 7),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::SineDown, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                coordinate: ButtonCoordinate::new(7, 6),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::HalfSineUp, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                coordinate: ButtonCoordinate::new(7, 5),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::HalfSineDown, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                coordinate: ButtonCoordinate::new(7, 4),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::ShortSquarePulse, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            // Dimmer sequences
            ButtonMapping {
                coordinate: ButtonCoordinate::new(6, 7),
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
                coordinate: ButtonCoordinate::new(6, 6),
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
                coordinate: ButtonCoordinate::new(6, 5),
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
                coordinate: ButtonCoordinate::new(6, 4),
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
                coordinate: ButtonCoordinate::new(6, 3),
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
                coordinate: ButtonCoordinate::new(6, 2),
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
                coordinate: ButtonCoordinate::new(2, 7),
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
                coordinate: ButtonCoordinate::new(2, 6),
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
                coordinate: ButtonCoordinate::new(2, 4),
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
                coordinate: ButtonCoordinate::new(2, 3),
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
                coordinate: ButtonCoordinate::new(2, 2),
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
                coordinate: ButtonCoordinate::new(2, 1),
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
                        coordinate: ButtonCoordinate::new(4, 7),
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
                        coordinate: ButtonCoordinate::new(4, 6),
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
                        coordinate: ButtonCoordinate::new(4, 5),
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
                        coordinate: ButtonCoordinate::new(4, 4),
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
                        coordinate: ButtonCoordinate::new(4, 3),
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
                        coordinate: ButtonCoordinate::new(4, 2),
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
                        coordinate: ButtonCoordinate::new(3, 7),
                        on_action: ButtonAction::UpdateBasePosition((0.0, 0.0).into()),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(3, 6),
                        on_action: ButtonAction::UpdateBasePosition(
                            ((-15.0, -30.0), BasePositionMode::MirrorPan).into(),
                        ),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(3, 5),
                        on_action: ButtonAction::UpdateBasePosition(
                            ((30.0, -30.0), BasePositionMode::MirrorPan).into(),
                        ),
                    },
                    ButtonMapping {
                        coordinate: ButtonCoordinate::new(3, 4),
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
                        coordinate: ButtonCoordinate::new(3, 0),
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
                        coordinate: ButtonCoordinate::new(3, 1),
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
                        coordinate: ButtonCoordinate::new(3, 2),
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
            // Pretending shift doesnt exist for now
            // MetaButtonMapping {
            //     location: ButtonGridLocation::MetaRight,
            //     coordinate: ButtonCoordinate::new(0, 0),
            //     on_action: MetaButtonAction::EnableShiftMode,
            //     off_action: Some(MetaButtonAction::DisableShiftMode),
            // },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaRight,
                coordinate: ButtonCoordinate::new(0, 0),
                on_action: MetaButtonAction::TapTempo,
                off_action: None,
            },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaBottom,
                coordinate: ButtonCoordinate::new(0, 0),
                on_action: MetaButtonAction::SelectFixtureGroupControl(FixtureGroupId::new(1)),
                off_action: None,
            },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaBottom,
                coordinate: ButtonCoordinate::new(1, 0),
                on_action: MetaButtonAction::SelectFixtureGroupControl(FixtureGroupId::new(2)),
                off_action: None,
            },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaBottom,
                coordinate: ButtonCoordinate::new(2, 0),
                on_action: MetaButtonAction::SelectFixtureGroupControl(FixtureGroupId::new(3)),
                off_action: None,
            },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaBottom,
                coordinate: ButtonCoordinate::new(4, 0),
                on_action: MetaButtonAction::SelectScene(SceneId::new(1)),
                off_action: None,
            },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaBottom,
                coordinate: ButtonCoordinate::new(5, 0),
                on_action: MetaButtonAction::SelectScene(SceneId::new(2)),
                off_action: None,
            },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaBottom,
                coordinate: ButtonCoordinate::new(6, 0),
                on_action: MetaButtonAction::SelectScene(SceneId::new(3)),
                off_action: None,
            },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaBottom,
                coordinate: ButtonCoordinate::new(7, 0),
                on_action: MetaButtonAction::SelectScene(SceneId::new(4)),
                off_action: None,
            },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaRight,
                coordinate: ButtonCoordinate::new(0, 7),
                on_action: MetaButtonAction::UpdateClockRate(Rate::new(1.0 / 3.0)),
                off_action: None,
            },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaRight,
                coordinate: ButtonCoordinate::new(0, 6),
                on_action: MetaButtonAction::UpdateClockRate(Rate::new(1.0 / 2.0)),
                off_action: None,
            },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaRight,
                coordinate: ButtonCoordinate::new(0, 5),
                on_action: MetaButtonAction::UpdateClockRate(Rate::new(1.0)),
                off_action: None,
            },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaRight,
                coordinate: ButtonCoordinate::new(0, 4),
                on_action: MetaButtonAction::UpdateClockRate(Rate::new(2.0)),
                off_action: None,
            },
            MetaButtonMapping {
                location: ButtonGridLocation::MetaRight,
                coordinate: ButtonCoordinate::new(0, 3),
                on_action: MetaButtonAction::UpdateClockRate(Rate::new(3.0)),
                off_action: None,
            },
        ],
    )
}
