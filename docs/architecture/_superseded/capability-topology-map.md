# NeoTrix 全能力多维拓扑地图

> ⛔ **已归档（2026-10-06）—— 与 `all-capability-analysis.md` 是同一组错数的双胞胎。**
>
> 两档同源同错（`2,413 Rust 文件` / `7 crates` / `68 skills` / `65`；
> 实测 `2,865` / `11` / `21` / —），且**均无生成器、无门、无维护触发**。
>
> 保留两份只会让错误数字有两个来源。当前代码拓扑正典：
> `CODE-TOPOLOGY.md`（生成器 `scripts/ops/nt_topology.py`）+ `map-check.sh`。

> 生成日期: 2026-09-20
> 数据源: 2,413 Rust 文件 + 7 crates + 68 skills + 65 前端组件
> 目的: 精准监控 + 统一管理 + 冗余检测 + 缺口识别

---

## 一、能力总览 (Top-Level Stats)

| 维度 | 数量 | 说明 |
|------|------|------|
| Rust 源文件 | 2,413 | neotrix-core/src/ |
| Workspace Crates | 7 | types, sysctl, consciousness, gateway, reasoning, multi-agent, nt-lang |
| 6层模块 | ~280 | L0(31) + L1(32+) + L2(13+) + L3(8+39shield) + L4(4+) + L5(40+) + L6(32+) |
| Skills | 68 | 9 categories, 68 indexed skills |
| 前端组件 | ~65 | SolidJS + Tauri |
| 前端 Store | 8 | chat, canvas, kb, insights, tags, theme, workflow, world |
| Canvas 模块 | 9 | SmartCanvas + node registry + panel renderers |

---

## 二、6层架构拓扑 (Layer Topology)

```
┌─────────────────────────────────────────────────────────────────────┐
│ L6 META (32+ modules)                                              │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│ │coordina- │ │ memory/  │ │ healing/ │ │evolution/│ │ nt_core_ │  │
│ │tion/ 19  │ │ 7 modules│ │ 12 modules│ │ 5 modules│ │capability│  │
│ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ │ 17 modules│ │
│      │            │            │            │        └────┬─────┘  │
│      └────────────┴────────────┴────────────┴────────────┘        │
├─────────────────────────────────────────────────────────────────────┤
│ L5 COGNITION (40+ modules)                                         │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│ │nt_mind/  │ │nt_core/  │ │nt_council│ │nt_goal/  │ │nt_agent/ │  │
│ │100+ sub  │ │20+ sub   │ │delibera- │ │goal_mgmt │ │ode loop  │  │
│ │modules   │ │modules   │ │tion      │ │6 modules │ │          │  │
│ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │
│      └────────────┴────────────┴────────────┴────────────┘        │
├─────────────────────────────────────────────────────────────────────┤
│ L4 EMOTION (4+ modules)                                            │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐                            │
│ │nt_feel/  │ │nt_memory/│ │emotion_  │                            │
│ │10 modules│ │KB+Spatial│ │reasoning │                            │
│ │          │ │88 modules│ │bridge    │                            │
│ └────┬─────┘ └────┬─────┘ └────┬─────┘                            │
│      └────────────┴────────────┘                                   │
├─────────────────────────────────────────────────────────────────────┤
│ L3 EMBODIMENT (8 + 39 shield modules)                              │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐               │
│ │nt_shield/│ │nt_compu- │ │nt_guard_ │ │nt_astar/ │               │
│ │39 modules│ │ter+fleet │ │chain     │ │pathfind  │               │
│ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘               │
│      └────────────┴────────────┴────────────┘                      │
├─────────────────────────────────────────────────────────────────────┤
│ L2 PERCEPTION (13+ modules)                                        │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│ │nt_world/ │ │nt_core_  │ │nt_core_  │ │nt_core_  │ │nt_routing│  │
│ │20+ files │ │e8/ 19    │ │hcube/ 21 │ │sense+    │ │+judgment │  │
│ │+12 intel │ │modules   │ │modules   │ │knowledge │ │          │  │
│ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │
│      └────────────┴────────────┴────────────┴────────────┘        │
├─────────────────────────────────────────────────────────────────────┤
│ L1 ACTION (32+ modules)                                            │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│ │nt_act/   │ │nt_io/    │ │nt_media/ │ │nt_infra/ │ │nt_memory_│  │
│ │20+ sub   │ │30+ sub   │ │13 modules│ │tracing+  │ │spatial   │  │
│ │modules   │ │modules   │ │          │ │breaker+  │ │          │  │
│ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │
│      └────────────┴────────────┴────────────┴────────────┘        │
├─────────────────────────────────────────────────────────────────────┤
│ L0 SUBSTRATE (31 modules)                                          │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│ │nt_core_  │ │nt_core_  │ │nt_core_  │ │nt_core_  │ │nt_core_  │  │
│ │error     │ │platform  │ │traits    │ │di+event  │ │ecs+tick  │  │
│ │22 variant│ │health+   │ │NativeTool│ │bus+cache │ │schedule  │  │
│ │          │ │metrics   │ │Capability│ │          │ │          │  │
│ └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 三、能力域拓扑 (Domain Topology)

### 11 个能力域 × 3 个成熟度层级

| 域 | L0_Primitive | L1_Composite | L2_Orchestrator | 模块数 | 状态 |
|----|-------------|-------------|-----------------|--------|------|
| **NtCore** | error, time, math, hex, traits, di, event, span, telemetry, cache, hot_data, ecs, tick | graph, edit, embed, harness, simulate | platform, schema_watchdog, self_test | 31 | 🟢 |
| **NtMind** | — | reasoning_engine, knowledge_engine, skill_tree, skill_chain | SEAL, self_evolver, reasoning_brain, automation | 100+ | 🟡 |
| **NtMemory** | — | kb_fts5, spatial_memory, memory_asset | knowledge_pipeline, experience_tree, wikiskill | 88+ | 🟡 |
| **NtWorld** | — | crawl, osint, sense, explore, source | unified_search, world_model_v2, JEPA | 20+ | 🟢 |
| **NtAct** | — | orchestrator, trade, code, autonomy, crypto, voice | parallel_task, agent_loop, mcp_protocol | 20+ | 🟢 |
| **NtIo** | — | provider, mcp_bridge, acp, agent_loop, hive | gateway_v2, universal_model, standalone | 30+ | 🟢 |
| **NtShield** | — | circuit_breaker, binary_analyzer, vulnerability_pipeline | web_scanner, pentest_swarm, sandbox, ztnet | 39 | 🟢 |
| **NtPhysical** | astar | computer, fleet | — | 3 | 🟢 |
| **NtFeel** | — | emotion_engine, affective_interface | writing_style, vtuber, salesperson | 10 | 🟡 |
| **NtFileAbility** | xlsx_read, pdf_merge, structured_read, encoding_detect | xlsx_parse, merge_tables, pdf_edit, image_sr, doc_parse, ocr | table_presenter, excel_tool_schema, visual_route | 22 | 🟡 |
| **NtDesign** | — | — | nt_design_visual/ (规划中) | 0→50 | 🔴 |

**图例**: 🟢 成熟 | 🟡 部分成熟 | 🔴 缺失/规划中

---

## 四、依赖拓扑 (Dependency Topology)

### Crate 依赖链 (自底向上)

```
nt-lang (standalone)
    │
