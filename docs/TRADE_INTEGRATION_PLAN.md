# 外贸能力融合架构设计方案

## 1. 架构映射：外贸能力 → NeoTrix 六层架构

### L1 行动层 (nt_act)
**现有模块**: `nt_act_trade/` (26个文件)
**新增能力**:
- `nt_act_trade/extractors/` - 数据提取引擎
  - `chrome_decrypt.rs` - Chrome密码解密 (PBKDF2+AES-128-CBC)
  - `selenium_automation.rs` - Selenium自动化登录
  - `api_discovery.rs` - API路由发现 (网络日志捕获)
  - `batch_extractor.rs` - 批量数据提取 (断点续传)

### L2 感知层 (nt_world)
**现有模块**: `nt_world/crawl/` (爬虫框架)
**新增能力**:
- `nt_world/crawl/crm_crawler.rs` - CRM平台爬虫
  - 富通天下适配器
  - Salesforce适配器
  - HubSpot适配器

### L3 具身层 (nt_shield)
**现有模块**: `nt_shield/` (安全防护)
**新增能力**:
- `nt_shield/credential_vault.rs` - 凭证保险库
  - 加密存储平台凭证
  - 访问控制

### L4 情感层 (nt_feel)
**现有模块**: `nt_feel/` (情感引擎)
**新增能力**:
- `nt_feel/salesperson_profiling.rs` - 业务员画像
  - 写作风格分析
  - 语言偏好识别
  - 沟通模式建模

### L5 认知层 (nt_core + nt_mind)
**现有模块**: `nt_core/` (核心推理) + `nt_mind/` (自我进化)
**新增能力**:
- `nt_core/trade_intelligence.rs` - 贸易智能
  - 客户意图识别
  - 商机评估
  - 风险预警
- `nt_mind/sales_coaching.rs` - 销售教练
  - 话术推荐
  - 跟进策略
  - 业绩预测

### L6 元认知层 (nt_meta)
**现有模块**: `nt_meta/` (元认知协调)
**新增能力**:
- `nt_meta/multi_agent_orchestrator.rs` - 多Agent协调器
  - Orchestrator-Worker模式
  - Router模式
  - Dynamic Handoff模式

## 2. 多Agent编排模式选择

### 推荐模式：Orchestrator-Worker + Router 混合

```
┌─────────────────────────────────────────────────────────────┐
│                    Trade Orchestrator                        │
│  (nt_act_trade/orchestrator.rs - 已存在)                    │
├─────────────────────────────────────────────────────────────┤
│  1. 任务分解 (LLM)                                          │
│  2. 路由决策 (Router)                                       │
│  3. 结果合成                                                │
└───────────────┬─────────────────────────────────────────────┘
                │
    ┌───────────┼───────────┬───────────┬───────────┐
    ▼           ▼           ▼           ▼           ▼
┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐
│Extract│  │ Analyze│  │  Write │  │  Send  │  │ Track │
│Agent  │  │ Agent  │  │ Agent  │  │ Agent  │  │ Agent │
└───────┘  └───────┘  └───────┘  └───────┘  └───────┘
```

### 模式优势
1. **聚焦冗余**: 单一Orchestrator控制，避免重复路由
2. **扁平缺陷**: Worker无状态，易于测试和替换
3. **跨域错位**: Router支持动态任务分配

## 3. 通用方案设计（适用所有外部模型）

### 3.1 统一数据提取接口

```rust
// crates/neotrix-types/src/trade_extract.rs
pub trait ExternalPlatformExtractor: Send + Sync {
    /// 平台标识
    fn platform_id(&self) -> &str;
    
    /// 认证方式
    fn auth_method(&self) -> AuthMethod;
    
    /// 提取客户列表
    async fn extract_customers(&self, config: ExtractConfig) -> Result<Vec<Customer>>;
    
    /// 提取沟通记录
    async fn extract_interactions(&self, customer_id: &str) -> Result<Vec<Interaction>>;
    
    /// 提取邮件
    async fn extract_emails(&self, config: EmailConfig) -> Result<Vec<Email>>;
    
    /// 增量同步
    async fn sync_incremental(&self, last_sync: DateTime) -> Result<SyncResult>;
}
```

### 3.2 平台适配器注册表

