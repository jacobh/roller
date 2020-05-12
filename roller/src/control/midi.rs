use async_std::prelude::*;
use midi::{MidiEvent, MidiInput, MidiOutput, Note};
use roller_protocol::FaderId;
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};

use roller_protocol::{ButtonCoordinate, ButtonGridLocation, InputEvent};

use crate::{
    control::{
        button::{AkaiPadState, ButtonGroup, ButtonMapping, MetaButtonMapping, PadMapping},
        fader::FaderControlMapping,
    },
    lighting_engine::ControlEvent,
};

// Specific for akai apc mini. really the internal button location should be specific with coords
fn coordinate_to_note(loc: &ButtonGridLocation, coord: &ButtonCoordinate) -> Note {
    match loc {
        ButtonGridLocation::Main => Note::new((8 * coord.row_idx + coord.column_idx) as u8),
        ButtonGridLocation::MetaRight => Note::new((89 - coord.row_idx) as u8),
        ButtonGridLocation::MetaBottom => Note::new((64 + coord.column_idx) as u8),
    }
}

fn note_to_coordinate(note: Note) -> Option<(ButtonGridLocation, ButtonCoordinate)> {
    let note = u8::from(note) as usize;
    if note < 64 {
        Some((
            ButtonGridLocation::Main,
            ButtonCoordinate {
                row_idx: note / 8,
                column_idx: note % 8,
            },
        ))
    } else if note < 72 {
        Some((
            ButtonGridLocation::MetaBottom,
            ButtonCoordinate {
                row_idx: 0,
                column_idx: note - 64,
            },
        ))
    } else if note < 90 {
        Some((
            ButtonGridLocation::MetaRight,
            ButtonCoordinate {
                row_idx: 89 - note,
                column_idx: 0,
            },
        ))
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteState {
    On,
    Off,
}

pub enum ButtonRef<'a> {
    Standard(&'a ButtonGroup, &'a ButtonMapping),
    Meta(&'a MetaButtonMapping),
}
impl<'a> ButtonRef<'a> {
    pub fn into_control_event(self, note_state: NoteState, now: Instant) -> Option<ControlEvent> {
        match (self, note_state) {
            (ButtonRef::Standard(group, button), _) => Some(button.clone().into_control_event(
                group.clone(),
                note_state,
                now,
            )),
            (ButtonRef::Meta(meta_button), NoteState::On) => {
                Some(meta_button.on_action.control_event(now))
            }
            (ButtonRef::Meta(meta_button), NoteState::Off) => meta_button
                .off_action
                .as_ref()
                .map(|action| action.control_event(now)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiMapping {
    pub faders: FxHashMap<FaderId, FaderControlMapping>,
    pub button_groups: Vec<ButtonGroup>,
    pub meta_buttons: FxHashMap<(ButtonGridLocation, ButtonCoordinate), MetaButtonMapping>,
}
impl MidiMapping {
    pub fn new(
        faders: Vec<FaderControlMapping>,
        button_groups: Vec<ButtonGroup>,
        meta_buttons: Vec<MetaButtonMapping>,
    ) -> MidiMapping {
        MidiMapping {
            faders: faders
                .into_iter()
                .map(|mapping| (mapping.id, mapping))
                .collect(),
            button_groups,
            meta_buttons: meta_buttons
                .into_iter()
                .map(|mapping| ((mapping.location, mapping.coordinate), mapping))
                .collect(),
        }
    }
    fn group_buttons(&self) -> impl Iterator<Item = (&'_ ButtonGroup, &'_ ButtonMapping)> {
        self.button_groups
            .iter()
            .flat_map(|group| group.buttons().map(move |button| (group, button)))
    }
    pub fn find_button(
        &self,
        location: ButtonGridLocation,
        coordinate: ButtonCoordinate,
    ) -> Option<ButtonRef<'_>> {
        if location == ButtonGridLocation::Main {
            self.group_buttons()
                .find(|(_, button)| button.coordinate == coordinate)
                .map(|(group, button)| ButtonRef::Standard(group, button))
        } else {
            self.meta_buttons
                .get(&(location, coordinate))
                .map(|meta_button| ButtonRef::Meta(meta_button))
        }
    }
    pub fn pad_mappings(&self) -> impl Iterator<Item = PadMapping<'_>> {
        self.group_buttons()
            .map(PadMapping::from)
            .chain(self.meta_buttons.values().map(PadMapping::from))
    }
}

pub struct MidiController {
    midi_input: MidiInput,
    midi_output: MidiOutput,
}
impl MidiController {
    pub fn new_for_device_name(name: &str) -> Result<MidiController, ()> {
        let midi_input = MidiInput::new(name).map_err(|_| ())?;
        let midi_output = MidiOutput::new(name).map_err(|_| ())?;

        Ok(MidiController {
            midi_input,
            midi_output,
        })
    }
    pub fn input_events(&self) -> impl Stream<Item = InputEvent> {
        // TODO this should be moved to a "control device mapping"
        fn midi_to_input_event(midi_event: &MidiEvent) -> Option<InputEvent> {
            match dbg!(midi_event) {
                MidiEvent::ControlChange { control, value } => {
                    let fader_id = FaderId::new((u8::from(*control) - 48) as usize);
                    let value = 1.0 / 127.0 * (*value as f64);
                    Some(InputEvent::FaderUpdated(fader_id, value))
                }
                MidiEvent::NoteOn { note, .. } => {
                    let (loc, coord) = note_to_coordinate(*note)?;
                    Some(InputEvent::ButtonPressed(loc, coord))
                }
                MidiEvent::NoteOff { note, .. } => {
                    let (loc, coord) = note_to_coordinate(*note)?;
                    Some(InputEvent::ButtonReleased(loc, coord))
                }
                _ => None,
            }
        }

        self.midi_input
            .clone()
            .map(move |midi_event| midi_to_input_event(&midi_event))
            .filter(|control_event| control_event.is_some())
            .map(|control_event| control_event.unwrap())
    }
    pub async fn set_pad_color(
        &self,
        location: ButtonGridLocation,
        coordinate: ButtonCoordinate,
        pad_color: AkaiPadState,
    ) {
        let note = coordinate_to_note(&location, &coordinate);

        self.midi_output
            .send_packet(vec![0x90, u8::from(note), pad_color.as_byte()])
            .await
    }
    pub async fn set_pad_colors(
        &self,
        pad_colors: impl IntoIterator<Item = (ButtonGridLocation, ButtonCoordinate, AkaiPadState)>,
    ) {
        for (location, coordinate, pad_color) in pad_colors {
            self.set_pad_color(location, coordinate, pad_color).await
        }
    }
    pub async fn reset_pads(&self) {
        for row_idx in 0..8 {
            for column_idx in 0..8 {
                self.set_pad_color(
                    ButtonGridLocation::Main,
                    ButtonCoordinate {
                        row_idx,
                        column_idx,
                    },
                    AkaiPadState::Off,
                )
                .await;
            }
        }
    }
    pub async fn run_pad_startup(&self) {
        for row_idx in 0..8 {
            for column_idx in 0..8 {
                self.set_pad_color(
                    ButtonGridLocation::Main,
                    ButtonCoordinate {
                        row_idx,
                        column_idx,
                    },
                    AkaiPadState::Green,
                )
                .await;
                async_std::task::sleep(Duration::from_millis(10)).await;
            }
        }
        async_std::task::sleep(Duration::from_millis(150)).await;
        self.reset_pads().await;
    }
}
