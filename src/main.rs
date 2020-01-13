pub mod ola {
    include!(concat!(env!("OUT_DIR"), "/ola.proto.rs"));
}

fn main() {
    println!("Hello, world!");
}
