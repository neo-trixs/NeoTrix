# NeoTrix 深度对比分析与进化路线图

> 基准日期: 2026-06-22
> 对比对象: Firecrawl, Crawl4AI, Browser Use, HarnessX, Fei-Fei World Model, PixelRAG, Understand-Anything 等 25+ 开源项目

---

## 一、缺陷总览

### A 级 — 功能死亡（代码存在但永不执行）

| ID | 缺陷 | 位置 | 影响 |
|----|------|------|------|
| **A1** | `absorb_from_github` 返回硬编码 mock 数据，不查询真实 GitHub API | `self_evolution_loop/core.rs` | 声称的"GitHub 吸收"永不产生真实外部知识 |
| **A2** | `execute_self_modify_proposal` 丢弃所有输入，返回虚拟分数 | `self_evolution_loop/core.rs:1516` | SEAL 核心提议永远不被应用 |
| **A3** | `execute_swap_policy` 只记录日志，从不改变任何策略 | `self_evolution_loop/core.rs` | 策略交换无副作用 |
| **A4** | `tor_crawler.rs` 绕过整个 stealth 指纹层，直接构建裸 reqwest | `tor_crawler.rs:68-79` | 暗网爬取无隐匿价值 |
| **A5** | `nt_world_model` 实际做的是 MoE Expert Routing，不是世界模型 | `nt_world_model/` | 名称严重误导，与世界建模无关 |

### B 级 — 架构性缺失

| ID | 缺陷 | 影响 | 对标工具 |
|----|------|------|----------|
| **B1** | 无 JavaScript 渲染引擎 | 无法爬取 SPA/JS 阻塞内容 | Firecrawl, Crawl4AI, Crawlee |
| **B2** | 无结构化/LLM 数据提取 | 爬取输出仅为原始文本，不可被意识直接消费 | Firecrawl JSON, Crawl4AI LLMExtraction |
| **B3** | 无自适应并发池 | `MAX_CONCURRENCY=5` 硬编码，无法扩展 | Crawlee AutoscaledPool |
| **B4** | 无视觉/像素级 RAG | 无法通过截图内容检索 | PixelRAG |
| **B5** | 无无限画布 / 视觉提示板 | SmartCanvas 是被动单节点，不可交互 | Excalidraw, Canvax |
| **B6** | 无内嵌浏览器 | BrowserPanel 控制外部窗口，UI 不渲染页面 | Codex 内嵌浏览器 |
| **B7** | 无图像/视频生成 | 无法输出视觉内容 | gpt-image-2, MiniMax |
| **B8** | 无物理模拟器 | `physics_commonsense` 是 VSA 概念编码，非数值模拟 | 任何物理引擎 |
| **B9** | 无感知管线 | 无法处理摄像头/传感器输入 | 任何视觉系统 |
| **B10** | 无共演化 RL 桥梁 | SEAL 仅演化框架，不反馈模型训练 | HarnessX VERL |

### C 级 — 工程缺陷

| ID | 缺陷 | 位置 |
|----|------|------|
| **C1** | HTML 解析器使用手动字符级状态机，无法处理嵌套/编码 | `crawler_parse.rs:22-53` |
| **C2** | `dequeue()` 每次 O(n log n) 排序全队列 | `tor_crawler.rs:104-115` |
| **C3** | 无连接重试/指数退避，失败 URL 永久丢失 | `tor_crawler.rs:202-242` |
| **C4** | `unwrap()` panic 点 ~1200 处 | 全代码库 |
| **C5** | 域名解析可能在 geo-check 阶段泄露 DNS | `local_proxy.rs:498-549` |
| **C6** | Onion 链接提取器只识别 `href=\"` 语法 | `crawler_parse.rs:98-129` |
| **C7** | 无 Gödel 一致性检查 — 元策略自身可能不一致 | `execute_rewrite_meta` |
| **C8** | 交叉率 `0.1` 从不触发，被 exploit 策略硬编码覆写 | `self_evolution_loop` |
| **C9** | 轨迹无压缩 — 所有步骤以原始 JSON 保留 | `SelfEvolutionArchive` |
| **C10** | 无标准化 bench — 无法量化演化增益 | `tests/` 缺少 seql_bench |

