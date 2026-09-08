# NeoTrix 媒体源架构重构 — 最终方案

> **日期**: 2026-09-08
> **版本**: v3.0
> **状态**: 执行中

---

## 一、问题诊断

### 1.1 冗余问题 (5 处)

| 问题 | 位置 | 影响 |
|------|------|------|
| AI/ML 重叠 | `ai/` + `ml/` | 维护成本 ×2 |
| 自进化重叠 | `ecosystem/` + `evolution/` | 职责不清 |
| 部署重叠 | `edge/` + `serverless/` + `cloud/` + `deployment/` | 4 处部署 |
| 合规重叠 | `compliance/` + `governance/` | 规则分散 |
| 测试重叠 | `simulation/` + `testing/` + `tests/` | 测试分散 |

### 1.2 扁平化问题 (40 目录)

```
问题: 40 个顶级目录，职责粒度不一致
- 有的目录只有 1 个文件 (cost)
- 有的目录有 12 个文件 (audio)
- 相似职责分散在多个目录
```

### 1.3 跨域错位 (3 处)

| 模块 | 当前位置 | 正确位置 | 理由 |
|------|----------|----------|------|
| DataClassifier | governance/ | security/ | 数据分类是安全 |
| ApiCostTracker | cost/ | analytics/ | 成本追踪是分析 |
| HealthCheckEndpoint | deployment/ | reliability/ | 健康检查是可靠性 |

---

## 二、重构方案

### 2.1 冗余清理

#### A. AI/ML 合并
```
现状:
  ai/ (9 文件)
  ml/ (5 文件)

方案:
  ml/ → ai/ml/ (子模块)
  ai/ 统一入口
```

#### B. 自进化合并
```
现状:
  ecosystem/ (5 文件)
  evolution/ (6 文件)

方案:
  ecosystem/ → evolution/ (子模块)
  evolution/ 统一入口
```

#### C. 部署合并
```
现状:
  edge/ (5 文件)
  serverless/ (4 文件)
  cloud/ (4 文件)
  deployment/ (4 文件)

方案:
  edge/ + serverless/ + cloud/ → deployment/ (子模块)
  deployment/ 统一入口
```

#### D. 合规合并
```
现状:
  compliance/ (5 文件)
  governance/ (5 文件)

方案:
  compliance/ → governance/ (子模块)
  governance/ 统一入口
```

#### E. 测试合并
```
现状:
  simulation/ (4 文件)
  testing/ (4 文件)
  tests/ (4 文件)

方案:
  simulation/ + testing/ → tests/ (子模块)
  tests/ 统一入口
```

### 2.2 扁平化合并

| 原目录 | → | 新目录 | 文件数 |
|--------|---|--------|--------|
| ml/ | → | ai/ml/ | 5 → 14 |
| ecosystem/ | → | evolution/ | 5 → 11 |
| edge/ | → | deployment/ | 5 → 9 |
| serverless/ | → | deployment/ | 4 → 9 |
| cloud/ | → | deployment/ | 4 → 9 |
| compliance/ | → | governance/ | 5 → 10 |
| simulation/ | → | tests/ | 4 → 8 |
| testing/ | → | tests/ | 4 → 8 |

**重构后目录数**: 40 → 32

### 2.3 跨域对齐

| 模块 | 原位置 | → | 新位置 |
|------|--------|---|--------|
| DataClassifier | governance/ | → | security/ |
| GdprCompliance | governance/ | → | security/ |
| ApiCostTracker | cost/ | → | analytics/ |
| HealthCheckEndpoint | deployment/ | → | reliability/ |

---

## 三、目标架构

### 3.1 目录结构 (32 个)

```
nt_world_media_source/
├── 核心层 (Core)
│   ├── types.rs              统一类型
│   ├── engine.rs             媒体引擎
│   ├── api.rs                统一 API
│   ├── resource_store.rs     KB 持久化
│   ├── lx_script.rs          LX 脚本协议
│   ├── playback.rs           播放控制
│   └── now_playing.rs        当前播放
│
├── 内容层 (Content)
│   ├── audio/                音频源 (11)
│   ├── video/                视频源 (3)
│   ├── image/                图片源 (4)
│   ├── document/             文档源 (2)
│   ├── book/                 图书源 (2)
│   ├── lyrics/               歌词源 (3)
│   ├── social/               社交源 (4)
│   └── feed/                 Feed 聚合 (9)
│
├── 智能层 (Intelligence)
│   ├── ai/                   AI + ML (14)
│   └── graph/                知识图谱 (4)
│
├── 基础设施层 (Infrastructure)
│   ├── reliability/          可靠性 (7)
│   ├── security/             安全 (7)
│   ├── observability/        可观测性 (8)
│   ├── media_cache/          缓存 (4)
│   ├── pipeline/             数据管道 (4)
│   └── performance/          性能优化 (5)
│
├── 集成层 (Integration)
│   ├── integration/          跨模块集成 (4)
│   ├── realtime/             实时能力 (4)
│   ├── api_gateway/          API 网关 (4)
│   ├── plugin/               插件系统 (3)
│   └── workflow/             工作流 (5)
│
├── 运营层 (Operations)
│   ├── deployment/           部署 (9)
│   ├── governance/           治理 (10)
│   ├── analytics/            分析 (5)
│   ├── multitenancy/         多租户 (4)
│   ├── i18n/                 国际化 (4)
│   └── disaster_recovery/    灾难恢复 (4)
│
├── 测试层 (Testing)
│   └── tests/                测试 (8)
│
└── 进化层 (Evolution)
    └── evolution/            自进化 (11)
```

### 3.2 依赖关系

```
核心层 ← 内容层 ← 智能层
  ↓
基础设施层 ← 集成层
  ↓
运营层 ← 测试层
  ↓
进化层 (跨层)
```

### 3.3 边界规则

| 层 | 允许依赖 | 禁止依赖 |
|----|----------|----------|
| 核心层 | 无 | 所有 |
| 内容层 | 核心层 | 基础设施层、运营层 |
| 智能层 | 核心层、内容层 | 基础设施层、运营层 |
| 基础设施层 | 核心层 | 内容层、智能层 |
| 集成层 | 核心层、基础设施层 | 内容层、智能层 |
| 运营层 | 核心层、基础设施层 | 内容层、智能层 |
| 测试层 | 所有 | 无 |
| 进化层 | 所有 | 无 |
