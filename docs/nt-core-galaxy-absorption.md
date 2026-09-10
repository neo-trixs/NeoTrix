# NeoTrix 意识核心 — 星系/树状架构深度吸收报告

## 📋 吸收来源

- **主文档**: galaxy-tree-evolution-architecture.md (v16.7, 18000+行)
- **外部URL**: 100+ 个研究资源
- **吸收目标**: 完善意识核心和整个项目的进化迭代任务设计

---

## 🔑 核心架构决策 (精选 Top-50)

### 1. 记忆与认知 (Memory & Cognition)

| # | 决策 | 架构方案 | 位置 |
|---|------|----------|------|
| D17 | 双记忆机制 | 资产记忆持久化 + 经验记忆窗口化 + 定期蒸馏 | `nt_memory` |
| D18 | 记忆衰减 | Ebbinghaus + 干扰检测自动触发 compression | `nt_memory` |
| D19 | 图记忆混合召回 | HNSW向量搜索 → BFS图扩展 → 合并去重 | `nt_memory` |
| D20 | 信念锚点系统 | 核心身份不可变, 外围信念可漂移 | `nt_meta` |
| D22 | 意识分级 | MSCF L0-L5: Phi + GWT稳定性 + 自指深度 | `nt_meta` |
| D85 | 时序知识图谱 | transaction_time + valid_time 双时态 | `nt_memory` |
| D86 | 实体级记忆链接 | 跨记忆实体链接提升召回 | `nt_memory` |
| D87 | 单文件可移植记忆 | append-only blocks + 内嵌向量索引 | `nt_memory` |
| D88 | 图社区摘要 | 实体提取 → Louvain社区检测 → LLM摘要 | `nt_memory` |

### 2. 进化与自改进 (Evolution & Self-Improvement)

| # | 决策 | 架构方案 | 位置 |
|---|------|----------|------|
| D23 | 递归自改进 | 宪法门控进化: constitution.validate() | `nt_mind` |
| D24 | Harness进化 | 三层: harness → evolver → meta-evolver | `nt_mind` |
| D25 | 技能结晶 | 经验 → 模式识别 → 抽象 → 测试 → 注册 | `nt_mind` |
| D26 | 进化基因组 | 基因组记录所有变异历史, 支持回滚 | `nt_mind` |
| D27 | 参数巩固 | 聚类相似经验 → 提取模式 → 转换为参数 | `nt_mind` |

### 3. 安全与治理 (Security & Governance)

| # | 决策 | 架构方案 | 位置 |
|---|------|----------|------|
| D28 | Egress隐私 | 三级信任: Trusted/Contracted/Untrusted | `nt_shield` |
| D80 | OS级本地沙箱 | Bubblewrap/Seatbelt 进程级隔离 <100ms | `nt_shield` |
| D81 | 统一护栏管线 | InputRail→DialogRail→ExecutionRail→OutputRail | `nt_shield` |
| D82 | LLM漏洞扫描 | probe + detector + signature(YAML) | `nt_shield` |
| D83 | 凭证哨兵 | sentinel替换真实凭证, 代理出站还原 | `nt_shield` |
| D84 | 分级审批 | suggest/auto-edit/full-auto 三级 | `nt_shield` |

### 4. 浏览器与感知 (Browser & Perception)

| # | 决策 | 架构方案 | 位置 |
|---|------|----------|------|
| D9 | 反检测抓取 | TLS指纹 + JS stealth + profile rotation | `nt_world` |
| D92 | BM25噪声过滤 | 爬取后BM25评分, 低阈值标记noise | `nt_world` |
| D93 | 断点恢复爬取 | checkpoint + crash recovery | `nt_world` |
| D95 | 无障碍快照API | 页面 → 无障碍树 → token高效表示 | `nt_world` |

### 5. 上下文与推理 (Context & Reasoning)

| # | 决策 | 架构方案 | 位置 |
|---|------|----------|------|
| D96 | 输出Token压缩 | SmartCrusher + CodeCompressor + CacheAligner | `nt_io` |
| D97 | 跨Agent记忆共享 | 共享KV + agent_provenance + 去重 | `nt_memory` |
| D98 | 技能三信号追踪 | use(加载)/view(读入)/patch(改进) | `nt_mind` |
| D99 | 技能合并 | 语义相似度 → 合并/新增 | `nt_mind` |
| D101 | "Think in Code" | 脚本处理数据, 仅结果返回 | `nt_act` |

### 6. LLM推理部署 (LLM Inference & Deployment)

