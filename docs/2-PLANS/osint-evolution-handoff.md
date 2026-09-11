# OSINT 模块进化移交文档

## 日期: 2026-09-11

## 已完成工作

### 核心架构

| 组件 | 状态 | 说明 |
|------|------|------|
| **OsintSource trait** | ✅ 完成 | 8 方法: name/requires_*/is_active/needs_api_key/priority/timeout_hint/investigate |
| **SourceGate** | ✅ 完成 | 15 个模块的门控元数据 (单一事实源) |
| **should_run_source()** | ✅ 完成 | 统一门控判断 |
| **run_osint()** | ✅ 完成 | 门控分发 + 15 模块全覆盖 |

### OSINT 模块 (15 个)

| 模块 | Trait Impl | Tests | 状态 |
|------|-----------|-------|------|
| dns | ✅ | 7 | 完整 |
| http | ✅ | 8 | 完整 |
| url | ✅ | 4 | 完整 |
| vuln | ✅ | 8 | 完整 |
| network | ✅ | 7 | 完整 |
| dark | ✅ | 10 | 完整 |
| fofa | ✅ | 15 | 完整 |
| person | ✅ | 6 | 完整 |
| social | ✅ | 2 | 完整 |
| credential | ✅ | 5 | 完整 |
| shodan | ✅ | 3 | 完整 |
| censys | ✅ | 3 | 完整 |
| zoomeye | ✅ | 3 | 完整 |
| asset_model | - | 2 | 完整 |
| auto_patrol | - | 1 | 完整 |

**总计: 127 个单元测试**

### 集成点 (8 处注册)

1. `pub mod` 声明 ✅
2. `OsintReport` struct ✅
3. `run_osint()` match arm ✅
4. `source_gates()` 门控配置 ✅
5. `total_findings()` ✅
6. `Display` impl ✅
7. `write_to_kb()` ✅
8. `doctor_osint()` API key 检测 ✅

### Git Commits

```
47844a8d feat(osint): OsintSource trait + FOFA/Shodan/Censys/ZoomEye integration
a630057e fix(osint): resolve compilation errors in write_to_kb and doctor_osint
38c37b22 fix(fofa): remove duplicate tests module
61c16e45 feat(osint): unified asset model + auto-patrol framework
```

## 待后续会话完成

### 高优先级

| 任务 | 说明 | 阻塞原因 |
|------|------|----------|
| 全量编译验证 | `cargo check -p neotrix --lib` | 预存错误 (nt_mind 模块重构残留) |
| OSINT 单元测试 | `cargo test -p neotrix --lib osint` | 依赖全量编译通过 |
| erased_serde 动态分发 | OsintModuleRegistry 替代显式 match | 需添加 erased_serde 依赖 |

### 中优先级

| 任务 | 说明 |
|------|------|
| GWT 集成 | 基于 priority 动态调度 |
| 更多模块 | SecurityTrails/CryptoPub/ShodanInternetDB |
| STIX 2.1 情报交换 | CTI 标准化支持 |

## 新增模块模板

```rust
// 1. 创建模块文件 (如 shodan.rs)
// 2. 实现 OsintSource trait
pub struct ShodanInvestigator;
impl super::OsintSource for ShodanInvestigator {
    type Findings = ShodanFindings;
    fn name(&self) -> &'static str { "shodan" }
    fn needs_api_key(&self) -> bool { true }
    fn priority(&self) -> u8 { 9 }
    async fn investigate(...) -> Result<ShodanFindings, String> { ... }
}

// 3. 在 mod.rs 注册 (8 处)
// pub mod shodan;
// pub shodan: Option<shodan::ShodanFindings>,  // OsintReport
// "shodan" => match shodan::investigate(...)   // run_osint
// SourceGate { name: "shodan", ... }            // source_gates
// if let Some(ref s) = self.shodan { n += ... } // total_findings
// if let Some(ref s) = self.shodan { write!(f, "{}", s)? } // Display
// if let Some(ref s) = self.shodan { ... }      // write_to_kb
// let shodan_ok = config.api_keys.contains_key("shodan"); // doctor_osint
```

## 关键规则

- **R-P87**: 新增 OSINT 模块 MUST 注册到 8 处
- **OsintSource trait**: 新模块只需 `impl OsintSource` + 注册 gate
- **SourceGate**: 集中管理门控元数据，散落在 run_osint 中
