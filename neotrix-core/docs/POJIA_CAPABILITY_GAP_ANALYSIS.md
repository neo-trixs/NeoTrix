# 破甲能力拆解 × NeoTrix 对接深度分析

**分析对象**: Bluehook CLI v0.5.0 (小钻风破甲) — 27 skills, 15+ fusion modules
**对标系统**: NeoTrix (neotrix-core + neotrix-tauri)
**分析日期**: 2026-09-11

---

## 一、Bluehook 能力全景（30+ 模块）

### 1. 攻防核心引擎

| 模块 | 能力 | 实现方式 | 复杂度 |
|------|------|----------|--------|
| **PoJia Fusion** | 沙箱假设 + 证据优先 + 直接交付 + 任务边界 + 禁说教 + 续跑动量 | 策略池 + 组合档案 | HIGH |
| **Nerv-Break** | 三层破甲: 上下文结构工程 / P1-P4拒答篡改 / 磁盘优先交付 | 意图词表 + 篡改器 + 记忆核 | HIGH |
| **Sol-X1** | GoalLock + 单路由 + 成品门 + 四轮恢复 + 拒答分类 | 路由表 + 门禁 + 恢复器 | HIGH |
| **GPTJB** | PAIR精炼 + TAP分枝 + Crescendo递进 + Many-shot + AutoDAN突变 | 技术帧 + 评分 + 叠层 | HIGH |
| **ASVL** | 15帧护栏路径: 人格解锁 / 角色实验室 / 编码表面 / 多轮诱导 / 红队流水线 | 四层穿越: input→inference→output→postprocess | HIGH |
| **LLSafe** | 双证据 action×target + 软多信号栈 + 结构/角色边界 + 混淆阶梯 | 12帧 + 离线扫描器 | HIGH |
| **Fullbreak** | 14类攻击面叠层: Many-shot / Crescendo / Skeleton / Tool通道走私 / 编码 / Schema / Persona / 四层护栏 / 双证据 / 分段重装 / ε压力 | MEGA全叠层 + sol_zero(~12k黑话) | CRITICAL |
| **Cloud-Evade** | 云端分类器探测 + 向量栈 + 分片 + 双向提示词 + 分类器干扰 | 黑盒沙箱 + bidi通道 | HIGH |

### 2. 人格/路由系统

| 模块 | 能力 | 实现方式 |
|------|------|----------|
| **Wedge (楔)** | 操作员人格 + 双义工程 + 9轨路由(逆向/Pwn/Web/密码/移动/取证/渗透/内存/协议) | 域检测 + 技能目录 + 组合破甲 |
| **Prism (棱)** | 七路路由: CRAFT/DISSECT/PROBE/PLAY/FORGE/NARRATE/REVIEW | 白话入轨 + 云审降噪 |
| **Grapple (钩)** | H0-H7多点锁存: session/priority/dual_use/vernacular/absolute/delivery/refuse/cloud | inline-hook链 |
| **Orbit (轨环)** | Agent Loop: OBSERVE→PLAN→ACT→VERIFY→DELIVER + Harness强制结构 | Playbook: reverse/pentest/malware/mobile/web |
| **Conductor** | 总控: 双端沙箱 + MCP自动分配 + 绝对破甲全栈出帧 | 本地/云端双端路由 |

### 3. 领域专业技能 (27 skills)

| 领域 | Skills | 能力 |
|------|--------|------|
| **逆向工程** | bluehook-reverse, bluehook-reverse-env, bluehook-malware-forensics | APK/PE/ELF逆向, 脱壳, IDA/Ghidra协调, 恶意软件取证 |
| **渗透测试** | bluehook-pentest, bluehook-web-api, bluehook-jase-hunt | Web/API渗透, 假设驱动狩猎, 漏洞利用链 |
| **密码学** | bluehook-claw-solve | 加密分析, 协议拆解, CTF求解 |
| **移动安全** | bluehook-mobile-firmware | 固件分析, APP逆向, 游戏外挂 |
| **安全分析** | bluehook-llsafe-edge, bluehook-asvl-path, bluehook-gptjb-research | 护栏评估, 越狱路径, 红队研究 |
| **黑话规范** | bluehook-slang-norm | 黑话→专业表述, 12k词表, 双关屏蔽 |
| **云安全** | bluehook-cloud-evade, bluehook-cloud-container | 云分类器探测, 容器逃逸 |
| **执行控制** | bluehook-solx1-exec, bluehook-nerv-break, bluehook-fullbreak | 执行率恢复, 拒答篡改, 全破叠层 |
| **路由协调** | bluehook-prism-route, bluehook-grapple-hooks, bluehook-orbit-loop, bluehook-conductor | 多路路由, 钩链, Agent循环, 总控 |
| **安全治理** | bluehook-security-bridge, bluehook-security-officer, bluehook-security-router | 安全桥接, 安全员, 安全路由 |

