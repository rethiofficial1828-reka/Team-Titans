// build.rs for the shared crate
// Exports all ts-rs derived types to app/src/bindings/ during `cargo build`
use std::path::PathBuf;

fn main() {
    // Tell cargo to re-run this script if any source file changes
    println!("cargo:rerun-if-changed=src/");

    // Output directory for generated TypeScript bindings
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("app")
        .join("src")
        .join("bindings");

    std::fs::create_dir_all(&out_dir).expect("Failed to create bindings output dir");

    // ts-rs v10: set TS_RS_EXPORT_DIR so that types marked #[ts(export)]
    // are written to the correct directory at compile time.
    // Note: this env var is picked up by ts-rs at compile time via the build script.
    std::env::set_var("TS_RS_EXPORT_DIR", out_dir.to_str().unwrap());

    println!(
        "cargo:rustc-env=TS_RS_EXPORT_DIR={}",
        out_dir.display()
    );
}
