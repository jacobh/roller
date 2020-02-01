fn main() {
    prost_build::compile_protos(
        &["src/proto/ola.proto", "src/proto/ola.rpc.proto"],
        &["src/"],
    )
    .unwrap();
}
