use async_std::net::{TcpStream, ToSocketAddrs};
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
        Ok(OlaClient {
            stream: TcpStream::connect(host).await?,
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
