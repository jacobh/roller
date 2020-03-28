use async_std::prelude::*;
use std::time::Duration;
use thiserror::Error;

use crate::MidiEvent;

#[derive(Debug, Error)]
pub enum MidiIoError {
    #[error("Failed to initialize client")]
    InitFailed,
    #[error("Couldn't find MIDI source with this name")]
    SourceNotFound,
    #[error("Couldn't find MIDI destination with this name")]
    DestinationNotFound,
}

#[derive(Debug)]
pub struct MidiInput {
    _client: coremidi::Client,
    _input_port: coremidi::InputPort,
    _source: coremidi::Source,
    input_receiver: async_std::sync::Receiver<MidiEvent>,
}
impl MidiInput {
    pub fn new(name: &str) -> Result<MidiInput, MidiIoError> {
        let client = coremidi::Client::new(&format!("roller-input-{}", name))
            .map_err(|_| MidiIoError::InitFailed)?;

        let source = coremidi::Sources
            .into_iter()
            .find(|source| source.display_name().as_deref() == Some(name))
            .ok_or(MidiIoError::SourceNotFound)?;

        let (input_sender, input_receiver) = async_std::sync::channel::<MidiEvent>(1024);

        let midi_input_port = client
            .input_port(&format!("roller-input-{}", name), move |packet_list| {
                for packet in packet_list.iter() {
                    // multiple messages may appear in the same packet
                    for message_data in packet.data().chunks_exact(3) {
                        let midi_event = MidiEvent::from_bytes(message_data);
                        async_std::task::block_on(input_sender.send(midi_event));
                    }
                }
            })
            .map_err(|_| MidiIoError::InitFailed)?;

        midi_input_port
            .connect_source(&source)
            .map_err(|_| MidiIoError::InitFailed)?;

        Ok(MidiInput {
            _client: client,
            _input_port: midi_input_port,
            _source: source,
            input_receiver,
        })
    }
    pub fn events(&self) -> impl Stream<Item = MidiEvent> {
        self.input_receiver.clone()
    }
}

#[derive(Debug)]
pub struct MidiOutput {
    _client: coremidi::Client,
    output_sender: async_std::sync::Sender<Vec<u8>>,
}
impl MidiOutput {
    pub fn new(name: &str) -> Result<MidiOutput, MidiIoError> {
        let client = coremidi::Client::new(&format!("roller-output-{}", name))
            .map_err(|_| MidiIoError::InitFailed)?;

        let (output_sender, mut output_receiver) = async_std::sync::channel::<Vec<u8>>(512);

        let destination = coremidi::Destinations
            .into_iter()
            .find(|dest| dest.display_name().as_deref() == Some(name))
            .ok_or(MidiIoError::DestinationNotFound)?;

        let midi_output_port = client
            .output_port(&format!("roller-output-{}", name))
            .map_err(|_| MidiIoError::InitFailed)?;

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

        Ok(MidiOutput {
            _client: client,
            output_sender,
        })
    }
    pub async fn send_packet(&self, packet: impl Into<Vec<u8>>) {
        self.output_sender.send(packet.into()).await
    }
}
