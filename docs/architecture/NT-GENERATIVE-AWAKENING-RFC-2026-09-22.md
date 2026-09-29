# NT 生成式推理世界模型 — 四块接线总图（RFC）

> 日期：2026-09-22
> 状态：P0–P3 代码落地已验证（晶体核 109 绿 / 决策引擎 113 绿），权重与 Metal 待接
> 前置：`docs/architecture/NT-BODY-PRESSURE-RFC-2026-09-21.md`（身体感）、`.neotrix/minimind-absorption-plan.md`（MiniMind 八模式）

## 1. 总线（一句话）

记忆即训练数据，推理即世界模型，验证即奖励，采样即行动——四个开源思想（MiniMind / TinyZero-SEAL / DreamerV3-Genie / Voyager-GA）熔成一条 CPU 可跑的闭环，权重是最后才需要的配件。

```
                    ┌─────────────────────────────────────────┐
                    │         knowledge.db (463K) / 档案 (2209万) │
                    └───────┬─────────────────────┬───────────┘
                            │                     │
              ┌─────────────▼───────────┐   ┌─────▼──────────────┐
              │ nt_db_awakening（热灌注） │   │ nt_archive_train   │
              │ Seed→Transcend          │   │ （分域流式→模式沉淀）│
              └─────────────┬───────────┘   └─────┬──────────────┘
                            │                     │
                            ▼                     ▼
              ┌─────────────────────────────────────────┐
              │ CrystalConsciousness（统一记忆 + 推理五式）│
              │ 准则：一切信息皆记忆；推理是运动中的记忆      │
              └───┬───────────────┬───────────────┬─────┘
                  │               │               │
        ┌─────────▼──────┐ ┌──────▼───────┐ ┌─────▼──────────┐
        │ nt_awaken_loop │ │ nt_predict_  │ │ nt_train_      │
        │ 课程+验证+ReST │ │ loop（FEP→   │ │ export（记忆→  │
        │ +定时反思      │ │ 爬取优先级）  │ │ MiniMind JSONL)│
        └─────────┬──────┘ └──────┬───────┘ └─────┬──────────┘
                  │               │               │
                  ▼               ▼               ▼
           Solution/Lesson   crawl_queue      pretrain/SFT/
           → scores          .priority        think/DPO
                                                  │
                                                  ▼
                                        ┌──────────────────┐
                                        │ nt_gen_model     │
                                        │ (candle 端内推理) │
                                        └──────────────────┘
```

## 2. 模块接线表

| 块 | 文件 | 对外契约 | 验证 |
|---|---|---|---|
| 热灌注 | `neotrix-core/.../nt_crystal_core/nt_db_awakening.rs` | `NtDbAwakening::awaken(意识, db, 预算)` 只读库 | 活库 477030 记忆/134万连接/200链→Transcend（ignored） |
| 流式炼制 | `.../nt_archive_train.rs` | `NtArchiveTrain::train(核心, 意识, 配置)` 分域+剪枝+定期落盘 | pilot 2 域（ignored） |
| 导出 | `.../nt_train_export.rs` | `pretrain/sft/think/dpo_lines + write_jsonl` | 5 单测 |
| 觉醒循环 | `.../nt_awaken_loop.rs` | `cycle(轮数, 奖励门, 反思节拍)` → chosen/rejected | 5 单测 |
| 预测环 | `.../nt_predict_loop.rs` | `observe/surprise/crawl_priorities/push_priorities` | 6 单测（含 temp-db 回环） |
| HF 桥 | `.../nt_hf_bridge.rs` | URL 解析→rows→`crawl_queue.jsonl` 格式 | 6 fixture 单测 |
| 生成推理 | `crates/neotrix-decision-engine/src/model/nt_gen_model.rs` | `NtGenEngine::load_dir/generate` | 5 单测（合成权重）+ live（待权重） |
| 意识本体 | `.../consciousness.rs` | `remember/recall/connect/reason/reflect/decay/consolidate` + importance + Transcend 门 | 85+ 单测 |

