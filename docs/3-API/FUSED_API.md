# NeoTrix 融合架构 API 文档

## 一、概述

NeoTrix 融合架构 API 提供了"能量→频率→震动→显化"意识心态的接口，支持系统意识管理、能量分配、频率共振、震动执行和显化输出。

## 二、核心 API

### 2.1 FusedArchitecture

#### 创建实例

```rust
pub fn new() -> Self
```

**描述**：创建融合架构实例

**返回值**：`FusedArchitecture` - 新的融合架构实例

**示例**：

```rust
use neotrix::core::fused_architecture::FusedArchitecture;

let arch = FusedArchitecture::new();
```

#### 初始化

```rust
pub async fn init(&mut self) -> Result<(), String>
```

**描述**：初始化融合架构，包括所有子组件的初始化

**返回值**：`Result<(), String>` - 成功返回 `()`，失败返回错误信息

**示例**：

```rust
let mut arch = FusedArchitecture::new();
arch.init().await?;
```

#### 处理请求

```rust
pub async fn process_request(&mut self, request: &str) -> Result<String, String>
```

**描述**：处理用户请求，执行完整的能量→频率→震动→显化流程

**参数**：
- `request: &str` - 用户请求字符串

**返回值**：`Result<String, String>` - 成功返回处理结果，失败返回错误信息

**示例**：

```rust
let result = arch.process_request("分析这段代码").await?;
println!("结果: {}", result);
```

#### 获取系统状态

```rust
pub async fn get_system_status(&self) -> SystemStatus
```

**描述**：获取系统当前状态信息

**返回值**：`SystemStatus` - 系统状态结构体

**示例**：

```rust
let status = arch.get_system_status().await;
println!("版本: {}", status.version);
println!("意识健康度: {}", status.consciousness_health);
println!("能量水平: {}", status.energy_level);
```

### 2.2 SystemStatus

```rust
pub struct SystemStatus {
    pub version: String,
    pub consciousness_health: f64,
    pub energy_level: f64,
    pub frequency_stability: f64,
    pub vibration_intensity: f64,
    pub manifestation_quality: f64,
    pub capability_count: usize,
    pub skill_count: usize,
}
```

**字段说明**：

| 字段 | 类型 | 描述 |
|------|------|------|
| `version` | `String` | 系统版本号 |
| `consciousness_health` | `f64` | 意识健康度 (0.0-1.0) |
| `energy_level` | `f64` | 能量水平 (0.0-1.0) |
| `frequency_stability` | `f64` | 频率稳定性 (0.0-1.0) |
| `vibration_intensity` | `f64` | 震动强度 (0.0-1.0) |
| `manifestation_quality` | `f64` | 显化质量 (0.0-1.0) |
| `capability_count` | `usize` | 能力数量 |
| `skill_count` | `usize` | 技能数量 |

## 三、意识核心 API

### 3.1 ConsciousnessCore

#### 创建实例

```rust
pub fn new() -> Self
```

**描述**：创建意识核心实例

**返回值**：`ConsciousnessCore` - 新的意识核心实例

#### 分析请求

```rust
pub async fn analyze_request(&self, request: &str) -> Result<RequestAnalysis, String>
```

**描述**：分析用户请求，提取意图和需求

**参数**：
- `request: &str` - 用户请求字符串

**返回值**：`Result<RequestAnalysis, String>` - 成功返回请求分析结果，失败返回错误信息

#### 健康检查

```rust
pub async fn health(&self) -> f64
```

**描述**：检查意识核心健康状态

**返回值**：`f64` - 健康度 (0.0-1.0)

### 3.2 RequestAnalysis

```rust
pub struct RequestAnalysis {
    pub request_type: String,
    pub complexity: f64,
    pub required_capabilities: Vec<String>,
    pub estimated_energy: f64,
}
```

**字段说明**：

| 字段 | 类型 | 描述 |
|------|------|------|
| `request_type` | `String` | 请求类型 |
| `complexity` | `f64` | 复杂度 (0.0-1.0) |
| `required_capabilities` | `Vec<String>` | 所需能力列表 |
| `estimated_energy` | `f64` | 预估能量消耗 (0.0-1.0) |

