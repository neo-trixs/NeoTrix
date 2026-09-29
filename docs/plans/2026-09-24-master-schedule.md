# 全域吸收总排期表（2026-09-24，第二批约 300 URL 去重归并）

> 输入：两批 URL 合计 ~340 条，去重后按簇归并。第一批 40 源结论见
> `2026-09-24-external-absorb-batch.md`（P0 16 / P1 19 / P2 7），本表不重复深吸，
> 只对冲增量。原则：簇级排期 + 长尾走情报管线（Dark Forest：无消费者不单独立项）。

## 0. 输入盘点

- 已吸对冲（跳过）：第一批 40 源全集 + minimind（底座）+ storm/agent-skills/eliza/hindsight/
  Graft/repowise/semantica/knowledge_graph/CyberStrike/agency-agents/open-code-review/
  tidings-rss/OpenExecutive/Panniantong/Agent-Reach（第一批已覆盖或入库）。
- 列表/话题/动态页（P2，不吸，当情报源）：topics/*（~25 个）、awesome-*（~15）、
  trendshift.io、producthunt、huggingface.co/spaces、figma 社区、drive.google ×2（无权限打不开，
  待主给内容）、微信文章/播客/视频站。
- 基础设施（P2 工具箱，用时再查）：ollama / vllm / llama.cpp / sglang（另见 P0）/ milvus /
  elasticsearch / syncthing / tauri / expo / hono / n8n / dify / outline / immich /
  supabase / open-webui / librechat / jan / ollama系。结论：只记存在，不排期。

## 1. 今晚内存窗（ftv4 落盘后，存量 9 件不变，+2 增量）

| # | 任务 | 增量源 | 门 |
|---|---|---|---|
| W0-1 | eval 验收 + 权重封存 | — | eval_v2.done |
| W0-2 | D3 MLX LoRA 首版 | laya-mlx（MLX-Jev 7–14ms 范式参考，6k★已验存在） | eval_lora + D5 |
| W0-3 | D1 实现 + Graft 格式 | —（设计齐） | 对照 7/10 |
| W0-4 | D4 L1/L2 | — | 有用占比口径 |
| W0-5 | **sglang 本地链打通**（NLL 打分/judge 吞吐就靠它；prefix 复用即 LLM2Jev 思想的生产版） | sgl-project/sglang | Qwen3-0.6B 本地 scoring 通 |

## 2. 本周（P0 新立 6 单）

| # | 任务 | 源（已验存在） | 门 |
|---|---|---|---|
| W1-1 | **个人 AI  harness 对标**：pi（108k★，统一 LLM API+loop+TUI）、openharness…openhuman（40k★，local-first memory+编排）、atlas（6k★，多 agent 变更追踪）、PI-Desktop | earendil-works/pi、tinyhumansai/openhuman、pacifio/atlas、vastsa/PI-Desktop | 对标表→MetaAgentShell 缺口单 |
| W1-2 | **Jev 生态第二轮 smelt**：KaLM-Jev（embedding+Jev）、fast-jev-compaction（上下文压缩！）、jev-review/jev-mcp×2/codex-router、BrowserSkill（腾讯）、TencentDB-Agent-Memory、super-hermes/hermes-workspace/dojo、council | smelt 管线复用（965 卡已验证） | 新 cards 入库数 |
| W1-3 | **记忆簇对标**：mem0、memvid、graphiti、TencentDB-Agent-Memory、COG-second-brain、claude-mem、ai-memory、memtensor | 晶体记忆（接 hindsight 五单之后） | 对标表→observations 下一刀 |
| W1-4 | **swarm 智能**：MiroFish（74k★，群体智能预测）、NousResearch/hermes-agent、openai/swarm、Sakshxm1/hermes-agency-orchestrator | NT-PLAY 群体 + MetaShell 派单 | 原型一单 |
| W1-5 | **感知扩展**：RuView（94k★，WiFi 空间智能）、VoiceStudio（34k★，本地语音克隆）、Qwen-MM/Moonshine/PaddleOCR 系 | l2_perception（world-sense） |  respiratory？不，先文本链：VoiceStudio 本地 TTS/STT 接入单 |
| W1-6 | **skill 批量引**（见 §3，第一批 60 个，按域映射表进 staging，agnix 式校验） | 下沉技能海 | staging 清单 + 校验通过率 |

## 3. skill 海 intake（上百个 *-skill，不逐个深读，管线化）

域映射（缺口→来源池）：
- design/ui：ui-ux-pro-max、taste-skill、antvis/Infographic、diagram-design×N、excalidraw、penpot、shadcn系、magicui、motion-primitives、vercel design-md、apple design、qiaomu-icon、ip-as-logo、vibe-designing-playbook → des 家族。
- security：trailofbits/skills、OWASP MCP、cloudflare/security-audit-skill、codex-security、defending harness、MCP-Governance → nt_shield。
- 办公文档：OfficeCLI（nt_file_ability 直系！）、dashi-ppt、guizang-ppt/social-card/product-video、moneyprinter、notebooklm、markitdown（微软）→ nt_file_ability。
- research：autoresearch（karpathy + uditgoenka + webfuse）、hyperresearch、deep-researcher-24x7、STORM（已吸）→ res-scholar。
- novel/游戏：chinese-novelist、novel-studio、godot/redot、blender-mcp、manim/remotion、cozy/indie topics → NT-PLAY/世界构建。
-  embedded/端：pi-* 系（computer-use、phone、mail、extensions）、macos-sysdata、disk-cleanup、mac-performance-monitor → reporter/系统位。
- 反 AI 味：kill-ai-slop、no-ai-slop、avoid-slop、humanizer、lieflat → des Anti-Slop 同盟（des-language 纪律）。
- 汉语生态：op7418 系、baoyu-skills、kunkun、humanizer-zh、reverse-skill（出现 8 次，高频，优先看）、ljg-skills → 中文任务验收。
管线：`skills-staging/` + 清单（源/域/星级/许可证）→ 校验（frontmatter+引用存在性，agnix 到来前手写 30 行）→ 每周一批 10 个进 `skills/`。
首批 10（高频+高星）：reverse-skill、ui-ux-pro-max-skill、taste-skill、cloudflare/security-audit-skill、OfficeCLI、markitdown、tidings-rss（已吸，先生效）、Pretty-mermaid-skills、antvis/Infographic、guizang-ppt-skill。

## 4. 论文阅读队列（按簇选读，不全下）

| 优先 | 论文 | 理由 |
|---|---|---|
| P0 | skilladam 2609.08944（skill 进化） | D2 复用→进化闭环的学术版 |
| P0 | wikiskill 2608.27454（经验编译进 wiki） | experience→wiki，D2 同源 |
| P0 | turingpost 9-paths RSI + RSIAgent（paper 2609.15364？）+ rsi-exam/Dream-RSI | 递归自改进簇，晶体自迭代的理论对照 |
| P1 | LightMem/LightMem2、GEPA（优化器）、parser-read 2609.06702（并行读+深度推理） | 记忆/优化/长上下文 |
| P1 | longhorizon 2609.11192、AMAP-ML 套件、dair.ai harness-engineering 合集 | 评估与 harness |
| P1 | jina-ocr、sam-3、Qwen-MM 插件 | doc-parse 多模态 |
| P2 | 其余 2608/2609 arxiv（~60 篇） | 按需检索，不预读；alphaxiv 文件夹 + trendshift 作持续情报 |

## 5. 远期（季度，不开工只记位）

- L3 judge → Gateway allowlist → DSH/跨机 → Veloren 世界架构 → mental models 开机页。
- 大厂 harness（claude-code/codex/opencode/kilocode/aider/goose/amp/tra LK?/trae/qwen-code/kimi-code）：只记存在，用时对标（DSH 思想已覆盖）。
- 安全红队（Exegol/ghidra/maigret/nuclei/pentest 系）：nt_shield 演习弹药库。
- 脑科学/cluster（果蝇连接组 Nature 系、ucl?/morpho）：意识隐喻库，不进工程。
- Godot/Redot + blender-mcp + manim：NT-PLAY 客户端技术栈候选。

## 6. 本轮已验证存在的新锚（API 实查，星级/语言/一句话见 §2 出处行）

pi 108k★TS / openhuman 40k★Rust / RuView 94k★Rust / VoiceStudio 34k★Py / MiroFish 74k★Py /
laya 21k★Py / laya-mlx 6k★Py / atlas 6k★TS / PI-Desktop 5k★TS / ASC 1.9k★Py / OpenResearch 5.6k★Rust /
splash 697★Py（Apple Silicon 本地推理引擎，MLX 路备选底座）。