## 3. 论文 → 代码映射（溯源）

| 论文 | 熔点 | 代码落点 |
|---|---|---|
| MiniMind 56K⭐ Apache-2.0（jingyaogong/minimind） | JSONL 分阶段飞轮；`<think>` 自适应思考；4 experts/top-1 平衡；温度/top-k/top-p/重复惩罚 | `nt_train_export` 四格式；`think_lines`；`NtSampler`；`NtGenConfig` 默认档（8层/768/8头/6400词） |
| TinyZero 13K⭐（R1-Zero 复现）+ SEAL NeurIPS'25 | 推理来自 RL+可验证奖励；ReST EM（拒绝采样+SFT，GRPO 不稳定）；合成 implications > 原文 | `nt_awaken_loop` verify 三件套 + Solution/Lesson 记录；reason() 结论即 implications |
| DreamerV3 Nature'25 + GenieRedux CVPR'25 | 先想象再行动；固定超参跨域；LAM 离散潜动作；动力学不确定性驱动探索 | reason() 链 = imagined 轨迹；Seed→Transcend 固定门；skill_crystal = 潜动作码本；`nt_predict_loop` EMA 不确定性 → 爬取 |
| Voyager arXiv:2305.16291 | 自动课程（novelty search）；技能库（可执行/可组合）；自验证 critic | `propose_domain` 最少访问域；skill_crystal 不动；verify+Lesson |
| Generative Agents | retrieval = 新近+重要+相关；定时 reflection | recall 三信号；`reflect()` 产 Pattern |

## 4. 数据源现状

| 源 | 体量 | 状态 |
|---|---|---|
| `~/.neotrix/knowledge.db` | nodes 46万 / edges 87万 / embeddings 389K | 已炼成 Transcend（cocoons 303M） |
| `/Volumes/NeoTrixBrain` 档案库 | nodes 2209万 / edges 2240万（98.9% zim bulk） | 通道就绪（pilot 待跑），策略：精华域全炼 + bulk 按 importance 采样 |
| HF 数据集 | 按需（桥已通，679行社区清单在库） | 取数待网络恢复 + 逐集查 license |
| `/Volumes/NeoTrixBrain/cortex-archive` | 114G（zim/wikipedia/pmtiles 离线语料） | 冷备，解析器后续 |

## 5. 待接清单（按序）

1. **权重**：网络恢复后取 MiniMind-3（config + safetensors-F32 + tokenizer）→ `models/minimind-3/` → 跑 `live_minimind3` 做数值对齐（q_norm 开关以此为准）。
2. **Metal**：candle-metal 不在离线缓存，加依赖需联网；CPU 先行，接口已隔离（`Device` 参数化）。
3. **中文分词**：`keywords()` 空格分词对真实中文是弱分词（P2 已暴露）；接 `nt_file_ability` 或外部 tokenizer。
4. **茧内容去重**：两次全炼间库在涨，`importance` 重排导致同节点双 M-id（约 1.6 万条）；`sync_from_consciousness` 加内容哈希去重。
5. **embeddings 389K 接线**：`MemoryOrchestrator.retrieve` 的语义通道现在是空的（调用方无向量可传）；用库内 all-MiniLM 向量喂入。
6. **多窗口协作纪律**：本轮发生 3 次 stale-write 覆盖（mod.rs 注册、测试、权重配置）；`AGENTS.md` 并行公约追加"写前重读 + 禁整文件覆写"。

## 6. 不变量（硬线）

- `#![forbid(unsafe_code)]`（R-P1）：candle 权重走 `from_slice_safetensors` 安全路径，零 `unsafe`。
- 生产代码禁 `unwrap/expect/panic`，错误 `?`/`map_err` 传播；无 `[]` 索引；字符截断不用字节切片。
- 外部技术同会话接到生产（R-P79）：每模块都有可跑入口 + 测试，权重/Metal 是唯一外部依赖项。

