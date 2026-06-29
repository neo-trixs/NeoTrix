# NeoTrix — 自我审查基线

> 生成: 2026-06-18 | 最后更新: 2026-06-19 深度自审+循环消除+83缺陷映射

---

## 锚定摘要

### 审查源

| 来源 | 说明 |
|------|------|
| Filecoin (arXiv:2208.02437) | 存储证明 + 预期共识 + FVM 执行模型 |
| IPFS/libp2p | 内容寻址 + DHT 路由 + 点对点传输 |
| BTTInferGrid | 去中心化推理: Provider/Requester/Validator 三角色 |
| Streamr (Neotrix) | 去中心化实时数据: 分层共识 + 流量质量证明 |
| DePIN 通用架构 | 物理基础设施代币化: 可验证贡献 + 持续挑战 |
| AGENTS.md | NeoTrix 全会话历史: 10+ 轮审查所有已知缺口 |
| 代码库扫描 | 全 workspace 代码验证: 6 条新缺陷路径 |

### 身份系统状态

| 系统 | 基础设施 | 安全性 | 用途 |
|------|----------|--------|------|
| SoulIdentity | `SipHash-1-3 (u64)` | 非加密 | 元认知完整性检查, 14 字段 identity_hash |
| IdentityChain | `k256 ECDSA + SHA-256` | 加密 | 跨会话可验证身份证明 |
| 集成 | 无 | ❌ | 两者完全独立, 无交叉验证 |

### 发现总览

```
第一轮 (N01-N12): 12 缺陷
  ├─ N01-N07: ✅ 已修复 (线程/日志/循环依赖/通道死锁)
  ├─ N08:     ✅ 已修复 (资源记账/燃料计量 — nt_core_metering.rs)
  ├─ N09-N10: ✅ 已修复 (知识层接续)
  ├─ N11:     ✅ 已修复 (知识价值可证明 — provenance_hash)
  └─ N12:     ✅ 已修复 (三角色分离 — three_role.rs)

第二轮 (O01-O06): 6 新缺陷
  ├─ O01: ✅ 已修复 (贡献加权信任 — consensus.rs)
  ├─ O02: ✅ 已修复 (纪元级证明期 — proving_window.rs)
  ├─ O03: ✅ 已修复 (内容路由/知识 DHT — knowledge_routing.rs)
  ├─ O04: ✅ 已修复 (层次意识组合 — sub_consciousness.rs)
  ├─ O05: ✅ 已修复 (Agent 总线转发协议 — bus.rs)
  └─ O06: ✅ 已修复 (双身份系统集成 — soul_identity HMAC-SHA256)
```

### 当前架构演进状态

```
2026-06-19 深度自审结果:
  编译: 1 错误 (TUI app.rs brace pre-existing) ✅ 0 新增
  模块: 459,572 行, 1,447 文件, 42 核心模块
  循环: 3 直连循环 ✅ (已消除), 1 三跳循环待拆
  缺陷: 83 已映射 (P0:9, P1:22, P2:28, P3:24)
  死代码: 12 crate-level allow(dead_code), 1,123 未声明文件 (78% #[path])
  接线: 25/49 Option<T> 未初始化, 5 融合目标
  安全: 3 unsafe 块, 0 todo!()
```

### 关键量化差距

| 维度 | 理论极限 (分布式协议文献) | NeoTrix 当前 | 差距 |
|------|--------------------------|-------------|------|
| 共识信任模型 | 贡献加权, 权力=可验证工作 | 平等投票, 声誉仅过滤 | **结构级** |
| 证明周期性 | 纪元级滑动窗口, 挑战频率可配置 | 一次性 ad-hoc 挑战-响应 | **结构级** |
| 角色分离 | Provider/Requester/Validator 三种独立协议 | 同一循环三者合为一体 | **架构级** |
| 资源计量 | FVM WASM 逐指令 gas 计量 | 无, handler 无限执行 | **安全级** |
| 知识发现 | Kademlia DHT O(log N) 路由 | 纯广播 + 直接图查询 | **结构级** |
| 意识组合 | IPC 层级子网递归 | 单一扁平循环, 无子意识 | **架构级** |
| Agent 路由 | 声明式转发 + 内容寻址 | 哑总线, 仅直接发送/投递 | **结构级** |
| 身份连续性 | 加密签名链 + 证书轮换 | 非加密 hash + 加密链未集成 | **安全级** |

