use async_std::{prelude::*, sync::Arc};
use midi::{ControlChannel, MidiEvent, MidiInput, MidiOutput, Note};
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};

use crate::{
    control::{
        button::{AkaiPadState, ButtonGroup, ButtonMapping, MetaButtonMapping, PadMapping},
        default_midi_mapping,
        fader::MidiFaderMapping,
    },
    lighting_engine::ControlEvent,
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
    pub fn new(
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
    fn midi_to_control_event(&self, midi_event: &MidiEvent) -> Option<ControlEvent> {
        let now = Instant::now();

        match dbg!(midi_event) {
            MidiEvent::ControlChange { control, value } => self
                .faders
                .get(control)
                .map(|fader| fader.control_event(1.0 / 127.0 * (*value as f64))),
            MidiEvent::NoteOn { note, .. } => self
                .group_buttons()
                .find(|(_, button)| button.note == *note)
                .map(|(group, button)| {
                    button
                        .clone()
                        .into_control_event(group.clone(), NoteState::On, now)
                })
                .or_else(|| {
                    self.meta_buttons
                        .get(note)
                        .map(|meta_button| meta_button.control_event(now))
                }),
            MidiEvent::NoteOff { note, .. } => self
                .group_buttons()
                .find(|(_, button)| button.note == *note)
                .map(|(group, button)| {
                    button
                        .clone()
                        .into_control_event(group.clone(), NoteState::Off, now)
                }),
            _ => None,
        }
    }
    pub fn pad_mappings(&self) -> impl Iterator<Item = PadMapping<'_>> {
        self.group_buttons()
            .map(PadMapping::from)
            .chain(self.meta_buttons.values().map(PadMapping::from))
    }
}

pub struct MidiController {
    pub midi_mapping: Arc<MidiMapping>,
    midi_input: MidiInput,
    midi_output: MidiOutput,
}
impl MidiController {
    pub fn new_for_device_name(name: &str) -> Result<MidiController, ()> {
        let midi_input = MidiInput::new(name).map_err(|_| ())?;
        let midi_output = MidiOutput::new(name).map_err(|_| ())?;

        Ok(MidiController {
            midi_mapping: Arc::new(default_midi_mapping()),
            midi_input,
            midi_output,
        })
    }
    pub fn control_events(&self) -> impl Stream<Item = ControlEvent> {
        let mapping = self.midi_mapping.clone();

        self.midi_input
            .clone()
            .map(move |midi_event| mapping.midi_to_control_event(&midi_event))
            .filter(|control_event| control_event.is_some())
            .map(|control_event| control_event.unwrap())
    }
    pub async fn set_pad_color(&self, note: Note, pad_color: AkaiPadState) {
        self.midi_output
            .send_packet(vec![0x90, u8::from(note), pad_color.as_byte()])
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