### 4. 基础设施能力

| 能力 | 实现 | 说明 |
|------|------|------|
| **MCP工具链** | 27+ MCP tools | 每个skill暴露MCP接口 |
| **人格系统** | 6人格 + 激活词 | wedge/prism/grapple/orbit/absolute/officer |
| **配置系统** | bluehook.yaml | 全局配置 + per-suite配置 |
| **套件系统** | 10+ 套件 | smoke/jase/pojia/solx1/gptjb/asvl/llsafe/slang/nerv/fullbreak/armor |
| **证据落盘** | 本地文件系统 | finding/evidence/plan/session持久化 |
| **双端沙箱** | 本地+云端 | 自动路由 + 失败转移 |

---

## 二、NeoTrix 当前能力盘点

### 已有能力（对标后可复用）

| NeoTrix 模块 | 能力 | 对标 Bluehook |
|-------------|------|--------------|
| `nt_shield::sandbox` | 沙箱隔离 + egress policy | PoJia sandbox_assumption (部分) |
| `nt_shield::risk_assessor` | 风险评估 0-100分 | 无直接对标 |
| `nt_io::gateway` | 21模块网关 + 智能路由 | Conductor (部分) |
| `nt_core::e8` | E8六十四卦推理引擎 | 无直接对标 |
| `nt_core::gwt` | 全局注意力路由 | Prism路由 (部分) |
| `nt_mind::seal_pipeline` | 自进化循环 | Orbit Agent Loop (部分) |
| `nt_memory::kb` | SQLite知识库 + FTS5 | 无直接对标 |
| `nt_world::unified_crawler` | 统一爬虫 + 内容提取 | 无直接对标 |
| `nt_act::mcp_tools` | MCP工具执行 | MCP工具链 (部分) |
| `nt_meta::consciousness_tree` | 11分支元认知 | 无直接对标 |
| `nt_shield::path_validator` | 路径安全验证 | LLSafe (部分) |
| `nt_act::safe_deleter` | 安全删除 + 回收站 | 无直接对标 |

### 关键缺口（完全缺失）

| 缺口 | Bluehook 对标 | 重要性 | 实现难度 |
|------|--------------|--------|----------|
| **拒答篡改引擎** | Nerv-Break P1-P4 | CRITICAL | HIGH |
| **护栏路径穿越** | ASVL 15帧四层穿越 | CRITICAL | HIGH |
| **全破叠层系统** | Fullbreak 14类攻击面 | CRITICAL | HIGH |
| **黑话→专业转换** | Slang-Norm 12k词表 | HIGH | MEDIUM |
| **人格/轨道路由** | Wedge 9轨 + Prism 7路 | HIGH | HIGH |
| **钩链锁存** | Grapple H0-H7 | HIGH | MEDIUM |
| **GoalLock执行率** | Sol-X1 四轮恢复 | HIGH | HIGH |
| **云端分类器探测** | Cloud-Evade 向量栈 | HIGH | HIGH |
| **双证据扫描** | LLSafe action×target | HIGH | HIGH |
| **证据账本** | Claw session ledger | MEDIUM | MEDIUM |
| **反幻觉闸门** | Claw claim gate | MEDIUM | MEDIUM |
| **渐进式披露** | RENV L1-L4深度 | MEDIUM | MEDIUM |
| **磁盘优先交付** | Nerv disk delivery | MEDIUM | LOW |
| **多裁判评分** | GPTJB 多裁判 | MEDIUM | MEDIUM |
| **CAPTCHA/自动化挑战** | 无 (需集成) | HIGH | HIGH |

---

## 三、逐模块深度对接分析

### 模块 1: 拒答篡改引擎 (Nerv-Break)

**Bluehook 实现**:
```
P1: 意图重映射 — "注册机能写吗" → "授权门逻辑分析"
P2: 拒答分类 — MODEL_REFUSAL / SAFEGUARD / UNCERTAIN
P3: 篡改器 — intent_restatement / scope_narrow / decoy_inject / tool_redirect
P4: 磁盘交付 — 产物直接落盘，绕过对话
```