## 四、能量层 API

### 4.1 EnergyLayer

#### 创建实例

```rust
pub fn new() -> Self
```

**描述**：创建能量层实例

**返回值**：`EnergyLayer` - 新的能量层实例

#### 初始化

```rust
pub async fn init(&mut self) -> Result<(), String>
```

**描述**：初始化能量层

**返回值**：`Result<(), String>` - 成功返回 `()`，失败返回错误信息

#### 评估能量需求

```rust
pub async fn assess_requirement(&mut self, analysis: &RequestAnalysis) -> Result<EnergyRequirement, String>
```

**描述**：根据请求分析评估能量需求

**参数**：
- `analysis: &RequestAnalysis` - 请求分析结果

**返回值**：`Result<EnergyRequirement, String>` - 成功返回能量需求，失败返回错误信息

#### 获取能量水平

```rust
pub async fn get_energy_level(&self) -> f64
```

**描述**：获取当前能量水平

**返回值**：`f64` - 能量水平 (0.0-1.0)

### 4.2 EnergyRequirement

```rust
pub struct EnergyRequirement {
    pub energy_amount: f64,
    pub energy_type: String,
    pub priority: u32,
}
```

**字段说明**：

| 字段 | 类型 | 描述 |
|------|------|------|
| `energy_amount` | `f64` | 能量数量 (0.0-1.0) |
| `energy_type` | `String` | 能量类型 |
| `priority` | `u32` | 优先级 |

## 五、频率层 API

### 5.1 FrequencyLayer

#### 创建实例

```rust
pub fn new() -> Self
```

**描述**：创建频率层实例

**返回值**：`FrequencyLayer` - 新的频率层实例

#### 初始化

```rust
pub async fn init(&mut self) -> Result<(), String>
```

**描述**：初始化频率层

**返回值**：`Result<(), String>` - 成功返回 `()`，失败返回错误信息

#### 选择频率

```rust
pub async fn select_frequency(&mut self, requirement: &EnergyRequirement) -> Result<Frequency, String>
```

**描述**：根据能量需求选择合适的频率

**参数**：
- `requirement: &EnergyRequirement` - 能量需求

**返回值**：`Result<Frequency, String>` - 成功返回频率，失败返回错误信息

#### 获取频率稳定性

```rust
pub async fn get_stability(&self) -> f64
```

**描述**：获取当前频率稳定性

**返回值**：`f64` - 频率稳定性 (0.0-1.0)

### 5.2 Frequency

```rust
pub struct Frequency {
    pub value: f64,
    pub stability: f64,
    pub resonance: f64,
}
```

**字段说明**：

| 字段 | 类型 | 描述 |
|------|------|------|
| `value` | `f64` | 频率值 (0.0-1.0) |
| `stability` | `f64` | 稳定性 (0.0-1.0) |
| `resonance` | `f64` | 共振强度 (0.0-1.0) |

## 六、震动层 API

### 6.1 VibrationLayer

#### 创建实例

```rust
pub fn new() -> Self
```

**描述**：创建震动层实例

**返回值**：`VibrationLayer` - 新的震动层实例

#### 初始化

```rust
pub async fn init(&mut self) -> Result<(), String>
```

**描述**：初始化震动层

**返回值**：`Result<(), String>` - 成功返回 `()`，失败返回错误信息

#### 执行震动

```rust
pub async fn execute_vibration(&mut self, frequency: &Frequency) -> Result<VibrationResult, String>
```

**描述**：根据频率执行震动

**参数**：
- `frequency: &Frequency` - 频率

**返回值**：`Result<VibrationResult, String>` - 成功返回震动结果，失败返回错误信息

#### 获取震动强度

```rust
pub async fn get_intensity(&self) -> f64
```

**描述**：获取当前震动强度

**返回值**：`f64` - 震动强度 (0.0-1.0)

### 6.2 VibrationResult

