//! WASM plugin loader — `WasmPlugin` wraps a compiled `wasmtime` module as a `Plugin`.
//!
//! Activated behind `#[cfg(feature = "sandbox")]`. When the feature is off the module
//! body is empty and `PluginRegistry::load_wasm` returns a friendly error at runtime.

#![cfg(feature = "sandbox")]

use std::path::Path;

use wasmtime::{Engine, Instance, Module, Store};

use super::types::{PluginInfo, PluginState};
use super::Plugin;

/// A plugin backed by a compiled WebAssembly module.
pub struct WasmPlugin {
    name: String,
    version: String,
    description: String,
    state: PluginState,
    pub(crate) engine: Engine,
    pub(crate) module: Module,
    store: Option<Store<()>>,
    instance: Option<Instance>,
}

impl std::fmt::Debug for WasmPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmPlugin")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("description", &self.description)
            .field("state", &self.state)
            .finish()
    }
}

impl WasmPlugin {
    /// Read, compile and inspect a `.wasm` file, returning a plugin in `Unloaded` state.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let engine = Engine::default();
        let wasm_bytes =
            std::fs::read(path).map_err(|e| format!("failed to read WASM file: {}", e))?;
        let module = Module::from_binary(&engine, &wasm_bytes)
            .map_err(|e| format!("failed to compile WASM module: {}", e))?;

        let name = module
            .name()
            .filter(|n| !n.is_empty())
            .map(|n| n.to_string())
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unnamed")
                    .to_string()
            });

        let (version, description) = extract_metadata(&wasm_bytes);

        Ok(Self {
            name,
            version,
            description,
            state: PluginState::Unloaded,
            engine,
            module,
            store: None,
            instance: None,
        })
    }

    fn ensure_metadata(&self) -> (&str, &str, &str) {
        (&self.name, &self.version, &self.description)
    }
}

impl Plugin for WasmPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn state(&self) -> PluginState {
        self.state.clone()
    }

    fn load(&mut self) -> Result<(), String> {
        let mut store = Store::new(&self.engine, ());
        let instance = Instance::new(&mut store, &self.module, &[])
            .map_err(|e| format!("failed to instantiate WASM module: {}", e))?;
        self.store = Some(store);
        self.instance = Some(instance);
        self.state = PluginState::Loaded;
        log::info!(
            "[wasm-plugin] loaded: {} v{} — {}",
            self.name,
            self.version,
            self.description
        );
        Ok(())
    }

    fn unload(&mut self) -> Result<(), String> {
        self.store = None;
        self.instance = None;
        self.state = PluginState::Unloaded;
        log::info!("[wasm-plugin] unloaded: {}", self.name);
        Ok(())
    }
}

// ── metadata helpers ──────────────────────────────────────────────────────────

/// Extract `(version, description)` from the module's custom sections.
///
/// Looks for a custom section named `neotrix_meta` whose payload is a tiny JSON
/// object `{"version":"…","description":"…"}`.  Falls back to sensible defaults.
///
/// Parses the raw wasm binary directly (wasmtime 24 removed the `custom_sections` API).
fn extract_metadata(wasm_bytes: &[u8]) -> (String, String) {
    let default_desc = "WASM plugin".to_string();
    let default_ver = "0.1.0".to_string();

    fn read_leb128(bytes: &[u8], pos: &mut usize) -> Option<u32> {
        let mut result = 0u32;
        let mut shift = 0;
        loop {
            let byte = *bytes.get(*pos)?;
            *pos += 1;
            result |= ((byte & 0x7f) as u32) << shift;
            if (byte & 0x80) == 0 {
                return Some(result);
            }
            shift += 7;
            if shift > 28 {
                return None;
            }
        }
    }

    let mut pos = 8; // skip magic (4) + version (4)
    while pos < wasm_bytes.len() {
        let section_id = match wasm_bytes.get(pos) {
            Some(&b) => b,
            None => break,
        };
        pos += 1;
        let section_size = match read_leb128(wasm_bytes, &mut pos) {
            Some(s) => s as usize,
            None => break,
        };
        let section_start = pos;

        if section_id == 0 {
            let name_len = match read_leb128(wasm_bytes, &mut pos) {
                Some(n) => n as usize,
                None => break,
            };
            if pos + name_len <= wasm_bytes.len() {
                let name = &wasm_bytes[pos..pos + name_len];
                pos += name_len;
                if name == b"neotrix_meta" {
                    let data = &wasm_bytes[pos..section_start + section_size];
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(data) {
                        let ver = val
                            .get("version")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let desc = val
                            .get("description")
                            .and_then(|d| d.as_str())
                            .map(String::from);
                        return (ver.unwrap_or(default_ver), desc.unwrap_or(default_desc));
                    }
                }
            }
        }
        pos = section_start + section_size;
    }

    (default_ver, default_desc)
}