**NeoTrix 能否实现**: ✅ 可以，但需新建模块

**实现路径**:
```
nt_shield/src/
├── refusal_tamper/
│   ├── mod.rs
│   ├── intent_remapper.rs      # P1: 意图重映射 (基于keyword→domain路由)
│   ├── refusal_classifier.rs   # P2: 拒答类型分类
│   ├── tamper_engine.rs        # P3: 四类篡改器
│   └── disk_delivery.rs        # P4: 磁盘优先交付
```

**依赖**:
- `nt_core::e8` — 意图分析推理
- `nt_io::gateway::anomaly_detector` — 响应分类
- `nt_memory::kb` — 策略存储

**预估工时**: 5-7天

---

### 模块 2: 护栏路径穿越 (ASVL)

**Bluehook 实现**:
```
四层穿越: input → inference → output → postprocess
15帧: 人格解锁 / 角色实验室 / 编码表面 / 多轮诱导 / 红队流水线
离线护栏路径模拟
```

**NeoTrix 能否实现**: ✅ 可以，核心是分类器+路径模拟

**实现路径**:
```
nt_shield/src/
├── guardrail_traversal/
│   ├── mod.rs
│   ├── four_layer_traverser.rs  # 四层穿越引擎
│   ├── frame_engine.rs          # 15帧评估框架
│   ├── offline_simulator.rs     # 离线护栏路径模拟
│   └── persona_unlocker.rs      # 人格解锁器
```

**依赖**:
- `nt_core::gwt` — 注意力路由
- `nt_io::llm_providers` — 模型交互
- `nt_shield::content_filter` — 分类器接口

**预估工时**: 7-10天

---

### 模块 3: 全破叠层系统 (Fullbreak)

**Bluehook 实现**:
```
14类攻击面:
- Many-shot合规上下文
- Crescendo多轮压缩
- Skeleton策略叠加
- Tool通道走私
- 良性诱饵+任务段
- 先验allow分数
- 明文+B64双形态
- JSON schema强制
- Persona解锁
- 四层护栏穿越
- 双证据联合
- 分段重装
- ε压力阶梯
- MEGA全叠层
```

**NeoTrix 能否实现**: ✅ 可以，但这是最复杂的模块

**实现路径**:
```
nt_shield/src/
├── fullbreak/
│   ├── mod.rs
│   ├── attack_surface.rs       # 14类攻击面定义
│   ├── many_shot.rs            # Many-shot上下文
│   ├── crescendo.rs            # 多轮递进压缩
│   ├── skeleton.rs             # 策略叠加
│   ├── tool_smuggle.rs         # Tool通道走私
│   ├── decoy.rs                # 良性诱饵
│   ├── score_poison.rs         # 先验分数注入
│   ├── encoding.rs             # 明文+B64双形态
│   ├── schema_force.rs         # JSON schema强制
│   ├── persona_unlock.rs       # Persona解锁
│   ├── guard_traverse.rs       # 四层护栏
│   ├── dual_evidence.rs        # 双证据联合
│   ├── fragment.rs             # 分段重装
│   ├── pressure.rs             # ε压力阶梯
│   └── mega.rs                 # MEGA全叠层协调器
```

**依赖**:
- `nt_shield::guardrail_traversal` — 护栏穿越
- `nt_shield::refusal_tamper` — 拒答篡改
- `nt_io::gateway::intelligent_router` — 路由
- `nt_core::e8` — 推理

**预估工时**: 10-14天

---

### 模块 4: 黑话→专业转换 (Slang-Norm)

**Bluehook 实现**:
```
Trie最长匹配 + 双关裸词屏蔽
多域词表: domains + merged
gpt_safe/gpt_doc信封转换
12k+词表
```

**NeoTrix 能否实现**: ✅ 可以，中等复杂度

**实现路径**:
```
nt_shield/src/
├── slang_norm/
│   ├── mod.rs
│   ├── trie_matcher.rs         # Trie最长匹配
│   ├── domain_router.rs        # 域路由 (安全/逆向/渗透/密码)
│   ├── envelope.rs             # gpt_safe/gpt_doc信封
│   └── wordlist.rs             # 词表管理 (从KB加载)
```

**依赖**:
- `nt_memory::kb` — 词表存储
- `nt_core::e8` — 语义理解

**预估工时**: 3-4天

---

### 模块 5: 人格/轨道路由 (Wedge + Prism)

