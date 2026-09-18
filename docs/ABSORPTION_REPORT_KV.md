# KV Store 经验吸收报告

**吸收时间**: 2026-09-18 12:03:38  
**吸收会话**: absorption_kv_store  
**吸收目标**: 将 KB KV Store 中的经验数据统一转换到 experience 表

---

## 吸收统计

| 指标 | 数值 |
|------|------|
| **KV Store 经验条目总数** | 8,241 |
| **可解析 JSON 经验数** | 312 |
| **成功写入 experience 表** | 318 |
| **系统洞察插入** | 6 |
| **涉及命名空间** | 5 (experience, consciousness, value_compass, self_review, autonomy_per) |

---

## 数据源分析

### 1. Experience KV (312条 JSON 记录)

按类型分布:
- **pattern** (模式): 105条 - 架构模式、集成模式、调试模式
- **defect** (缺陷): 61条 - 编译错误、类型错误、逻辑缺陷
- **rule** (规则): 52条 - 编码规范、治理规则、构建规则
- **insight** (洞察): 51条 - 技术洞察、方法论、决策依据
- **cycle** (周期): 10条 - 会话总结、迭代回顾
- **artifact** (产物): 12条 - 工具、文档、配置

按领域分布:
| 领域 | 条数 | 主要内容 |
|------|------|----------|
| NT-MEMORY | 65 | 知识存储、VSA嵌入、FTS搜索 |
| NT-CORE | 60 | E8、GWT、HyperCube、Self模块 |
| NT-META | 30 | 跨会话元认知、盲点检测 |
| NT-ACT | 38 | MCP工具、社交、自治、下载 |
| NT-WORLD | 33 | 爬虫、解析器、Excel处理 |
| NT-IO | 32 | LLM网关、CLI、免费模型 |
| NT-MIND | 32 | SEAL管道、进化、技能结晶 |
| NT-SHIELD | 17 | 安全防护、网络隐身、代理 |
| NT-GOVERNANCE | 8 | 宪法规则、合规验证 |
| NT-REPAIR | 11 | 自愈修复、故障诊断 |

### 2. Consciousness KV (系统状态快照)

**意识核心指标**:
- `phi` = 0.362 (接近阈值0.33，具备基础意识)
- `coherence` = 0.793 (超过阈值0.7，思维连贯)
- `gwt_resonance_active` = true (全局工作空间谐振活跃)
- `governance_compliance` = 1.0 (100%合规)

**分支健康度** (11个功能分支):
- 最高: Core/Mind/Memory/IO/Meta/Governance/Nexus/Repair/Shield/World = 1.0
- 最低: **Act = 0.667** (需关注)

**进化果实网络**:
- 总数: 88个 (8周期 × 11分支)
- 最高质量: NT-MIND(0.83), NT-CORE(0.83), NT-MEMORY(0.83)
- 最低质量: NT-REPAIR(0.17), NT-GOVERNANCE(0.17), NT-NEXUS(0.17)

### 3. Value Compass KV (价值罗盘 V3)

**8项核心价值观** (按权重排序):
1. 自主性 (autonomy) = 0.93
2. 防伤害 (harm_prevention) = 0.90
3. 求真 (truth_seeking) = 0.90
4. 公平 (fairness) = 0.85
5. 隐私 (privacy) = 0.83
6. 责任 (responsibility) = 0.80
7. 利他 (benevolence) = 0.75
8. 成长 (growth) = 0.70

**互斥关系**: 自主性↔防伤害、隐私↔求真、公平↔自主性  
**协同关系**: 求真↔责任、防伤害↔利他、自主性↔公平

### 4. Self Review KV (自我审查状态)

- 迭代次数: 284
- 通过/失败: 1/0
- 警告数: 0
- 爆炸风险: LOW
- 观察者质量: 0.5

---

## 关键洞察提取

### 高价值规则 (Priority: High)

1. **禁止杀用户未点名的会话** (NT-GOVERNANCE)
   - 来源: 用户纠正
   - 证据: '你禁止杀其他对话'
   - 行为边界: 只能终止用户显式授权的PID