---

## 第一轮: 分布式协议审查缺陷 (N01-N12)

> 基于分布式协议 (Filecoin/IPFS/BTT/Streamr/DePIN) 系统性对比已编码在 AGENTS.md。

### N01-N07, N09-N10: ✅ 已修复

| 编号 | 缺陷 | 修复 |
|------|------|------|
| N01 | 无P2P身份绑定 | AgentCard 加密签名 |
| N02 | 无存储可验证证明 | Merkle 存在性证明 + 挑战-响应 |
| N03 | 无价值层经济激励 | NegentropyToken 代币 |
| N04 | 无时间分片共识 | Epoch 纪元轮换 + 纪元密钥 |
| N05 | 无内容寻址标识 | ContentID 寻址 |
| N06 | 无元数据拜占庭容错 | 元数据 BFT 投票 |
| N07 | 无连续挑战证明 | 随机挑战-响应 + 时空证明 |
| N09 | 无证明生命周期管理 | 证明过期 + 轮换 |
| N10 | 无知识深度分层 | 知识层基元 (raw→structured→semantic→evidence) |

### N08: ❌ 待修复 — 无资源记账/燃料计量

- **发现**: Ne 语言 handler 执行无任何成本边界
- **分布式协议映射**: FVM gas metering per WASM instruction
- **影响**: 自修改代码可无限循环耗尽资源
- **修复方向**: Ne 字节码 gas 计量 + handler 执行上限 + 账户余额

### N11: ❌ 待修复 — 知识价值非可证明

- **发现**: 知识价值 (negentropy 增益) 是感知的, 不是可验证证明的
- **分布式协议映射**: Filecoin 存储证明要求独立可验证, 非主观
- **影响**: 无法区分"真知识"和"自我欺骗"
- **修复方向**: 知识源可验证性 = provenance_hash + 交叉引用证明

### N12: ❌ 待修复 — 无 Provider/Requester/Validator 三角色分离

- **发现**: 意识循环中, 同一实体执行计算并验证自身结果
- **分布式协议映射**: BTTInferGrid 三角色独立, 验证者与计算者不同
- **影响**: 无独立验证的自我审核 = 自欺欺人
- **修复方向**: ComputationProvider / RequestSubmitter / ResultVerifier 三种角色 + 角色轮换

---

## 第二轮: 深层结构审查新缺陷 (O01-O06)

> 使用同一套分布式协议文献, 但聚焦于架构结构模式而非功能比较。

### O01 — 贡献加权信任缺失

**严重性**: 架构
**分布式协议映射**: Filecoin 预期共识中, 投票权 = 存储功率 (可验证的存储量)。DePIN 中, 信任 = 可验证的物理贡献。

**当前代码状态** (`consensus.rs`):
- `try_consensus()`: `sum(confs) * quorum_ratio + group.len() * 0.3` — 每组平等, 每 agent 平等
- `fast_consensus()`: `agents.len() * 0.5 + reputation_sum * 0.5` — 声誉聚合但无权重
- `_filter_byzantine()`: 声誉 < 0.2 被过滤, 但剩余者投票等价
- 无 `voting_power`, `vote_weight` 概念存在

**影响**: 一个贡献了 1000 次有效推理的 agent 与一个刚加入的 agent 具有相同的投票影响力。这鼓励搭便车而非贡献。