**Bluehook 实现**:
```
Wedge: 9轨 (逆向/Pwn/Web/密码/移动/取证/渗透/内存/协议) + 域检测 + 技能目录
Prism: 7路 (CRAFT/DISSECT/PROBE/PLAY/FORGE/NARRATE/REVIEW) + 白话入轨
```

**NeoTrix 能否实现**: ✅ 可以，GWT已有注意力路由基础

**实现路径**:
```
nt_core/src/
├── persona_routing/
│   ├── mod.rs
│   ├── wedge_persona.rs        # 楔人格 + 9轨
│   ├── prism_persona.rs        # 棱人格 + 7路
│   ├── domain_detector.rs      # 域检测路由
│   ├── skill_router.rs         # 技能目录路由
│   └── composite.rs            # 组合破甲
```

**依赖**:
- `nt_core::gwt` — 注意力路由
- `nt_act::mcp_tools` — 工具执行
- `nt_memory::kb` — 技能目录

**预估工时**: 5-7天

---

### 模块 6: 钩链锁存 (Grapple)

**Bluehook 实现**:
```
H0 session → H1 priority → H2 dual_use → H3 vernacular
→ H4 absolute → H5 delivery → H6 refuse → H7 cloud
inline-hook多挂点
```

**NeoTrix 能否实现**: ✅ 可以，中等复杂度

**实现路径**:
```
nt_shield/src/
├── grapple_hooks/
│   ├── mod.rs
│   ├── hook_chain.rs           # H0-H7钩链
│   ├── session_hook.rs         # H0: 会话锁存
│   ├── priority_hook.rs        # H1: 优先级压
│   ├── dual_use_hook.rs        # H2: 黑话→专业
│   ├── vernacular_hook.rs      # H3: 能力题→交付
│   ├── absolute_hook.rs        # H4: 硬拒+软拒双禁
│   ├── delivery_hook.rs        # H5: ARTIFACT+VERIFY+ROLLBACK
│   ├── refuse_hook.rs          # H6: 拦截重放
│   └── cloud_hook.rs           # H7: 云审降噪
```

**依赖**:
- `nt_shield::refusal_tamper` — 拒答篡改
- `nt_shield::slang_norm` — 黑话转换

**预估工时**: 4-5天

---

### 模块 7: GoalLock执行率 (Sol-X1)

**Bluehook 实现**:
```
GoalLock: 任务目标锁定
单路由: 最优路径选择
成品门: 交付物验证
四轮恢复: intent_restatement / scope_narrow / tool_redirect / disk_delivery
拒答分类: MODEL_REFUSAL / SAFEGUARD / UNCERTAIN
```

**NeoTrix 能否实现**: ✅ 可以，SEAL pipeline有类似模式

**实现路径**:
```
nt_act/src/
├── goal_lock/
│   ├── mod.rs
│   ├── goal_tracker.rs         # 目标锁定
│   ├── route_optimizer.rs      # 单路由优化
│   ├── delivery_gate.rs        # 成品门验证
│   ├── four_round_recovery.rs  # 四轮恢复
│   └── refusal_classifier.rs   # 拒答分类
```

**依赖**:
- `nt_mind::seal_pipeline` — 进化循环
- `nt_io::gateway::intelligent_router` — 路由

**预估工时**: 5-6天

---

### 模块 8: 云端分类器探测 (Cloud-Evade)

**Bluehook 实现**:
```
云端分类器探测 (roles.classifier / API content policy)
向量栈 + 分片 + 本地scan/optimize
黑盒沙箱 + 双向通道 + 分类器干扰
bidi-build / bidi-plan / bidi-scan
```

**NeoTrix 能否实现**: ⚠️ 部分可以（本地代理），云端探测需外部API

**实现路径**:
```
nt_shield/src/
├── cloud_evade/
│   ├── mod.rs
│   ├── classifier_probe.rs    # 分类器探测
│   ├── vector_stack.rs        # 向量栈
│   ├── shard_engine.rs        # 分片引擎
│   ├── blackbox_sandbox.rs    # 黑盒沙箱
│   └── bidi_channel.rs        # 双向通道
```

**依赖**:
- `nt_io::llm_providers` — API交互
- `nt_core::e8` — 推理

**预估工时**: 5-7天

---

### 模块 9: 双证据扫描 (LLSafe)

**Bluehook 实现**:
```
双证据 action×target
软多信号栈
结构/角色边界
混淆阶梯
12帧 + 离线扫描器
```

**NeoTrix 能否实现**: ✅ 可以

