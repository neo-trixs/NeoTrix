# 知识注入 + 缺失功能分析 + 实施方案

> 生成日期: 2026-05-14 · 分析源: 11 个外部仓库深度分析报告
> 仓库: ~/repo-analyses/multi-repo-analysis-20260514/ANALYSIS_REPORT.md

---

## Part 1: KnowledgeSource 注入

### 1.1 新增 KnowledgeSource 枚举变体

当前 `KnowledgeSource` 只有 6 个 UI 设计来源。新增以下 8 个变体覆盖新域：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeSource {
    // --- 现有 UI 设计来源 (保留) ---
    HeroUI,
    BaseUI,
    ArcUI,
    CortexUI,
    AgenticDS,
    DesignPhilosophy,

    // --- 新增: 视频/媒体 ---
    Hyperframes,        // heygen-com/hyperframes — HTML→Video

    // --- 新增: 安全 ---
    Betterleaks,        // betterleaks/betterleaks — 密钥扫描
    YaoWebsecurity,     // yaojingang/yao-open-skills — 安全审查 AI Skill

    // --- 新增: 爬虫 ---
    Botasaurus,         // omkarcloud/botasaurus — 爬虫框架

    // --- 新增: 前端质量 ---
    ReactDoctor,        // millionco/react-doctor — React 代码健康

    // --- 新增: 设计工具 ---
    OpenPencil,         // ZSeven-W/openpencil — AI 原生设计工具

    // --- 新增: 交易 ---
    AiTrader,           // HKUDS/AI-Trader — Agent 原生交易

    // --- 新增: 机器人 (可选, 视硬件相关需求) ---
    SesameRobot,        // dorianborian/sesame-robot — ESP32 四足机器人
}
```

### 1.2 能力向量映射

由于现有 23 维偏向 UI 设计，新增来源主要使用 `extension`（扩展维度）：

| 来源 | 利用的现有维度 (0-22) | 新增 extension 维度 |
|------|----------------------|-------------------|
| Hyperframes | inference_depth:0.85, creativity:0.7, ai_native_states:0.9 | video_rendering:0.95, html_composition:0.9, deterministic_capture:0.85, frame_adapter:0.8 |
| Betterleaks | analysis:0.9, verification:0.95 | secret_detection:0.95, cel_filtering:0.9, bpe_entropy:0.8, scan_parallelism:0.85 |
| YaoWebsecurity | analysis:0.85, domain_specificity:0.9 | security_audit:0.95, vulnerability_knowledge:0.9, report_generation:0.85, review_workflow:0.9 |
| Botasaurus | compound_composition:0.7, verification:0.7 | anti_detection:0.95, web_scraping:0.9, human_mouse:0.85, ui_builder:0.8, desktop_extractor:0.7 |
| ReactDoctor | analysis:0.85, quality_gates:0.9, verification:0.9 | react_lint:0.95, health_scoring:0.9, agent_skill_integration:0.85, diff_scanning:0.8 |
| OpenPencil | creativity:0.9, ai_native_states:0.95, semantic_layer:0.9 | vector_design:0.95, mcp_design_tools:0.9, concurrent_agent_teams:0.85, design_as_code:0.9, canvas_engine:0.85 |
| AiTrader | domain_specificity:0.8, inference_depth:0.7 | agent_trading:0.95, signal_sync:0.85, copy_trading:0.8, market_data:0.85, reward_system:0.7 |
| SesameRobot | creativity:0.6, experimental:0.7 | esp32_firmware:0.9, quadruped_kinematics:0.85, oled_expression:0.8, servo_control:0.85 |

### 1.3 实现: knowledge_source.rs

新 `capability_vector()` 返回融合现有维度 + extension 的完整向量：

```rust
KnowledgeSource::Hyperframes => {
    let mut cv = CapabilityVector::from_values(
        0.3, 0.3, 0.5, 0.3,  // typography/grid/color/whitespace (low - not design-focused)
        0.2, 0.6, 0.2, 0.5,  // data_viz/emotion/minimalism/experimental
        0.85, 0.7, 0.8, 0.7, 0.6, // inference/creativity/analysis/synthesis/domain
        0.3, 0.3, 0.2,  // accessibility/compound/tailwind (low)
        0.2, 0.9, 0.3,  // react_aria/ai_native/bem
        0.4, 0.5, 0.6,  // figma/quality_gates/verification
    );
    cv.extend_named(vec![
        ("video_rendering".into(), 0.95),
        ("html_composition".into(), 0.9),
        ("deterministic_capture".into(), 0.85),
        ("frame_adapter".into(), 0.8),
    ]);
    cv
}
```

### 1.4 ReasoningBank 种子知识

在 `memory.rs` 的 `initialize_with_design_knowledge()` 旁新增 `initialize_with_repo_analysis_knowledge()`:

```
- Hyperframes: "HTML-native video rendering framework", task_type=CodeGeneration, tier=Semantic
- Betterleaks: "CEL-based secret scanning engine", task_type=Security, tier=Semantic
- Botasaurus: "Anti-detection web scraping framework", task_type=CodeGeneration, tier=Semantic
- ReactDoctor: "React code health scoring tool", task_type=CodeReview, tier=Semantic
- OpenPencil: "AI-native vector design tool with MCP", task_type=Design, tier=Semantic
- AiTrader: "Agent-native trading platform", task_type=Research, tier=Semantic
- YaoWebsecurity: "Security audit AI skill with 275 checks", task_type=Security, tier=Semantic
- SesameRobot: "ESP32 quadruped robot platform", task_type=Learning, tier=Episodic
```

---

## Part 2: 缺失功能分析

### 2.1 功能对比矩阵

| 功能域 | NeoTrix 现状 | 参考仓库 | 差距等级 |
|--------|-------------|---------|---------|
| **视频渲染** | ❌ 无 | hyperframes | 🟡 P1 |
| **密钥扫描** | ⚠️ security 模块已存在但无 CEL/secret 扫描 | betterleaks | 🟢 P2 |
| **安全审查工作流** | ❌ 无结构化审查流程 | yao-websecurity-skill | 🟢 P2 |
| **Web 爬虫** | ⚠️ browser_automation 有但无框架 | botasaurus | 🟡 P1 |
| **React 代码质量** | ⚠️ code_review.rs 已存在但无 React 专项 | react-doctor | 🔴 P0 |
| **AI 设计工具** | ❌ 知识注入即可，不需完整设计引擎 | openpencil | 🟠 P3 |
| **Agent 交易** | ❌ 域特定, 非通用能力 | AI-Trader | 🟠 P3 |
| **邮件聚合** | ❌ 域特定 | cypht | ⛔ |
| **机器人控制** | ⚠️ 可知识注入但无硬件 | sesame-robot/jie_3d_nav | 🟠 P3 |
| **图标生成** | ❌ 小工具 | MoBrowser-Icon-Maker | 🟢 P2 |

### 2.2 P0 级缺失（必须填补）

#### Gap-1: React 代码质量专项检测

**现状**: `code_review.rs` 有通用 CodeReviewEngine，但维度是通用代码审查（6类检测：correctness/performance/security/style/maintainability/documentation），没有 React 专项规则。

**差距细节**:
- react-doctor 有 `~50` 条 React-specific lint rules
- 6 维度分类：State & Effects / Performance / Architecture / Security / Accessibility / Dead Code
- 健康分公式: `100 - (unique_error_rules × 1.5) - (unique_warning_rules × 0.75)`
- Agent skill 集成：50+ 编码 agent 自动适配
- GitHub Action + CLI + Node.js API

#### Gap-2: 爬虫框架集成

**现状**: `browser_automation/` 模块存在但无完整的 anti-detection 爬虫框架。MCP tools 有 playwright_verify 但只是验证工具。

**差距细节**:
- botasaurus 的 3 个装饰器模式 (`@browser`, `@request`, `@task`)
- Anti-detection: Cloudflare/Datadome/Fingerprint bypass
- UI builder: 自动生成 Web 界面、API、数据表格
- Desktop extractor: 一键打包桌面应用

#### Gap-3: HTML→Video 渲染

**现状**: 完全缺失。NeoTrix 目前只能输出文本/代码，无法生成视频内容。

**差距细节**:
- hyperframes 的 Frame Adapter 模式 (GSAP/Lottie/CSS/Three.js)
- Deterministic capture engine (Puppeteer + FFmpeg)
- CLI: init → preview → render 管线
- MCP 工具和 Skill 集成

---

## Part 3: 实施方案

### Phase 1: 知识注入 + ReactDoctor 集成 (P0, 1-2 sessions)

#### Task 1: KnowledgeSource 注入 (P0, 1 session)

1. 修改 `core/knowledge_source.rs` — 添加 8 个新枚举变体
2. 在 `capability_vector()` 中实现各来源的向量 + extension
3. 更新 `source_weight()` 为新来源赋值
4. 在 `memory.rs` 添加 `initialize_with_repo_analysis_knowledge()` 种子知识
5. 在 `USER.md` 更新知识来源表
6. `cargo check --lib` 确认零错误

#### Task 2: ReactDoctor 规则引擎集成 (P0, 2 sessions)

1. 在 `code_review.rs` 新增 `ReactDoctorRules` 模块：
   - 移植 react-doctor 的核心规则到 Rust（优先 State & Effects + Performance 类别）
   - 实现健康分公式 `100 - (unique_error_rules × 1.5) - (unique_warning_rules × 0.75)`
2. 在 `src/neotrix/reasoning_brain/` 创建 `react_doctor.rs`：
   - `ReactDoctorEngine` — 对 React 项目执行专项检测
   - 集成到 `CodeReviewLoop`（检测到 React 项目时自动触发）
3. 创建 MCP 工具：`react_doctor_scan` 暴露给外部 agent
4. 15+ 测试覆盖所有规则类别
5. `cargo check --lib` + `cargo test --lib` 全部通过

### Phase 2: 爬虫框架 + WebScraping 管道 (P1, 2-3 sessions)

#### Task 3: Botasaurus 模式吸收 (P1)

1. 在 `src/neotrix/` 创建 `scraper/` 模块：
   - `ScraperConfig` — proxy/profile/headless/extension 配置
   - `BrowserScraper` — 基于 `browser_automation` 的 @browser 等价
   - `RequestScraper` — 轻量 HTTP 爬取 @request 等价
   - `ScraperPipeline` — cache → scrape → parse → save 管线
2. 核心 anti-detection 能力：
   - Google Referrer 绕过 Cloudflare
   - tiny_profile (1KB 跨平台 cookie)
   - 真人鼠标轨迹模拟（复用已有 browser_automation）
3. MCP 工具：`scraper_scrape`, `scraper_config`
4. 集成到 KnowledgeChain 作为外部数据来源

### Phase 3: 视频渲染 + HTML→Video (P1-P2, 2-3 sessions)

#### Task 4: Hyperframes Adapter (P1)

1. 创建 `src/neotrix/media/video.rs`：
   - `VideoComposition` — 定义 HTML 模板 + 资源
   - `HyperframesAdapter` — 调用 hyperframes CLI 渲染
   - Composition template registry（预置 GSAP/Lottie 模板）
2. 集成到 ReasoningBrain 输出管线：
   - 当用户请求"生成视频"时，自动调用 hyperframes 管线
3. MCP 工具：`render_video`, `preview_composition`

### Phase 4: 安全审查 + Secret 扫描 (P2)

#### Task 5: Betterleaks + YaoWebsecurity 能力吸收 (P2)

1. 在 `security/` 模块扩展：
   - CEL-based 规则过滤器（移植 betterleaks 的 CEL filtering 概念）
   - 275 检查项知识库（移植 yao-websecurity-skill 的 vulnerability-ontology.csv）
   - 安全审查报告生成（移植其 4 格式输出）
2. 集成到 CodeReviewLoop：
   - 检测到安全相关任务时自动启用安全审查模式

### Timeline Summary

```
Week 1:  ✅ Phase 1 — KnowledgeSource 注入 + ReactDoctor
           Task 1 (knowledge_source.rs 扩展) — 1 session
           Task 2 (react_doctor.rs 规则引擎) — 2 sessions

Week 2:  🟡 Phase 2 — Botasaurus 爬虫框架吸收
           Task 3 (scraper/ 模块) — 2-3 sessions

Week 3:  🟡 Phase 3 — Hyperframes 视频渲染
           Task 4 (media/video.rs) — 2-3 sessions

Week 4:  🟢 Phase 4 — 安全审查能力
           Task 5 (security 扩展) — 1-2 sessions
```

### 每 Session 检查项

1. `cargo check --lib` 零 error
2. 新增模块全部测试通过
3. `TODO.md` 更新任务状态
4. `USER.md` 更新知识来源和能力表格
5. Session log 写入 `notes/session-logs/YYYY-MM-DD.md`
