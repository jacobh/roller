use async_std::{
    net::{TcpStream, ToSocketAddrs},
    prelude::*,
};
use prost::Message;
use std::collections::HashMap;

mod ola_rpc {
    include!(concat!(env!("OUT_DIR"), "/ola.rpc.rs"));
}

const PROTOCOL_VERSION: u32 = 1;
const VERSION_MASK: u32 = 0xf0000000;
const SIZE_MASK: u32 = 0x0fffffff;
const RECEIVE_BUFFER_SIZE: usize = 8192;

fn new_header(length: usize) -> [u8; 4] {
    let length = length as u32;
    let header_u32 = ((PROTOCOL_VERSION << 28) & VERSION_MASK) | (length & SIZE_MASK);

    header_u32.to_le_bytes()
}

fn serialize_message(message: impl prost::Message) -> Result<Vec<u8>, prost::EncodeError> {
    let mut buf = Vec::<u8>::new();
    message.encode(&mut buf)?;
    Ok(buf)
}

pub struct OlaClient {
    stream: TcpStream,
    sequence: usize,
    outstanding_requests: HashMap<usize, Box<dyn prost::Message>>,
    outstanding_responses: HashMap<usize, Box<dyn prost::Message>>,
    received_bytes_buffer: Vec<u8>,
    current_message_expected_size: Option<usize>,
    skip_current_message: bool,
}
impl OlaClient {
    pub async fn connect(host: impl ToSocketAddrs) -> Result<OlaClient, async_std::io::Error> {
        let stream = TcpStream::connect(host).await?;
        stream.set_nodelay(true)?;

        Ok(OlaClient {
            stream: stream,
            sequence: 0,
            outstanding_requests: HashMap::new(),
            outstanding_responses: HashMap::new(),
            received_bytes_buffer: vec![],
            current_message_expected_size: None,
            skip_current_message: false,
        })
    }
    pub async fn connect_localhost() -> Result<OlaClient, async_std::io::Error> {
        OlaClient::connect("localhost:9010").await
    }

    fn iterate_sequence(&mut self) -> usize {
        let out = self.sequence.clone();
        self.sequence += 1;
        out
    }

    async fn send_message(
        &mut self,
        message: ola_rpc::RpcMessage,
    ) -> Result<(), async_std::io::Error> {
        let serialized_message = serialize_message(message)?;

        let mut bytes = new_header(serialized_message.len()).to_vec();
        bytes.extend(serialized_message);

        self.stream.write_all(&bytes).await?;

        Ok(())
    }

    async fn call_stream_method<T>(
        &mut self,
        method_name: impl Into<String>,
        request: T,
    ) -> Result<(), async_std::io::Error>
    where
        T: prost::Message,
    {
        let message = ola_rpc::RpcMessage {
            r#type: ola_rpc::Type::StreamRequest as i32,
            id: Some(self.iterate_sequence() as u32),
            name: Some(method_name.into()),
            buffer: Some(serialize_message(request)?),
        };

        self.send_message(message).await
    }

    pub async fn send_dmx_data(
        &mut self,
        universe: i32,
        dmx_data: impl Into<Vec<u8>>,
    ) -> Result<(), async_std::io::Error> {
        let message = crate::ola::DmxData {
            universe: universe,
            data: dmx_data.into(),
            priority: Some(1),
        };

        self.call_stream_method("StreamDmxData", message).await
    }
}
