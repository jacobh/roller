use async_std::prelude::*;

mod ola;
mod ola_client;
mod fixture;

fn pad_512(mut vec: Vec<u8>) -> Vec<u8> {
    while vec.len() < 512 {
        vec.push(0)
    }
    vec
}

#[async_std::main]
async fn main() -> Result<(), async_std::io::Error> {
    let red = vec![255, 0, 0, 0];
    let blue = vec![0, 0, 255, 0];

    let mut ola_client = ola_client::OlaClient::connect_localhost().await?;

    loop {
        println!("red");
        ola_client.send_dmx_data(10, red.clone()).await?;
        async_std::task::sleep(std::time::Duration::from_millis(100)).await;

        println!("blue");
        ola_client.send_dmx_data(10, blue.clone()).await?;
        async_std::task::sleep(std::time::Duration::from_millis(100)).await;
    }

    Ok(())
}
