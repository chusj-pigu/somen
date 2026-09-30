//! Generates the FlatBuffers bindings used by somen-pod5.
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/io/flatbuffers/footer.fbs");

    let status = Command::new("flatc")
        .args([
            "--rust",
            "-o",
            std::env::var("OUT_DIR").unwrap().as_str(),
            "src/io/flatbuffers/footer.fbs",
        ])
        .status()
        .expect(
            "failed to execute `flatc`; please install the FlatBuffers compiler",
        );

    assert!(status.success());
}
