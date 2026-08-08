// uniffi-bindgen CLI entry point for NeoTrix
// Generates Swift/Kotlin bindings from the proc-macro metadata.
// Usage: cargo run -p neotrix --bin uniffi-bindgen -- generate --library <path-to-cdylib> --language swift --out-dir <dir>

fn main() {
    uniffi::uniffi_bindgen_main()
}