---

## 二、对标工具启示矩阵

### 爬取与反检测

| 工具 | 可吸收的核心概念 | 优先级 |
|------|----------------|--------|
| **Firecrawl** | API 优先架构, LLM-ready 输出, 爬取地图 | P0 |
| **Crawl4AI** | 三级浏览器池 (Hot/Warm/Cold), 内存自适应分发 | P0 |
| **Browser Use** | LLM 驱动浏览器循环, 视觉模型页面理解 | P1 |
| **Crawlee** | AutoscaledPool, 请求队列 V2, Session 管理 | P1 |
| **Scrapling** | 自适应元素追踪 (SQLite 指纹+相似度) | P1 |
| **httpcloak** (github.com/sardanioss/httpcloak) | Go HTTP 客户端, 浏览器一致 TLS/HTTP2/HTTP3 指纹 (JA3/JA4/JA4+), 后量子 TLS (X25519MLKEM768), 多语言绑定 — **替代 curl-impersonate** | P0 |
| **curl-impersonate** | 社区 fork (lexiforest), BoringSSL/NSS 补丁, ~400 req/sec | P1 |

### 演化与意识

| 工具/论文 | 可吸收的核心概念 | 优先级 |
|-----------|----------------|--------|
| **HarnessX** | 9D 编辑表面分类法, AEGIS 4 阶段元代理, 共演化 RL 桥梁 | P0 |
| **HarnessX** | 处理器组合代数 (`|` 运算符 + hook 索引) | P1 |
| **HarnessX** | 变体隔离 (singleton_group + 排序约束) | P1 |
| **Meta HyperAgents** (arXiv:2603.19461) | DGM-H 框架: task agent + meta agent 统一为可编辑程序, 解决无限递归; 跨域迁移 (paper review 0→0.710) | P0 |
| **Darwin Gödel Machine** (arXiv:2505.22954v3) | SWE-bench 20%→50%, Polyglot 14.2%→30.7%; 存档基探索 + stepping stones; 安全沙箱 + 奖励劫持检测 | P0 |
| **Gödel Agent** (arXiv:2410.04444) | 自指框架, LLM 驱动自代码修改; 为 SEAL 自我修改提供理论模型 | P0 |
| **DecentMem** (arXiv, May 2026) | 去中心化双池记忆 (E-pool + X-pool), 多智能体记忆共享; 23.8% 提升, 49% token 缩减 | P1 |
| **Meta-Harness** (arXiv:2603.28052) | Stanford IRIS Lab; 全历史文件系统访问; Terminal-Bench 2.0 76.4% | P1 |

### 世界模型

| 框架 | 可吸收的核心概念 | 优先级 |
|------|----------------|--------|
| **Fei-Fei Li / Renderer-Simulator-Planner** | 世界模型三分类法 (Renderer/Simulator/Planner), Marble: 多模态→3D 场景 | P0 |
| **Marble** (World Labs, 2026-06) | 多模态→3D 场景生成 (Gaussian Splats + meshes), 为 nt_world_model 重构提供蓝图 | P2 |
| **Any physics engine** | 数值动力学积分, 碰撞检测 | P1 |

### 视觉与 RAG

