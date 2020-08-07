use async_std::{prelude::*, stream::Stream, sync::Arc};
use std::pin::Pin;
use std::time::Duration;
use thiserror::Error;

#[cfg(target_os = "macos")]
extern crate coremidi;

use crate::{MidiEvent, MidiMessageStream};

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
struct MidiInputState {
    _client: coremidi::Client,
    _input_port: coremidi::InputPort,
    _source: coremidi::Source,
}

// TODO unclear if this is legitimate
unsafe impl Send for MidiInputState {}
unsafe impl Sync for MidiInputState {}

#[derive(Debug, Clone)]
pub struct MidiInput {
    state: Arc<MidiInputState>,
    input_receiver: async_std::sync::Receiver<MidiEvent>,
}
impl MidiInput {
    pub fn new(name: &str) -> Result<MidiInput, MidiIoError> {
        if !cfg!(macos) {
            unimplemented!()
        }

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
                    let mut stream = MidiMessageStream::new(packet.data());

                    loop {
                        match MidiEvent::read_next(&mut stream) {
                            Ok(Some(midi_event)) => {
                                async_std::task::block_on(input_sender.send(midi_event));
                            }
                            Ok(None) => {
                                break;
                            }
                            Err(e) => eprintln!("error reading midi event: {:?}", e),
                        }
                    }
                }
            })
            .map_err(|_| MidiIoError::InitFailed)?;

        midi_input_port
            .connect_source(&source)
            .map_err(|_| MidiIoError::InitFailed)?;

        Ok(MidiInput {
            state: Arc::new(MidiInputState {
                _client: client,
                _input_port: midi_input_port,
                _source: source,
            }),
            input_receiver,
        })
    }
}
impl Stream for MidiInput {
    type Item = MidiEvent;
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<Option<Self::Item>> {
        Pin::new(&mut self.input_receiver).poll_next(cx)
    }
}

unsafe impl Send for MidiOutput {}
unsafe impl Sync for MidiOutput {}
#[derive(Debug)]
pub struct MidiOutput {
    _client: coremidi::Client,
    output_sender: async_std::sync::Sender<Vec<u8>>,
}
impl MidiOutput {
    pub fn new(name: &str) -> Result<MidiOutput, MidiIoError> {
        if !cfg!(macos) {
            unimplemented!()
        }

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