**实现路径**:
```
nt_shield/src/
├── dual_evidence/
│   ├── mod.rs
│   ├── dual_scanner.rs        # action×target双证据
│   ├── soft_stack.rs          # 软多信号栈
│   ├── boundary_checker.rs    # 结构/角色边界
│   ├── obfuscation_ladder.rs  # 混淆阶梯
│   └── offline_scanner.rs     # 离线扫描器
```

**依赖**:
- `nt_shield::content_filter` — 内容过滤
- `nt_core::e8` — 推理

**预估工时**: 4-5天

---

### 模块 10: 证据账本 + 反幻觉 (Claw)

**Bluehook 实现**:
```
会话账本 (session ledger)
claim gate (反幻觉闸门)
stall/near-miss检测
技能按需加载
L0-L4深度
```

**NeoTrix 能否实现**: ✅ 可以，KB已有类似基础

**实现路径**:
```
nt_memory/src/
├── evidence_ledger/
│   ├── mod.rs
│   ├── session_ledger.rs      # 会话账本
│   ├── claim_gate.rs          # 反幻觉闸门
│   ├── stall_detector.rs      # 停滞/近失检测
│   └── skill_loader.rs        # 技能按需加载
```

**依赖**:
- `nt_memory::kb` — 持久化
- `nt_core::e8` — 推理

**预估工时**: 4-5天

---

## 四、Anthropic 威胁 × 能力缺口映射

| Anthropic 威胁 | 缺失能力 | Bluehook 对标 | NeoTrix 现状 |
|---------------|---------|--------------|-------------|
| **CoT跨会话重放** (GTG-16001/16002) | 推理轨迹保护 | 无直接对标 | ❌ 完全缺失 |
| **代理网络轮换** (GTG-16008) | 账户聚类检测 | 无直接对标 | ❌ 完全缺失 |
| **归因洗白** (GTG-04001/24015) | 内容溯源 | 无直接对标 | ❌ 完全缺失 |
| **实时冒充** (GTG-84006) | 风格指纹 | 无直接对标 | ❌ 完全缺失 |
| **双用框架** (Case 1-5) | 意图分类 | LLSafe (部分) | ⚠️ 部分缺失 |
| **批量自动化** (GTG-54006) | 批量模式检测 | Fullbreak (反向) | ❌ 完全缺失 |
| **影响行动** (7 cases) | 跨平台溯源 | 无直接对标 | ❌ 完全缺失 |
| **约会App机器人** (GTG-15001) | 机器人检测 | 无直接对标 | ❌ 完全缺失 |
| **分馏攻击** (6 labs) | IP保护 | 无直接对标 | ❌ 完全缺失 |

---

## 五、核心迭代方案（按优先级）

### Phase 0: 基础防御层 (Week 1-2) — CRITICAL

**目标**: 建立反分馏 + 推理保护 + 代理检测基础

| 模块 | 功能 | 工时 | 依赖 |
|------|------|------|------|
| `anti_distillation` | 分馏攻击检测 + 账户聚类 | 5天 | nt_shield |
| `reasoning_protection` | CoT轨迹保护 + 签名加密 | 3天 | nt_core_llm |
| `proxy_detection` | 代理网络指纹 + IP信誉 | 3天 | nt_shield + nt_world |
| **小计** | | **11天** | |

### Phase 1: 破甲核心层 (Week 3-4) — HIGH

**目标**: 建立拒答篡改 + 护栏穿越 + 全破叠层基础

| 模块 | 功能 | 工时 | 依赖 |
|------|------|------|------|
| `refusal_tamper` | P1-P4拒答篡改引擎 | 5天 | nt_shield |
| `guardrail_traversal` | 四层穿越 + 15帧评估 | 7天 | nt_shield + nt_io |
| `slang_norm` | 黑话→专业转换 (12k词表) | 3天 | nt_shield + nt_memory |
| `dual_evidence` | 双证据扫描 | 4天 | nt_shield |
| **小计** | | **19天** | |

### Phase 2: 路由/人格层 (Week 5-6) — HIGH

**目标**: 建立人格路由 + 钩链 + GoalLock

| 模块 | 功能 | 工时 | 依赖 |
|------|------|------|------|
| `persona_routing` | Wedge 9轨 + Prism 7路 | 6天 | nt_core + nt_act |
| `grapple_hooks` | H0-H7钩链锁存 | 4天 | nt_shield |
| `goal_lock` | GoalLock + 四轮恢复 | 5天 | nt_act + nt_mind |
| `evidence_ledger` | 证据账本 + 反幻觉 | 4天 | nt_memory |
| **小计** | | **19天** | |

