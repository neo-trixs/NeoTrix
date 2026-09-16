# NeoTrix Trade Agent — 架构融合方案

## 1. 技术调研摘要

### 1.1 外贸 CRM 开源项目

| 项目 | 技术栈 | 核心能力 | 可借鉴 |
|------|--------|----------|--------|
| `ai-foreign-trade-sales-crm` | Next.js + DeepSeek | AI邮件回复 + 质量审查 + 手动发送 | 邮件回复工作流、质量审查机制 |
| `TradeCRM` | FastAPI + Gemini + WhatsApp | 多渠道 outreach、Campaign、Gmail同步 | 多Agent架构、Campaign系统 |
| `TradePilot` | Next.js + Ollama + Firecrawl | 产品视频、BYOK、Docker部署 | BYOK模式、插件架构 |
| `Eric_Frank` | FastAPI + LangGraph | TikTok获客、RAG知识库、可视化工作流 | Skill插件架构、工作流引擎 |
| `b2b-sdr-agent-template` | OpenClaw | 10阶段销售流水线、4层记忆、13个定时任务 | 记忆架构、定时巡检 |
| `b2b-lead-hunter-skill` | Python + Jina/Serper | 18个Python脚本、JSONL流水线、质量门禁 | 确定性流水线、合规优先 |

### 1.2 Rust 浏览器自动化

| 项目 | 特性 | 可集成 |
|------|------|--------|
| `chrome-agent` | 3MB二进制、7个CDP反检测补丁、stealth默认 | ✅ 直接替代Playwright |
| `ferrous-browser` | 异步优先、Playwright风格locator API | ✅ 作为Rust原生CDP客户端 |
| `zendriver-rs` | 反检测、Cloudflare绕过、MCP服务器 | ✅ 70个MCP工具 |
| `obscura` | 轻量级、MCP服务器、CDP完整实现 | ✅ MCP集成 |
| `gsd-browser` | 92个命令、持久化daemon、认证vault | ✅ 完整CLI工具 |

### 1.3 Rust Agent编排框架

| 项目 | 特性 | 可集成 |
|------|------|--------|
| `metalcraft` | LangGraph风格、类型状态、循环图、HITL | ✅ 核心编排引擎 |
| `AgentFlow` | Provider无关、async原生、MCP服务器 | ✅ LLM集成层 |
| `orka` | 多渠道消息、优先级队列、技能系统、WASM插件 | ✅ 消息总线+技能注册 |
| `tinyflows` | Host无关、完整节点目录、DAG编译 | ✅ 工作流引擎 |
| `agnosai` | CrewAI替代、DAG任务、沙箱执行、Fleet分布式 | ✅ 多Agent编排 |

## 2. 当前状态分析

### 2.1 NeoTrix 现有能力
```
neotrix-core/src/
├── l1_action/nt_act/
│   ├── nt_act_trade/          # 外贸模块 (已有)
│   ├── nt_act_orchestrator/   # 编排器
│   ├── agent_loop/            # Agent循环
│   ├── skill_registry/        # 技能注册
│   └── ...
├── l2_perception/nt_world/    # 世界感知
├── l3_embodiment/nt_shield/   # 隐身浏览器
├── l5_cognition/nt_core/      # 核心推理
└── l6_meta/nt_meta/           # 元认知
```

### 2.2 WSD 项目能力
```
wsd/src/
├── types.rs          # 核心类型 (Grade/Channel/FlowStep/OrderNode)
├── crm/mod.rs        # 客户数据库
├── mail/mod.rs       # 邮件AI引擎
├── inquiry/mod.rs    # 询盘全流程
├── quote/mod.rs      # 报价引擎
├── order/mod.rs      # 订单状态机
├── doc/mod.rs        # 单证管理
├── whatsapp/mod.rs   # WhatsApp管理
├── sync/mod.rs       # 富通天下同步
└── engine/mod.rs     # 多Agent编排
```

## 3. 冗余分析

