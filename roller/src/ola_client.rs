use async_std::{
    net::{TcpStream, ToSocketAddrs},
    prelude::*,
};

mod ola {
    include!(concat!(env!("OUT_DIR"), "/ola.proto.rs"));
}

mod ola_rpc {
    include!(concat!(env!("OUT_DIR"), "/ola.rpc.rs"));
}

const PROTOCOL_VERSION: u32 = 1;
const VERSION_MASK: u32 = 0xf0000000;
const SIZE_MASK: u32 = 0x0fffffff;

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
}
impl OlaClient {
    pub async fn connect(host: impl ToSocketAddrs) -> Result<OlaClient, async_std::io::Error> {
        let stream = TcpStream::connect(host).await?;
        stream.set_nodelay(true)?;

        Ok(OlaClient {
            stream: stream,
            sequence: 0,
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
        let message = ola::DmxData {
            universe: universe,
            data: dmx_data.into(),
            priority: Some(1),
        };

        self.call_stream_method("StreamDmxData", message).await
    }
}
