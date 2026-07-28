fn main() {
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    let profile = std::env::var("PROFILE").unwrap_or_default();
    let is_release = profile == "release";

    let native = std::env::var("MFKEY_NATIVE")
        .map(|v| v == "1")
        .unwrap_or(false);
    println!("cargo:rerun-if-env-changed=MFKEY_NATIVE");

    let mut build = cc::Build::new();
    build.file("csrc/crypto1.c");
    build.include("csrc");

    if target_env == "msvc" {
        if is_release {
            build.flag_if_supported("/O2");
            build.flag_if_supported("/Oi");
            build.flag_if_supported("/Ot");
        }
        if native {
            build.flag_if_supported("/arch:AVX2");
        }
    } else {
        build.flag_if_supported("-std=c11");
        build.flag_if_supported("-Wall");

        if is_release {
            build.flag_if_supported("-O3");
            build.flag_if_supported("-funroll-loops");
            build.flag_if_supported("-fomit-frame-pointer");
            if target_os == "linux" {
                build.flag_if_supported("-fno-plt");
            }
        }
        if native {
            build.flag_if_supported("-march=native");
            build.flag_if_supported("-mtune=native");
        }
    }

    build.compile("crypto1");

    println!("cargo:rerun-if-changed=csrc/crypto1.c");
    println!("cargo:rerun-if-changed=csrc/crypto1.h");
    println!("cargo:rerun-if-changed=csrc/parity.h");

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

    cfg.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    cfg.compile_protos(&protos, &[proto_dir])
        .expect("prost-build failed");
}