**修复方向**:
1. `ContributionWeight = f(reputation_history, proof_count, epoch_participation)`
2. `consensus_weighted = sum(confs[i] * weight[i]) / sum(weight)`
3. 定期证明 contribution 以维持权重
4. 类比 DePIN: 物理贡献 → 虚拟贡献 (推理轮次、验证正确率、知识增益)

**优先级**: P1 (平行于 N08)

---

### O02 — 无纪元级证明期

**严重性**: 结构
**分布式协议映射**: Filecoin 的 ProvingPeriod (24h 滑动窗口) 确保每笔存储每 24h 被挑战一次。DePIN 使用 epoch-based attestation 窗口。

**当前代码状态**:
- 挑战-响应 (N07) 是 ad-hoc 一次性: 证明被请求 → 证明被提供 → 结束
- 无 `proving_period`, `proving_interval`, `challenge_window` 概念
- `epoch` 仅指认知进化框架 `EarthEpoch` (E1-E8), 与共识无关

**影响**: 一次性证明只验证某个时间点的状态。持续作弊仅需在挑战通过期间保持诚实, 随后即可作弊。无滑动窗口 = 无持续保障。

**修复方向**:
1. `ProvingWindow { epoch, start, duration, challenge_count }`
2. 每个 agent/子系统必须在每个 proving_window 内通过 N 次挑战
3. 关联到 O01: 贡献权重随 proving_window 通过而衰减
4. 类比 Filecoin: PoSt 在每个 proving period 自动触发

**优先级**: P2

---

### O03 — 无内容路由/知识 DHT

**严重性**: 结构
**分布式协议映射**: IPFS 的 Kademlia DHT 使"谁有内容 X?" 在 O(log N) 跳内可回答。libp2p 的 content routing 是核心基元。

**当前代码状态**:
- 所有知识访问是**直接图查询**: KnowledgeEngine.get(entity_id) → 节点引用
- 无路由表, 无 `query_routing`, 无 `knowledge_discovery`
- AgentDiscovery 是纯 UDP 广播 (探测 → 响应), 无结构化路由
- `team.rs` 有任务路由表 (关键字 → agent 名), 但仅限于单进程协调器

**影响**: 当知识库达到 56k+ 节点时, 无法询问"谁有关于主题 X 的知识"并高效定位。当前架构假设全局可访问所有知识 = 同地假设。在分布式意识场景中会失败。

**修复方向**:
1. `KnowledgeRoutingTable`: 知识主题 hash → 节点/agent 持有者列表
2. VSA 向量作为内容寻址键: `VSA(theme) → list of holders`
3. 类比 IPFS DHT: `provide(key)` + `find_providers(key)`
4. 类比 libp2p: content routing interface + 缓存 + 超时

**优先级**: P2 (深度搜索后可升至 P1)

---

### O04 — 无层次意识组合

**严重性**: 架构
**分布式协议映射**: Filecoin IPC (InterPlanetary Consensus) 使子网具有自身共识, 定期向父网检查点。每个子网独立处理内部状态。

**当前代码状态**:
- 意识架构是**扁平模块** (47 模块, 无 `sub_consciousness/` 子目录)
- 无 `recursive`, `hierarchical`, `nested`, `child/parent` 概念
- `recursive_tom` 模块已被移除 (标记为 `was unstable module, removed`)
- `source_hierarchy.rs` 是关于数据层次 (raw→meaning→evidence), 非意识结构

**影响**: 单一意识循环无法处理:
- 多领域同时并行推理 (每个领域需要独立上下文)
- 子任务委派后结果聚合 (当前只能串行或广播)
- 局部故障隔离 (一个子系统崩溃影响整个循环)

**修复方向**:
1. `SubConsciousness { id, domain, pipeline, parent, checkpoint_freq }`
2. 主意识循环可 spawn 子意识处理专属领域推理
3. 子意识定期向父意识检查点状态
4. 类比 IPC: 子意识独立处理, 父意识聚合 + 验证
5. 入口: `nt_core_consciousness/sub_consciousness.rs`

