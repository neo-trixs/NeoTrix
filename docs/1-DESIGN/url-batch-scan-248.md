# URL Batch Scan #248

**Scan Date**: 2026-09-11
**Sources**: 6 URLs (2 GitHub repos, 1 arxiv paper, 3 GitHub repos)

---

## 1. FreePEP (siknet/FreePEP)

**URL**: https://github.com/siknet/FreePEP
**Stars**: 885 | **License**: MIT

### Core Function
人教社中小学电子教材批量下载器。自动解析人教社电子教材平台、批量抓取高清图片并合成 PDF。支持 WebUI + CLI 双模式，内置 780+ 本教材目录，含 AES 解密引擎和阿里云 WAF 滑块验证码自动破解。

### Technical Innovations
- **AES 逆向解密引擎**: 对人教社教材加密协议的逆向工程实现
- **WAF 滑块验证码自动破解**: Playwright 驱动的自动化绕过机制
- **多线程断点续传**: 支持层级目录归档 (学段/年级/教材名.pdf)
- **单文件 EXE 打包**: PyInstaller + 内嵌 Chromium 绿色便携包

### NeoTrix Relevance
- **低** — 垂直领域爬虫工具，针对特定教育资源平台
- 爬虫架构 (Playwright + AES 解密) 可作为 NT-WORLD 定向抓取参考
- 多线程断点续传模式与 nt_act::production_orchestrator 的批量任务管理同构
- 但平台特异性强，通用价值有限

### Fusion Priority: **P2**
> 参考其 Playwright WAF 绕过和 AES 逆向模式，作为 NT-WORLD 特定平台抓取的 skill 模板

---

## 2. visual-explainer (nicobailon/visual-explainer)

**URL**: https://github.com/nicobailon/visual-explainer
**Stars**: 9.7k | **License**: MIT

### Core Function
Agent skill — 将终端输出转化为富 HTML 页面/幻灯片。支持架构图、diff review、plan audit、数据表、项目回顾等场景的可视化渲染。通过 Mermaid/CSS Grid/Chart.js 自动路由到正确渲染方案。

### Technical Innovations
- **多 agent harness 适配**: 原生支持 Claude Code / Pi / MCP / Cursor / OpenCode / Codex CLI / Antigravity 等 7+ agent 平台
- **Quick Mode JSON schema**: 紧凑 JSON spec → 确定性本地渲染 (无需 LLM)
- **11 主题 + 字体对**: 运行时切换，Mermaid SVG 颜色自动同步
- **PPTX 静态导出**: HTML deck → .pptx 最佳努力转换
- **自动触发**: 4+ 行/3+ 列表格自动升级为 HTML 渲染
- **MCP stdio server**: 本地 stdio 模式，无 HTTP 监听，安全隔离

### NeoTrix Relevance
- **高** — 与 NT-IO (界面使徒) + nt_file_ability (PPT 能力) 直接同构
- **Skill 接口契约**: SKILL.md (<200 lines) + references/ + templates/ 结构 → 验证 NT SKILL-SPEC.md 契约
- **多 harness 适配模式**: 可复用到 NeoTrix 的多 agent 适配层 (opencode/Claude Code/Pi)
- **Quick Mode JSON→HTML**: 可替代 fireworks-tech-graph 的部分场景，零依赖确定性渲染
- **Mermaid 主题联动**: 可整合进 des/ui 的 design-language token 系统
- **MCP server 安全隔离**: jail 目录 + symlink 拒绝 + 临时文件写入 → 值得借鉴

### Fusion Priority: **P0**
> 直接吸收 Skill 接口契约 + Quick Mode 渲染模式 + 多 harness 适配架构。整合进 NT-IO 可视化能力，替代/增强 data-viz 和 ppt-design 的输出层

---

## 3. DeerFlow (bytedance/deer-flow)

**URL**: https://github.com/bytedance/deer-flow
**Stars**: 82.3k | **License**: MIT

