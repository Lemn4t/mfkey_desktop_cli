use std::env;

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
        build.define("MFKEY_NO_BUILTIN_PARITY", None);
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
}