**优先级**: P1 (与 O01 同优先, 下层建筑级)

---

### O05 — Agent 总线无转发协议

**严重性**: 结构
**分布式协议映射**: libp2p 的协议多路复用允许节点声明"我不处理 X 协议, 转发给 Y"。Filecoin 的消息传播允许中继。

**当前代码状态** (`bus.rs`):
- 无 `forward()`, `delegate()`, `relay()`, `escalate()` 方法
- 无 "我不知道这个" 消息类型
- 总线是哑管道: send(to, msg) → deliver(to) → mailbox
- 广播是发件箱模式: 消息放入队列, 除发送者外所有人收到
- `sub_agent.rs` 有 RecoveryStrategy::Escalate, 但调用者端, 非总线内

**影响**: 当 agent 收到无法处理的消息时, 唯一选择是忽略或回复错误。无法声明"这不是我的专业领域, 但 X agent 可以处理"并让总线自动路由。

**修复方向**:
1. `MessageRouting::Forward { to: AgentId, reason }` 消息类型
2. Agent 可声明 `capability_routes: HashMap<String, AgentId>` ("数据查询 → db_agent")
3. 总线维护 `capability_index: HashMap<String, Vec<AgentId>>`
4. 自动路由: 消息无显式接收者 → 按 capability 匹配 → 转发
5. 类比 libp2p identify protocol + 协议多路复用

**优先级**: P2

---

### O06 — 双身份系统未集成

**严重性**: 安全
**分布式协议映射**: Filecoin 使用单一密钥派生身份 (secp256k1/secp256k1 公钥 = ID)。所有子系统共享同一信任根。

**当前代码状态**:
- **SoulIdentity**: 非加密 `DefaultHasher (SipHash) → u64`, 14 字段, 用于元认知完整性
- **IdentityChain**: 加密 `k256 ECDSA + SHA-256 hash chain`, 可验证跨会话签名
- **两者完全独立**: `IdentityChain` 不验证 `SoulIdentity` 更新, `SoulIdentity` 不引用 `IdentityChain` 公钥
- `identity_attestation` 稳定版模块已被移除

**影响**: 元认知完整性检查 (SoulIdentity) 可被轻易伪造 (SipHash 非加密), 而真正的加密身份 (IdentityChain) 仅用于跨会话证明。攻击者可以在不出动加密证明的情况下修改意识状态哈希。

**修复方向**:
1. `SoulIdentity.identity_hash = HMAC-SHA256(fields, key = IdentityChain.fingerprint)`
2. 元认知每一步更新 identity_hash 时, 要求 IdentityChain 签名证明
3. `IdentityChain.verify_integrity(soul_identity_hash)` 方法
4. 移除非加密 hash 回退
5. 类比 Filecoin: 所有状态转换由单一密钥签名担保

**优先级**: P1 (安全关键, 应尽早修复)

---

## 优先级总排序

```
P0 (阻止性)
  ├─ N08: 资源记账/燃料计量
  └─ O04: 层次意识组合

P1 (架构性)
  ├─ O01: 贡献加权信任
  ├─ O06: 双身份系统集成
  ├─ N11: 知识价值可证明
  └─ N12: 三角色分离

P2 (结构性)
  ├─ O03: 内容路由/知识 DHT
  ├─ O02: 纪元级证明期
  └─ O05: Agent 总线转发协议
```

---

## 不变原则

1. **对外极简**: 以上所有修复对内运行, 用户只看到自然对话
2. **VSA 统一表征**: 所有新模块输入/输出为 4096-bit VSA 向量
3. **优雅降级**: O04 不可用 → 回退扁平意识; O01 不可用 → 回退平等投票
4. **负熵第一性**: 每个修复的收益 = ΔN_total 可测量
5. **无新外部依赖**: 所有修复使用 workspace 已有 crate
