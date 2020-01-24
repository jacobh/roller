fn main() {
    prost_build::compile_protos(&["src/ola.proto", "src/ola.rpc.proto"], &["src/"]).unwrap();
}