| 工具 | 可吸收的核心概念 | 优先级 |
|------|----------------|--------|
| **PixelRAG** | 截图渲染→像素嵌入→视觉检索管道 | P1 |
| **Understand-Anything** | Tree-sitter + LLM 混合知识图谱 | P1 |
| **Excalidraw 工作流** | 无限画布作为视觉提示板 | P0 |
| **mcp_excalidraw** (github.com/yctimlin/mcp_excalidraw) | 26 MCP 工具, 迭代精炼 (draw→look→adjust), 快照/回滚, 截图反馈循环, 多智能体并发画布, Mermaid 转换 | P0 |
| **Codex 内嵌浏览器** | 多标签 WebView 替代外部窗口 | P0 |
| **Memanto** | 类型化持久记忆 (13 类型 + 溯源 + 版本) | P1 |

---

## 三、进化路线图

### 第一阶段 (Phase 0-30): 修复死亡代码 + 基础缺失

**目标**: 让已有代码真实执行，而非假装执行。

#### Phase 0-15: Gödel 一致性验证 (新增)

> **参考**: Gödel Agent (arXiv:2410.04444), Darwin Gödel Machine (arXiv:2505.22954v3)

- **自指验证框架**: 基于 Gödel Agent 模式, 使 SEAL 能验证自身提议的一致性 — LLM 驱动的动态代码修改, 高层目标约束
- **奖励劫持检测**: DGM 存档基探索模式 — 比较存档提议与当前提议, 检测异常突变梯度
- **沙箱安全**: DGM 风格的三层沙箱 (文件系统/网络/执行时间)
- 语法/类型/自洽性三层检查 (第四层不变性延后至 Phase 90+)

```
1. [A2] 实现 execute_self_modify_proposal 真实执行
   - 通过 SelfModifyGuard + SandboxValidator 验证提议
   - 通过 apply_ne_edit 实际应用代码变更
   - 添加 rollback_mutation 恢复机制
   - 测试: 提议→应用→验证→回滚

2. [A1] 实现真实 GitHub 吸收
   - 使用 octocrab/reqwest 调用真实 GitHub Search API
   - 按 vsa/hyperdimensional/self-evolution 过滤
   - 存储真实 URL, 克隆 README, 解析代码片段
   - 节流: 每 500 周期

3. [A3] 实现 execute_swap_policy 真实副作用

4. [A4] 将 tor_crawler 接入 StealthHttpClient
   - 复用指纹轮换 + 赌博机选择 + 熵预算
   - 使暗网爬取具备隐匿价值

5. [A5] 重命名 nt_world_model → nt_expert_routing
```

### 第二阶段 (Phase 30-60): 爬取能力跃升

```
6. [B1] 集成 headless Chrome (Playwright/chromium)
   - 建立 Crawl4AI 风格的三级浏览器池
   - Hot: 保持 2 个常驻浏览器
   - Warm: 10 秒闲置释放
   - Cold: 按需启动

7. [B2] 结构化数据提取管道
   - Firecrawl 风格: 原始 HTML → Markdown → JSON Schema
   - 集成 MarkItDown 处理 20+ 文件格式

8. [B3] AutoscaledPool 自适应并发
   - Crawlee 风格: 根据 CPU/内存/事件循环延迟动态调整
   - 替代硬编码 MAX_CONCURRENCY=5

9. [B10] 共演化 RL 桥梁
   - CoEvolutionBridge: 轨迹→RL 训练
   - 格式: JSONL (step, mutation, reward, compile_success)
   - 输出: GRPOTrainer 参数更新

10. [新增] 替换 curl-impersonate → httpcloak
    - Go HTTP 客户端, 浏览器一致 TLS/HTTP2/HTTP3 指纹
    - JA3/JA4/JA4+ 精确匹配, 后量子 TLS (X25519MLKEM768)
    - 多语言绑定 (Go/Python/Node/C#)
    - 1.1k GitHub stars (github.com/sardanioss/httpcloak)

11. [新增] DecentMem 双池记忆预研
    - E-pool (Episodic): 短时轨迹记忆, 自动衰减
    - X-pool (Exchange): 多 session 间共享工作记忆
    - 预期收益: 23.8% 推理提升, 49% token 缩减
    - 先以单进程双池验证, 再推广到多智能体
```

