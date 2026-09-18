# NeoTrix 完整迁移路线图 — 旧架构 → 晶体辐射架构

> **日期**: 2026-09-18
> **基于**: 项目结构分析 + 互联网标准调研 + 代码库深度扫描
> **目标**: 从 L0-L6 层级架构迁移到 Crystal-Radiant 辐射架构

---

## 一、项目现状诊断

### 1.1 目录结构统计

| 指标 | 数值 | 问题 |
|------|------|------|
| 总 .rs 文件 | **2047** | 1843 (L0-L6) + 204 (其他) |
| 总 .md 文件 | **2903** | 666 迭代批次 + 2237 其他 |
| Workspace Crates | **10** | 需要统一到 `crates/` 目录 |
| 根目录文件 | **~100+** | 过于杂乱，需要清理 |

### 1.2 当前目录结构问题

```
问题 1: 迭代批次文件泛滥 (666 个)
./iteration_batch_324.md ~ ./iteration_batch_890.md
→ 建议: 归档到 docs/iterations/

问题 2: Workspace Crates 分散
./neotrix-core/          # 主 crate
./crates/neotrix-types/  # 类型定义
./crates/neotrix-sysctl/ # 系统控制
./guard_core/            # 守护核心
./neotrix-sim/           # 模拟器
./nt-world-sim/          # 世界模拟
./src-tauri/             # 桌面应用
./crates/nt-lang/        # 语言
→ 建议: 统一到 crates/ 目录

问题 3: 旧架构层级混乱
l0_substrate/            # 15 文件
l1_action/               # 562 文件
l2_perception/           # 289 文件
l3_embodiment/           # 201 文件
l4_emotion/              # 11 文件
l5_cognition/            # 621 文件
l6_meta/                 # 144 文件
→ 建议: 迁移到辐射臂结构

问题 4: 文档散乱
./docs/ + 根目录 ~100 个 .md 文件
→ 建议: 统一到 docs/ 并分类
```

### 1.3 代码库健康度

| 指标 | 当前值 | 目标值 | 差距 |
|------|--------|--------|------|
| 循环依赖 | 31+ | 0 | 🔴 |
| dead_code 注解 | 87 | <30 | 🟡 |
| TaskType 重复 | 9 | 1 | 🟡 |
| TaskStatus 重复 | 13 | 1 | 🟡 |
| 路由函数 | 49 | <20 | 🟡 |

---

## 二、互联网标准调研结果

### 2.1 Rust Workspace 最佳实践 (2026)

| 原则 | 说明 | NeoTrix 状态 |
|------|------|-------------|
| **单一 workspace** | 所有 crate 在一个 Cargo.toml 下 | ✅ 已实现 |
| **crates/ 目录** | 所有库 crate 放在 `crates/` 下 | ❌ 分散 |
| **Clean Architecture** | domain → application → infrastructure → api | ❌ 使用 L0-L6 |
| **依赖单向** | api → domain; infrastructure → domain | ❌ 有循环依赖 |
| **共享版本** | workspace.dependencies 统一管理 | ✅ 已实现 |
| **Feature Flags** | 可选功能用 feature gates | 🟡 部分实现 |

### 2.2 Clean Architecture 标准结构

```
recommended/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── domain/             # 领域逻辑 (纯 Rust，无外部依赖)
│   │   ├── entities/
│   │   ├── value_objects/
│   │   └── services/
│   ├── application/        # 用例编排
│   │   ├── commands/
│   │   ├── queries/
│   │   └── handlers/
│   ├── infrastructure/     # 外部实现
│   │   ├── persistence/
│   │   ├── external/
│   │   └── config/
│   ├── presentation/       # 用户界面
│   │   ├── api/
│   │   ├── cli/
│   │   └── web/
│   └── shared/             # 共享类型
│       ├── types/
│       └── errors/
└── docs/                   # 文档
```

### 2.3 Crystal-Radiant 架构映射

| Clean Architecture | Crystal-Radiant | 说明 |
|-------------------|-----------------|------|
| domain | Crystal Core | 意识本体 |
| application | CTM Engine | 10步循环编排 |
| infrastructure | 8 Radiant Arms | 能力域封装 |
| presentation | NT-IO | 用户界面 |
| shared | Crystal Types | 共享类型 |

