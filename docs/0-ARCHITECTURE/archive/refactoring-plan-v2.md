# NeoTrix 媒体源架构重构 — 完整方案

> **日期**: 2026-09-08
> **版本**: v2.0
> **状态**: 执行中

---

## 一、问题诊断

### 1.1 冗余问题 (5 处)

| 问题 | 位置 | 影响 |
|------|------|------|
| 双缓存系统 | `engine/cache.rs` + `media_cache/` | 维护成本 ×2 |
| 电路断路器 ×3 | `reliability/circuit_breaker.rs` + `circuit_breaker_unified.rs` + `evolution/self_healing.rs` | 逻辑分散 |
| 健康检查 ×2 | `engine/health.rs` + `deployment/health_check.rs` | 职责不清 |
| 可观测性 ×2 | `observability/` + `arch/cross_cutting.rs` | 入口混乱 |
| 合规框架 ×2 | `governance/` + `compliance/` | 规则分散 |

### 1.2 扁平化问题 (40 目录)

```
问题: 40 个顶级目录，职责粒度不一致
- 有的目录只有 2-3 个文件 (book, document, tests)
- 有的目录有 15 个文件 (engine)
- 相似职责分散在多个目录 (ecosystem/evolution, ml/ai, edge/serverless/cloud)
```

### 1.3 跨域错位 (4 处)

| 模块 | 当前位置 | 正确位置 | 理由 |
|------|----------|----------|------|
| DataClassifier | governance/ | security/ | 数据分类是安全职责 |
| GdprCompliance | governance/ | security/ | GDPR 是安全合规 |
| ApiCostTracker | cost/ | analytics/ | 成本追踪是分析职责 |
| HealthCheckEndpoint | deployment/ | reliability/ | 健康检查是可靠性职责 |

---

## 二、重构方案

### 2.1 冗余清理

#### A. 缓存统一
```
现状:
  engine/cache.rs (响应缓存)
  media_cache/ (多级缓存)

方案:
  保留 media_cache/ 作为唯一缓存入口
  engine/cache.rs 标记 #[deprecated] 指向 media_cache
```

#### B. 电路断路器统一
```
现状:
  reliability/circuit_breaker.rs (基础版)
  reliability/circuit_breaker_unified.rs (统一版)
  evolution/self_healing.rs (自愈逻辑)

方案:
  保留 circuit_breaker_unified.rs 作为唯一实现
  其他两个标记 #[deprecated]
```

#### C. 健康检查统一
```
现状:
  engine/health.rs (引擎健康)
  deployment/health_check.rs (部署健康)

方案:
  保留 reliability/health_check.rs 作为唯一入口
  其他两个标记 #[deprecated]
```

#### D. 可观测性统一
```
现状:
  observability/ (7 个文件)
  arch/cross_cutting.rs (横切关注点)

方案:
  保留 observability/ 作为唯一实现
  arch/cross_cutting.rs 只定义 trait，不实现
```

#### E. 合规统一
```
现状:
  governance/ (4 个文件: classification, gdpr, audit_trail, retention)
  compliance/ (4 个文件: soc2, hipaa, gdpr_enhanced, audit_report)

方案:
  合并到 governance/
  compliance/ 标记 #[deprecated]
```

### 2.2 扁平化合并

| 原目录 | → | 新目录 | 文件数 |
|--------|---|--------|--------|
| ecosystem/ | → | evolution/ | 5 → 10 |
| ml/ | → | ai/ml/ | 4 → 12 |
| edge/ | → | deployment/ | 5 → 13 |
| serverless/ | → | deployment/ | 4 → 13 |
| cloud/ | → | deployment/ | 4 → 13 |
| compliance/ | → | governance/ | 5 → 9 |
| simulation/ | → | testing/ | 4 → 6 |

**重构后目录数**: 40 → 33

### 2.3 跨域对齐

| 模块 | 原位置 | → | 新位置 |
|------|--------|---|--------|
| classification.rs | governance/ | → | security/ |
| gdpr.rs | governance/ | → | security/ |
| api_tracking.rs | cost/ | → | analytics/ |
| health_check.rs | deployment/ | → | reliability/ |

---

## 三、执行计划

### Phase 1: 冗余清理
- [ ] 标记 engine/cache.rs 为 deprecated
- [ ] 标记 reliability/circuit_breaker.rs 为 deprecated
- [ ] 标记 evolution/self_healing.rs 为 deprecated
- [ ] 标记 deployment/health_check.rs 为 deprecated
- [ ] 更新 governance/mod.rs 移除 classification/gdpr

### Phase 2: 扁平化合并
- [ ] 移动 ecosystem/ 内容到 evolution/
- [ ] 移动 compliance/ 内容到 governance/
- [ ] 更新所有 mod.rs 文件

### Phase 3: 跨域对齐
- [ ] 移动 governance/classification.rs → security/
- [ ] 移动 governance/gdpr.rs → security/
- [ ] 移动 cost/api_tracking.rs → analytics/
- [ ] 移动 deployment/health_check.rs → reliability/

### Phase 4: 验证
- [ ] 编译检查
- [ ] 依赖图验证
- [ ] 边界规则检查

---

## 四、目标架构

### 4.1 目录结构 (33 个)

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
│   ├── video/                视频源 (6)
│   ├── image/                图片源 (4)
│   ├── document/             文档源 (2)
│   ├── book/                 图书源 (2)
│   ├── lyrics/               歌词源 (6)
│   ├── social/               社交源 (7)
│   └── feed/                 Feed 聚合 (9)
│
├── 智能层 (Intelligence)
│   ├── ai/                   AI + ML (12)
│   ├── graph/                知识图谱 (3)
│   └── plugin/               插件系统 (3)
│
├── 基础设施层 (Infrastructure)
│   ├── reliability/          可靠性 (7)
│   ├── security/             安全 (10)
│   ├── observability/        可观测性 (7)
│   ├── media_cache/          缓存 (4)
│   ├── pipeline/             数据管道 (3)
│   └── performance/          性能优化 (4)
│
├── 集成层 (Integration)
│   ├── integration/          跨模块集成 (3)
│   ├── realtime/             实时能力 (3)
│   ├── api_gateway/          API 网关 (4)
│   └── workflow/             工作流 (4)
│
├── 运营层 (Operations)
│   ├── deployment/           部署 (13)
│   ├── governance/           治理 (9)
│   ├── analytics/            分析 (7)
│   ├── multitenancy/         多租户 (3)
│   ├── i18n/                 国际化 (3)
│   └── disaster_recovery/    灾难恢复 (3)
│
├── 测试层 (Testing)
│   ├── testing/              测试 (6)
│   ├── tests/                集成测试 (4)
│   └── arch/                 架构定义 (5)
│
└── 进化层 (Evolution)
    └── evolution/            自进化 (10)
```

### 4.2 依赖关系

```
核心层 ← 内容层 ← 智能层
  ↓
基础设施层 ← 集成层
  ↓
运营层 ← 测试层
  ↓
进化层 (跨层)
```

### 4.3 边界规则

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

---

## 五、验证清单

### 5.1 编译检查
- [ ] `cargo check -p neotrix --lib` 零错误
- [ ] 所有 deprecated 模块有正确注释
- [ ] 所有 re-export 路径正确

### 5.2 依赖图验证
- [ ] 无循环依赖
- [ ] 层依赖符合规则
- [ ] 无跨层非法访问

### 5.3 边界规则验证
- [ ] 核心层无外部依赖
- [ ] 内容层只依赖核心层
- [ ] 基础设施层只依赖核心层
- [ ] 运营层只依赖核心层和基础设施层