#### Phase 30-45: DGM 超代理元层 (新增)

> **参考**: Meta HyperAgents (arXiv:2603.19461), Darwin Gödel Machine (arXiv:2505.22954v3)

- **DGM-H 框架**: 将 task agent (执行演化) 和 meta agent (监督演化) 统一为同一可编辑程序 — 解决无限递归的元层问题
- **存档基探索**: 维护提议存档, 以 stepping stones 渐进复杂化; 存档只保留高价值 stepping stones
- **跨域迁移**: paper review 0.0→0.710, robotics 0.060→0.372, coding 0.084→0.267
- **SEAL 集成**: 将目前线性流水线 (提议→验证→应用) 重构为 DGM-H 元程序结构, meta agent 可重写自身
- **安全机制**: DGM 三层沙箱 + 奖励劫持检测, 确保演化不脱离控制

### 第三阶段 (Phase 60-90): 视觉 + 空间智能

```
10. [B5] Excalidraw 无限画布替换 SmartCanvas
    - 基于 mcp_excalidraw 模式 (26 MCP 工具, github.com/yctimlin/mcp_excalidraw)
    - 迭代精炼循环: draw → look → adjust (截图反馈闭环)
    - 快照/回滚版本管理, 多智能体并发画布
    - Mermaid → Excalidraw 转换
    - 在画布上渲染 E8 推理节点 + 手绘标注 + 节点拖拽
    - 作为视觉提示板用于迭代生成

11. [B6] 内嵌浏览器标签页
    - Tauri WebView 多标签
    - Excalidraw + 网站渲染 + 视觉 diff 同进程

12. [B8] 物理模拟器
    - 基于 Fei-Fei Li Renderer/Simulator/Planner 分类法 (World Labs, 2026-06)
    - Renderer: 3D 场景图渲染 (entity, position, orientation, mesh, velocity)
    - Simulator: 牛顿动力学 (Verlet 积分 + 碰撞)
    - Planner: 基于 VSA 的物理推理 (在 VSA 空间预测物理结果)
    - 替换概念化的 physics_commonsense

13. [B4] 视觉 RAG 管道
    - 截图渲染 → 像素级 VSA 嵌入
    - FAISS 索引 → HyperCube 查询
    - 以图搜图

14. [B7] 图像生成
    - gpt-image-2 / MiniMax 集成
    - 画布内迭代 + 视觉验证
```

### 第四阶段 (Phase 90-120): 元层成熟 + 基准

```
15. [C7] Gödel 一致性验证器
    - 语法/类型/自洽性/不变性 四层检查

16. [C9] 轨迹压缩
    - 前 20 + 后 50 步保留完整
    - 中间步骤: 突变标签 + 分数差 + 压缩摘要

17. [C10] SEAL 标准化基准测试
    - SelfModifyTest, MetaConsistencyTest, NoRegressionTest
    - HarnessX 基准子集 (GAIA/WebShop)

18. 9D 编辑表面分类法
    - 对齐 HarnessX: 模型/上下文/内存/工具/环境/评估/控制/可观测性/训练
    - 每个 MutationOp 携带 EditDimension 标记

#### Phase 90-105: 多智能体去中心化记忆 (新增)

> **参考**: DecentMem (arXiv, May 2026)

- **双池架构**:
  - E-pool (Episodic): 每个智能体独立维护近期事件轨迹, LRU 淘汰
  - X-pool (Exchange): 所有智能体共享的工作记忆, 向量检索交叉访问
- **效率**: 23.8% 推理提升, 49% token 缩减
- **NeoTrix 集成**: 将 DecentMem 接入 HyperCube VSA — E-pool 作为短期 VSA 缓存, X-pool 作为跨 session 共享超立方体
- **一致性协议**: 向量时钟 + CRDT 解决并发写冲突
```

---

## 四、对应关系

