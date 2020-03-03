use async_std::{prelude::*, sync::Arc};
use midi::{ControlChannel, MidiEvent, Note};
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};

use crate::{
    control::{
        button::{AkaiPadState, ButtonGroup, ButtonMapping, MetaButtonMapping, PadMapping},
        default_midi_mapping,
        fader::MidiFaderMapping,
    },
    lighting_engine::LightingEvent,
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
            midi_mapping: Arc::new(default_midi_mapping()),
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
