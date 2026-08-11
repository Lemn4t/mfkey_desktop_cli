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
    build.file("csrc/crapto1.c");
    build.file("csrc/ffi_contract.c");
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

    build.compile("crapto1");

    println!("cargo:rerun-if-changed=csrc/crapto1.c");
    println!("cargo:rerun-if-changed=csrc/ffi_contract.c");
    println!("cargo:rerun-if-changed=csrc/hardnested_entry.h");
    println!("cargo:rerun-if-changed=csrc/crapto1.h");
    println!("cargo:rerun-if-changed=csrc/parity.h");

    let vendor = "csrc/hardnested_vendor";
    let mut hn = cc::Build::new();
    hn.warnings(false);
    hn.include(vendor);
    hn.include(format!("{vendor}/minlzlib"));
    hn.include(format!("{vendor}/pm3"));
    hn.file("csrc/hardnested_entry.c");
    hn.file(format!("{vendor}/parity.c"));
    hn.file(format!("{vendor}/hardnested.c"));
    hn.file(format!("{vendor}/hardnested/hardnested_bf_core.c"));
    hn.file(format!("{vendor}/hardnested/hardnested_bitarray_core.c"));
    hn.file(format!("{vendor}/hardnested/hardnested_bruteforce.c"));
    hn.file(format!("{vendor}/hardnested/tables.c"));
    hn.file(format!("{vendor}/pm3/util.c"));
    hn.file(format!("{vendor}/pm3/util_posix.c"));
    hn.file(format!("{vendor}/pm3/commonutil.c"));
    hn.file(format!("{vendor}/minlzlib/dictbuf.c"));
    hn.file(format!("{vendor}/minlzlib/inputbuf.c"));
    hn.file(format!("{vendor}/minlzlib/lzma2dec.c"));
    hn.file(format!("{vendor}/minlzlib/lzmadec.c"));
    hn.file(format!("{vendor}/minlzlib/rangedec.c"));
    hn.file(format!("{vendor}/minlzlib/xzcrc.c"));
    hn.file(format!("{vendor}/minlzlib/xzstream.c"));

    if target_env == "msvc" {
        hn.include("csrc/win_compat");
        hn.file("csrc/win_compat/pthread_shim.c");
        hn.flag("/FImsvc_compat.h");
        hn.define("__BIGGEST_ALIGNMENT__", "16");
        if is_release {
            hn.flag_if_supported("/O2");
        }
    } else {
        hn.flag_if_supported("-std=gnu11");
        if is_release {
            hn.flag_if_supported("-O3");
        }
    }

    hn.compile("hardnested");

    println!("cargo:rerun-if-changed={vendor}");
    println!("cargo:rerun-if-changed=csrc/win_compat/pthread_shim.c");
    println!("cargo:rerun-if-changed=csrc/win_compat/pthread.h");
}