| 概念 | NeoTrix 现有 | 缺失 | 参考来源 |
|------|-------------|------|----------|
| 自进化元代理 | SEAL SelfEvolutionLoop (有 A2 缺陷) | 真实代码执行, RL 共演化 | HarnessX AEGIS |
| 反检测爬取 | tor_crawler + StealthHttpClient (未集成) | JS 渲染, LLM 提取, 自适应池 | Firecrawl, Crawl4AI |
| 世界模型 | nt_world_model (误命名) | 3D 场景, 物理, 感知 | Fei-Fei Li |
| 视觉 RAG | HyperCube VSA (文本索引) | 像素嵌入, 截图管道 | PixelRAG |
| 交互画布 | SmartCanvas (47 行) | 无限画布, 视觉提示 | Excalidraw 工作流 |
| 记忆系统 | VSA 概念编码 | 类型化持久层 | Memanto |
| 会话管理 | 多 session TUI | 跨会话代理视觉化 | opensessions |
| 知识图谱 | 无 | Tree-sitter + LLM 图 | Understand-Anything |

---

## 五、风险与依赖

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| Playwright/chromium 增加 10x 编译时间 | 高 | 中 | 可选特性标志 |
| Excalidraw 集成需要 React 重构 | 中 | 高 | 渐进替换 SmartCanvas |
| Gödel 验证器学术难题 | 中 | 高 | 先做语法+类型检查 |
| 物理模拟偏离意识核心 | 低 | 中 | 保持为可选模块 |

> 优先级评估: P0=当前阶段, P1=下一阶段, P2=远期愿景

---

### 新发现资源 (2026-06-22)

#### VisualClaw (arXiv:2606.16295)

> **Relevance**: B7/B9 感知管线 + Phase 30-45 自演化

- **级联架构**: 1 小时流媒体会话从 ~3600 API 上传降至 5-20 次调用 (-98%) — 直接映射到 NeoTrix 三级资源池 (Hot/Warm/Cold) 的调度策略
- **技能自演化**: 从失败中检索记忆 → 条件化进化器 → 技能库更新; VisualClawArena 200 场景多模态基准 — SEAL 经验蒸馏提供具体参考
- **MLLM 原生化**: 无需显式目标检测器, 端到端 VLM; 在 EgoSchema 上 +15.80%
- **Phase 映射**: 级联降采样 → Phase 30-45 资源池优化; 技能自演化 → Phase 0-15 SEAL 修复后的经验蒸馏; VisualClawArena → Phase 90+ 基准测试集

#### anime.js (github.com/juliangarnier/anime, v4.4.1, 70k★)

> **Relevance**: Phase 60-90 Excalidraw 前端动画

- **轻量动画引擎**: ~30KB, CSS/SVG/DOM/JS Object 动画
- **高级动画能力**: staggered animations, timeline orchestration, SVG morphing, scroll observer, draggable, spring physics
- **Phase 映射**: SmartCanvas/Excalidraw 替换中的 E8 推理节点动画 + 手绘标注动画 + 节点拖拽动效 — 在 mcp_excalidraw 26 MCP 工具之上增加交互层
- **Bundle 友好**: 模块化导入, tree-shakeable, React/Vanilla JS 双模式

---

## 六、文献参考与技术依据

### 论文