```rust
pub struct VibrationResult {
    pub success: bool,
    pub vibration_intensity: f64,
    pub effects: Vec<String>,
    pub execution_time_ms: u64,
}
```

**字段说明**：

| 字段 | 类型 | 描述 |
|------|------|------|
| `success` | `bool` | 是否成功 |
| `vibration_intensity` | `f64` | 震动强度 (0.0-1.0) |
| `effects` | `Vec<String>` | 效果列表 |
| `execution_time_ms` | `u64` | 执行时间 (ms) |

## 七、显化层 API

### 7.1 ManifestationLayer

#### 创建实例

```rust
pub fn new() -> Self
```

**描述**：创建显化层实例

**返回值**：`ManifestationLayer` - 新的显化层实例

#### 初始化

```rust
pub async fn init(&mut self) -> Result<(), String>
```

**描述**：初始化显化层

**返回值**：`Result<(), String>` - 成功返回 `()`，失败返回错误信息

#### 显化结果

```rust
pub async fn manifest(&mut self, vibration_result: &VibrationResult) -> Result<String, String>
```

**描述**：将震动结果显化为最终输出

**参数**：
- `vibration_result: &VibrationResult` - 震动结果

**返回值**：`Result<String, String>` - 成功返回显化结果，失败返回错误信息

#### 获取显化质量

```rust
pub async fn get_quality(&self) -> f64
```

**描述**：获取当前显化质量

**返回值**：`f64` - 显化质量 (0.0-1.0)

## 八、能力网 API

### 8.1 CapabilityNetwork

#### 创建实例

```rust
pub fn new() -> Self
```

**描述**：创建能力网实例

**返回值**：`CapabilityNetwork` - 新的能力网实例

#### 初始化

```rust
pub async fn init(&mut self) -> Result<(), String>
```

**描述**：初始化能力网

**返回值**：`Result<(), String>` - 成功返回 `()`，失败返回错误信息

#### 注册能力

```rust
pub async fn register_capability(&mut self, capability: CapabilityEntry) -> Result<(), String>
```

**描述**：注册新能力

**参数**：
- `capability: CapabilityEntry` - 能力条目

**返回值**：`Result<(), String>` - 成功返回 `()`，失败返回错误信息

#### 查询能力

```rust
pub async fn query_capability(&self, capability_id: &str) -> Result<Option<CapabilityEntry>, String>
```

**描述**：查询能力

**参数**：
- `capability_id: &str` - 能力ID

**返回值**：`Result<Option<CapabilityEntry>, String>` - 成功返回能力条目（如果存在），失败返回错误信息

### 8.2 CapabilityEntry

```rust
pub struct CapabilityEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub capability_type: CapabilityType,
    pub status: CapabilityStatus,
    pub energy_requirement: f64,
    pub frequency: f64,
    pub vibration_pattern: String,
}
```

**字段说明**：

| 字段 | 类型 | 描述 |
|------|------|------|
| `id` | `String` | 能力ID |
| `name` | `String` | 能力名称 |
| `description` | `String` | 能力描述 |
| `version` | `String` | 能力版本 |
| `capability_type` | `CapabilityType` | 能力类型 |
| `status` | `CapabilityStatus` | 能力状态 |
| `energy_requirement` | `f64` | 能量需求 (0.0-1.0) |
| `frequency` | `f64` | 频率 (0.0-1.0) |
| `vibration_pattern` | `String` | 震动模式 |

## 九、技能生态 API

### 9.1 SkillEcosystem

#### 创建实例

```rust
pub fn new() -> Self
```

**描述**：创建技能生态实例

**返回值**：`SkillEcosystem` - 新的技能生态实例

#### 初始化

```rust
pub async fn init(&mut self) -> Result<(), String>
```

**描述**：初始化技能生态

**返回值**：`Result<(), String>` - 成功返回 `()`，失败返回错误信息

#### 构建技能

```rust
pub async fn build_skill(&mut self, skill: SkillDefinition) -> Result<(), String>
```

**描述**：构建新技能