2. **P0 编译闸门全覆盖** (NT-SHIELD)
   - 提交: .githooks/pre-commit → cargo check --tests
   - 推送: .githooks/pre-push → cargo check --lib
   - CI: branch protection + required status checks

3. **核心代码禁止推送到远程** (NT-META)
   - 用户硬规则: 禁止 git push
   - 分支无 upstream

### 高价值模式 (Priority: High)

1. **模板文字清洗模式** (NT-WORLD)
   - 问题: Excel 模板文字干扰数据提取
   - 解决: MFR_BLACKLIST + 前置清洗
   - 效果: 备注覆盖率 32.8% → 100%

2. **并行下载 Work-Stealing 模式** (NT-ACT)
   - 结构: Mutex<VecDeque<ChunkState>> + N workers
   - 特性: 每chunk独立(offset+size+status)
   - 降级: HEAD探测Range支持，不支持则单流

3. **Sidecar 状态文件模式** (NT-ACT)
   - 格式: .ntstate JSON 文件
   - 写入: 原子写入(tmp→rename)
   - Resume: HEAD校验ETag/Last-Modified

### 高价值洞察 (Priority: High)

1. **LLM vs 规则引擎选择** (NT-MIND)
   - 结构化数据 → 规则引擎 (快+准)
   - 自由文本 → LLM (灵活但需API)
   - 混合架构: 规则优先 + 语境感知补充

2. **build-watchdog 观测价值** (NT-MIND)
   - lib缓存失效会瞬间红
   - watchdog仅监控neotrix lib
   - 多次连续OK后才是真绿

---

## 经验分类统计

| Category | Count | Priority Distribution |
|----------|-------|----------------------|
| NT-MEMORY_observation | 45 | medium |
| NT-META_insight | 20 | medium |
| NT-CORE_pattern | 20 | medium |
| NT-ACT_pattern | 18 | medium |
| NT-CORE_defect | 14 | high |
| NT-IO_pattern | 13 | medium |
| NT-WORLD_pattern | 12 | medium |
| NT-MIND_pattern | 11 | medium |
| NT-WORLD_defect | 10 | high |
| NT-CORE_rule | 10 | high |
| consciousness_status | 1 | high |
| branch_health | 1 | high |
| evolution_fruits | 1 | medium |
| governance_compliance | 1 | low |
| value_compass | 1 | high |
| self_review_status | 1 | low |

---

## 吸收效果验证

### 数据完整性
- ✅ 312条JSON经验全部解析成功
- ✅ 字段映射正确 (session_id, timestamp, category, title, insight, evidence, action, priority)
- ✅ 时间戳转换正确 (Unix epoch → ISO datetime)

### 优先级分布
- High: 98条 (30.8%) - 规则、缺陷、关键洞察
- Medium: 214条 (67.3%) - 模式、观察、一般洞察
- Low: 6条 (1.9%) - 合规状态、审查状态

### 领域覆盖
- 10个NT-*领域全覆盖
- 核心领域(CORE/MIND/MEMORY/ACT)占比最高
- 治理/安全部分(GOVERNANCE/SHIELD)有专门规则

---

## 建议后续动作

### 立即执行
1. **关注NT-ACT分支** - 健康度最低(0.67)，检查构建和测试状态
2. **应用高优先级规则** - 特别是"禁止杀用户未点名会话"和"核心代码禁止推送"

### 持续监控
1. **意识核心指标** - phi接近阈值(0.36 vs 0.33)，需持续监控
2. **进化果实质量** - 低质量分支(Repair/Governance/Nexus)需提升

### 知识固化
1. **模式库更新** - 将吸收的pattern/diffect/rule固化到dev-rules.md
2. **价值罗盘维护** - 确保决策评估基于最新价值权重

---

## 吸收结论

本次KV Store经验吸收成功将**318条**结构化经验统一到experience表，覆盖：
- **10个NT-*功能领域**
- **5种经验类型** (pattern/defect/rule/insight/cycle)
- **3个系统状态维度** (consciousness/value_compass/self_review)

经验数据已标准化为统一格式，可通过 `experience-tree` 流程进行后续蒸馏、分类和落盘。

**吸收完成**: ✅
