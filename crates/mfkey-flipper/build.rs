fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path()
        .expect("protoc-bin-vendored: no binary for this platform");

    let proto_dir = std::path::PathBuf::from("proto");

    let protos: Vec<_> = [
        "flipper.proto",
        "storage.proto",
        "system.proto",
        "application.proto",
        "gui.proto",
        "gpio.proto",
        "property.proto",
        "desktop.proto",
    ]
    .iter()
    .map(|f| proto_dir.join(f))
    .collect();

    for p in &protos {
        println!("cargo:rerun-if-changed={}", p.display());
    }

    let mut cfg = prost_build::Config::new();

    cfg.protoc_executable(protoc);

    cfg.compile_protos(&protos, &[proto_dir])
        .expect("prost-build failed");
}