neotrix-types ←────── (leaf, 0 crate deps)
    │
neotrix-sysctl ←────── (leaf, 0 crate deps)
    │
neotrix-consciousness ←── types + sysctl
    │
neotrix-gateway ←────── types + sysctl + consciousness
    │
neotrix-reasoning ←─── types + sysctl + consciousness + gateway
    │
neotrix-multi-agent ←─ types + sysctl + consciousness + reasoning + gateway
    │
neotrix-core ←────── (aggregates all above)
```

### 跨层依赖违规检测点

| 检测点 | 规则 | 当前状态 |
|--------|------|---------|
| L0→L1 | L0 不应引用 L1 | ⚠️ nt_core_kb_primitives (L0 re-export from L6) |
| L0→L6 | L0 不应引用 L6 | ⚠️ nt_core_memory_asset (L0 re-export from L6) |
| L1→L2 | L1 不应引用 L2 | ✅ 无违规 |
| L2→L3 | L2 不应引用 L3 | ✅ 无违规 |
| L3→L4 | L3 不应引用 L4 | ✅ 无违规 |
| L4→L5 | L4 不应引用 L5 | ⚠️ nt_emotion_reasoning_bridge (L4→L5) |
| L5→L6 | L5 不应引用 L6 | ✅ 无违规 |

---

## 五、Skill 依赖拓扑 (Skill Dependency Graph)

```
external-absorption (root, 0 deps)
    ↑
architecture-auditor → external-absorption
    ↑
diagnose → architecture-auditor
    ↑
improve-architecture → diagnose

self-iteration-agent → external-absorption

neotrix-types (leaf, 0 deps)
    ↑
neotrix-sysctl → neotrix-types
    ↑
nt-lang → neotrix-types

design-core (0 deps, standalone)
design-audit (0 deps, standalone)
design-identity (0 deps, standalone)
design-patrol (0 deps, standalone)

