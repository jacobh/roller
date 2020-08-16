use roller_protocol::{
    clock::{
        offset::{ClockOffset, ClockOffsetMode},
        Beats, Rate,
    },
    color::Color,
    control::{ButtonCoordinate, ButtonGridLocation, FaderId},
    effect::EffectDirection,
    effect::{
        ColorEffect, ColorModulation, ColorModulator, DimmerEffect, DimmerModulator, PixelEffect,
        PixelModulator, PositionEffect, PositionModulator, Waveform,
    },
    fixture::FixtureGroupId,
    position::BasePositionMode,
};

use crate::{
    control::{
        button::{
            ButtonAction, ButtonGroup, ButtonMapping, ButtonType, MetaButtonAction,
            MetaButtonMapping,
        },
        control_mapping::ControlMapping,
        fader::{FaderControlMapping, FaderCurve, FaderType},
    },
    lighting_engine::SceneId,
};

pub fn default_control_mapping() -> ControlMapping {
    ControlMapping::new(
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
                        label: "White".to_owned(),
                        coordinate: ButtonCoordinate::new(0, 7),
                        on_action: ButtonAction::UpdateGlobalColor(Color::White),
                    },
                    ButtonMapping {
                        label: "Yellow".to_owned(),
                        coordinate: ButtonCoordinate::new(0, 6),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Yellow),
                    },
                    ButtonMapping {
                        label: "Deep Orange".to_owned(),
                        coordinate: ButtonCoordinate::new(0, 5),
                        on_action: ButtonAction::UpdateGlobalColor(Color::DeepOrange),
                    },
                    ButtonMapping {
                        label: "Red".to_owned(),
                        coordinate: ButtonCoordinate::new(0, 4),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Red),
                    },
                    ButtonMapping {
                        label: "Violet".to_owned(),
                        coordinate: ButtonCoordinate::new(0, 3),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Violet),
                    },
                    ButtonMapping {
                        label: "Dark Blue".to_owned(),
                        coordinate: ButtonCoordinate::new(0, 2),
                        on_action: ButtonAction::UpdateGlobalColor(Color::DarkBlue),
                    },
                    ButtonMapping {
                        label: "Teal".to_owned(),
                        coordinate: ButtonCoordinate::new(0, 1),
                        on_action: ButtonAction::UpdateGlobalColor(Color::Teal),
                    },
                    ButtonMapping {
                        label: "Green".to_owned(),
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
                        label: "White (Secondary)".to_owned(),
                        coordinate: ButtonCoordinate::new(1, 7),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::White),
                    },
                    ButtonMapping {
                        label: "Yellow (Secondary)".to_owned(),
                        coordinate: ButtonCoordinate::new(1, 6),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Yellow),
                    },
                    ButtonMapping {
                        label: "Deep Orange (Secondary)".to_owned(),
                        coordinate: ButtonCoordinate::new(1, 5),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::DeepOrange),
                    },
                    ButtonMapping {
                        label: "Red (Secondary)".to_owned(),
                        coordinate: ButtonCoordinate::new(1, 4),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Red),
                    },
                    ButtonMapping {
                        label: "Violet (Secondary)".to_owned(),
                        coordinate: ButtonCoordinate::new(1, 3),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Violet),
                    },
                    ButtonMapping {
                        label: "Dark Blue (Secondary)".to_owned(),
                        coordinate: ButtonCoordinate::new(1, 2),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::DarkBlue),
                    },
                    ButtonMapping {
                        label: "Teal (Secondary)".to_owned(),
                        coordinate: ButtonCoordinate::new(1, 1),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Teal),
                    },
                    ButtonMapping {
                        label: "Green (Secondary)".to_owned(),
                        coordinate: ButtonCoordinate::new(1, 0),
                        on_action: ButtonAction::UpdateGlobalSecondaryColor(Color::Green),
                    },
                ],
            ),
            // Dimmer Effects
            ButtonMapping {
                label: "1/4 Sine Down".to_owned(),
                coordinate: ButtonCoordinate::new(7, 7),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::SineDown, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                label: "1/4 Half Sine Up".to_owned(),
                coordinate: ButtonCoordinate::new(7, 6),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::HalfSineUp, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                label: "1/4 Half Sine Down".to_owned(),
                coordinate: ButtonCoordinate::new(7, 5),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::HalfSineDown, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            ButtonMapping {
                label: "1/4 Short Square Pulse".to_owned(),
                coordinate: ButtonCoordinate::new(7, 4),
                on_action: ButtonAction::ActivateDimmerEffect(
                    DimmerModulator::new(Waveform::ShortSquarePulse, Beats::new(1.0), 1.0).into(),
                ),
            }
            .into_group(ButtonType::Toggle),
            // Dimmer sequences
            ButtonMapping {
                label: "1/4 Offset Pulse".to_owned(),
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
                label: "1/2 Alternating Pulse".to_owned(),
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
                label: "1/4 Three Up One Down".to_owned(),
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
                label: "1/1 Offset Saw Down".to_owned(),
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
                label: "1/1 Offset Sine Up".to_owned(),
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
                label: "1/4 5/4 Offset Step".to_owned(),
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
                label: "1/2 120deg Shift Up".to_owned(),
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
                label: "1/4 -90deg Shift Down".to_owned(),
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
                label: "1/4 6 Step Hue".to_owned(),
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
                label: "1/4 Offset Random White Flash".to_owned(),
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
                label: "1/4 Fast 180deg Hue Shifts".to_owned(),
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
                label: "1/2 Hue to Secondary".to_owned(),
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
                        label: "1/4 From Center".to_owned(),
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
                            Some(ClockOffset::new(
                                ClockOffsetMode::Location(EffectDirection::FromCenter),
                                Beats::new(0.1),
                            )),
                        )),
                    },
                    ButtonMapping {
                        label: "1/2 Offset From Center".to_owned(),
                        coordinate: ButtonCoordinate::new(4, 6),
                        on_action: ButtonAction::ActivatePixelEffect(PixelEffect::new(
                            vec![PixelModulator::new(
                                Waveform::SineDown,
                                Beats::new(2.0),
                                EffectDirection::FromCenter,
                            )],
                            Some(ClockOffset::new(
                                ClockOffsetMode::Location(EffectDirection::FromCenter),
                                Beats::new(1.0),
                            )),
                        )),
                    },
                    ButtonMapping {
                        label: "1/4 Sigmoid Wave".to_owned(),
                        coordinate: ButtonCoordinate::new(4, 5),
                        on_action: ButtonAction::ActivatePixelEffect(PixelEffect::new(
                            vec![PixelModulator::new(
                                Waveform::SigmoidWaveDown,
                                Beats::new(2.0),
                                EffectDirection::BottomToTop,
                            )],
                            Some(ClockOffset::new(
                                ClockOffsetMode::Location(EffectDirection::LeftToRight),
                                Beats::new(1.0),
                            )),
                        )),
                    },
                    ButtonMapping {
                        label: "1/4 1.5 Root Wave".to_owned(),
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
                                ClockOffsetMode::Location(EffectDirection::LeftToRight),
                                Beats::new(0.25),
                            )),
                        )),
                    },
                    ButtonMapping {
                        label: "1/4 Offset 3 Up 1 Down".to_owned(),
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
                                ClockOffsetMode::Location(EffectDirection::FromCenter),
                                Beats::new(2.0),
                            )),
                        )),
                    },
                    ButtonMapping {
                        label: "1/4 2 Down 1 Up".to_owned(),
                        coordinate: ButtonCoordinate::new(4, 2),
                        on_action: ButtonAction::ActivatePixelEffect(PixelEffect::new(
                            vec![
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
                                PixelModulator::new(
                                    Waveform::OnePointFiveRootUp,
                                    Beats::new(1.0),
                                    EffectDirection::BottomToTop,
                                ),
                            ],
                            None,
                        )),
                    },
                    ButtonMapping {
                        label: "1/8 Offset 4 Up 4 Down".to_owned(),
                        coordinate: ButtonCoordinate::new(4, 0),
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
                                ClockOffsetMode::Location(EffectDirection::LeftToRight),
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
                        label: "0/0".to_owned(),
                        coordinate: ButtonCoordinate::new(3, 7),
                        on_action: ButtonAction::UpdateBasePosition((0.0, 0.0).into()),
                    },
                    ButtonMapping {
                        label: "-15/-30 Mirrored".to_owned(),
                        coordinate: ButtonCoordinate::new(3, 6),
                        on_action: ButtonAction::UpdateBasePosition(
                            ((-15.0, -30.0), BasePositionMode::MirrorPan).into(),
                        ),
                    },
                    ButtonMapping {
                        label: "30/-30 Mirrored".to_owned(),
                        coordinate: ButtonCoordinate::new(3, 5),
                        on_action: ButtonAction::UpdateBasePosition(
                            ((30.0, -30.0), BasePositionMode::MirrorPan).into(),
                        ),
                    },
                    ButtonMapping {
                        label: "50/-75 Mirrored".to_owned(),
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
                        label: "Movement Effect #1".to_owned(),
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
                        label: "Movement Effect #2".to_owned(),
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
                        label: "Movement Effect #3".to_owned(),
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
