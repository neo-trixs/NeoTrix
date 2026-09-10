# GitHub Batch 210 — 16 Repos Research Report

> Generated: 2026-09-10 | Source: GitHub repos | Priority mapping: P0 (critical) → P3 (low)

---

## 1. StrikeAgent_AtkBrain-Flash

- **URL**: https://github.com/Yean-Sec/StrikeAgent_AtkBrain-Flash
- **Stars**: 312 ⭐
- **Core Function**: AI 渗透测试平台，攻击图驱动自循环 + 监督轮次边界 + 技法蒸馏记忆库
- **Architecture**: 控制台调度猎面 → 攻击图驱动自循环 → 监督轮次出方案 → 技法蒸馏回灌
- **Absorbable Patterns**: 攻击图驱动闭环自进化、红队二次评级/验证机制、skill 嵌套深度平衡、蒸馏回灌记忆库
- **NeoTrix Mapping**: NT-SHIELD (影卫) — 渗透测试自动化、攻击面收敛、红队协作
- **Priority**: P2 — NT-SHIELD 可吸收攻击图驱动 + 蒸馏回灌模式

---

## 2. hyperresearch

- **URL**: https://github.com/jordan-gibbs/hyperresearch
- **Stars**: 2.1k ⭐
- **Core Function**: Agent 驱动深度研究平台，16 步 pipeline + 对抗审阅 + 持久化知识 vault
- **Architecture**: 16-step tier-adaptive pipeline → 4 parallel critics → surgical patcher → cite-check → persist vault (markdown+SQLite)
- **Absorbable Patterns**: 16-step分级研究pipeline、对引用的怀疑性验证、去重/独立性审计、vault持久化+可搜索、tool-lock只允许Read+Edit禁止重写、pageRank源质量排序
- **NeoTrix Mapping**: NT-MIND (进化工匠) — 研究蒸馏管线；NT-MEMORY (知识守护者) — vault 持久化
- **Priority**: P0 — 高度同构 SEAL pipeline + 知识吸收 + 对抗审阅

---

## 3. OpenHands

- **URL**: https://github.com/OpenHands/OpenHands
- **Stars**: 87.2k ⭐
- **Core Function**: 自托管开发者控制中心，支持多 agent (OpenHands/Claude Code/Codex/Gemini) + ACP 协议 + 自动化编排
- **Architecture**: Agent Canvas frontend + Agent Server REST API + Automation Server + 多后端切换 (Docker/VM/Cloud)
- **Absorbable Patterns**: ACP协议统一agent接入、多后端无缝切换、自动化调度 (schedule+webhook)、Docker沙箱隔离、前端控制中心
- **NeoTrix Mapping**: NT-IO (界面使徒) — 多agent调度控制台；NT-ACT (行动执行者) — 自动化编排
- **Priority**: P1 — 架构对齐度高，NT-IO 可参考 Agent Canvas 模式

---

## 4. OpenBiliClaw

- **URL**: https://github.com/whiteguo233/OpenBiliClaw
- **Stars**: 3.3k ⭐
- **Core Function**: 本地私有跨平台 AI 内容发现 Agent，心理画像+五层灵魂画像+跨平台主动推荐
- **Architecture**: 浏览器插件(信号采集) → 本地Python后端(画像+推荐) → 桌面/移动端Web UI → SQLite
- **Absorbable Patterns**: 心理画像五层架构(事件→偏好→觉察→洞察→灵魂)、跨平台信号聚合、本地优先隐私、猜测兴趣主动破茧、DeepSeek Harness 插件
- **NeoTrix Mapping**: NT-WORLD (虚空探索者) — 跨平台内容感知；NT-FEEL (情感中枢) — 用户画像
- **Priority**: P1 — 心理画像+跨平台感知可扩展 NT-WORLD 和 NT-FEEL

---

## 5. nopus

- **URL**: https://github.com/Vistyy/nopus
- **Stars**: 282 ⭐
- **Core Function**: 确定性散文检查工具，检测 AI agent 输出的抽象/冗余模式并请求重写
- **Architecture**: 确定性度量 (词频+抽象度+名词堆叠+短语负载) → 阈值判定 → 触发重写请求
- **Absorbable Patterns**: 确定性prose质量检查、AI slop模式库 (20+模式)、tool-lock限制重写行为、敏感度分级(low/medium/high)
- **NeoTrix Mapping**: NT-SHIELD (影卫) — 输出质量守卫；NT-META (元认知) — 自我检查
- **Priority**: P2 — 可吸收 prose 质量检查模式到 NT-META 输出门禁

---

## 6. AIPOCH Open-Science

- **URL**: https://github.com/aipoch/open-science
- **Stars**: 4k ⭐
- **Core Function**: 开源本地优先AI科研工作台，科学agent + Python/R notebook + 可溯源 artifact 版本
- **Architecture**: Electron (React+TypeScript) + Prisma/SQLite + ACP agent runtime + immutable artifact versions with provenance
- **Absorbable Patterns**: 不可变 artifact 版本 + provenance 溯源、科研数据连接器 (24 built-in)、Python/R notebook 集成、HPC远程计算、reviewer审计循环
- **NeoTrix Mapping**: NT-MEMORY (知识守护者) — artifact 溯源；NT-ACT (行动执行者) — notebook 执行
- **Priority**: P1 — provenance 溯源 + artifact 版本管理同构 NT-MEMORY 设计