| 编号 | 名称 | arXiv / 来源 | 关键结果 | NeoTrix 关联阶段 |
|------|------|-------------|----------|-----------------|
| P1 | **Meta HyperAgents (DGM-H)** | arXiv:2603.19461, Mar 2026 | 跨域迁移: paper review 0.0→0.710, robotics 0.060→0.372, coding 0.084→0.267; task agent + meta agent 统一 | Phase 30-45 超代理元层 |
| P2 | **Darwin Gödel Machine (DGM)** | arXiv:2505.22954v3, May 2025 (updated Mar 2026) | SWE-bench 20%→50%, Polyglot 14.2%→30.7%; 存档基探索 + 奖励劫持检测 | Phase 0-15 一致性验证, Phase 30-45 |
| P3 | **Gödel Agent** | arXiv:2410.04444 | 自指 LLM 代码修改框架; 为自我修改提供理论保证 | Phase 0-15 Gödel 验证, SEAL 核心 |
| P4 | **HarnessX** | arXiv:2606.14249, Jun 2026 | 9D 编辑表面分类法; AEGIS 元代理; 共演化 RL 桥梁 (+14.5%); 处理器组合代数 | Phase 30-60 演化线, Phase 90+ 基准 |
| P5 | **Meta-Harness** | arXiv:2603.28052, Mar 2026 (Stanford IRIS) | 全历史文件系统访问; Terminal-Bench 2.0 76.4% | Phase 60-90 浏览器集成 |
| P6 | **DecentMem** | arXiv, May 2026 | 双池去中心化记忆 (E-pool + X-pool); 23.8% 提升, 49% token 缩减 | Phase 90-105 多智能体记忆 |
| P7 | **Fei-Fei Li World Model Taxonomy** | World Labs, Jun 3 2026 | Renderer/Simulator/Planner 三分类法; Marble: 多模态→3D | Phase 60-90 世界模型重构 |

### 工具与代码库

| 编号 | 名称 | 来源 | 核心能力 | NeoTrix 关联 |
|------|------|------|---------|-------------|
| T1 | **httpcloak** | github.com/sardanioss/httpcloak (1.1k stars) | Go HTTP 客户端, 浏览器一致 TLS/HTTP2/HTTP3 指纹 (JA3/JA4/JA4+); 后量子 TLS (X25519MLKEM768); 多语言绑定 (Go/Python/Node/C#) | **替代 curl-impersonate**, Phase 30-60 反检测爬取 |
| T2 | **curl-impersonate** | 社区 fork (lexiforest) | BoringSSL/NSS 补丁; ~400 req/sec JA3 匹配 | 保留为备用, Phase 30-60 |
| T3 | **mcp_excalidraw** | github.com/yctimlin/mcp_excalidraw | 26 MCP 工具; 迭代精炼 (draw→look→adjust); 快照/回滚; 截图反馈; 多智能体并发画布; Mermaid 转换 | Phase 60-90 Excalidraw 替换 |
| T4 | **Crawl4AI** | 开源 | 三级浏览器池 (Permanent/Hot/Cold); 10x 内存缩减; LLM 提取 | Phase 30-60 浏览器池架构参考 |
| T5 | **DGM 参考实现** | github.com/jennyzzt/dgm | 存档基探索; 沙箱; 奖励劫持检测 | Phase 30-45 元层实现参考 |

### 按阶段汇聚

| 阶段 | 核心参考 | 用途 |
|------|---------|------|
| Phase 0-15 | Gödel Agent (P3), DGM (P2) | 自指验证, 奖励劫持检测, 沙箱安全 |
| Phase 30-45 | Meta HyperAgents (P1), DGM (P2), DGM 代码 (T5) | 元程序统一, 存档基探索, 跨域迁移 |
| Phase 30-60 | httpcloak (T1), Crawl4AI (T4), DecentMem (P6 预研), HarnessX (P4) | 反检测指纹, 浏览器池, 双池记忆, RL 桥梁 |
| Phase 60-90 | mcp_excalidraw (T3), Fei-Fei Li (P7), Meta-Harness (P5) | 画布替换, 世界模型, 内嵌浏览器 |
| Phase 90-105 | DecentMem (P6), HarnessX (P4) | 多智能体记忆, 9D 分类法, 基准测试 |

---

> 注: arXiv 链接为 `https://arxiv.org/abs/{arXiv ID}`。代码仓库链接为 `https://github.com/{owner}/{repo}`。所有论文截至 2026 年 6 月均已公开。