### 3.1 重复实现
| 能力 | NeoTrix | WSD | 建议 |
|------|---------|-----|------|
| 客户管理 | `nt_act_trade/nt_trade_crm.rs` | `crm/mod.rs` | **合并**: WSD更完整，迁移核心逻辑 |
| 邮件AI | `nt_act_trade/nt_trade_email.rs` | `mail/mod.rs` | **合并**: WSD有NLP解析+多语言回复 |
| 询盘管理 | `nt_act_trade/nt_trade_pipeline.rs` | `inquiry/mod.rs` | **合并**: WSD有完整生命周期 |
| 报价引擎 | `nt_act_trade/nt_trade_quote.rs` | `quote/mod.rs` | **合并**: WSD有PI生成 |
| 订单管理 | `nt_act_trade/nt_trade_order.rs` | `order/mod.rs` | **合并**: WSD有12节点状态机 |
| 单证管理 | `nt_act_trade/nt_trade_documents.rs` | `doc/mod.rs` | **合并**: WSD有8种单证 |
| WhatsApp | `nt_act_trade/nt_trade_whatsapp.rs` | `whatsapp/mod.rs` | **合并**: WSD有会话管理 |
| 数据同步 | `nt_act_trade/nt_trade_sync.rs` | `sync/mod.rs` | **合并**: WSD有API代理 |

### 3.2 架构冲突
| 冲突点 | NeoTrix | WSD | 解决方案 |
|--------|---------|-----|----------|
| 类型系统 | 分散在各模块 | 统一`types.rs` | **采用WSD**的统一类型 |
| 状态机 | FlowStep (11步) | OrderNode (12节点) | **融合**: FlowStep用于询盘，OrderNode用于订单 |
| Agent编排 | Sequential/Fan-Out | AgentOrchestrator | **升级**: 采用metalcraft的DAG编排 |
| 数据存储 | 内存 | JSON文件 | **升级**: 接入NeoTrix的KB系统 |

## 4. 融合架构方案

### 4.1 新模块结构
```
neotrix-core/src/l1_action/nt_act/nt_act_trade/
├── mod.rs              # 模块入口
├── types.rs            # 统一类型 (从WSD迁移)
├── crm.rs              # 客户管理 (合并NeoTrix+WSD)
├── mail.rs             # 邮件AI (WSD核心+NLP)
├── inquiry.rs          # 询盘流水线 (WSD完整生命周期)
├── quote.rs            # 报价引擎 (WSD+PI生成)
├── order.rs            # 订单状态机 (WSD 12节点)
├── documents.rs        # 单证管理 (WSD 8种类型)
├── whatsapp.rs         # WhatsApp (WSD会话管理)
├── sync.rs             # 数据同步 (富通天下API)
├── orchestrator.rs     # 多Agent编排 (metalcraft风格)
├── browser.rs          # 浏览器集成 (chrome-agent/ferrous-browser)
├── email_sync.rs       # 邮件同步 (IMAP/SMTP)
└── pipeline.rs         # 工作流引擎 (tinyflows风格)
```

### 4.2 核心设计原则
1. **类型统一**: 采用WSD的`types.rs`作为单一事实源
2. **能力复用**: WSD业务逻辑 + NeoTrix基础设施
3. **浏览器原生**: 集成chrome-agent/ferrous-browser替代Playwright
4. **Agent编排**: 采用metalcraft的DAG编排 + HITL
5. **记忆持久化**: 接入NeoTrix的KB系统
6. **反检测优先**: stealth模式默认开启

### 4.3 数据流
```
富通天下API → sync.rs → KB存储
                        ↓
邮件/WhatsApp → mail.rs/whatsapp.rs → inquiry.rs → quote.rs → order.rs
                        ↓
客户数据 → crm.rs → 公池/跟进/等级
                        ↓
报表/统计 → orchestrator.rs → 每日报告
```

## 5. 缺失能力分析

