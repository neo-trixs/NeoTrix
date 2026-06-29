use std::path::Path;
use std::sync::OnceLock;

use super::{Plugin, PluginEvent};

static WASM_ENGINE: OnceLock<wasmtime::Engine> = OnceLock::new();

fn global_wasm_engine() -> &'static wasmtime::Engine {
    WASM_ENGINE.get_or_init(|| wasmtime::Engine::default())
}

pub struct WasmPluginWrapper {
    name: String,
    version: String,
    wasm_bytes: Vec<u8>,
}

impl WasmPluginWrapper {
    pub fn new(path: &Path) -> Result<Self, String> {
        let wasm_bytes =
            std::fs::read(path).map_err(|e| format!("Failed to read wasm file: {}", e))?;
        let engine = global_wasm_engine();
        let _module = wasmtime::Module::new(engine, &wasm_bytes)
            .map_err(|e| format!("Invalid wasm module: {}", e))?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let version = "0.1.0-wasm".to_string();

        Ok(Self {
            name,
            version,
            wasm_bytes,
        })
    }

    fn call_export(&self, func_name: &str, arg: &str) -> Result<String, String> {
        let engine = global_wasm_engine();
        let module = wasmtime::Module::new(engine, &self.wasm_bytes)
            .map_err(|e| format!("Module error: {}", e))?;

        let mut store = wasmtime::Store::new(engine, ());
        let linker = wasmtime::Linker::new(engine);

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| format!("Instantiation error: {}", e))?;

        let func = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, func_name)
            .map_err(|_| format!("Export '{}' not found in wasm plugin", func_name))?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or("No memory export")?;

        let input_bytes = arg.as_bytes();
        let input_len = input_bytes.len() as i32;
        let ptr = memory.data_mut(&mut store).len() as i32 - input_len - 1;
        memory.data_mut(&mut store)[ptr as usize..(ptr + input_len) as usize]
            .copy_from_slice(input_bytes);

        let _result_ptr = func
            .call(&mut store, (ptr, input_len))
            .map_err(|e| format!("Call error: {}", e))?;

        Ok(format!("wasm:{}({}):ok", func_name, arg))
    }
}

impl Plugin for WasmPluginWrapper {
    fn name(&self) -> &'static str {
        Box::leak(self.name.clone().into_boxed_str())
    }

    fn version(&self) -> &'static str {
        Box::leak(self.version.clone().into_boxed_str())
    }

    fn on_load(&self) -> Result<(), String> {
        self.call_export("_on_load", "")?;
        Ok(())
    }

    fn on_unload(&self) -> Result<(), String> {
        self.call_export("_on_unload", "")?;
        Ok(())
    }

    fn on_event(&self, event: &PluginEvent) -> Result<(), String> {
        self.call_export("_on_event", &format!("{}", event))?;
        Ok(())
    }
}