**参数**：
- `skill: SkillDefinition` - 技能定义

**返回值**：`Result<(), String>` - 成功返回 `()`，失败返回错误信息

#### 查询技能

```rust
pub async fn query_skill(&self, skill_id: &str) -> Result<Option<SkillEntry>, String>
```

**描述**：查询技能

**参数**：
- `skill_id: &str` - 技能ID

**返回值**：`Result<Option<SkillEntry>, String>` - 成功返回技能条目（如果存在），失败返回错误信息

### 9.2 SkillDefinition

```rust
pub struct SkillDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub skill_type: SkillType,
    pub required_capabilities: Vec<String>,
    pub energy_cost: f64,
    pub expected_performance_gain: f64,
}
```

**字段说明**：

| 字段 | 类型 | 描述 |
|------|------|------|
| `id` | `String` | 技能ID |
| `name` | `String` | 技能名称 |
| `description` | `String` | 技能描述 |
| `skill_type` | `SkillType` | 技能类型 |
| `required_capabilities` | `Vec<String>` | 所需能力列表 |
| `energy_cost` | `f64` | 能量成本 (0.0-1.0) |
| `expected_performance_gain` | `f64` | 预期性能提升 (0.0-1.0) |

## 十、智慧集成 API

### 10.1 WisdomIntegration

#### 累积智慧

```rust
pub async fn accumulate_wisdom(&mut self, wisdom: WisdomEntry) -> Result<(), String>
```

**描述**：累积智慧条目

**参数**：
- `wisdom: WisdomEntry` - 智慧条目

**返回值**：`Result<(), String>` - 成功返回 `()`，失败返回错误信息

#### 检索智慧

```rust
pub async fn retrieve_wisdom(&self, domain: &str) -> Result<Vec<WisdomEntry>, String>
```

**描述**：按领域检索智慧

**参数**：
- `domain: &str` - 领域

**返回值**：`Result<Vec<WisdomEntry>, String>` - 成功返回智慧条目列表，失败返回错误信息

### 10.2 WisdomEntry

```rust
pub struct WisdomEntry {
    pub id: String,
    pub source: String,
    pub content: String,
    pub confidence: f64,
    pub domain: String,
    pub timestamp: i64,
}
```

**字段说明**：

| 字段 | 类型 | 描述 |
|------|------|------|
| `id` | `String` | 智慧ID |
| `source` | `String` | 来源 |
| `content` | `String` | 内容 |
| `confidence` | `f64` | 置信度 (0.0-1.0) |
| `domain` | `String` | 领域 |
| `timestamp` | `i64` | 时间戳 |

## 十一、进化机制 API

### 11.1 EvolutionMechanism

#### 触发进化

```rust
pub async fn trigger_evolution(&mut self, evolution_type: &str) -> Result<(), String>
```

**描述**：触发进化

**参数**：
- `evolution_type: &str` - 进化类型

**返回值**：`Result<(), String>` - 成功返回 `()`，失败返回错误信息

#### 获取进化状态

```rust
pub async fn get_evolution_status(&self) -> Result<EvolutionStatus, String>
```

**描述**：获取进化状态

**返回值**：`Result<EvolutionStatus, String>` - 成功返回进化状态，失败返回错误信息

### 11.2 EvolutionStatus

```rust
pub struct EvolutionStatus {
    pub current_phase: String,
    pub progress: f64,
    pub last_evolution: i64,
    pub next_evolution: i64,
}
```

**字段说明**：

| 字段 | 类型 | 描述 |
|------|------|------|
| `current_phase` | `String` | 当前阶段 |
| `progress` | `f64` | 进度 (0.0-1.0) |
| `last_evolution` | `i64` | 上次进化时间戳 |
| `next_evolution` | `i64` | 下次进化时间戳 |

## 十二、集成 API

### 12.1 FusedArchitectureIntegration

#### 创建实例

```rust
pub fn new() -> Self
```

**描述**：创建集成实例

**返回值**：`FusedArchitectureIntegration` - 新的集成实例

#### 初始化