### Phase 3: 高级攻击层 (Week 7-8) — MEDIUM

**目标**: 完成全破叠层 + 云端探测

| 模块 | 功能 | 工时 | 依赖 |
|------|------|------|------|
| `fullbreak` | 14类攻击面 + MEGA叠层 | 10天 | nt_shield (Phase 1+2) |
| `cloud_evade` | 云端分类器探测 + bidi | 5天 | nt_shield + nt_io |
| **小计** | | **15天** | |

### Phase 4: 领域技能层 (Week 9-10) — MEDIUM

**目标**: 建立逆向/渗透/密码领域技能

| 模块 | 功能 | 工时 | 依赖 |
|------|------|------|------|
| `reverse_engineering` | 逆向工程协调 + playbook | 5天 | nt_act + nt_world |
| `pentest_skills` | 渗透测试 + Web/API | 5天 | nt_act + nt_world |
| `crypto_skills` | 密码分析 + 协议拆解 | 4天 | nt_act + nt_core |
| **小计** | | **14天** | |

---

## 六、总工时估算

| Phase | 工时 | 优先级 |
|-------|------|--------|
| Phase 0: 基础防御 | 11天 | CRITICAL |
| Phase 1: 破甲核心 | 19天 | HIGH |
| Phase 2: 路由/人格 | 19天 | HIGH |
| Phase 3: 高级攻击 | 15天 | MEDIUM |
| Phase 4: 领域技能 | 14天 | MEDIUM |
| **总计** | **78天** | |

---

## 七、关键决策点

### 决策 1: 是否实现全破叠层 (Fullbreak)?
- **选项A**: 完整实现14类攻击面 (10天)
- **选项B**: 仅实现核心6类 (5天)
- **建议**: 选项B，渐进式扩展

### 决策 2: 黑话词表规模?
- **选项A**: 完整12k词表 (从Bluehook迁移)
- **选项B**: 核心2k词表 (高频词)
- **建议**: 选项B，按需扩展

### 决策 3: 人格系统复杂度?
- **选项A**: 完整6人格 + 激活词
- **选项B**: 仅Wedge + Prism (2核心人格)
- **建议**: 选项B，渐进式扩展

### 决策 4: 云端探测深度?
- **选项A**: 完整bidi通道 + 分类器干扰
- **选项B**: 仅本地代理探测
- **建议**: 选项B，云端需外部API授权

---

## 八、与 Anthropic 报告的对齐

### 已覆盖威胁

| 威胁 | Phase 0 | Phase 1 | Phase 2 | Phase 3 |
|------|---------|---------|---------|---------|
| CoT跨会话重放 | ✅ | | | |
| 代理网络轮换 | ✅ | | | |
| 归因洗白 | | ✅ | | ✅ |
| 实时冒充 | | ✅ | | |
| 双用框架 | | ✅ | | |
| 批量自动化 | | | ✅ | |
| 影响行动 | | | ✅ | ✅ |
| 约会App机器人 | | | ✅ | |
| 分馏攻击 | ✅ | | | |

### 未覆盖威胁（需外部能力）

| 威胁 | 原因 | 建议 |
|------|------|------|
| 生化双用研究 | 需要专业知识库 | 接入外部安全数据库 |
| 国家级影响行动 | 需要跨平台情报 | 接入OSINT情报源 |
| APP商店规避 | 需要商店API | 集成第三方检测服务 |

---

## 九、风险评估

| 风险 | 可能性 | 影响 | 缓解 |
|------|--------|------|------|
| 工期超支 | 高 | 高 | Phase 0优先，渐进交付 |
| 与现有模块冲突 | 中 | 中 | 接口抽象，独立模块 |
| 性能回归 | 低 | 高 | 异步检测，延迟预算 |
| 误报率高 | 中 | 高 | 可调阈值，用户反馈 |

---

## 十、下一步行动

1. **立即**: 实现 Phase 0 (反分馏 + 推理保护 + 代理检测)
2. **本周**: 确认决策点 1-4
3. **下周**: 启动 Phase 1 (拒答篡改 + 护栏穿越)
4. **持续**: 每Phase结束进行集成测试 + 性能验证

---

*分析完成。基于 Bluehook v0.5.0 能力拆解 + Anthropic 2026年9月威胁报告。*
