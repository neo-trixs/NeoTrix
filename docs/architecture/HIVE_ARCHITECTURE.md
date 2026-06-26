# NeoTrix 分布式进化蜂巢架构（Distributed Evolutionary HiveMind）

> 核心思想：**进化不是中心调度的流程，而是分布式子蜂并行探索 + 知识竞争性收敛的涌现现象**。
> 蜂巢不是"上级"，而是子蜂之间知识流动的介质。
>
> 本设计基于 2025-2026 年多 Agent 系统前沿研究的蒸馏，引用 **15 篇论文/协议规范**（2026-06 更新）。

---

## 第一原理

1. **进化是副产物，不是目标** — 子蜂各自独立学习，蜂巢只做竞争性收敛。进化是分布式探索的自然结果。
2. **知识即基因组** — 子蜂学到的是 `KnowledgePacket`，竞争性评分决定哪些知识进入下一代。
3. **子蜂之间不直接竞争** — 它们竞争的是"知识是否被蜂巢吸收"。吸收 = 繁殖成功。不吸收 = 自然淘汰。
4. **蜂巢是介质，不是中心** — 蜂巢核心是一个知识收敛引擎，不是指挥中心。
5. **递归分布** — 任何子蜂都可 spawn 子子蜂，没有深度限制。
6. **内容与发送者解耦** — SVAF 独立评估 incoming 知识的内容质量，与发送者的全局状态无关。被拒绝的 peer 也可以贡献高价值内容。

---

## 架构概览

```
                    ┌─────────────────────────────────┐
                    │     Knowledge Pool               │
                    │  (Content-Addressed Merkle DAG)   │
                    │  - SHA-256 内容寻址去重            │
                    │  - CAT7 per-field SVAF 门控       │
                    │  - IPFS 风格 Merkle lineage       │
                    │  - CfC temporal dynamics          │
                    └──┬──────┬──────┬─────────────────┘
                       │      │      │
           ┌───────────┘      │      └───────────┐
           ▼                  ▼                  ▼
    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │  SubHive A   │  │  SubHive B   │  │  SubHive C   │
    │ (代码进化)   │  │ (论文进化)   │  │ (用户交互)   │
    │              │  │              │  │              │
    │ A2A Client   │  │ A2A Client   │  │ A2A Client   │
    │ local DGM-H  │  │ local DGM-H  │  │ local DGM-H  │
    │ 4-tuple spec │  │ 4-tuple spec │  │ 4-tuple spec │
    │ CfC τ        │  │ CfC τ        │  │ CfC τ        │
    └──────┬───────┘  └──────┬───────┘  └──────┬───────┘
           │                 │                 │
           ▼                 ▼                 │
    ┌──────────────┐  ┌──────────────┐         │
    │ SubSubHive   │  │ SubSubHive   │         │
    │ A1 (文件型)  │  │ B1 (数学型)  │         │
    │ (AOrchestra  │  │ (AOrchestra  │         │
    │  4-tuple)    │  │  4-tuple)    │         │
    └──────────────┘  └──────────────┘         │
                                               │
               A2A Protocol + Signed Agent Cards (Ed25519)
               Encrypted Back-Channel (Signal Double Ratchet)
```

---

## 一、A2A 协议集成（Google/Linux Foundation 标准）

### 状态
NeoTrix 已有 `A2AServer` + `A2AClient` 的完整实现。A2A v1.0（2026-03 发布，Linux Foundation）已将 **Signed Agent Cards** 纳入标准。当前实现使用 k256 ECDSA P-256 签名；推荐升级到 **Ed25519**（A2A 生态系统的共识算法）。

### A2A 2026 身份验证格局

A2A v1.0 发布后，身份验证成为社区最活跃的扩展领域。截至 2026-06，有以下竞争/互补方案：