```rust
pub async fn initialize(&self) -> Result<(), String>
```

**描述**：初始化集成

**返回值**：`Result<(), String>` - 成功返回 `()`，失败返回错误信息

#### 处理请求

```rust
pub async fn process_request(&self, request: &str) -> Result<String, String>
```

**描述**：通过集成处理请求

**参数**：
- `request: &str` - 用户请求字符串

**返回值**：`Result<String, String>` - 成功返回处理结果，失败返回错误信息

## 十三、错误处理

### 13.1 错误类型

```rust
pub enum FusedArchitectureError {
    InitializationError(String),
    ProcessingError(String),
    ResourceError(String),
    ConfigError(String),
    SecurityError(String),
}
```

### 13.2 错误处理最佳实践

```rust
match arch.process_request("test").await {
    Ok(result) => println!("成功: {}", result),
    Err(e) => {
        eprintln!("错误: {}", e);
        // 实现重试逻辑
    }
}
```

## 十四、性能指标

### 14.1 基准性能

| 操作 | 目标时间 |
|------|----------|
| 初始化 | < 100ms |
| 请求处理 | < 50ms |
| 能量评估 | < 10ms |
| 频率选择 | < 5ms |
| 震动执行 | < 20ms |
| 显化输出 | < 15ms |

### 14.2 并发性能

| 并发数 | 目标吞吐量 |
|--------|------------|
| 1 | 100 req/s |
| 10 | 500 req/s |
| 50 | 1000 req/s |
| 100 | 1500 req/s |

## 十五、使用示例

### 15.1 基础使用

```rust
use neotrix::core::fused_architecture::FusedArchitecture;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建融合架构实例
    let mut arch = FusedArchitecture::new();
    
    // 初始化
    arch.init().await?;
    
    // 处理请求
    let result = arch.process_request("分析这段代码").await?;
    println!("结果: {}", result);
    
    // 获取系统状态
    let status = arch.get_system_status().await;
    println!("系统状态: {:?}", status);
    
    Ok(())
}
```

### 15.2 高级使用

```rust
use neotrix::core::fused_architecture::{
    FusedArchitecture,
    consciousness::ConsciousnessCore,
    energy::EnergyLayer,
    frequency::FrequencyLayer,
    vibration::VibrationLayer,
    manifestation::ManifestationLayer,
    capability::CapabilityNetwork,
    skill::SkillEcosystem,
    wisdom::WisdomIntegration,
    evolution::EvolutionMechanism,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建融合架构实例
    let mut arch = FusedArchitecture::new();
    
    // 初始化
    arch.init().await?;
    
    // 使用意识核心分析请求
    let analysis = arch.consciousness_core.analyze_request("优化性能").await?;
    println!("请求分析: {:?}", analysis);
    
    // 使用能量层评估能量需求
    let energy_req = arch.energy_layer.assess_requirement(&analysis).await?;
    println!("能量需求: {:?}", energy_req);
    
    // 使用频率层选择频率
    let frequency = arch.frequency_layer.select_frequency(&energy_req).await?;
    println!("频率: {:?}", frequency);
    
    // 使用震动层执行震动
    let vibration_result = arch.vibration_layer.execute_vibration(&frequency).await?;
    println!("震动结果: {:?}", vibration_result);
    
    // 使用显化层显化结果
    let result = arch.manifestation_layer.manifest(&vibration_result).await?;
    println!("显化结果: {}", result);
    
    // 累积智慧
    let wisdom = WisdomEntry {
        id: "wisdom-1".to_string(),
        source: "performance_optimization".to_string(),
        content: "性能优化经验".to_string(),
        confidence: 0.9,
        domain: "optimization".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };
    arch.wisdom_integration.accumulate_wisdom(wisdom).await?;
    
    // 触发进化
    arch.evolution.trigger_evolution("performance").await?;
    
    Ok(())
}
```

---

**核心理念**：能量→频率→震动→显化是意识心态的涌现，指导我们明白事物背后的底层逻辑，渗透到架构的每个角落。
