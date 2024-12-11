fn main() {
    prost_build::compile_protos(&["src/plugin.proto"], &["src/"]).unwrap();
}