```rust
// crates/neotrix-types/src/platform_registry.rs
pub struct PlatformRegistry {
    extractors: HashMap<String, Box<dyn ExternalPlatformExtractor>>,
}

impl PlatformRegistry {
    pub fn register(&mut self, platform: &str, extractor: Box<dyn ExternalPlatformExtractor>) {
        self.extractors.insert(platform.to_string(), extractor);
    }
    
    pub fn get(&self, platform: &str) -> Option<&dyn ExternalPlatformExtractor> {
        self.extractors.get(platform).map(|e| e.as_ref())
    }
}
```

### 3.3 数据标准化管道

```rust
// crates/neotrix-types/src/data_pipeline.rs
pub struct TradeDataPipeline {
    registry: PlatformRegistry,
    normalizer: DataNormalizer,
    storage: TradeStorage,
}

impl TradeDataPipeline {
    /// 统一数据提取入口
    pub async fn extract_all(&self, platform: &str) -> Result<ExtractionResult> {
        let extractor = self.registry.get(platform)
            .ok_or_else(|| Error::PlatformNotSupported(platform.into()))?;
        
        // 1. 提取原始数据
        let raw_customers = extractor.extract_customers(ExtractConfig::default()).await?;
        let raw_interactions = extractor.extract_interactions("*").await?;
        
        // 2. 标准化
        let customers = self.normalizer.normalize_customers(raw_customers)?;
        let interactions = self.normalizer.normalize_interactions(raw_interactions)?;
        
        // 3. 存储
        self.storage.save_customers(&customers).await?;
        self.storage.save_interactions(&interactions).await?;
        
        Ok(ExtractionResult {
            customers: customers.len(),
            interactions: interactions.len(),
        })
    }
}
```

## 4. 冗余清理清单

### 4.1 代码冗余
| 模块 | 冗余类型 | 清理方案 |
|------|----------|----------|
| `nt_trade_crm.rs` | 与`unified_types.rs`类型重复 | 合并到`unified_types.rs` |
| `nt_trade_email.rs` | 与`nt_act_email`功能重叠 | 统一到`nt_trade_email.rs` |
| `knowledge_base.rs` | 与`sqlite_knowledge_base.rs`实现重复 | 保留SQLite实现，删除trait定义 |

### 4.2 架构冗余
| 问题 | 影响 | 重构方案 |
|------|------|----------|
| Engine过多 (100+) | 编译慢，维护难 | 合并相关Engine |
| 模块间耦合高 | 测试困难 | 引入trait边界 |
| 类型定义分散 | 重复定义 | 统一到`unified_types.rs`

## 5. 架构重构路线

### Phase 1: 类型统一 (1周)
- [ ] 合并所有trade类型到`unified_types.rs`
- [ ] 删除重复类型定义
- [ ] 更新所有引用

### Phase 2: 接口标准化 (2周)
- [ ] 实现`ExternalPlatformExtractor` trait
- [ ] 创建`PlatformRegistry`
- [ ] 实现`TradeDataPipeline`

### Phase 3: Agent编排 (2周)
- [ ] 实现`TradeOrchestrator`多Agent模式
- [ ] 创建Worker Agent池
- [ ] 实现Router动态路由

### Phase 4: 冗余清理 (1周)
- [ ] 删除重复Engine
- [ ] 合并相关模块
- [ ] 优化编译时间

### Phase 5: 测试覆盖 (1周)
- [ ] 单元测试覆盖
- [ ] 集成测试
- [ ] 性能测试

## 6. 核心路线任务清单

### 高优先级 (P0)
1. 实现富通天下适配器 (`nt_act_trade/extractors/joinf.rs`)
2. 实现数据标准化管道 (`TradeDataPipeline`)
3. 实现多Agent编排器 (`TradeOrchestrator`)

### 中优先级 (P1)
4. 实现Salesforce适配器
5. 实现HubSpot适配器
6. 实现业务员画像引擎

### 低优先级 (P2)
7. 实现销售教练引擎
8. 实现业绩预测模型
9. 实现风险预警系统

## 7. 多Agent自动巡检修复

### 巡检维度
1. **编译检查**: `cargo check --all-targets`
2. **测试检查**: `cargo test --lib`
3. **安全检查**: `cargo audit`
4. **性能检查**: `cargo bench`
5. **文档检查**: `cargo doc`

### 自动修复策略
1. **编译错误**: 自动定位并修复
2. **测试失败**: 分析失败原因并修复
3. **安全漏洞**: 自动升级依赖
4. **性能回归**: 标记并通知

### 巡检频率
- 每次提交: 编译+测试
- 每日: 安全+性能
- 每周: 全量巡检
