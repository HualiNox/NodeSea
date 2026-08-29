const ENGINE_PROTO: &str = "proto/nodesea/v1/engine.proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Keep Cargo's watch set precise. Watching the whole proto directory can
    // cause unrelated generated/editor files to retrigger rust-analyzer.
    println!("cargo:rerun-if-changed={ENGINE_PROTO}");

    let proto_files = [ENGINE_PROTO];
    let includes = ["proto"];

    tonic_prost_build::configure()
        .emit_rerun_if_changed(false)
        .compile_protos(&proto_files, &includes)?;

    Ok(())
}
