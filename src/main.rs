use async_std::prelude::*;

mod ola;
mod ola_client;

fn pad_512(mut vec: Vec<u8>) -> Vec<u8> {
    while vec.len() < 512 {
        vec.push(0)
    }
    vec
}

#[async_std::main]
async fn main() -> Result<(), async_std::io::Error> {
    let red = crate::ola::DmxData {
        universe: 10,
        data: pad_512(vec![255, 0, 0, 0]),
        priority: Some(1),
    };

    let blue = crate::ola::DmxData {
        universe: 10,
        data: pad_512(vec![0, 0, 255, 0]),
        priority: Some(1),
    };

    let mut ola_client = ola_client::OlaClient::connect_localhost().await?;

    loop {
        println!("red");
        ola_client
            .call_stream_method("StreamDmxData", red.clone())
            .await?;
        async_std::task::sleep(std::time::Duration::from_millis(100)).await;

        println!("blue");
        ola_client
            .call_stream_method("StreamDmxData", blue.clone())
            .await?;
        async_std::task::sleep(std::time::Duration::from_millis(100)).await;
    }

    Ok(())
}