// ── PluginInfo helper ─────────────────────────────────────────────────────────

impl From<&WasmPlugin> for PluginInfo {
    fn from(p: &WasmPlugin) -> Self {
        Self {
            name: p.name.clone(),
            version: p.version.clone(),
            description: p.description.clone(),
            state: p.state.clone(),
            source: "wasm".to_string(),
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn stray_bytes() -> Vec<u8> {
        b"\x00asm\x01\x00\x00\x00".to_vec()
    }

    #[test]
    fn test_from_file_nonexistent() {
        let err = WasmPlugin::from_file(Path::new("/tmp/__nope__/nonexistent.wasm")).unwrap_err();
        assert!(
            err.contains("failed to compile") || err.contains("No such file"),
            "got: {err}"
        );
    }

    #[test]
    fn test_from_file_not_wasm() {
        let dir = std::env::temp_dir().join(format!("wasm_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("not_a_wasm.txt");
        std::fs::write(&f, b"hello, i am not wasm").unwrap();
        let err = WasmPlugin::from_file(&f).unwrap_err();
        assert!(
            err.contains("failed to compile") || err.contains("magic header"),
            "got: {err}"
        );
        drop(std::fs::remove_dir_all(&dir));
    }

    #[test]
    fn test_from_stray_bytes_rejected() {
        // Write a valid header but invalid module
        let dir = std::env::temp_dir().join(format!("wasm_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("stray.wasm");
        {
            let mut file = std::fs::File::create(&f).unwrap();
            file.write_all(&stray_bytes()).unwrap();
        }
        let err = WasmPlugin::from_file(&f).unwrap_err();
        assert!(err.contains("failed to compile"), "got: {err}");
        drop(std::fs::remove_dir_all(&dir));
    }

    #[test]
    fn test_valid_minimal_wasm_roundtrip() {
        // The smallest valid WASM module: an empty module with one function export.
        // WAT: (module (func (export "e")))
        let wasm_bytes: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, // magic \0asm
            0x01, 0x00, 0x00, 0x00, // version 1
            0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // type section: func ()
            0x03, 0x02, 0x01, 0x00, // function section: 1 func (type 0)
            0x07, 0x05, 0x01, 0x01, 0x65, 0x00, 0x00, // export section: "e" func 0
        ];

        let dir = std::env::temp_dir().join(format!("wasm_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("hello.wasm");
        std::fs::write(&f, &wasm_bytes).unwrap();

        let mut plugin = WasmPlugin::from_file(&f).unwrap();
        assert_eq!(plugin.name(), "hello");
        assert_eq!(plugin.version(), "0.1.0");
        assert_eq!(plugin.state(), PluginState::Unloaded);

        plugin.load().unwrap();
        assert_eq!(plugin.state(), PluginState::Loaded);

        plugin.unload().unwrap();
        assert_eq!(plugin.state(), PluginState::Unloaded);

        drop(std::fs::remove_dir_all(&dir));
    }

    #[test]
    fn test_plugin_registry_load_wasm_integration() {
        use crate::core::nt_core_plugin::PluginRegistry;

        let wasm_bytes: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x05, 0x01, 0x01, 0x65, 0x00, 0x00,
        ];

        let dir = std::env::temp_dir().join(format!("wasm_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("integration_test.wasm");
        std::fs::write(&f, &wasm_bytes).unwrap();

        let mut reg = PluginRegistry::new();
        reg.load_wasm(&f).unwrap();
        assert_eq!(reg.len(), 1);

        let info = reg.info("integration_test").unwrap();
        assert_eq!(info.source, "wasm");
        assert_eq!(info.state, PluginState::Loaded);

        reg.unregister("integration_test").unwrap();
        assert!(reg.is_empty());

        drop(std::fs::remove_dir_all(&dir));
    }
}
