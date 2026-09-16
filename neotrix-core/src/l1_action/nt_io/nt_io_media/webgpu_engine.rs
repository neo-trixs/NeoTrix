use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WebGPUConfig {
    pub device: DeviceType,
    pub backend: Backend,
    pub max_buffer_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Discrete,
    Integrated,
    Cpu,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    WebGpu,
    Vulkan,
    Metal,
    Dx12,
    Auto,
}

impl Default for WebGPUConfig {
    fn default() -> Self {
        Self {
            device: DeviceType::Auto,
            backend: Backend::Auto,
            max_buffer_size: 256 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComputePipeline {
    pub name: String,
    pub bind_group_layout: Vec<BindGroupEntry>,
    pub workgroup_size: [u32; 3],
}

#[derive(Debug, Clone)]
pub struct BindGroupEntry {
    pub binding: u32,
    pub visibility: u32,
    pub ty: BindingType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingType {
    Buffer,
    Texture,
    Sampler,
}

#[derive(Debug, Clone)]
pub struct ShaderModule {
    pub code: String,
    pub entry_point: String,
}

#[derive(Debug, Clone)]
pub struct BufferDescriptor {
    pub label: String,
    pub size: usize,
    pub usage: u32,
}

#[derive(Debug, Clone)]
pub struct GPUBuffer {
    pub id: u32,
    pub size: usize,
    pub usage: u32,
}

pub struct WebGPUEngine {
    config: WebGPUConfig,
    buffers: HashMap<String, GPUBuffer>,
    pipelines: HashMap<String, ComputePipeline>,
    initialized: bool,
}

impl WebGPUEngine {
    pub fn new(config: WebGPUConfig) -> Self {
        Self {
            config,
            buffers: HashMap::new(),
            pipelines: HashMap::new(),
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        // Simulate GPU initialization
        self.initialized = true;
        Ok(())
    }

    pub fn create_buffer(&mut self, desc: &BufferDescriptor) -> Result<GPUBuffer, String> {
        if !self.initialized {
            return Err("Engine not initialized".into());
        }
        if desc.size > self.config.max_buffer_size {
            return Err(format!(
                "Buffer size {} exceeds max {}",
                desc.size, self.config.max_buffer_size
            ));
        }
        let buffer = GPUBuffer {
            id: self.buffers.len() as u32,
            size: desc.size,
            usage: desc.usage,
        };
        self.buffers.insert(desc.label.clone(), buffer.clone());
        Ok(buffer)
    }

    pub fn create_shader(&self, module: &ShaderModule) -> Result<u32, String> {
        if !self.initialized {
            return Err("Engine not initialized".into());
        }
        if module.code.is_empty() {
            return Err("Empty shader code".into());
        }
        Ok(0) // Simulate shader compilation
    }

    pub fn create_compute_pipeline(&mut self, pipeline: ComputePipeline) -> Result<(), String> {
        if !self.initialized {
            return Err("Engine not initialized".into());
        }
        self.pipelines.insert(pipeline.name.clone(), pipeline);
        Ok(())
    }

    pub fn dispatch(&self, pipeline_name: &str, workgroups: [u32; 3]) -> Result<(), String> {
        if !self.initialized {
            return Err("Engine not initialized".into());
        }
        if !self.pipelines.contains_key(pipeline_name) {
            return Err(format!("Pipeline not found: {}", pipeline_name));
        }
        let _ = workgroups;
        Ok(())
    }

    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }

    pub fn pipeline_count(&self) -> usize {
        self.pipelines.len()
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn config(&self) -> &WebGPUConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let mut e = WebGPUEngine::new(WebGPUConfig::default());
        assert!(e.initialize().is_ok());
        assert!(e.is_initialized());
    }

    #[test]
    fn test_create_buffer() {
        let mut e = WebGPUEngine::new(WebGPUConfig::default());
        e.initialize().unwrap();
        let b = e
            .create_buffer(&BufferDescriptor {
                label: "test".into(),
                size: 1024,
                usage: 1,
            })
            .unwrap();
        assert_eq!(b.size, 1024);
    }

    #[test]
    fn test_buffer_too_large() {
        let mut e = WebGPUEngine::new(WebGPUConfig::default());
        e.initialize().unwrap();
        assert!(e
            .create_buffer(&BufferDescriptor {
                label: "big".into(),
                size: 512 * 1024 * 1024,
                usage: 1,
            })
            .is_err());
    }

    #[test]
    fn test_pipeline() {
        let mut e = WebGPUEngine::new(WebGPUConfig::default());
        e.initialize().unwrap();
        e.create_compute_pipeline(ComputePipeline {
            name: "add".into(),
            bind_group_layout: vec![],
            workgroup_size: [64, 1, 1],
        })
        .unwrap();
        assert!(e.dispatch("add", [1, 1, 1]).is_ok());
    }
}