---

## 7. autoresearch (uditgoenka)

- **URL**: https://github.com/uditgoenka/autoresearch
- **Stars**: 6.3k ⭐
- **Core Function**: Claude Code/OpenCode/Codex 自主迭代引擎，基于 Karpathy autoresearch 理念
- **Architecture**: 薄路由SKILL.md (41行) + 12个自包含子命令文件 + git作为记忆 + TSV结果日志
- **Absorbable Patterns**: 约束+度量+自主迭代=复合增益、git作为记忆(experiment前缀commit)、自动回滚、有界默认迭代、guard安全网、explorer子命令模式
- **NeoTrix Mapping**: NT-MIND (进化工匠) — 自主迭代循环；NT-REPAIR (自愈工程师) — debug/fix命令
- **Priority**: P0 — 核心迭代循环同构 SEAL pipeline 的 modify→verify→keep/discard

---

## 8. Robin

- **URL**: https://github.com/apurvsinghgautam/robin
- **Stars**: 7.1k ⭐
- **Core Function**: AI 驱动暗网 OSINT 工具，LLM 精炼查询 + 暗网搜索引擎 + 调查摘要
- **Architecture**: Streamlit UI → 多模型支持 (OpenAI/Claude/Gemini/Ollama) → Tor 搜索 → LLM 分析 → 会话式追问
- **Absorbable Patterns**: LLM精炼OSINT查询、会话式追问(不重跑搜索)、一键pivot(从发现中推荐后续)、模块化search→scrape→LLM pipeline
- **NeoTrix Mapping**: NT-SHIELD (影卫) — 暗网情报采集；NT-WORLD (虚空探索者) — 深度搜索
- **Priority**: P2 — OSINT采集模式可增强 NT-WORLD 搜索能力

---

## 9. browser-use-pi

- **URL**: https://github.com/browser-use/browser-use-pi
- **Stars**: 256 ⭐
- **Core Function**: 基于 Pi Mono 的 TypeScript web agent，持久V8 REPL + raw CDP + WebGPU
- **Architecture**: Task → Pi Mono → persistent V8 REPL → raw CDP → Chrome; AX tree + screenshots feedback loop
- **Absorbable Patterns**: 持久V8 REPL for可编程agent、raw CDP直接控制、session管理(保存登录)、typed results + compaction、streaming
- **NeoTrix Mapping**: NT-WORLD (虚空探索者) — 浏览器自动化；NT-ACT (行动执行者) — web交互
- **Priority**: P3 — 浏览器自动化可参考但优先级较低

---

## 10. Comet

- **URL**: https://github.com/rpamis/comet
- **Stars**: 3.0k ⭐
- **Core Function**: 可恢复长运行任务工作流 + Skill 平台，Native/Classic双模式 + 评估平台
- **Architecture**: .comet/config.yaml → Native(强模型自主) / Classic(OpenSpec+Superpowers五阶段) → state machine + phase checks + resumable archive
- **Absorbable Patterns**: Native vs Classic双模式、Supervisor Changes (DAG分解+并行worktree实现)、Skill创建/分发/评估、Rubric/Pass@k/Pass^k科学评估、LangSmith集成
- **NeoTrix Mapping**: NT-MIND (进化工匠) — 工作流编排 + 评估；NT-ACT (行动执行者) — 多agent协作
- **Priority**: P0 — Supervisor Changes + Skill评估平台 + 双模式工作流高度同构

---

## 11. SwarmLLM

- **URL**: https://github.com/Nehanth/swarmllm
- **Stars**: 195 ⭐
- **Core Function**: P2P LLM推理，浏览器tab间拆分27B模型，WebGPU引擎+WebRTC运行时
- **Architecture**: WebGPU engine (~50 WGSL kernels) + WebRTC room + 层级拆分 (每个设备持有一层切片) + 10KB激活向量传递
- **Absorbable Patterns**: 设备级模型层拆分、speculative decoding验证、bit-exact golden tests、内存带宽极限优化、无服务器P2P推理
- **NeoTrix Mapping**: NT-PHYSICAL (具身骨架) — 分布式推理；NT-IO (界面使徒) — WebGPU引擎
- **Priority**: P3 — 前沿技术但与NeoTrix核心域距离较远

---

## 12. emilkowalski/skills

- **URL**: https://github.com/emilkowalski/skills
- **Stars**: 36.6k ⭐
- **Core Function**: 面向设计师和工程师的AI技能集，聚焦动画/设计/UI质量
- **Architecture**: Skill packages (SKILL.md) + 12个设计/动画专用技能 + npx skills 安装
- **Absorbable Patterns**: 领域专家技能封装 (动画曲线/easing选择)、anti-slop设计模式、skill作为品味放大器、prototype多版本对比
- **NeoTrix Mapping**: NT-IO (界面使徒) — UI/动画技能；NT-META (元认知) — 设计品味
- **Priority**: P2 — 设计技能可参考 NT-IO UI模式