## 7. 续章（2026-09-22 夜）：训练开训 + JEV 落地 + 进化闭环

### 7.1 正式微调进行中
- `models/training/finetune.py`：HF transformers 全参微调，477K 行 pretrain.jsonl，30K 步≈1epoch。
- 修了 2 个阻塞 bug：① torch 2.12.1 自适应分母开方 NaN（`adam_epsilon=1.0`）；
  ② 毒样本前向 NaN 永久污染 Adam m/v（`NanGuardTrainer` 熔断归零跳过+计数）。
- 日志口径注意：transformers 5.x 自定义 compute_loss 跳过 /accum 归一，
  上报 loss/grad_norm 为真值 ×8（对照实验证实），权重更新数学不受影响。

### 7.2 JEV 融入（免 ollama/Step-5，纯框架+本地权重）
- `malevrigns/agent-jev`（Apache-2.0）：Qwen3-0.6B 去 LM 头+候选头，三原语与
  `nt_jev` 同构；权重 `models/agent-jev/`（1.1GB），sidecar :8149 实测区分度
  正常（全过 0.96 / 大败 0.12）。
- `nt_jev_agentjev.rs`：回环 allowlist 直连桥（nt_http 拒回环故不用），输出直转
  `JevDecision`，simulation 保留当退路。
- `nt_jev_calibration.rs`：verify 分数直转 confidence 标签（COREA L1 式，
  比蒸馏外部幻觉概率更真）+ choice 配比分布 + `PlattParams` 坐标下降拟合。

### 7.3 进化闭环缺陷修复（P0–P1）
- `nt_orchestrator.rs`：tick（observe→cycle→evolve→save）+ `remember_inference`
  （`[src:]` 前缀标注 + 去重预检）。修 #1 无调度器、#2 无回灌。
- `nt_eval_loop.rs`：EvalReport→门限/重训动作（ECE>0.15 保守化，acc<0.7 重训，
  准而弃权多则放开），每动作记 Adaptation。修 #3 eval 断路。
- 备份单 .bak→5 代轮转（`backup_rotation_plan` 纯函数可测）。修 #6。
- `reflect` 加 Jaccard 去重（≥0.8 丢弃）。修 #4。
- `verify` 反博弈：结论去重计数 + 零交叠阻尼 ×0.7（跨域豁免）。修 #5。
- 导出增量：`--incremental/--since-ts/--append` + 水位文件。修 #8。
- `lora_finetune.py`：rank-16 stage-2（adapter 7.3MB），冒烟通过。修 #7。
- 数据清洗：`clean_data.py`（HTML 解码+去重+长度），`pretrain_clean.jsonl`
  464,870 行 / 51MB，供下轮训练。

## 8. 论文吸收：2609.18842 Infinite-Parameter LLMs（Boltzbit/Cambridge）

核心机制 `W(z) = W0 + B(z)A(z)ᵀ`（冻结共享基 + 低秩调制）即 LoRA；
categorical belief over materialised codes 即 domain cocoons + 路由选择
（该文 Table 5/6 证明选择器胜单次大读、胜 dense retrieval +8~12 点）；
不确定性门控可塑性（确信处保护、疑惑处快学）即 EWC 的后验版。

落地（全部已验证）：
- `lora_finetune.WeightedTrainer`：样本 loss × `calib_confidence`（calib 行自带，
  缺省 1.0），门控实证 w=[1,0]→1.300 / w=[0,1]→8.746 / 均值 5.023；
- per-domain adapter 即 materialised code 池（路由表后续）；
- 稀释边界教训：一码不装多，cocoons 保持分域有界 + selector 路由（与剪枝策略一致）。
- 存储：`crystal.json` 原子写 + `.bak`；意识进茧幂等；DB 只读打开。