### Core Function
字节跳动开源的 Super Agent Harness。编排 sub-agents、memory、sandboxes、skills 处理分钟到小时级别的长程任务。v2.0 全面重写，从 Deep Research 框架进化为通用 Super Agent 编排器。

### Technical Innovations
- **Session Goals + Context Compaction**: 会话级目标管理 + 手动上下文压缩
- **Sub-Agent 编排**: 独立子 agent 编排，支持 Docker/K8s 沙箱隔离
- **Long-Term Memory**: 持久化记忆系统 (SQLite/Postgres + Redis stream bridge)
- **Sandbox 多模式**: Local / Docker / K8s 三级沙箱执行
- **Gateway 架构**: nginx 统一入口 + SSE streaming + lease-based 多 worker 协调
- **MCP Extensions**: 运行时动态 MCP server 管理 + tool_search 路由
- **ACP Agent 集成**: 支持 Codex CLI / Claude Code / MiniMax Code 的 ACP 协议
- **Checkpoint Delta**: 增量检查点存储 (JSONL/Redis) + 快照频率控制
- **LangGraph Studio**: 开发时图检查 + 生产运行时分离

### NeoTrix Relevance
- **极高** — 架构层面与 NeoTrix 高度同构:
  - DeerFlow Session Goals ≈ NT-MIND SEAL pipeline 阶段管理
  - DeerFlow Sub-Agents ≈ NT-ACT orchestration + capability routing
  - DeerFlow Memory ≈ NT-MEMORY KB + experience 持久化
  - DeerFlow Sandbox ≈ NT-SHIELD 沙箱隔离
  - DeerFlow Gateway ≈ NT-IO 统一入口
- **Skill 系统**: `.agent/skills/` + `skills/public/` → 验证 NT SKILL-SPEC.md 方向
- **Checkpoint Delta + Snapshot Frequency**: 可优化 NT-MEMORY 的增量存储策略
- **Lease-based 多 Worker**: 值得借鉴到 NT-ACT 的并行任务协调
- **Context Compaction**: 与 KVMem paged KV 互补，短任务用 compaction

### Fusion Priority: **P0**
> 架构级参考价值。重点吸收: (1) Skill 系统契约 (2) Checkpoint Delta 模式 (3) Sandbox 多模式编排 (4) Gateway lease-based 协调。作为 NT-ACT + NT-MIND 编排层的对标实现

---

## 4. HoneyRoute (arXiv:2609.08306)

**URL**: https://arxiv.org/abs/2609.08306
**Paper**: HoneyRoute: Honeypot-Model Routing for Adversarial LLM Serving

### Core Function
推理服务层安全方案 — 检测恶意请求并路由到蜜罐模型，保护生产模型同时收集攻击情报。三层架构: streaming router (0.8B embedding + MLP heads) → dual honeypot → analysis loop (attacker fingerprint → router retraining)。

### Technical Innovations
- **Streaming Router**: 冻结 0.8B embedding backbone + per-domain MLP heads, F1=0.911, 38ms 中位延迟
- **Dual Honeypot**: 规则/prompt-engineered 代码蜜罐 或 同族副本专用蜜罐
- **Analysis Loop**: 蜜罐交互 → 攻击者指纹 → 路由器重训练闭环
- **Fidelity-Traceability Frontier**: 选择性伪装注入可恢复 88.9% 保真度 vs 7.6% 无条件诱饵
- **Loop-trained Correction Head**: 安全研究误路由降低 9x, F1 提升至 0.933

### NeoTrix Relevance
- **高** — 与 NT-SHIELD (影卫) + NT-IO (LLM 路由) 直接相关:
  - **Egress Privacy Guard 同构**: NeoTrix 已有出站隐私防护，HoneyRoute 提供入侵侧检测
  - **蜜罐模型 → GWT salience 路由**: 恶意请求检测可整合进 GWT 注意力路由的 cost-aware 维度
  - **Attacker Fingerprint → KB**: 攻击指纹可写入 NT-MEMORY KB，支持跨 session 威胁情报
  - **Fidelity-Traceability Frontier**: 为 NT-SHIELD 的 honeypot 策略提供理论边界
