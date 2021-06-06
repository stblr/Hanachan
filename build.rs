use std::env;

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    if target_arch == "aarch64" {
        println!("cargo:rerun-if-changed=src/enable_ftz_aarch64.s");
        cc::Build::new()
            .file("src/enable_ftz_aarch64.s")
            .compile("enable_ftz_aarch64");
    }
}