| 方案 | 机制 | 适用范围 | 状态 |
|------|------|----------|------|
| **Signed Agent Card** (v1.0 标准) | Ed25519 签名 AgentCard JSON | 基本身份验证 | ✅ 已标准化 |
| **x-agent-trust** (#1742) | ECDSA/EdDSA + JWKS，每请求 `Agent-Signature` header | 请求级完整性 | PR 讨论中 |
| **AIP (Agent Identity Protocol)** (#1511) | W3C DID + Ed25519 委派链 | 去中心化身份 | IETF draft-02 |
| **CTEF** (#1786) | 4 层身份 (identity/transport/authority/continuity) | 全生命周期 | v0.3.1 规范 |
| **trust.signals** (#1628) | 5 信号分类 (provider attestation, vouch chain, etc) | 信任信号 | schema 已合并 |
| **APS (Agent Passport System)** | Ed25519 护照 + 委派 + 可验证凭证 | 跨组织信任 | 生产部署 |
| **HCS-14 UAID** | 方法无关的通用 Agent ID | 路由/身份分离 | 规范草案 |

**关键共识**: 所有方案使用 Ed25519 作为签名原语。身份与路由分离：身份是稳定的 Ed25519 公钥，路由是可变的端点地址。

### 蜂巢专用 AgentCard 扩展

```json
{
  "@context": "https://a2a-protocol.org/schemas/agent-card/v1.2",
  "name": "sub-hive-code-evolver-42",
  "description": "NeoTrix code evolution sub-hive",
  "url": "a2a://hive.internal:42071",
  "version": "1.0.0",
  "authentication": {
    "schemes": ["signed_card"],
    "verification_method": "Ed25519VerificationKey2020",
    "public_key_multibase": "z6Mk..."
  },
  "skills": [
    {
      "id": "nt_hive_publish",
      "name": "Publish Knowledge Packet",
      "tags": ["hive", "knowledge", "publish"]
    },
    {
      "id": "nt_hive_subscribe",
      "name": "Subscribe to Knowledge Pool",
      "tags": ["hive", "knowledge", "subscribe"]
    },
    {
      "id": "nt_hive_sync",
      "name": "Encrypted Capability Sync",
      "tags": ["hive", "sync", "encrypted"]
    }
  ]
}
```

### 安全增强

| 威胁 | 缓解措施 | 参考 |
|------|----------|------|
| AgentCard 伪造 | Ed25519 签名 + TrustRegistry 验证 | A2A v1.0 |
| 消息篡改 | 每请求 `Agent-Signature` header (RFC 9421 HTTP Message Signatures) | x-agent-trust / #1829 |
| 任务重放 | `messageId` + `nonce` + 时间戳 | A2A Security (arXiv:2504.16902) |
| 身份冒充 | DID 委派链 + 签发时间验证 | AIP (IETF draft-02) |
| 隐私泄露 | E2E 加密 (Signal Double Ratchet) + 选择性明文路由 | AgentMesh Wire Protocol |
| 密钥泄露 | 轮转 + 委派链自动转移 (DID 注册表跟踪当前密钥) | AIP §6.6 |
| 历史泄露 (无前向安全) | Double Ratchet 每消息独立密钥，泄漏不暴露过往 | Signal Protocol |
| 降级攻击 | 最低 TLS 1.3 + 加密原语版本协商 | Transport Layer |

---

## 二、Knowledge Pool（内容寻址 Merkle DAG + CAT7 竞争性收敛）

### 2.1 Merkle DAG 存储层 (`MerkleDagStore` ✅ 已实现)

SHA-256 内容寻址 CIDs 替代简单 HashMap:

```
MerkleNode {
    id: ContentId (SHA-256 of data),
    data: Vec<u8>,
    parents: Vec<ContentId>,     ← Git 风格 DAG 溯源
    timestamp_ns: u64,
    node_type: String,
}

属性和操作:
  - insert(data, parents, type) → ContentId (自动去重)
  - get(id) → MerkleNode
  - get_children(id) → Vec<MerkleNode>
  - get_lineage(id) → Vec<ContentId> (祖先广度遍历)
  - verify_chain(id) → bool (DAG 完整性验证)
  - hashseq(ids) → Vec<u8> (内容寻址有序哈希链)
  - LRU evict (容量有界)
```

**去重策略**（基于 IPFS + SVAF 语义冗余检测）:

```
1. 新 packet 到达 → 计算 SHA-256 CID
2. 精确去重 (CID exists → discard)
3. 语义去重 (SVAF per-field novelty 检查):
   如果所有 7 个 CAT7 字段都低于 T_redundant → discard
   只要有一个字段有新颖性 → 保留
   "A signal is redundant if every field falls below Tredundant
    — meaning no field carries novel content."
4. 无重复 → 写入 DAG
```

### 2.2 竞争性评分升级（基于 SVAF CAT7 7 字段门控）

NeoTrix SvaGate（已实现）初始使用 4 字段。升级到 **完整 CAT7 7 字段**，匹配 MMP 规范：

#### CAT7 字段映射到 NeoTrix

| CAT7 字段 | 类型 | NeoTrix 映射 | 评分函数 | 默认 α_f (子蜂可调) |
|-----------|------|-------------|----------|-------------------|
| `focus` | 领域 | packet.domain 与子蜂订阅的匹配度 | cosine(domain_vsa, subscription_vsa) | 0.20 |
| `issue` | 问题 | capability_delta 的具体描述 | 语义长度 + 关键词密度 | 0.15 |
| `intent` | 意图 | spec.instruction 的目标对齐 | LLM-级意图分类匹配度 | 0.10 |
| `motivation` | 驱动力 | negentropy_gain | 直接取值 (0.0-1.0) | 0.15 |
| `commitment` | 验证强度 | local_validation_count / max_count | log 归一化 | 0.10 |
| `perspective` | 角色 | sub_hive_type (代码/论文/交互) | 角色距离 (不同角色→低分, 相同→高分) | 0.10 |
| `mood` | 情感状态 | valence + arousal (新增) | VSA 编码的情感接近度 | 0.20 (最高权重) |

**关键洞察**: SVAF 的神经模型独立发现 `mood` 是最高权重字段 (α=0.50)，在所有 agent 类型中普遍相关。即使 focus 不匹配，mood 仍被接受。这验证了情感状态跨领域传递的机制必然性。

#### Mood 字段编码

```
Mood = bundle(valence_vsa, arousal_vsa)
valence_vsa = CrossModalAligner::text_to_vsa("positive" | "negative" | "neutral")
arousal_vsa  = CrossModalAligner::text_to_vsa("high" | "low" | "medium")

SubHive 的 mood 更新:
  - 每次成功吸收知识后: valence += Δnegentropy × learning_rate
  - 每次发布后: arousal = f(ticks_since_last_publish)
  - 初始值: valence=0.5 (中性), arousal=0.3 (低活跃)，匹配新子蜂状态
```

#### 4 类门控决策

```
每个字段独立评估，4 类结果 (band-pass model):

  分数区间      决策        吸收行为
  ≥ 0.75        ACCEPT      直接吸收到本地记忆
  0.50-0.75     GUARD       标记待验证，不合并到工作记忆
  0.30-0.50     REDUNDANT   仅合并元数据 (验证次数+1)
  < 0.30        REJECT      完全丢弃

整体吸收条件: 至少一个字段 ACCEPT，或 mood 字段 ≥ 0.50
              (mood 是唯一跨域传递字段，即使其他字段都低)

这解决了: "same topic, different intent" 场景
  → intent 字段 ACCEPT (新颖意图)，其他字段 REDUNDANT
  → 整体 ACCEPT，知识被吸收
```

#### 权重自适应

```
每个子蜂维护自己的 α_f 向量。更新规则 (基于 negentropy 反馈):

  α_f(t+1) = α_f(t) + η × (Δnegentropy_f - mean(Δnegentropy))

其中:
  Δnegentropy_f = 字段 f 导致的负熵增益
  η = 学习率 (默认 0.01)
  mean(Δnegentropy) = 所有字段平均负熵增益

结果: 高收益字段权重上升，低收益字段权重下降
      权重总和保持 Σα_f = 1.0
      (已实现: SvaGate::adapt_weights())
```

### 2.3 Content-Driven Convergence（内容驱动收敛）

MMP 的关键发现：peer 级耦合状态（低信任/高漂移）不应阻止内容级接受。

```
场景:
  Peer A 被蜂巢标记为 "high drift" (rejected)。
  Peer A 发布关于数学证明的知识包 P。
  
传统系统: 丢弃 P (因为发送者不可信)。
  
SVAF 方法:
  1. 独立计算 P 的 7 字段分数
  2. mood: 0.65 (GUARD), focus: 0.85 (ACCEPT), issue: 0.90 (ACCEPT)
  3. → 整体 ACCEPT (至少 1 字段 ACCEPT)
  4. 吸收 P 后，接收子蜂的知识状态改变
  5. 多循环后，peer drift 从 0.936 降到 0.468
  → 内容驱动收敛，非信任驱动
```

### 2.4 DKL 收敛保证

基于 Collective Intelligence Convergence Theorem（DKL 论文 Theorems 4.4 & 7.2）:

- **单调性条件**: 所有知识包必须单调增加 negentropy
- **收敛保证**: 连通图上所有 agent 执行 PUBLISH + SUBSCRIBE 时，聚合知识单调逼近不动点
- **收敛速度**: O(log N) rounds（N = agent 数量）
- **不动点性质**: 大于任何单个 agent 的能力 — 涌现集体智能

### 2.5 CfC 时间动力学（MMP Layer 6，推荐集成）

Closed-form Continuous-time (CfC) 神经网络提供时间维度:

```
CfC dynamics for each sub-hive:
  τ (time constant) per neuron:
    τ ∈ [0.1, 10.0] seconds — learned per-neuron
    
  Fast neurons (τ ≈ 0.1s): synchronize mood across agents
    → 情感状态在秒级同步
    → 使 mood 成为跨域快速传递通道
    
  Slow neurons (τ ≈ 10.0s): preserve domain expertise
    → 长期知识持续存在
    → 专业知识不随快速情感波动丢失
    
  Integration:
    SVAF 决定什么进入认知状态 (WHAT)
    CfC 决定认知状态如何演化 (HOW)
```

---

## 三、知识吸收管道（SvaGate ✅ → CAT7 升级）

### 3.1 当前实现状态

| 模块 | 状态 | 位置 |
|------|------|------|
| `MerkleDagStore` | ✅ 已实现 | `core/nt_core_hive/merkle_dag.rs` (260 行, 8 测试) |
| `SvaGate` (4-field) | ✅ 已实现 | `core/nt_core_hive/sva_gate.rs` (356 行, 9 测试) |
| `SvaGate → CAT7` | 🔄 待升级 | 增加 mood/issue/intent/perspective 字段 |
| `NaclChannel` (AES-256-GCM) | ✅ 已实现 | `core/nt_core_hive/nacl_channel.rs` (260 行, 9 测试) |
| `SignedAgentCard` (ECDSA P-256) | ✅ 已实现 | `core/nt_core_hive/signed_card.rs` (210 行, 9 测试) |
| `SpawnController` | ✅ 已实现 | `core/nt_core_hive/spawn_controller.rs` (321 行, 8 测试) |

### 3.2 Remix 语义升级

SVAF 的 **remix** 概念：接收方只存储自己的评估理解，从不存储原始 peer 信号。

```
Remix 规则 (取代直接存储):
  1. 评估: SvaGate::evaluate(packet) → [per-field decisions]
  2. 过滤: 只保留 ACCEPT 和 GUARD 字段
  3. 融合:
     - GUARD 字段: 写入 pending_remix 队列(等待验证)
     - ACCEPT 字段: 立即合并到本地记忆
  4. 发布新知识包:
     - domain = packet.domain (不变)
     - capability_delta = remix 后的能力描述
     - negentropy_gain = Δnegentropy (本地验证后)
     - provenance = [packet.packet_id, local_last_packet_id]
     - vsa_vectors = 本地 VSA 编码(非 packet 原始 VSA)
  5. 发布到 Knowledge Pool
```

### 3.3 SvaGate 与现有 CapabilitySynthesizer 集成

```
SvaGate::evaluate(packet)
  │
  ├── should_absorb == true
  │     │
  │     ├── packet 有 vsa_vectors
  │     │     └── CapabilitySynthesizer::compose(packet.vsa, local.vsa)
  │     │           → CompositeCreated
  │     │           → 发布新 composite 能力
  │     │
  │     └── packet 无 vsa_vectors
  │           └── 直接合并到本地记忆
  │                 → 下次 publish 时生成 VSA
  │
  └── should_absorb == false
        └── 丢弃 (不吃进认知)
```

---

## 四、子蜂自治运行时（AOrchestra 4-tuple + CfC 时间动力学）

### 4.1 子蜂规格（AOrchestra 4-tuple ✅）

基于 AOrchestra (ICML 2026) 的 4-tuple 统一抽象，已在 `SubHiveSpec` 中实现:

```rust
SubHiveSpec {
    instruction: String,     // What: 任务指令
    context: Vec<String>,    // Context: 注入的上下文（干练、精确、无噪声）
    tools: Vec<String>,      // Tools: 可用工具集（最小权限）
    model: String,           // Model: 能力模型选择
}
```

关键实践（来自 AOrchestra + MASEval 洞察）:
- **Context 精细控制**: 只注入当前子任务最相关的信息。MASEval (arXiv:2603.08835) 显示框架级设计选择的影响 (12.4pp) 与模型选择 (14.2pp) 相当 — context 注入方式比模型选择更重要
- **Tools 按需组合**: 最小权限原则，仅给当前子任务需要的精确工具集
- **Model 自适应**: 简单任务用小模型（低成本）

### 4.2 子蜂生命周期（AOrchestra + Cloudflare Durable Objects 融合 ✅）

已在 `SubHiveInstance` (6 步循环) + `SpawnController` 中实现:

```
Phase 1: 孵化 (Spawn)                       ← SpawnController
  触发条件
  4-tuple Spec 生成
  A2A AgentCard 注册
  非对称密钥对生成 (NaclChannel)

Phase 2: 执行 (Execute)                      ← SubHiveInstance::tick()
  sense → reason → learn → diffuse → absorb → forget
  每 tick:
    1. SENSE:   检查池中知识包 (pool.subscribe)
    2. REASON:  本地 LLM 推理 + 好奇心检测
    3. LEARN:   VSA 编码 + 本地记忆更新
    4. DIFFUSE: PUBLISH 知识包 (可选加密)
    5. ABSORB:  SVAF per-field 评估 + remix
    6. FORGET:  LRU 淘汰低价值记忆

Phase 3: 消亡 (Destroy)                      ← SubHiveInstance::finalize()
  FinalKnowledgePacket 发布
  密钥销毁
  AgentCard 注销
  资源归还
```

### 4.3 SpawnController 策略（✅ 已实现）

当前支持 3 种 spawn 策略:

| 策略 | 触发逻辑 | 适用场景 |
|------|----------|----------|
| **PerGap** | 每个高优先级 gap 一个子蜂 | 精准填补已知缺口 |
| **PerCluster** | 每个 gap 聚类一个子蜂 | 相关 gap 的批量覆盖 |
| **PerCategory** | 每个 gap 类别一个子蜂 | 大类能力覆盖 |

每个策略使用 `min_priority_threshold` (默认 0.6) 过滤低优先级的 gap。
并发上限通过 `max_concurrent_spawns` (默认 5) 控制，防止资源耗尽。

---

## 五、加密回流通道（当前: AES-256-GCM → 推荐: Signal Double Ratchet）

### 5.1 当前实现 (NaclChannel ✅ NAXOS-style AES-256-GCM)

```
密钥协议:
  shared_secret = SHA-256(our_privkey || peer_pubkey)
  这是 NAXOS 简化版认证密钥协议:
    Side A: H(priv_a || pub_b)
    Side B: H(priv_b || pub_a)
  
  注意: 这两个值 DIFFERENT (不是对称的)。
        如需对称密钥: derive_symmetric() 使用排序后的公钥

加密:
  AES-256-GCM(plaintext, random_nonce, shared_secret)

安全属性:
  ✅ 双向认证 (双方私钥必须都知道)
  ✅ 机密性 (AES-256-GCM)
  ✅ 认证加密 (GCM tag)
  ❌ 无前向安全 (私钥泄露→所有过往通信可解密)
  ❌ 无后向安全 (私钥泄露→未来所有通信可解密)
  ❌ 无可否认性
```

### 5.2 推荐升级: Signal Double Ratchet

微软 AgentMesh Wire Protocol (2026-04) 使用 Signal Protocol 的 Double Ratchet 实现 E2EE:
- 每消息独立密钥 (泄漏不暴露过往)
- DH ratchet 提供后向安全 (泄漏后恢复)
- 与 A2A 身份系统兼容 (Ed25519 长期身份 → X25519 会话密钥)

```
推荐双轨升级路径:

┌─ 短期 (当前) ──────────────────────┐
│ NaclChannel (AES-256-GCM + NAXOS)   │
│ 无前向安全，无后向安全               │
│ 适用于: 原型验证 / 低安全环境         │
└──────────────────────────────────────┘
                  ↓ 升级
┌─ 长期 (推荐) ────────────────────────┐
│ Signal Double Ratchet                 │
│ X25519 + AES-256-GCM + HMAC-SHA256    │
│ 前向安全 + 后向安全 + 防重放          │
│ 匹配 AgentMesh Wire Protocol 标准      │
│ 适用于: 生产部署 / 合规环境             │
└────────────────────────────────────────┘
```

### 5.3 密钥生命周期 (需升级)

```
HiveKeyPair:
  - Ed25519 长期签名密钥 (用于 AgentCard 签名)
  - 存储在现有 keyvault
  - 周期轮转 (建议 90 天)

SubHiveKeyPair:
  - X25519 短期会话密钥 (用于加密)
  - 每次 spawn 生成，子蜂消亡时销毁

密钥交换 (Double Ratchet):
  1. 蜂巢发布 Signed AgentCard (含 X25519 公钥)
  2. 子蜂验证签名 → 提取公钥
  3. 子蜂生成临时 X25519 密钥对
  4. 执行 X3DH 初始密钥交换 → 根密钥
  5. 每消息: 对称密钥派生 → 加密 → ratchet 推进
  6. 每 3 消息: DH ratchet step (后向安全)
```

### 5.4 加密包格式

```
EncryptedKnowledgePacket {
    // 明文头部 (用于路由)
    sender_id: HiveId,
    packet_id: PacketId,
    domain: String,
    encrypted_size: u32,
    
    // Double Ratchet 加密载荷
    dh_ratchet_pubkey: [u8; 32],    // 当前 ratchet 公钥
    message_number: u32,             // 发送方向消息计数
    previous_message_number: u32,    // 前一个 ratchet 周期的消息数
    ciphertext: Vec<u8>,             // AES-256-GCM (key from ratchet)
    
    // 身份签名
    sender_signature: Ed25519Signature,  // 签 ciphertext header
}
```

---

## 六、收敛加速机制（D³MAS + FoT + Symphony）

### 6.1 知识冗余消除（D³MAS 启发）

D³MAS（arXiv:2510.10585）证明分布式 agent 间知识冗余率达 47.3%。
应对方案:

| 冗余类型 | D³MAS 方法 | 蜂巢实现 |
|----------|-----------|----------|
| 任务分解重复 | 异构图上三层协调 | SpawnController 的 gap 重复检测 |
| 推理路径冗余 | 互补推理路径选择 | Knowledge Pool 的 novelty 评分 |
| 记忆检索重叠 | 结构化消息传递过滤 | SVAF per-field 门控 (冗余检测) |
| 语义重复 (新增) | CAT7 per-field redundancy check | "所有字段 < T_redundant → discard" |

### 6.2 语义级联邦学习（FoT 启发）

FoT（arXiv:2604.16778）实现不传梯度的语义级联邦:

```
1. 子蜂本地推理 → 生成 reasoning trace (可读文本)
2. 子蜂 PUBLISH trace 到 Pool (非梯度)
3. SpawnController 选择高分 traces
4. 聚合为跨任务洞察库 (FoTInsightLibrary)
5. 新子蜂 spawn 时继承 insight library

FoT 论文: accuracy +24%, reasoning tokens -28%
```

### 6.3 多路径投票收敛（Symphony 启发）

Symphony 的多 CoT 投票机制提高 12-15% 准确率:

```
多个子蜂独立解决同一问题:
  1. 每个子蜂产生 solution + confidence
  2. 所有 solutions → Pool (通过 problem_id 关联)
  3. Weighted voting (confidence × negentropy_gain)
  4. 胜出方案 → 蜂巢核心 KB

分歧检测:
  - 如果 solutions 分歧大 → spawn 仲裁子蜂
  - 仲裁子蜂 → 分析分歧 → 产生融合版本
```

---

## 七、Graceful Degradation（优雅降级策略）

基于 SWARM+ 的三层弹性 + adaptive quorum:

| 故障模式 | 影响 | 降级策略 |
|----------|------|----------|
| 单一子蜂崩溃 | 该子蜂知识丢失 | 其他子蜂补全 (Pool 中有 lineage) |
| 50% 子蜂失效 | 收敛速度减慢 | adaptive quorum 自动调整 (SWARM+: <7.5% 影响) |
| Pool 节点故障 | 新知识无法发布 | 子蜂本地缓存 + 重试 |
| 网络分区 | 子蜂孤立 | 本地 DGM-H 继续进化, 恢复后合并 via Merkle DAG |
| 加密密钥泄露 | 回流通道不安全 | 立即轮转密钥 + 子蜂重新身份验证 |
| 全部子蜂失效 | 蜂巢停止进化 | 保留最后状态, 新子蜂从 checkpoint 恢复 |

---

## 八、代码映射（当前实现状态）

### 已实现（✅）

| 组件 | 文件 | 行数 | 测试 |
|------|------|------|------|
| `MerkleDagStore` | `core/nt_core_hive/merkle_dag.rs` | 260 | 8 |
| `SvaGate` (CAT7 8-field) | `core/nt_core_hive/sva_gate.rs` | 456 | 22 |
| `SpawnController` | `core/nt_core_hive/spawn_controller.rs` | 321 | 8 |
| `NaclChannel` (AES-256-GCM + Ratchet) | `core/nt_core_hive/nacl_channel.rs` | 662 | 20 |
| `SignedAgentCard` | `core/nt_core_hive/signed_card.rs` | 210 | 9 |
| `TrustRegistry` | `core/nt_core_hive/signed_card.rs` | (同上) | (同上) |
| `KnowledgePool` (SVAF-wired) | `core/nt_core_hive/pool.rs` | 387 | 8 |
| `SubHiveInstance` + `Registry` (+ SVAF) | `core/nt_core_hive/sub_hive.rs` | 470 | 9 |
| `ReputationTracker` | `core/nt_core_hive/reputation.rs` | 215 | 9 |
| `ConsciousnessIntegration` 接线 | `nt_mind_background_loop/consciousness.rs` | — | — |
| `run.rs` tick 注册 | `nt_mind_background_loop/run.rs` | — | — |
| `Hyperedge` / `HypergraphStore` / `NaryRelationExtractor` | `core/nt_core_knowledge/hypergraph.rs` | 667 | 15 |
| `ByzantineConsensusLayer` / `ConsensusConfig` | `core/nt_core_agent/consensus.rs` | 459 | 8 |
| `AdaptiveVsaEncoder` / `EncodingMode` | `core/nt_core_hcube/adapt_encoder.rs` | 275 | 13 |
| `MemoryGraph` / `MemoryNode` / `MemoryEdge` | `core/nt_core_knowledge/spread_activation.rs` | 506 | 16 |
| `EFEMinimizer` / `Policy` / `TransitionModel` | `core/nt_core_negentropy/efe_minimizer.rs` | 414 | 12 |

### 需升级（🔄）

| 组件 | 升级内容 | 优先级 |
|------|----------|--------|
| `SvaGate` | 4-field → 完整 CAT7 7-field (add mood/issue/intent/perspective) | P0 ✅ |
| `SvaGate` | Add CfC time-constant state evolution | P2 |
| `NaclChannel` | AES-256-GCM → Signal Double Ratchet (k256 ECDH + 前向安全) | P1 ✅ |
| `NaclChannel` | Full X3DH (DID-based prekey bundles) for PCS | P2 |
| `ReputationTracker` | Per-sub-hive reputation via SVAF scores | P1 ✅ |
| `SignedAgentCard` | k256 ECDSA → Ed25519 (匹配 A2A 生态共识) | P1 (k256 保留, 双算法) |
| `SignedAgentCard` | Add DID-based identity per AIP spec | P2 |

### 推荐新建（📋）

| 模块 | 参考 | 优先级 |
|------|------|--------|
| `CfCState` (per-sub-hive CfC time constants τ) | MMP Layer 6, CfC (arXiv:2408.15539) | P2 |
| `FoTInsightLibrary` (跨子蜂推理洞察库) | FoT (arXiv:2604.16778) | P2 |
| `SymphonyVoter` (多路径加权投票收敛) | Symphony (MultiAgent 2026) | P2 |
| `ReputationTracker` (wired into SubHiveRegistry via SVAF) | DKL | P1 ✅ |
| `RatchetSession` (per-link ratchet state machine in NaclChannel) | Signal / AgentMesh | P1 ✅ |

---

## 九、核心技术参考（2026-06 更新）

| 技术 | 来源 | 年份 | 关键发现 |
|------|------|------|----------|
| **A2A Protocol v1.0** | Google / Linux Foundation | 2025-26 | Signed Agent Cards, 150+ 组织, v1.0 稳定版 (2026-03) |
| **A2A Identity (#1672)** | A2A Community | 2026 | 身份验证缺口, 73+ 评论, 5+ 扩展方案 |
| **A2A x-agent-trust (#1742)** | A2A Community | 2026 | ECDSA+JWKS 每请求签名, OpenAPI 扩展注册 |
| **AIP (Agent Identity Protocol)** | IETF draft-02 | 2025-26 | W3C DID + Ed25519, 委派链, 39 端点服务 |
| **CTEF (#1786)** | A2A Community | 2026 | 4 层身份 (identity/transport/authority/continuity) |
| **APS (Agent Passport System)** | aeoess | 2026 | Ed25519 护照, JCS 规范化, 双边委派 |
| **AgentMesh Wire Protocol** | Microsoft | 2026 | Signal Double Ratchet E2EE, 离线投递, 5 SDK 语言 |
| **AgentCrypt** | eprint 2025/2216 | 2025 | 3 级加密框架 (IBE/ABE/FHE), 100% 隐私, 84% 任务正确 |
| **SS-ZKR** | arXiv:2606.00962 | 2026 | 盲路由 + DP 语义向量 + ZK 策略编译器 |
| **SWARM+** | arXiv:2603.19431 | 2026 | 分层共识 O(n²)→O(log n), 1000 agents, <7.5% 降级 |
| **MMP SVAF** | meshcognition.org / arXiv:2604.19540 | 2026 | CAT7 7 字段, per-field 门控, 78.7% 准确率, remix 图 |
| **MMP CfC** | meshcognition.org / arXiv:2604.03955 | 2026 | CfC τ 时间常数, 快慢神经元, 情感秒级同步 |
| **IPFS Merkle DAG** | Protocol Labs | 2014-26 | 内容寻址、防篡改、去重、Git 风格 DAG |
| **MASEval** | arXiv:2603.08835 | 2026 | 框架级设计影响 (12.4pp) ≈ 模型选择影响 (14.2pp) |
| **DKL** | OpenReview | 2026 | 集体智能收敛定理, 多项式检索, O(log N) 收敛 |
| **FoT** | arXiv:2604.16778 | 2026 | 语义级联邦, accuracy +24%, tokens -28% |
| **Symphony** | MultiAgent 2026 | 2026 | O(log N) gossip, multi-CoT accuracy +12-15% |
| **AOrchestra** | ICML 2026 | 2026 | 4-tuple 子 agent, 16.28% rel. improvement |
| **D³MAS** | arXiv:2510.10585 | 2025 | 47.3% 冗余率 → 分层降至 ~26% |
| **Signal Double Ratchet** | Signal / OMEMO | 2013-26 | 前向安全 + 后向安全, 每消息独立密钥 |

---

## 十、关键缺口与路线图

### P0 — 已完成 ✅

| 缺口 | 修复 |
|------|------|
| SvaGate CAT7 7 字段升级 | 8-field (CAT7 + TextSummary), MoodVA 编码, α=0.20 权重, sentiment 分析 |
| SvaGate ↔ KnowledgePool 接线 | publish() 集成 SVAF, content-driven convergence, absorption recording |
| **P0.1 KROP Cleanup** (O(N log N)) | KronekerCodebook in `kroneker_cleanup.rs` — FWHT-based, theoretical limit reached ✅ |
| **P0.2 N-ary Hypergraph RAG** | `hypergraph.rs` — `Hyperedge`/`HypergraphStore`/`NaryRelationExtractor`, 15 tests ✅ |
| **P0.3 BFT Consensus Layer** | `consensus.rs` — `ByzantineConsensusLayer`/`ConsensusConfig`, PBFT-style 3-phase commit, 8 tests ✅ |

### P1 — 已完成 ✅

| 缺口 | 修复 |
|------|------|
| AES-256-GCM 无前向安全 | k256 ECDH ephemeral + SHA-256 链 ratchet, RatchetState/RatchetEncryptedPacket |
| 无子蜂声誉追踪 | ReputationTracker: EMA 分数, consecutive_high/low, grace_period, weight_factor |
| **P1.1 Adaptive VSA Encoder** | `adapt_encoder.rs` — `AdaptiveVsaEncoder` supports Orthogonal/Correlated/Hybrid modes, 13 tests ✅ |
| **P1.2 Spreading Activation Memory** | `spread_activation.rs` — `MemoryGraph` with node/edge kinds, BFS decay-based activation, 16 tests ✅ |
| **P1.3 EFE Minimizer** | `efe_minimizer.rs` — `EFEMinimizer` with risk/ambiguity/info-gain free-energy components, 12 tests ✅ |

### P1 — 待办

| 缺口 | 影响 | 修复方案 |
|------|------|----------|
| k256 ECDSA 非 A2A 生态共识 | 与外部 A2A Agent 签名不兼容 | 添加 Ed25519 支持 (双算法, ed25519-dalek 可选) |

### P2 — 中长期

| 缺口 | 影响 | 修复方案 |
|------|------|----------|
| 无 CfC 时间动力学 | 缺失情感同步/专业知识稳定的时间维度 | 实现 CfCState: per-子蜂 τ 向量 + 状态更新 |
| 无 FoT 洞察库 | 跨子蜂推理模式无法复用 | 实现 FoTInsightLibrary: 高分 reasoning traces 聚合 |
| 无 Symphony 投票 | 多子蜂同任务时浪费收敛机会 | 实现 SymphonyVoter: weighted voting + 分歧仲裁 |
| DID 身份未集成 | 无法与外部 A2A 生态互认身份 | 实现 DIDResolver + did:key 方法 |