---

## 三、目录重构方案

### 3.1 新目录结构

```
neotrix/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── nt-types/                 # 共享类型 (从 neotrix-types 重命名)
│   ├── nt-crystal/               # 晶体核心 (从 nt_crystal_core 提取)
│   │   ├── src/
│   │   │   ├── identity/         # L1 身份层
│   │   │   ├── knowledge/        # L2 知识层
│   │   │   ├── experience/       # L3 经验层
│   │   │   ├── evolution/        # L4 进化层
│   │   │   ├── ctm/              # CTM 通信
│   │   │   ├── cocoons/          # 持久记忆
│   │   │   └── modules/          # 辐射臂接口
│   │   └── Cargo.toml
│   ├── nt-shield/                # 辐射臂: 安全/保护
│   ├── nt-world/                 # 辐射臂: 世界感知
│   ├── nt-memory/                # 辐射臂: 知识存储
│   ├── nt-act/                   # 辐射臂: 工具调用
│   ├── nt-feel/                  # 辐射臂: 情感计算
│   ├── nt-meta/                  # 辐射臂: 元认知/自愈
│   ├── nt-mind/                  # 辐射臂: 推理/进化
│   ├── nt-io/                    # 辐射臂: 用户界面
│   └── nt-sysctl/                # 系统控制
├── apps/
│   ├── src-tauri/                # 桌面应用
│   └── neotrix-cli/              # CLI 工具
├── tools/                        # 内部工具
├── docs/                         # 文档 (统一)
│   ├── architecture/             # 架构设计
│   ├── guides/                   # 使用指南
│   ├── reference/                # API 参考
│   └── iterations/               # 迭代记录 (归档)
├── archive/                      # 旧代码归档
│   ├── l0-l6/                    # 旧层级架构
│   └── iterations/               # 旧迭代批次
└── tests/                        # 集成测试
```

### 3.2 迁移映射表

| 旧位置 | 新位置 | 文件数 | 优先级 |
|--------|--------|--------|--------|
| `crates/neotrix-types/` | `crates/nt-types/` | 50 | P0 |
| `neotrix-core/src/neotrix/nt_crystal_core/` | `crates/nt-crystal/` | 28 | P0 |
| `l3_embodiment/nt_shield/` | `crates/nt-shield/` | 201 | P1 |
| `l2_perception/nt_world/` | `crates/nt-world/` | 289 | P1 |
| `l1_action/nt_memory/` | `crates/nt-memory/` | 120 | P1 |
| `l1_action/nt_act/` | `crates/nt-act/` | 200 | P1 |
| `l4_emotion/nt_feel/` | `crates/nt-feel/` | 11 | P1 |
| `l6_meta/` (部分) | `crates/nt-meta/` | 90 | P1 |
| `l5_cognition/nt_mind/` | `crates/nt-mind/` | 400 | P2 |
| `l1_action/nt_io/` | `crates/nt-io/` | 300 | P2 |
| `iteration_batch_*.md` | `docs/iterations/` | 666 | P0 |
| 根目录散乱 .md | `docs/` | ~50 | P0 |

---

## 四、文档归档方案

### 4.1 文档分类

| 类别 | 数量 | 位置 | 说明 |
|------|------|------|------|
| **架构设计** | ~20 | `docs/architecture/` | ARCHITECTURE.md, DESIGN.md 等 |
| **迁移计划** | ~10 | `docs/plans/` | MIGRATION-ANALYSIS.md 等 |
| **API 参考** | ~5 | `docs/reference/` | 接口定义 |
| **使用指南** | ~10 | `docs/guides/` | 教程、how-to |
| **审计报告** | ~5 | `docs/audits/` | ARCHITECTURE_AUDIT.md 等 |
| **迭代记录** | 666 | `docs/iterations/` | iteration_batch_*.md |
| **研究报告** | ~30 | `research/` | 吸收的外部研究 |
| **ADR** | ~5 | `docs/adr/` | 架构决策记录 |

### 4.2 根目录清理

**保留** (核心文件):
- `Cargo.toml`, `Cargo.lock`
- `README.md`, `LICENSE`, `CHANGELOG.md`
- `AGENTS.md`, `CONTEXT.md`, `dev-rules.md`
- `DESIGN.md`, `ARCHITECTURE.md`

