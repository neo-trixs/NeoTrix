# 核心路线任务清单

## 总览
基于外贸数据提取经验 + 多Agent架构研究 + NeoTrix现有能力，生成以下核心任务清单。

## Phase 1: 数据提取能力集成 (Week 1-2)

### P0: 富通天下适配器
- [ ] **TASK-001**: 实现`JoinfExtractor` struct
  - 文件: `neotrix-core/src/l1_action/nt_act/nt_act_trade/extractors/joinf.rs`
  - 功能: Chrome密码解密、Selenium登录、API发现、批量提取
  - 依赖: `chrome_decrypt.rs`, `selenium_automation.rs`
  - 验证: 能成功提取富通天下数据

- [ ] **TASK-002**: 实现`ChromeDecryptor`模块
  - 文件: `neotrix-core/src/l1_action/nt_act/nt_act_trade/extractors/chrome_decrypt.rs`
  - 功能: PBKDF2+AES-128-CBC解密Chrome密码
  - 验证: 能解密Chrome Safe Storage密码

- [ ] **TASK-003**: 实现`SeleniumAutomation`模块
  - 文件: `neotrix-core/src/l1_action/nt_act/nt_act_trade/extractors/selenium_automation.rs`
  - 功能: 浏览器自动化登录、表单提交、网络日志捕获
  - 验证: 能自动登录富通天下

### P1: 数据标准化
- [ ] **TASK-004**: 实现`TradeDataPipeline`
  - 文件: `neotrix-core/src/l1_action/nt_act/nt_act_trade/data_pipeline.rs`
  - 功能: 统一数据提取入口、标准化、存储
  - 验证: 能通过Pipeline提取数据

- [ ] **TASK-005**: 实现`PlatformRegistry`
  - 文件: `neotrix-core/src/l1_action/nt_act/nt_act_trade/platform_registry.rs`
  - 功能: 平台适配器注册、动态加载
  - 验证: 能注册和获取适配器

## Phase 2: 多Agent编排 (Week 3-4)

### P0: 编排器实现
- [ ] **TASK-006**: 实现`TradeOrchestrator`多Agent模式
  - 文件: `neotrix-core/src/l1_action/nt_act/nt_act_trade/orchestrator_v2.rs`
  - 功能: Orchestrator-Worker + Router混合模式
  - 验证: 能编排多个Worker Agent

- [ ] **TASK-007**: 实现`TradeWorker` Agent池
  - 文件: `neotrix-core/src/l1_action/nt_act/nt_act_trade/workers/`
  - 功能: ExtractWorker, AnalyzeWorker, WriteWorker, SendWorker, TrackWorker
  - 验证: 各Worker能独立执行任务

- [ ] **TASK-008**: 实现`TradeRouter`动态路由
  - 文件: `neotrix-core/src/l1_action/nt_act/nt_act_trade/router.rs`
  - 功能: 基于任务类型的动态路由决策
  - 验证: 能正确路由到对应Worker

### P1: 通信协议
- [ ] **TASK-009**: 实现`TradeMessage`协议
  - 文件: `neotrix-core/src/l1_action/nt_act/nt_act_trade/message.rs`
  - 功能: Agent间通信消息格式
  - 验证: 消息能正确序列化/反序列化

- [ ] **TASK-010**: 实现`TradeEventBus`事件总线
  - 文件: `neotrix-core/src/l1_action/nt_act/nt_act_trade/event_bus_v2.rs`
  - 功能: 异步事件发布/订阅
  - 验证: 事件能正确传播

## Phase 3: 业务能力增强 (Week 5-6)

### P0: 业务员画像
- [ ] **TASK-011**: 实现`SalespersonProfiler`
  - 文件: `neotrix-core/src/l4_emotion/nt_feel/salesperson_profiling.rs`
  - 功能: 写作风格分析、语言偏好识别、沟通模式建模
  - 验证: 能生成业务员画像报告

- [ ] **TASK-012**: 实现`WritingStyleAnalyzer`
  - 文件: `neotrix-core/src/l4_emotion/nt_feel/writing_style.rs`
  - 功能: 邮件/WhatsApp写作风格分析
  - 验证: 能识别写作风格特征

### P1: 销售智能
- [ ] **TASK-013**: 实现`TradeIntelligence`
  - 文件: `neotrix-core/src/l5_cognition/nt_core/trade_intelligence.rs`
  - 功能: 客户意图识别、商机评估、风险预警
  - 验证: 能评估商机质量

- [ ] **TASK-014**: 实现`SalesCoach`
  - 文件: `neotrix-core/src/l5_cognition/nt_mind/sales_coaching.rs`
  - 功能: 话术推荐、跟进策略、业绩预测
  - 验证: 能生成销售建议

## Phase 4: 冗余清理 (Week 7)

### P0: 类型统一
- [ ] **TASK-015**: 合并重复类型到`unified_types.rs`
  - 清理: `nt_trade_crm.rs`中的重复类型
  - 清理: `nt_trade_email.rs`中的重复类型
  - 验证: 编译通过，测试通过

- [ ] **TASK-016**: 删除冗余Engine
  - 清理: 合并`knowledge_base.rs`和`sqlite_knowledge_base.rs`
  - 清理: 删除未使用的Engine
  - 验证: 编译时间减少

### P1: 接口标准化
- [ ] **TASK-017**: 统一 trait 边界
  - 清理: 所有Engine实现统一trait
  - 清理: 删除重复的trait定义
  - 验证: 接口一致性

## Phase 5: 测试覆盖 (Week 8)

### P0: 单元测试
- [ ] **TASK-018**: 为所有新模块编写单元测试
  - 覆盖: extractors, pipeline, orchestrator, workers
  - 目标: 80%+ 代码覆盖率
  - 验证: `cargo test --lib` 通过

- [ ] **TASK-019**: 为业务能力编写单元测试
  - 覆盖: profiler, intelligence, coach
  - 目标: 80%+ 代码覆盖率
  - 验证: `cargo test --lib` 通过

### P1: 集成测试
- [ ] **TASK-020**: 编写端到端集成测试
  - 场景: 从数据提取到销售建议的完整流程
  - 验证: 集成测试通过

## 依赖关系

```
TASK-001 → TASK-002, TASK-003 → TASK-004 → TASK-005
    ↓
TASK-006 → TASK-007, TASK-008 → TASK-009, TASK-010
    ↓
TASK-011 → TASK-012 → TASK-013 → TASK-014
    ↓
TASK-015 → TASK-016 → TASK-017
    ↓
TASK-018 → TASK-019 → TASK-020
```

## 验收标准

### 编译检查
```bash
cargo check --all-targets -p neotrix
cargo build -p neotrix
```

### 测试检查
```bash
cargo test -p neotrix --lib
```

### 安全检查
```bash
cargo audit
```

### 性能检查
```bash
cargo bench -p neotrix
```

## 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Selenium依赖过重 | 编译慢，体积大 | 考虑用headless浏览器替代 |
| Chrome密码解密跨平台 | Windows/Linux不兼容 | 实现平台特定适配器 |
| 多Agent通信开销 | 性能下降 | 使用消息队列，批量处理 |
| 类型合并破坏兼容性 | 现有代码无法编译 | 渐进式迁移，保持向后兼容 |