---

## 13. no-ai-slop

- **URL**: https://github.com/petergyang/no-ai-slop
- **Stars**: 8.0k ⭐
- **Core Function**: 移除 AI 写作中 20+ 种 slop 模式，保留个人风格
- **Architecture**: SKILL.md 编辑规则 + eval.md 检查规则 + plugin.json 元数据
- **Absorbable Patterns**: 20+ AI slop模式库 (二元对比/清嗓开头/伪洞察/冒号揭示等)、voice preservation、slop检测+重写
- **NeoTrix Mapping**: NT-SHIELD (影卫) — 输出质量门禁；NT-META (元认知) — 自我审查
- **Priority**: P2 — slop 模式库可增强 NT-META/NT-SHIELD 输出质量检查

---

## 14. PostBot

- **URL**: https://github.com/gitcoffee-os/postbot
- **Stars**: 1.4k ⭐
- **Core Function**: 多平台内容同步分发工具，一键发布至微信/微博/B站/YouTube等10+平台
- **Architecture**: 浏览器扩展(登录态复用) → 本地后端 → 多平台API/页面自动发布 → 桌面/移动端UI
- **Absorbable Patterns**: 浏览器登录态本地复用(安全)、多平台适配引擎、一键同步分发、智能网页阅读器、CLI工具
- **NeoTrix Mapping**: NT-ACT (行动执行者) — 多平台发布；NT-IO (界面使徒) — 内容管理
- **Priority**: P3 — 内容分发与NeoTrix核心域距离较远

---

## 15. DeepSelect

- **URL**: https://github.com/deepseek-ai/DeepSelect
- **Stars**: 135 ⭐
- **Core Function**: DeepSeek 稀疏注意力 TopK 内核，2-20x 加速 torch.topk
- **Architecture**: CUDA kernels + Lightning Indexer (bf16) / Sampling (fp32) 两种场景优化
- **Absorbable Patterns**: TopK kernel优化、稀疏注意力内存带宽极限、变长行支持、NaN安全检查
- **NeoTrix Mapping**: NT-CORE (E8引导者) — 推理优化；NT-PHYSICAL (具身骨架) — GPU内核
- **Priority**: P3 — 底层推理优化，短期与NeoTrix距离较远

---

## 16. LLM-Agent-Paper-List

- **URL**: https://github.com/WooooDyy/LLM-Agent-Paper-List
- **Stars**: 8.2k ⭐
- **Core Function**: LLM-based Agent 综述论文列表，覆盖 Brain/Perception/Action/社会模拟
- **Architecture**: 论文分类目录 (Brain知识→记忆→推理→规划; Perception视觉/音频; Action工具/具身; Society行为/模拟)
- **Absorbable Patterns**: Agent三组件框架 (Brain+Perception+Action)、记忆能力分类 (长度扩展/总结/向量压缩/检索)、规划反射机制、Agent社会模拟
- **NeoTrix Mapping**: 全域参考 — 架构设计方法论、记忆/推理/规划模式库
- **Priority**: P1 — 架构设计参考 + 模式库

---

## Priority Summary

| Priority | Repos | Key Patterns |
|----------|-------|-------------|
| **P0** | hyperresearch, autoresearch, Comet | SEAL pipeline同构、自主迭代、Skill评估平台、对抗审阅 |
| **P1** | OpenHands, OpenBiliClaw, AIPOCH Open-Science, LLM-Agent-Paper-List | ACP协议、心理画像、artifact溯源、架构方法论 |
| **P2** | StrikeAgent, nopus, Robin, emilkowalski/skills, no-ai-slop | 攻击图、prose质量、OSINT、设计技能、slop检测 |
| **P3** | browser-use-pi, SwarmLLM, PostBot, DeepSelect | 浏览器自动化、P2P推理、内容分发、GPU内核 |

## Top Absorbable Patterns for NeoTrix

1. **SEAL Pipeline 同构** (hyperresearch + autoresearch): 16步分级研究 + 自主迭代循环 → NT-MIND 进化引擎
2. **Skill 评估平台** (Comet): Rubric + Pass@k + Pass^k + LangSmith → NT-MIND 技能质量门禁
3. **Artifact Provenance** (AIPOCH): 不可变版本 + 溯源 + reviewer审计 → NT-MEMORY 知识版本管理
4. **Agent Canvas** (OpenHands): ACP协议 + 多后端切换 + 自动化调度 → NT-IO 多agent控制台
5. **心理画像五层** (OpenBiliClaw): 事件→偏好→觉察→洞察→灵魂 → NT-FEEL 用户理解
6. **Prose质量门禁** (nopus + no-ai-slop): 20+ slop模式 + 确定性检查 → NT-SHIELD/NT-META 输出守卫
