// NeoTrix build.rs
// iOS bridge uses uniffi proc-macros (#[uniffi::export], #[derive(uniffi::Record)])
// No UDL scaffolding generation needed — proc-macro metadata is compiled into the lib.

fn main() {
    // The UDL file is kept as documentation of the FFI surface.
    // Actual bindings are generated from proc-macro metadata via:
    //   uniffi-bindgen-swift --library target/.../libneotrix.a
    println!("cargo:rerun-if-changed=src/neotrix/ffi/neotrix.udl");
}