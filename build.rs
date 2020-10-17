extern crate prost_build;

fn main() {
    prost_build::compile_protos(&["proto/client.proto", "proto/server.proto"], &["proto"]).unwrap();
}