(all other skills: 0 deps, standalone)
```

### Skill 层级映射

| 层级 | Skills |
|------|--------|
| L0 | config/*, docs/*, crates/* (24 skills) |
| L1 | dev-tools/* (7 skills) |
| L4 | mcp-gateway, self-health, caveman (3 skills) |
| L5 | architecture-auditor/*, root/*, trending/* (22 skills) |
| L6 | design-core, design-audit, design-identity, design-patrol (4 skills) |

---

## 六、前端组件拓扑 (Frontend Component Topology)

### 组件分类矩阵

| 类别 | 组件数 | 关键组件 | 设计能力依赖 |
|------|--------|---------|-------------|
| **Layout & Shell** | 8 | Sidebar, RightBar, TrafficLights | — |
| **Chat & Messaging** | 6 | NeoTrixCore, StreamingText, Markdown, SlashMenu, CommandPalette | — |
| **Agent & Multi-Agent** | 5 | AgentActivityBar, KanbanAgentBoard, OfficeFloor, AutonomyMeter | — |
| **Canvas & Visualization** | 7 | SmartCanvas, CausalMap, KbGraph, GlobeView, FlowerOfLife, SproutIcon | 🟡 部分需要 nt_design_visual |
| **Design & Annotation** | 2 | DesignModePanel, AnnotatedImage | 🔴 需要 nt_design_visual |
| **Code & File** | 8 | CodeBlock, FileTreeView, TerminalPanel, LivePreview | — |
| **Tool & Approval** | 5 | ToolCallCard, ApprovalPanel, PermissionModeSelector | — |
| **UI Controls** | 12 | ModelSwitcher, SettingsModal, ThemeToggle, Icon, neo-icons | 🟡 Icon 需要 nt_design_visual |
| **Data & Visualization** | 6 | CostDashboard, ContextUsageReport, TaskList, CheckpointTimeline | — |
| **Other** | 6 | OnboardingWizard, GenUIView, VoicePanel, PlanMode | — |

---

## 七、监控维度 (Monitoring Dimensions)

### 7.1 健康度监控

| 监控项 | 检测方法 | 阈值 | 当前状态 |
|--------|---------|------|---------|
| 编译状态 | `cargo check` | 0 errors | ⚠️ 待验证 |
| 测试覆盖 | `cargo test` | >80% | ⚠️ 待统计 |
| Clippy 警告 | `cargo clippy` | 0 warnings | ⚠️ 待统计 |
| 依赖审计 | `cargo deny` | 0 vulnerabilities | ⚠️ 待统计 |
| 构建时间 | `cargo build` | <5min | ⚠️ 待测量 |

### 7.2 架构健康度监控

| 监控项 | 检测方法 | 阈值 | 当前状态 |
|--------|---------|------|---------|
| 跨层违规 | PATROL-003 | 0 | ⚠️ 2 个已知 (L0 re-export) |
| 能力孤岛 | PATROL-002 | 0 | ⚠️ 待扫描 |
| 依赖环 | PATROL-001 | 0 | ⚠️ 待扫描 |
| 冗余簇 | PATROL-005 | <3 | ⚠️ 已识别 4 簇 |
| 扁平缺陷 | PATROL-004 | 0 | ⚠️ 待统计 |

### 7.3 设计能力监控

| 监控项 | 检测方法 | 阈值 | 当前状态 |
|--------|---------|------|---------|
| nt_design_visual 实现进度 | 模块计数 | 10/10 | 🔴 0/10 |
| Glass 效果集成 | 前端检查 | 已集成 | 🔴 未开始 |
| 图标库索引 | lib/ 模块 | 5 库 | 🔴 未开始 |
| Token 系统 | construct/token.rs | 已实现 | 🔴 未开始 |
| 设计审计维度 | audit/ 模块 | 11 维度 | 🔴 未开始 |

### 7.4 前端健康度监控

| 监控项 | 检测方法 | 阈值 | 当前状态 |
|--------|---------|------|---------|
| TypeScript 错误 | `tsc --noEmit` | 0 | ⚠️ 待验证 |
| 组件数 | 文件计数 | >60 | ✅ ~65 |
| Bundle 大小 | `vite build` | <2MB | ⚠️ 待测量 |
| 首屏加载 | Lighthouse | <3s | ⚠️ 待测量 |

---

## 八、冗余检测矩阵 (Redundancy Detection)

### 已识别冗余簇

| 簇ID | 涉及模块 | 重叠度 | 处理状态 |
|------|---------|--------|---------|
| R1 | 5 design skills | 70% | ✅ 已合并为 3 |
| R2 | nt_file_ability XLSX 三路径 | 60% | ⚠️ 待统一 |
| R3 | nt_memory_kb 5 种检索范式 | 40% | ⚠️ 待选择器 |
| R4 | Agent-Reach vs neotrix-gateway | 50% | ⚠️ 待桥接 |
| R5 | nt_media/thumbnail vs nt_file_ability/image_sr | 30% | ⚠️ 待合并到 visual |
| R6 | frontend AnnotatedImage vs DesignModePanel | 40% | ⚠️ 待统一 |
| R7 | frontend KbGraph vs CausalMap (SVG graph) | 30% | ⚠️ 保留(不同用途) |

### 待检测冗余 (需 PATROL-005 扫描)

```
# 所有 provides 列表中的重复 tag
grep -r "provides" .neotrix/capability_registry.json | sort | uniq -d
```

---

## 九、缺口矩阵 (Gap Matrix)

| 缺口 | 影响层 | 优先级 | 预估工时 | 目标模块 |
|------|--------|--------|---------|---------|
| nt_design_visual 未实现 | L1 | **P0** | 52h | nt_design_visual/ |
| 色彩调色板生成 | L1 | **P0** | 4h | visual/construct |
| 设计 Token 管线 | L1 | **P0** | 6h | visual/pipeline |
| 图标生成管线 | L1 | **P0** | 8h | visual/generate |
| 字体系统 | L1 | P1 | 3h | visual/construct |
| 布局/间距系统 | L1 | P1 | 3h | visual/construct |
| 截图→代码 | L1 | P1 | 10h | visual/pipeline |
| 动效 Token | L1 | P2 | 4h | visual/animate |
| 视觉回归测试 | L1 | P3 | 6h | visual/validate |

---

## 十、精准监控仪表盘 (Dashboard Schema)

```json
{
  "project": "neotrix",
  "version": "0.21.0",
  "timestamp": "2026-09-20T00:00:00Z",
  "dimensions": {
    "build": {
      "rust_errors": 0,
      "rust_warnings": 0,
      "ts_errors": 0,
      "clippy_warnings": 0,
      "deny_vulnerabilities": 0,
      "build_time_seconds": 0,
      "test_pass_rate": 0.0
    },
    "architecture": {
      "total_modules": 280,
      "cross_layer_violations": 2,
      "capability_islands": 0,
      "dependency_cycles": 0,
      "redundancy_clusters": 4,
      "flat_defects": 0,
      "layer_distribution": {
        "L0": 31, "L1": 32, "L2": 13, "L3": 47,
        "L4": 4, "L5": 40, "L6": 32
      }
    },
    "design": {
      "nt_design_visual_modules": 0,
      "nt_design_visual_target": 10,
      "design_skills": 3,
      "glass_integrated": false,
      "icon_libs_indexed": 0,
      "token_system": false,
      "audit_dimensions": 0
    },
    "frontend": {
      "components": 65,
      "stores": 8,
      "canvas_modules": 9,
      "bundle_size_kb": 0,
      "first_paint_ms": 0
    },
    "skills": {
      "total": 68,
      "categories": 9,
      "with_dependencies": 4,
      "orphan_skills": 0
    },
    "crates": {
      "total": 7,
      "dependency_depth": 5,
      "leaf_crates": 2,
      "unsafe_crates": 1
    }
  },
  "health_score": {
    "build": 0,
    "architecture": 0,
    "design": 0,
    "frontend": 0,
    "overall": 0
  }
}
```

---

## 十一、行动路线图 (Action Roadmap)

### Phase 1: 基础监控 (Week 1)
- [ ] 运行 `cargo check` 获取编译状态基线
- [ ] 运行 `cargo test` 获取测试覆盖率基线
- [ ] 运行 `cargo clippy` 获取警告基线
- [ ] 运行 `cargo deny` 获取安全基线
- [ ] 生成初始监控仪表盘 JSON

### Phase 2: 冗余清理 (Week 1-2)
- [ ] 统一 XLSX 读取入口 (R2)
- [ ] 聚合 PDF 子能力 (R3)
- [ ] 合并 thumbnail + image_sr 到 visual (R5)

### Phase 3: 跨域修复 (Week 2-3)
- [ ] 修复 L0 re-export 违规 (L0→L6)
- [ ] 桥接 L4→L5 emotion_reasoning_bridge
- [ ] 创建 VisualBackend trait 桥接 10 个模块

### Phase 4: nt_design_visual 实现 (Week 3-6)
- [ ] 实现 core/ (类型定义)
- [ ] 实现 construct/ (矢量构建 + 色彩 + 字体)
- [ ] 实现 generate/ (AI 生成 + 图标管线)
- [ ] 实现 transform/ (变换 + 超分 + PDF)
- [ ] 实现 animate/ (动画 + 动效 Token)
- [ ] 实现 validate/ (验证 + 质量)
- [ ] 实现 export/ (多平台导出)
- [ ] 实现 lib/ (图标库管理)
- [ ] 实现 diagram/ (图表/架构图)
- [ ] 实现 pipeline/ (统一管线)

### Phase 5: 自动巡检 (持续)
- [ ] 每次 commit: PATROL-001 + PATROL-003
- [ ] 每日: PATROL-002 + PATROL-005
- [ ] 每周: PATROL-004 + PATROL-006 + 全量仪表盘更新
