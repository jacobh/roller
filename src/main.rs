use async_std::{
    net::{TcpStream, ToSocketAddrs},
    prelude::*,
};
use byteorder::{ByteOrder, LittleEndian};
use prost::Message;
use std::collections::HashMap;

fn pad_512(mut vec: Vec<u8>) -> Vec<u8> {
    while vec.len() < 512 {
        vec.push(0)
    }
    vec
}

pub mod ola {
    include!(concat!(env!("OUT_DIR"), "/ola.proto.rs"));
}

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

    let mut buf = [0; 4];
    LittleEndian::write_u32(&mut buf, header_u32);
    buf
}

struct OlaClient {
    stream: TcpStream,
    sequence: usize,
    outstanding_requests: HashMap<usize, Box<dyn prost::Message>>,
    outstanding_responses: HashMap<usize, Box<dyn prost::Message>>,
    received_bytes_buffer: Vec<u8>,
    current_message_expected_size: Option<usize>,
    skip_current_message: bool,
}
impl OlaClient {
    async fn connect(host: impl ToSocketAddrs) -> Result<OlaClient, async_std::io::Error> {
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
    async fn connect_localhost() -> Result<OlaClient, async_std::io::Error> {
        OlaClient::connect("localhost:9010").await
    }

    async fn send_message(
        &mut self,
        message: ola_rpc::RpcMessage,
    ) -> Result<(), async_std::io::Error> {
        let serialized_message = {
            let mut buf = Vec::<u8>::new();
            message.encode(&mut buf)?;
            buf
        };

        let mut bytes = new_header(serialized_message.len()).to_vec();
        bytes.extend(serialized_message);

        self.stream.write_all(&bytes).await?;

        Ok(())
    }
}

fn main() {
    let _red = ola::DmxData {
        universe: 10,
        data: pad_512(vec![255, 0, 0, 0]),
        priority: Some(1),
    };

    let _blue = ola::DmxData {
        universe: 10,
        data: pad_512(vec![0, 0, 255, 0]),
        priority: Some(1),
    };

    println!("Hello, world!");
}
