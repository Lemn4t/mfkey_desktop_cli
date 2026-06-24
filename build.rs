use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    let mut build = cc::Build::new();
    build.file("csrc/crypto1.c");
    build.include("csrc");

    let profile = env::var("PROFILE").unwrap_or_default();
    let is_release = profile == "release";

    if target_env == "msvc" {
        if is_release {
            build.flag_if_supported("/O2");
        }
    } else {
        build.flag_if_supported("-std=c11");
        build.flag_if_supported("-Wall");
        if is_release {
            build.flag_if_supported("-O3");
            build.flag_if_supported("-funroll-loops");
        }
    }

    match target_os.as_str() {
        "macos" => {}
        "linux" => {}
        "windows" => {}
        _ => {}
    }

    build.compile("crypto1");

    println!("cargo:rerun-if-changed=csrc/crypto1.c");
    println!("cargo:rerun-if-changed=csrc/crypto1.h");
    println!("cargo:rerun-if-changed=csrc/parity.h");

    let proto_dir = Path::new("proto");
    if proto_dir.exists() {
        let mut protos: Vec<PathBuf> = Vec::new();
        for entry in std::fs::read_dir(proto_dir).expect("read proto dir") {
            let path = entry.expect("proto entry").path();
            if path.extension().and_then(|e| e.to_str()) == Some("proto") {
                protos.push(path);
            }
        }
        protos.sort();

        if !protos.is_empty() {
            println!("cargo:rerun-if-changed=proto");

            let file_descriptors = protox::compile(&protos, &[proto_dir])
                .expect("failed to compile flipper .proto files with protox");

            prost_build::Config::new()
                .compile_fds(file_descriptors)
                .expect("prost-build failed to generate code");
        }
    }
}