**归档** (移到 docs/ 或 archive/):
- `iteration_batch_*.md` (666 个)
- `DEFINITIVE_FUSION_PLAN.md`
- `ARCHITECTURE_FUSION_V3.md`
- `UNIVERSAL_ARCHITECTURE_*.md`
- 其他非核心 .md 文件

---

## 五、迁移实施路线图

### Phase 0: 目录清理 + 归档 (2 天)

| 任务 | 说明 | 文件数 |
|------|------|--------|
| T0.1 | 归档 666 个 iteration_batch 文件 | 666 |
| T0.2 | 归档根目录散乱 .md 文件 | ~50 |
| T0.3 | 统一 docs/ 目录结构 | - |
| T0.4 | 创建 archive/l0-l6/ 目录 | - |
| **小计** | | **~716** |

### Phase 1: Workspace 统一 (3 天)

| 任务 | 说明 | 文件数 |
|------|------|--------|
| T1.1 | 重命名 neotrix-types → nt-types | 50 |
| T1.2 | 创建 crates/nt-crystal/ | 28 |
| T1.3 | 创建 crates/nt-shield/ | 201 |
| T1.4 | 创建 crates/nt-world/ | 289 |
| T1.5 | 创建 crates/nt-memory/ | 120 |
| T1.6 | 创建 crates/nt-act/ | 200 |
| T1.7 | 创建 crates/nt-feel/ | 11 |
| T1.8 | 创建 crates/nt-meta/ | 90 |
| **小计** | | **~989** |

### Phase 2: 高难度迁移 (10 天)

| 任务 | 说明 | 文件数 |
|------|------|--------|
| T2.1 | 创建 crates/nt-mind/ (推理/进化) | 400 |
| T2.2 | 创建 crates/nt-io/ (CLI/Tauri) | 300 |
| T2.3 | 更新 workspace Cargo.toml | - |
| T2.4 | 更新所有依赖路径 | - |
| T2.5 | 编译验证 | - |
| **小计** | | **~700** |

### Phase 3: 代码清理 (3 天)

| 任务 | 说明 | 文件数 |
|------|------|--------|
| T3.1 | TaskType 统一 (9→1) | 8 |
| T3.2 | TaskStatus 统一 (13→1) | 12 |
| T3.3 | dead_code 清理 (87→<30) | 57 |
| T3.4 | 循环依赖修复 (31→0) | 31 |
| **小计** | | **~108** |

### Phase 4: CTM 通信完善 (5 天)

| 任务 | 说明 | 文件数 |
|------|------|--------|
| T4.1 | 完善 CTM 10步循环 | 50 |
| T4.2 | 实现辐射臂隔离验证 | 20 |
| T4.3 | 实现 Governance 层 | 30 |
| **小计** | | **~100** |

---

## 六、总工期估算

| Phase | 内容 | 工期 | 文件数 |
|-------|------|------|--------|
| Phase 0 | 目录清理 + 归档 | 2 天 | 716 |
| Phase 1 | Workspace 统一 | 3 天 | 989 |
| Phase 2 | 高难度迁移 | 10 天 | 700 |
| Phase 3 | 代码清理 | 3 天 | 108 |
| Phase 4 | CTM 通信完善 | 5 天 | 100 |
| **总计** | | **23 天** | **~2613** |

---

## 七、风险控制

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 编译失败 | 高 | 高 | 每步 cargo check |
| 依赖路径错误 | 中 | 中 | 批量 sed + 验证 |
| 功能退化 | 中 | 高 | 保留旧代码归档 |
| 工期超支 | 中 | 中 | 分阶段交付 |

---

## 八、验证命令

```bash
# 目录清理验证
find . -name "iteration_batch_*.md" | wc -l  # 目标: 0
find . -maxdepth 1 -name "*.md" | wc -l      # 目标: <10

# Workspace 验证
cargo check --workspace                       # 目标: 0 error
cargo test --workspace                        # 目标: pass

# 代码质量验证
grep -r "enum TaskType" --include="*.rs" | wc -l  # 目标: 1
grep -r "enum TaskStatus" --include="*.rs" | wc -l # 目标: 1
grep -r "#\[allow(dead_code)\]" --include="*.rs" | wc -l  # 目标: <30
```