- **延迟预算**: 38ms 中位延迟可接受，适合 NT-IO 实时路由场景

### Fusion Priority: **P0**
> 安全核心论文。吸收: (1) Streaming Router 架构 → NT-SHIELD 入侵检测 (2)蜜罐模型路由 → GWT cost-aware 路由扩展 (3) Attacker fingerprint → KB 威胁情报 (4) Fidelity-Traceability 理论 → NT-SHIELD 策略边界

---

## 5. Stirling-PDF (Stirling-Tools/Stirling-PDF)

**URL**: https://github.com/Stirling-Tools/Stirling-PDF
**Stars**: 91.7k | **License**: MIT (open-core)

### Core Function
GitHub #1 PDF 应用平台。50+ PDF 工具 (编辑/合并/拆分/签名/OCR/转换/压缩)，支持桌面客户端/浏览器/自托管服务器 + 私有 API。企业级 SSO + 审计 + 40+ 语言 UI。

### Technical Innovations
- **50+ PDF 操作引擎**: Java/Spring Boot 全功能 PDF 处理
- **无代码工作流**: UI 内直接编排 PDF 处理管线
- **Docker 一键部署**: `docker run -p 8080:8080` 即用
- **REST API 平台**: 几乎所有工具都有 API 端点
- **OpenSSF Scorecard**: 安全审计透明
- **Taskfile 统一命令运行器**: build/dev/test 一体化

### NeoTrix Relevance
- **中** — PDF 处理与 NT-IO (文档处理) + nt_file_ability 有交集
- **PDF 操作引擎**: 可作为 NT-IO PDF 处理的参考实现或 MCP server 集成
- **无代码工作流编排**: 与 SEAL pipeline 的 stage 编排思想同构
- **REST API 设计**: 可参考其 API 路由设计优化 NT-IO 接口层
- **但**: Java 栈 (Spring Boot) 与 NeoTrix Rust 栈差异大，吸收方式为 API 设计参考 + MCP server 集成，非代码级

### Fusion Priority: **P1**
> (1) 作为 MCP server 集成到 NT-IO PDF 处理管线 (2) 参考其 50+ 工具的 API 路由设计 (3) 无代码工作流编排模式参考

---

## 6. Crawl4AI (unclecode/crawl4ai)

**URL**: https://github.com/unclecode/crawl4ai
**Stars**: 82.1k | **License**: Apache-2.0

### Core Function
LLM-friendly 开源 Web Crawler & Scraper。将网页转化为干净 Markdown 供 RAG/agent/数据管线使用。异步浏览器池 + 缓存 + 自适应智能抓取，50K+ 开发者社区。

### Technical Innovations
- **Fit Markdown**: BM25 启发式过滤 → 去噪 AI-friendly Markdown
- **Async Browser Pool**: 异步浏览器池 + 页面预热，3 级架构 (permanent/hot/cold)
- **MemoryAdaptive Dispatcher**: 自适应内存调度，流式爬取关闭时无泄漏
- **Deep Crawl Crash Recovery**: `resume_state` + `on_state_change` 回调，JSON 可序列化状态
- **Prefetch Mode**: 跳过 markdown/extraction/media 处理，5-10x 加速 URL 发现
- **Anti-Bot Detection + Proxy Escalation**: 3 级检测 (已知厂商/通用指标/结构完整性) + 自动代理链
- **Shadow DOM Flattening**: 提取 Shadow DOM 内隐藏内容
- **LLM Extraction**: 支持任意 LLM 的结构化数据提取 (schema-driven)
- **Docker 3-tier Browser Pool**: permanent/hot/cold 浏览器分级管理 + Janitor 自动清理
- **DomainMapper**: 域名映射特性
- **Security Hardening**: JWT auth + loopback binding + request body 不信任边界

