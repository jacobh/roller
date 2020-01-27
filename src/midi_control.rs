pub struct MidiController {
    client: coremidi::Client,
    source: coremidi::Source,
    input_port: coremidi::InputPort,
    output_port: coremidi::OutputPort,
}
impl MidiController {
    pub fn new_for_device_name(name: &str) -> Result<MidiController, ()> {
        let midi_client = coremidi::Client::new(&format!("roller-{}", name)).unwrap();

        let source = coremidi::Sources
            .into_iter()
            .find(|source| source.display_name() == Some(name.to_owned()))
            .unwrap();

        let midi_input_port = midi_client
            .input_port(&format!("roller-input-{}", name), |packet_list| {
                dbg!(packet_list);
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
        })
    }
}
