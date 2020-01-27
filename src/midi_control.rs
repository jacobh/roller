use async_std::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum MidiEvent {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8, velocity: u8 },
    ControlChange { control: u8, value: u8 },
    Other(rimd::Status),
}
impl From<rimd::MidiMessage> for MidiEvent {
    fn from(message: rimd::MidiMessage) -> MidiEvent {
        match message.status() {
            rimd::Status::NoteOn => MidiEvent::NoteOn {
                note: message.data(1),
                velocity: message.data(2),
            },
            rimd::Status::NoteOff => MidiEvent::NoteOff {
                note: message.data(1),
                velocity: message.data(2),
            },
            rimd::Status::ControlChange => MidiEvent::ControlChange {
                control: message.data(1),
                value: message.data(2),
            },
            _ => MidiEvent::Other(message.status()),
        }
    }
}

pub struct MidiController {
    client: coremidi::Client,
    source: coremidi::Source,
    input_port: coremidi::InputPort,
    output_port: coremidi::OutputPort,

    pub input_receiver: async_std::sync::Receiver<MidiEvent>,
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
                    let midi_message = rimd::MidiMessage::from_bytes(packet.data().to_vec());
                    let midi_event = MidiEvent::from(midi_message);
                    async_std::task::block_on(input_sender.send(midi_event));
                }
            })
            .unwrap();
        midi_input_port.connect_source(&source).unwrap();

        let midi_output_port = midi_client
            .output_port(&format!("roller-input-{}", name))
            .unwrap();

        Ok(MidiController {
            client: midi_client,
            source: source,
            input_port: midi_input_port,
            output_port: midi_output_port,
            input_receiver: input_receiver,
        })
    }
    pub fn incoming_events(&self) -> impl Stream<Item = MidiEvent> {
        self.input_receiver.clone()
    }
}
