fn main() {
    // oboe-sys's cc::Build emits cargo:rustc-link-lib=c++_static (dynamic, the Cargo default),
    // but libc++_static.so doesn't exist in the Android NDK — only libc++_static.a does.
    // With -Wl,-Bdynamic the linker won't fall back to .a, so we:
    //   1. Add the NDK sysroot lib dir to the search path so the .a is reachable.
    //   2. Create stub .so linker scripts in OUT_DIR that redirect the dynamic
    //      request to the actual static archive (same trick Cargokit uses for libgcc).
    // Cargokit sets CARGO_ENCODED_RUSTFLAGS which overrides .cargo/config.toml,
    // so build.rs is the only place we can influence the link.
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "android" {
        let target = std::env::var("TARGET").unwrap_or_default();
        let clang = std::env::var("_CARGOKIT_NDK_LINK_CLANG")
            .or_else(|_| std::env::var(format!("CC_{}", target)));

        if let Ok(cc) = clang {
            if let Some(prebuilt) = std::path::Path::new(&cc).parent().and_then(|b| b.parent()) {
                let sysroot_lib = prebuilt.join("sysroot/usr/lib").join(ndk_sysroot_arch(&target));

                let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
                let stub_dir = std::path::Path::new(&out_dir).join("ndk_stubs");
                std::fs::create_dir_all(&stub_dir).ok();

                // libc++_static.a references __cxa_pure_virtual which is defined in
                // libc++abi.a (not inside libc++_static.a). Use GROUP() to pull both
                // archives together so the cross-reference resolves statically.
                let static_a = sysroot_lib.join("libc++_static.a");
                let abi_a = sysroot_lib.join("libc++abi.a");
                if static_a.exists() {
                    let stub = stub_dir.join("libc++_static.so");
                    let content = if abi_a.exists() {
                        format!("GROUP({} {})\n", static_a.display(), abi_a.display())
                    } else {
                        format!("INPUT({})\n", static_a.display())
                    };
                    std::fs::write(&stub, content).ok();
                }
                // Standalone libc++abi stub for targets that link it directly.
                if abi_a.exists() {
                    let stub = stub_dir.join("libc++abi.so");
                    std::fs::write(&stub, format!("INPUT({})\n", abi_a.display())).ok();
                }

                // Add versioned sysroot subdirs (21, 22, … 35) BEFORE the parent dir.
                // The parent dir has libc.a but no libc.so stub; the .so stubs live in
                // the versioned subdirs. Without this, -lc with -Wl,-Bdynamic falls back
                // to static libc.a which contains non-PIC x86_64 code (NDK 28 regression).
                if let Ok(entries) = std::fs::read_dir(&sysroot_lib) {
                    let mut api_dirs: Vec<u32> = entries
                        .flatten()
                        .filter(|e| e.path().is_dir())
                        .filter_map(|e| e.file_name().to_string_lossy().parse::<u32>().ok())
                        .collect();
                    api_dirs.sort();
                    if let Some(min_api) = api_dirs.first() {
                        println!("cargo:rustc-link-search=native={}", sysroot_lib.join(min_api.to_string()).display());
                    }
                }

                // Stub dir must come before sysroot so the .so scripts are found first.
                println!("cargo:rustc-link-search=native={}", stub_dir.display());
                println!("cargo:rustc-link-search=native={}", sysroot_lib.display());
            }
        }

    }
}

fn ndk_sysroot_arch(target: &str) -> &'static str {
    if target.starts_with("armv7") {
        "arm-linux-androideabi"
    } else if target.starts_with("aarch64") {
        "aarch64-linux-android"
    } else if target.starts_with("i686") {
        "i686-linux-android"
    } else if target.starts_with("x86_64-linux-android") {
        "x86_64-linux-android"
    } else {
        "aarch64-linux-android"
    }
}
