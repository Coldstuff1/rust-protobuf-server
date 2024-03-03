use protobuf_codegen::{Codegen, Customize};

fn main() {
    Codegen::new()
        .protoc()
        .protoc_path(&protoc_bin_vendored::protoc_bin_path().unwrap())
        .customize(Customize::default().tokio_bytes(true))
        .includes(&["src/protos"])
        .input("src/protos/client.proto")
        .cargo_out_dir("protos")
        .run_from_script();

    println!("built!");
}