### NeoTrix Relevance
- **极高** — 与 NT-WORLD (虚空探索者) 直接同构:
  - **Crawl4AI Async Browser Pool ≈ NT-WORLD UnifiedCrawler 浏览器池**: 3 级架构 (permanent/hot/cold) 完全同构
  - **Fit Markdown (BM25) ≈ NT-MEMORY BM25 搜索**: 内容过滤策略可复用
  - **Deep Crawl Crash Recovery ≈ SEAL pipeline 断点续传**: 状态持久化 + 回调模式
  - **Prefetch Mode ≈ NT-WORLD 两阶段爬取**: URL 发现阶段 → 选择性处理阶段
  - **Anti-Bot 3 级检测 ≈ NT-SHIELD stealth net**: 检测 + 代理升级 + 回退
  - **Shadow DOM Flattening**: NT-WORLD 缺失能力，需补充
  - **LLM Extraction (schema-driven)**: 与 NT-MEMORY KB embedding 管线互补
  - **Docker 安全加固**: JWT + loopback + trust boundary → NT-SHIELD 参考

### Fusion Priority: **P0**
> NT-WORLD 核心对标。吸收: (1) 3 级浏览器池架构 (2) BM25 Fit Markdown 过滤 (3) Deep Crawl Crash Recovery (4) Prefetch 两阶段模式 (5) Anti-Bot 3 级检测 (6) Shadow DOM Flattening (7) LLM schema-driven extraction (8) Docker 安全加固模式

---

## Fusion Matrix

| Source | Priority | Target Domain | Key Absorption |
|--------|----------|---------------|----------------|
| **visual-explainer** | P0 | NT-IO, nt_file_ability | Skill 接口契约, Quick Mode JSON→HTML, 多 harness 适配, MCP 安全隔离 |
| **DeerFlow** | P0 | NT-ACT, NT-MIND, NT-MEMORY | Skill 系统, Checkpoint Delta, Sandbox 编排, Gateway lease 协调, Context Compaction |
| **HoneyRoute** | P0 | NT-SHIELD, NT-IO | Streaming Router, 蜜罐路由, Attacker Fingerprint → KB, Fidelity-Traceability |
| **Crawl4AI** | P0 | NT-WORLD, NT-SHIELD | 3 级浏览器池, BM25 Fit Markdown, Crash Recovery, Prefetch, Anti-Bot, Shadow DOM, LLM extraction |
| **Stirling-PDF** | P1 | NT-IO | PDF 工具 API 设计, 无代码工作流, MCP server 集成 |
| **FreePEP** | P2 | NT-WORLD | Playwright WAF 绕过, AES 逆向, 多线程断点续传 (特定平台参考) |

## Cross-Source Patterns

| Pattern | Sources | NeoTrix Integration |
|---------|---------|---------------------|
| **3-tier Browser Pool** | Crawl4AI + DeerFlow (sandbox tiers) | NT-WORLD 浏览器池架构标准化 |
| **Skill Interface Contract** | visual-explainer + DeerFlow | NT SKILL-SPEC.md (<200 lines) 验证 |
| **Crash Recovery / Checkpoint** | Crawl4AI + DeerFlow | SEAL pipeline + NT-MEMORY 增量存储 |
| **Streaming Router** | HoneyRoute + DeerFlow (GWT salience) | NT-SHIELD 入侵检测 + GWT cost-aware 路由 |
| **Anti-Bot + Security** | Crawl4AI + HoneyRoute + Stirling-PDF | NT-SHIELD 安全纵深防御 |
| **MCP Server Isolation** | visual-explainer + DeerFlow + Crawl4AI | NT-IO MCP server 安全架构 |