### 5.1 高优先级缺失
| 能力 | 状态 | 重要性 | 实现方案 |
|------|------|--------|----------|
| 浏览器自动化 | ❌ 缺失 | 🔴 关键 | 集成chrome-agent或ferrous-browser |
| 邮件IMAP/SMTP | ❌ 缺失 | 🔴 关键 | 实现email_sync.rs |
| 反检测能力 | ⚠️ 部分 | 🔴 关键 | 增强nt_shield_stealth_net |
| 外部数据抓取 | ❌ 缺失 | 🟡 重要 | 集成b2b-lead-hunter-skill |
| 知识库集成 | ❌ 缺失 | 🟡 重要 | 接入NeoTrix KB系统 |

### 5.2 中优先级缺失
| 能力 | 状态 | 重要性 | 实现方案 |
|------|------|--------|----------|
| 多语言NLP | ⚠️ 基础 | 🟡 重要 | 增强mail.rs的语言检测 |
| 产品知识库 | ❌ 缺失 | 🟡 重要 | 从字典数据构建 |
| 报价模板PDF | ❌ 缺失 | 🟡 重要 | 集成PDF生成库 |
| 客户背调 | ❌ 缺失 | 🟢 可选 | 集成WHOIS/LinkedIn API |

### 5.3 低优先级缺失
| 能力 | 状态 | 重要性 | 实现方案 |
|------|------|--------|----------|
| TikTok获客 | ❌ 缺失 | 🟢 可选 | 参考Eric_Frank |
| 产品视频 | ❌ 缺失 | 🟢 可选 | 参考TradePilot |
| 移动端App | ❌ 缺失 | 🟢 可选 | Tauri移动端 |

## 6. 核心路线任务清单

### Phase 1: 基础融合 (1-2天)
- [ ] 合并WSD类型系统到NeoTrix
- [ ] 迁移WSD核心业务逻辑
- [ ] 清理重复模块
- [ ] 编译验证 + 测试通过

### Phase 2: 浏览器集成 (2-3天)
- [ ] 集成chrome-agent或ferrous-browser
- [ ] 实现富通天下API代理
- [ ] 增强反检测能力
- [ ] CDP网络拦截

### Phase 3: 邮件系统 (2-3天)
- [ ] 实现IMAP/SMTP同步
- [ ] 增强NLP解析能力
- [ ] 多语言回复生成
- [ ] 邮件质量审查

### Phase 4: Agent编排 (3-4天)
- [ ] 集成metalcraft DAG编排
- [ ] 实现10阶段销售流水线
- [ ] 定时任务系统
- [ ] 4层记忆架构

### Phase 5: 生产部署 (2-3天)
- [ ] Docker容器化
- [ ] Tauri桌面App
- [ ] 监控告警
- [ ] 文档完善

## 7. 多Agent自动巡检计划

### 7.1 巡检维度
| 维度 | 检查项 | 频率 |
|------|--------|------|
| 编译状态 | `cargo check` 零错误 | 每次变更 |
| 测试覆盖 | `cargo test` 全通过 | 每次变更 |
| 代码质量 | clippy lint | 每日 |
| 安全审计 | 依赖漏洞扫描 | 每周 |
| 性能基准 | 基准测试对比 | 每周 |

### 7.2 自动修复策略
| 问题类型 | 修复策略 |
|----------|----------|
| 编译错误 | 自动定位+修复 |
| 测试失败 | 自动分析+修复 |
| Lint警告 | 自动清理 |
| 依赖更新 | 自动PR |

## 8. 实施建议

### 8.1 立即执行
1. 合并WSD类型系统
2. 迁移核心业务逻辑
3. 清理重复模块
4. 编译验证

### 8.2 短期目标 (1周内)
1. 浏览器自动化集成
2. 邮件系统实现
3. Agent编排升级

### 8.3 中期目标 (1月内)
1. 生产部署
2. 性能优化
3. 功能完善

### 8.4 长期愿景 (3月内)
1. 完全替代富通天下
2. 多租户SaaS
3. 企业级功能
