# Trending Rankings Report — 2026-09-11

## 概览

数据来源：TrendShift / GitHub Trending / GitHub Trending Rust / GitHub Topics: AI Agent / Hugging Face Daily Papers

---

## 🔥 新 P0 项目识别

### 1. NVlabs/SoL-Pi (⭐144, New)
- **链接**: https://github.com/NVlabs/SoL-Pi
- **标签**: AI agent, AI workflow
- **P0 理由**: NVIDIA Labs 出品，AI workflow 自动化，与 NT-MIND SEAL pipeline 高度相关
- **吸收价值**: workflow orchestration patterns

### 2. TencentCloud/CubeSandbox (⭐95, New)
- **链接**: https://github.com/TencentCloud/CubeSandbox
- **标签**: AI agent
- **描述**: Instant, Concurrent, Secure & Lightweight Sandbox for AI Agents
- **P0 理由**: 腾讯云出品的 AI agent 沙箱，轻量级隔离执行环境，与 NT-SHIELD sandbox 机制可对标
- **吸收价值**: sandbox isolation architecture

### 3. arcboxlabs/arcbox (⭐121, New)
- **链接**: https://github.com/arcboxlabs/arcbox
- **描述**: Run AI agents on real and isolated machines — own kernel, filesystem, and network — with <200ms boot. Local first, OCI compatible, pure Rust.
- **P0 理由**: **纯 Rust** 实现，<200ms 启动，kernel 隔离，OCI 兼容，与 NeoTrix 技术栈完全一致
- **吸收价值**: microVM isolation, Rust-native agent runtime

### 4. sapientinc/PRAXIST (⭐158, New)
- **链接**: https://github.com/sapientinc/PRAXIST
- **描述**: Autonomous research system for measurable, computer-executable research.
- **P0 理由**: 自主研究系统，可执行计算机实验，与 NT-MIND 自主进化理念一致
- **吸收价值**: autonomous research pipeline

### 5. EvoMap/AutoResearch (⭐126, New)
- **链接**: https://github.com/EvoMap/AutoResearch
- **描述**: AI/ML research agents from idea to paper-ready evidence
- **P0 理由**: 从 idea 到 paper 的全流程自动化，与 SEAL pipeline 的 distillation phase 契合
- **吸收价值**: research-to-evidence workflow

### 6. github/spec-kit (⭐203)
- **链接**: https://github.com/github/spec-kit
- **描述**: Toolkit to help you get started with Spec-Driven Development
- **P0 理由**: GitHub 官方出品，Spec-Driven Development 方法论，与 NeoTrix 架构决策可参考
- **吸收价值**: spec-driven methodology

### 7. multimodal-art-projection/YuE (⭐92)
- **链接**: https://github.com/multimodal-art-projection/YuE
- **描述**: Open Full-song Music Generation Foundation Model (类似 Suno.ai 开源版)
- **P0 理由**: 开源音乐生成模型，多模态能力扩展参考
- **吸收价值**: multimodal generation patterns

---

## 📊 GitHub Trending Top (All Languages)

| # | 项目 | Stars | 语言 | 关键标签 |
|---|------|-------|------|----------|
| 1 | hermes-agent | 244k | Python | AI agent, hermes, codex |
| 2 | Agent-Reach | 79.3k | Python | Web scraping, MCP |
| 3 | learn-claude-code | 76.5k | Python | Agent harness |
| 4 | career-ops | 71.2k | JS | AI job search |
| 5 | daily_stock_analysis | 64.9k | Python | LLM quant trading |
| 6 | ppt-master | 53.6k | Python | AI PPT generation |
| 7 | cherry-studio | 51.7k | TS | AI productivity |
| 8 | nanobot | 48k | Python | Personal AI agent |
| 9 | CowAgent | 46.9k | Python | Multi-agent harness |
| 10 | siyuan | 46.3k | TS | Knowledge workspace |

---

## 🦀 GitHub Trending Rust

| # | 项目 | Stars | 描述 |
|---|------|-------|------|
| 1 | arcbox | 121 | AI agent isolation (pure Rust) |
| 2 | googleworkspace/cli | 30.9k | Google Workspace CLI + AI agent skills |
| 3 | Hmbown/Codewhale | 40.9k | Open-source coding agent (Rust) |

---

## 🤖 GitHub Topics: AI Agent (Top by Stars)

| # | 项目 | Stars | 语言 | 核心能力 |
|---|------|-------|------|----------|
| 1 | NousResearch/hermes-agent | 244k | Python | 自进化 agent |
| 2 | Panniantong/Agent-Reach | 79.3k | Python | 多平台数据抓取 |
| 3 | shareAI-lab/learn-claude-code | 76.5k | Python | Agent harness 教程 |
| 4 | career-ops-hq/career-ops | 71.2k | JS | AI job search |
| 5 | hugohe3/ppt-master | 53.6k | Python | AI PPT 生成 |
| 6 | CherryHQ/cherry-studio | 51.7k | TS | AI productivity |
| 7 | HKUDS/nanobot | 48k | Python | 个人 AI agent |
| 8 | zhayujie/CowAgent | 46.9k | Python | 多 agent harness |
| 9 | siyuan-note/siyuan | 46.3k | TS | 知识工作空间 |
| 10 | bojieli/ai-agent-book | 45.7k | Python | AI Agent 教材 |

---

## 📄 Hugging Face Daily Papers (Sep 11)

| # | 论文 | 热度 | 关键词 |
|---|------|------|--------|
| 1 | NCP-ArchPreview: Next Concept Prediction | 38 | Latent Space Language Models |
| 2 | SpatialBlock: Spatial Intelligence in LVLMs | 19 | Spatial reasoning |
| 3 | SenseNova-U1.5: Native Unified Visual Intelligence | 4 | Multimodal |
| 4 | CARDEA: Auditable Reasoning for Coronary Angiography | 2 | Medical AI |
| 5 | Recursive Code World Models | 1 | World models |
| 6 | HyQuant: Hybrid-Precision Quantization for LLM | 1 | Quantization |
| 7 | World in World: Explore with World Models | - | World models |
| 8 | Open Recipe for IMO Gold: Nemotron Olympiad Math | - | Math reasoning |

---

## 🔬 P0 吸收优先级总结

| 优先级 | 项目 | 吸收目标 | NeoTrix 映射 |
|--------|------|----------|--------------|
| P0 | arcbox | Rust-native microVM isolation | NT-SHIELD sandbox |
| P0 | CubeSandbox | AI agent sandbox architecture | NT-SHIELD sandbox |
| P0 | PRAXIST | Autonomous research pipeline | NT-MIND SEAL |
| P0 | AutoResearch | Research-to-evidence workflow | NT-MIND distillation |
| P0 | SoL-Pi | AI workflow patterns | NT-MIND pipeline |
| P1 | spec-kit | Spec-driven methodology | Architecture decision |
| P1 | YuE | Multimodal generation | NT-IO multimodal |

---

*Generated: 2026-09-11T00:00:00Z*
*Sources: TrendShift, GitHub Trending, GitHub Topics, HuggingFace Papers*