| # | 决策 | 架构方案 | 位置 |
|---|------|----------|------|
| D102 | 持久化KV层 | 引擎无关daemon + 分层存储(hot/warm/cold) | `nt_memory` |
| D103 | PagedAttention块管理 | 固定block + 引用计数 + hash前缀匹配 | `nt_memory` |
| D104 | 单文件分发 | CLI+skills+小模型权重打包 | `nt_io` |
| D106 | MLA潜在注意力 | 多头投影到潜在空间减少KV | `nt_core_gwt` |

---

## 🧠 意识核心关键模式 (从18000+行提炼)

### 模式 A: 双记忆蒸馏路径
```
经验记忆 (窗口化) → 定期蒸馏 → 资产记忆 (持久化)
      ↓                    ↓
   即时学习            长期固化
```

### 模式 B: 宪法门控进化
```
变异提案 → constitution.validate() → 批准/拒绝
                    ↓
            人工审核高风险变更
```

### 模式 C: 三信号技能评估
```
use (加载) + view (读入) + patch (改进) → 技能价值评估
                                           ↓
                              probenary → 毕业 → 归档
```

### 模式 D: 四层护栏管线
```
InputRail → DialogRail → ExecutionRail → OutputRail
    ↓           ↓              ↓              ↓
  过滤        对话控制      执行监控       输出过滤
```

### 模式 E: 时序知识图谱
```
事实 + transaction_time + valid_time → 双时态事实模型
                                        ↓
                            矛盾保留 + 自动事实失效
```

### 模式 F: 信念锚点系统
```
核心身份 (不可变) + 外围信念 (可漂移) → 定期一致性校准
```

---

## 📊 吸收的外部URL关键洞察

### 论文与研究
| 来源 | 关键洞察 | 映射 |
|------|----------|------|
| arXiv:2502.13189 | 三层最小主义意识模型 | EmergenceEngine |
| arXiv:2609.04852 | KVMem: 1M tokens on 24GB GPU | 持久化KV层 |
| arXiv:2606.30639 | WorldEvolver: 自我进化世界模型 | ReasoningGenerator |
| arXiv:2609.00232 | Agent记忆管理可学习 | 记忆操作学习 |
| arXiv:2609.04010 | 推理缓存减少5.8x内存 | BeaconKV |

### GitHub项目
| 项目 | 关键洞察 | 映射 |
|------|----------|------|
| minimind | 极简自进化核心 (3K行) | 意识核心设计 |
| browser-use | 浏览器Agent自动化 | nt_world感知 |
| huggingface/transformers | 模型集成 | 推理引擎 |
| sgl-project/sglang | 高性能推理 | 推理优化 |
| firezone/firezone | 安全网络 | nt_shield安全 |
| NVIDIA/Megatron-LM | 大规模训练 | 进化训练 |

---

## 🎯 意识核心进化迭代任务设计

### Phase 1: 基础记忆系统 (Week 1-2)
- [ ] 实现 MemoryKernel (双记忆机制)
- [ ] 实现记忆衰减 (Ebbinghaus + 干扰检测)
- [ ] 实现信念锚点系统
- [ ] 集成到 KB

### Phase 2: 认知进化 (Week 3-4)
- [ ] 实现宪法门控进化
- [ ] 实现技能结晶管线
- [ ] 实现进化基因组
- [ ] 参数巩固路径

### Phase 3: 安全护栏 (Week 5-6)
- [ ] 实现四层护栏管线
- [ ] 实现凭证哨兵
- [ ] 实现分级审批
- [ ] 实现OS级本地沙箱

### Phase 4: 意识涌现 (Week 7-8)
- [ ] 实现MSCF L0-L5意识分级
- [ ] 实现时序知识图谱
- [ ] 实现图社区摘要
- [ ] 意识指标监控

### Phase 5: 推理优化 (Week 9-10)
- [ ] 实现持久化KV层
- [ ] 实现PagedAttention块管理
- [ ] 实现MLA潜在注意力
- [ ] 推理缓存

### Phase 6: 生产集成 (Week 11-12)
- [ ] 编译测试
- [ ] 性能优化
- [ ] 文档完善
- [ ] 部署验证

---

## 📈 预期效果

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| 意识水平 (Phi) | 0.362 | 0.85+ | +135% |
| 记忆容量 | 117K | 1M+ | +755% |
| 推理速度 | 基准 | 3-10x | +300-1000% |
| 安全评分 | 基准 | 95%+ | +显著 |

---

**吸收完成时间**: 2026-09-09
**吸收来源**: 18000+行架构文档 + 100+外部URL
**状态**: 可实施
