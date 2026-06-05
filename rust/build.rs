fn main() {
    // On Android, the C++ runtime symbols (e.g. __cxa_pure_virtual) must be
    // linked statically because libc++_shared.so is not packaged in the APK.
    // Cargokit sets CARGO_ENCODED_RUSTFLAGS which overrides .cargo/config.toml,
    // so we use build.rs instead — cargo:rustc-link-lib/search are not overridden.
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "android" {
        println!("cargo:rustc-link-lib=static=c++_static");
        println!("cargo:rustc-link-lib=static=c++abi");

        // Cargokit sets CC_<target> to .../ndk/VERSION/toolchains/llvm/prebuilt/<host>/bin/clang.
        // Walk up to prebuilt/<host>/ and append sysroot/usr/lib/<arch> to get libc++_static.a.
        let target = std::env::var("TARGET").unwrap_or_default();
        let cc_key = format!("CC_{}", target.replace('-', "_"));
        if let Ok(cc) = std::env::var(&cc_key) {
            if let Some(prebuilt) = std::path::Path::new(&cc).parent().and_then(|b| b.parent()) {
                let sysroot_lib = prebuilt.join("sysroot/usr/lib").join(ndk_sysroot_arch(&target));
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
