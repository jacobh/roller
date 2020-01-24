fn pad_512(mut vec: Vec<u8>) -> Vec<u8> {
    while vec.len() < 512 {
        vec.push(0)
    }
    vec
}

pub mod ola {
    include!(concat!(env!("OUT_DIR"), "/ola.proto.rs"));
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
