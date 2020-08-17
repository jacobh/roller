use async_std::prelude::*;
use futures::pin_mut;
use futures::stream::{self, StreamExt};
use midi::{MidiEvent, MidiInput, MidiOutput, Note};
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};

use roller_protocol::control::{
    ButtonCoordinate, ButtonGridLocation, ButtonState, FaderId, InputEvent,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AkaiPadState {
    Off,
    Green,
    GreenBlink,
    Red,
    RedBlink,
    Yellow,
    YellowBlink,
}
impl AkaiPadState {
    fn as_byte(self) -> u8 {
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

pub struct MidiController {
    midi_input: MidiInput,
    button_update_send: async_std::sync::Sender<Vec<(Note, AkaiPadState, Illumination)>>,
}
impl MidiController {
    pub fn new_for_device_name<'a>(name: impl Into<String>) -> Result<MidiController, ()> {
        let name = name.into();
        let midi_input = MidiInput::new(&name).map_err(|_| ())?;

        let (button_update_send, button_update_recv) =
            async_std::sync::channel::<Vec<(Note, AkaiPadState, Illumination)>>(8);

        async_std::task::spawn(async move {
            let started_at = Instant::now();
            let midi_output = MidiOutput::new(&name).map_err(|_| ()).unwrap();
            let mut current_button_strobes: FxHashMap<Note, AkaiPadState> = FxHashMap::default();

            enum Event {
                ButtonUpdates(Vec<(Note, AkaiPadState, Illumination)>),
                Tick,
            }

            let ticks = crate::utils::tick_stream(Duration::from_millis(40)).map(|_| Event::Tick);
            let button_updates = button_update_recv.map(Event::ButtonUpdates);
            let events = stream::select(ticks, button_updates);

            pin_mut!(events);

            while let Some(event) = events.next().await {
                match event {
                    Event::ButtonUpdates(updates) => {
                        for (note, state, illumination) in updates.into_iter() {
                            match illumination {
                                Illumination::Solid => {
                                    current_button_strobes.remove(&note);
                                    midi_output
                                        .send_packet(vec![0x90, u8::from(note), state.as_byte()])
                                        .await;
                                }
                                Illumination::Strobe => {
                                    current_button_strobes.insert(note, state);
                                }
                            }
                        }
                    }
                    Event::Tick => {
                        for (note, state) in current_button_strobes.iter() {
                            let state = if started_at.elapsed().as_millis() % 250 < 100 {
                                *state
                            } else {
                                AkaiPadState::Off
                            };

                            midi_output
                                .send_packet(vec![0x90, u8::from(*note), state.as_byte()])
                                .await;
                        }
                    }
                }
            }
        });

        Ok(MidiController {
            midi_input,
            button_update_send,
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
            .filter_map(move |midi_event| futures::future::ready(midi_to_input_event(&midi_event)))
    }
    pub async fn set_button_state(
        &self,
        location: ButtonGridLocation,
        coordinate: ButtonCoordinate,
        state: ButtonState,
    ) {
        let note = coordinate_to_note(&location, &coordinate);
        let (pad_color, illumination) = button_state_to_akai_pad_color(location, state);

        self.button_update_send
            .send(vec![(note, pad_color, illumination)])
            .await;
    }
    pub async fn set_button_states(
        &self,
        states: impl IntoIterator<Item = (ButtonGridLocation, ButtonCoordinate, ButtonState)>,
    ) {
        for (location, coordinate, state) in states {
            self.set_button_state(location, coordinate, state).await
        }
    }
    pub async fn reset_pads(&self) {
        for row_idx in 0..8 {
            for column_idx in 0..8 {
                self.set_button_state(
                    ButtonGridLocation::Main,
                    ButtonCoordinate {
                        row_idx,
                        column_idx,
                    },
                    ButtonState::Unused,
                )
                .await;
            }
        }
    }
    pub async fn run_pad_startup(&self) {
        for row_idx in 0..8 {
            for column_idx in 0..8 {
                self.set_button_state(
                    ButtonGridLocation::Main,
                    ButtonCoordinate {
                        row_idx,
                        column_idx,
                    },
                    ButtonState::Active,
                )
                .await;
                async_std::task::sleep(Duration::from_millis(10)).await;
            }
        }
        async_std::task::sleep(Duration::from_millis(150)).await;
        self.reset_pads().await;
    }
}

enum Illumination {
    Solid,
    Strobe,
}

fn button_state_to_akai_pad_color(
    location: ButtonGridLocation,
    state: ButtonState,
) -> (AkaiPadState, Illumination) {
    match location {
        ButtonGridLocation::Main => match state {
            ButtonState::Active => (AkaiPadState::Green, Illumination::Solid),
            ButtonState::Inactive => (AkaiPadState::Yellow, Illumination::Solid),
            ButtonState::Deactivated => (AkaiPadState::Red, Illumination::Solid),
            ButtonState::Unused => (AkaiPadState::Off, Illumination::Solid),
        },
        ButtonGridLocation::MetaBottom | ButtonGridLocation::MetaRight => match state {
            ButtonState::Active => (AkaiPadState::Green, Illumination::Strobe),
            ButtonState::Inactive => (AkaiPadState::Yellow, Illumination::Solid),
            ButtonState::Deactivated => (AkaiPadState::Red, Illumination::Solid),
            ButtonState::Unused => (AkaiPadState::Off, Illumination::Solid),
        },
    }
}
