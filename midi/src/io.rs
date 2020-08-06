use async_std::{prelude::*, stream::Stream, sync::Arc};
use std::pin::Pin;
use std::time::Duration;
use thiserror::Error;

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

unsafe impl Send for MidiInput {}
unsafe impl Sync for MidiInput {}
#[derive(Clone)]
pub struct MidiInput {
    midi_conn: Arc<midir::MidiInputConnection<()>>,
    input_receiver: async_std::sync::Receiver<MidiEvent>,
}
impl MidiInput {
    pub fn new(name: &str) -> Result<MidiInput, MidiIoError> {
        let (input_sender, input_receiver) = async_std::sync::channel::<MidiEvent>(1024);

        let midi_input = midir::MidiInput::new(&format!("roller-input-{}", name))
            .map_err(|_| MidiIoError::InitFailed)?;

        let midi_input_port = midi_input
            .ports()
            .into_iter()
            .find(|port| midi_input.port_name(port).unwrap_or(String::new()) == name)
            .ok_or(MidiIoError::SourceNotFound)?;

        let midi_conn = midi_input
            .connect(
                &midi_input_port,
                &format!("roller-input-port-{}", name),
                move |_timestamp, bytes, ()| {
                    let mut stream = MidiMessageStream::new(bytes);

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
                },
                (),
            )
            .map_err(|_| MidiIoError::InitFailed)?;

        Ok(MidiInput {
            midi_conn: Arc::new(midi_conn),
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
    output_sender: async_std::sync::Sender<Vec<u8>>,
}
impl MidiOutput {
    pub fn new(name: &str) -> Result<MidiOutput, MidiIoError> {
        let midi_output = midir::MidiOutput::new(&format!("roller-output-{}", name)).unwrap();

        let midi_output_port = midi_output
            .ports()
            .into_iter()
            .find(|port| midi_output.port_name(port).unwrap_or(String::new()) == name)
            .ok_or(MidiIoError::DestinationNotFound)?;

        let mut midi_conn = midi_output
            .connect(&midi_output_port, &format!("roller-output-conn-{}", name))
            .map_err(|_| MidiIoError::InitFailed)?;

        let (output_sender, mut output_receiver) = async_std::sync::channel::<Vec<u8>>(512);

        async_std::task::spawn(async move {
            while let Some(packet) = output_receiver.next().await {
                midi_conn
                    .send(&packet)
                    .map_err(|_| "failed to send packets")
                    .unwrap();
                async_std::task::sleep(Duration::from_millis(1)).await;
            }
        });

        Ok(MidiOutput { output_sender })
    }
    pub async fn send_packet(&self, packet: impl Into<Vec<u8>>) {
        self.output_sender.send(packet.into()).await
    }
}
