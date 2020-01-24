use async_std::net::{TcpStream, ToSocketAddrs};

fn pad_512(mut vec: Vec<u8>) -> Vec<u8> {
    while vec.len() < 512 {
        vec.push(0)
    }
    vec
}

pub mod ola {
    include!(concat!(env!("OUT_DIR"), "/ola.proto.rs"));
}

struct OlaClient {
    stream: TcpStream,
}
impl OlaClient {
    async fn connect(host: impl ToSocketAddrs) -> Result<OlaClient, async_std::io::Error> {
        Ok(OlaClient {
            stream: TcpStream::connect(host).await?,
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
