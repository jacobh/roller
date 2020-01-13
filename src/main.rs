use ola::ola_server_service_client::OlaServerServiceClient;
use ola::DmxData;

pub mod ola {
    tonic::include_proto!("ola.proto");
}

fn pad_512(mut vec: Vec<u8>) -> Vec<u8> {
    while vec.len() < 512 {
        vec.push(0)
    }
    vec
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OlaServerServiceClient::connect("http://127.0.0.1:9010").await?;

    loop {
        println!("red");
        client.stream_dmx_data(DmxData {
            universe: 10,
            data: pad_512(vec![255, 0, 0, 0]),
            priority: Some(1),
        }).await?;

        std::thread::sleep(std::time::Duration::from_secs(1));

        println!("blue");
        client.stream_dmx_data(DmxData {
            universe: 10,
            data: pad_512(vec![0, 255, 0, 0]),
            priority: Some(1),
        }).await?;

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}
