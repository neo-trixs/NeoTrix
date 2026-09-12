# NeoTrix 媒体源架构重构计划

> **日期**: 2026-09-08
> **目标**: 冗余清理 + 扁平化 + 跨域对齐

---

## 一、当前状态分析

### 目录统计 (40 个)

| 目录 | 文件数 | 问题 |
|------|--------|------|
| audio/ | 11 | ✅ 正常 |
| video/ | 6 | ✅ 正常 |
| image/ | 4 | ✅ 正常 |
| document/ | 2 | ✅ 正常 |
| book/ | 2 | ✅ 正常 |
| lyrics/ | 6 | ✅ 正常 |
| social/ | 7 | ✅ 正常 |
| feed/ | 9 | ✅ 正常 |
| engine/ | 15 | ⚠️ 过大 |
| ai/ | 6 | ✅ 正常 |
| plugin/ | 3 | ✅ 正常 |
| graph/ | 3 | ✅ 正常 |
| cli/ | 2 | ✅ 正常 |
| integration/ | 3 | ✅ 正常 |
| realtime/ | 3 | ✅ 正常 |
| pipeline/ | 3 | ✅ 正常 |
| media_cache/ | 3 | ⚠️ 与 engine/cache.rs 重复 |
| observability/ | 7 | ✅ 正常 |
| api_gateway/ | 3 | ✅ 正常 |
| security/ | 6 | ⚠️ 与 governance 重叠 |
| tests/ | 3 | ✅ 正常 |
| deployment/ | 3 | ⚠️ health_check 应在 reliability |
| reliability/ | 4 | ⚠️ 与 self_healing 重叠 |
| governance/ | 4 | ⚠️ 与 security 重叠 |
| cost/ | 3 | ⚠️ 应在 analytics |
| multitenancy/ | 3 | ✅ 正常 |
| i18n/ | 3 | ✅ 正常 |
| performance/ | 4 | ✅ 正常 |
| disaster_recovery/ | 3 | ✅ 正常 |
| analytics/ | 4 | ⚠️ 缺 cost |
| ecosystem/ | 4 | ⚠️ 与 evolution 重叠 |
| evolution/ | 4 | ⚠️ 与 ecosystem 重叠 |
| ml/ | 4 | ⚠️ 与 ai 重叠 |
| edge/ | 4 | ⚠️ 与 deployment 重叠 |
| serverless/ | 3 | ⚠️ 与 deployment 重叠 |
| cloud/ | 3 | ⚠️ 与 deployment 重叠 |
| compliance/ | 4 | ⚠️ 与 governance 重叠 |
| workflow/ | 4 | ✅ 正常 |
| simulation/ | 3 | ⚠️ 与 testing 重叠 |
| testing/ | 3 | ⚠️ 与 simulation 重叠 |

---

## 二、冗余问题

### 2.1 电路断路器 (3 处)
```
reliability/circuit_breaker.rs     ← 主实现
reliability/circuit_breaker_unified.rs ← 统一版本
evolution/self_healing.rs          ← 自愈逻辑
engine/health.rs                   ← 健康检查
```
**方案**: 保留 `circuit_breaker_unified.rs`，其他标记 `#[deprecated]`

### 2.2 缓存 (2 处)
```
media_cache/                       ← 多级缓存
engine/cache.rs                    ← 响应缓存
```
**方案**: 合并到 `media_cache/`

### 2.3 可观测性 (2 处)
```
observability/                     ← 7 个文件
arch/cross_cutting.rs              ← 横切关注点
```
**方案**: 保留 `observability/`，`arch/cross_cutting.rs` 只定义 trait

### 2.4 合规/治理 (3 处)
```
governance/                        ← 数据治理
compliance/                        ← 合规框架
security/                          ← 安全
```
**方案**: 合并到 `governance/`

---

## 三、扁平化问题

### 3.1 目录合并

| 原目录 | → | 新目录 | 理由 |
|--------|---|--------|------|
| ecosystem/ | → | evolution/ | 都是自进化 |
| ml/ | → | ai/ | 都是 AI/ML |
| edge/ | → | deployment/ | 都是部署 |
| serverless/ | → | deployment/ | 都是部署 |
| cloud/ | → | deployment/ | 都是部署 |
| compliance/ | → | governance/ | 都是规则 |
| simulation/ | → | testing/ | 都是测试 |

### 3.2 重构后目录 (25 个)

```
nt_world_media_source/
├── audio/              (11) 音频源
├── video/              (6)  视频源
├── image/              (4)  图片源
├── document/           (2)  文档源
├── book/               (2)  图书源
├── lyrics/             (6)  歌词源
├── social/             (7)  社交源
├── feed/               (9)  Feed 聚合
├── engine/             (15) 引擎核心
├── ai/                 (10) AI + ML
├── plugin/             (3)  插件系统
├── graph/              (3)  知识图谱
├── cli/                (2)  CLI 命令
├── integration/        (3)  跨模块集成
├── realtime/           (3)  实时能力
├── pipeline/           (3)  数据管道
├── media_cache/        (5)  缓存统一
├── observability/      (7)  可观测性
├── api_gateway/        (3)  API 网关
├── security/           (9)  安全 + 合规
├── reliability/        (4)  可靠性
├── deployment/         (13) 部署 + 边缘 + 云
├── governance/         (8)  治理 + 合规
├── analytics/          (7)  分析 + 成本
├── performance/        (4)  性能优化
├── disaster_recovery/  (3)  灾难恢复
├── multitenancy/       (3)  多租户
├── i18n/               (3)  国际化
├── workflow/           (4)  工作流
├── testing/            (6)  测试 + 模拟
├── arch/               (5)  架构定义
└── (core files)
```

---

## 四、跨域错位问题

| 模块 | 当前位置 | 正确位置 | 理由 |
|------|----------|----------|------|
| classification.rs | governance/ | security/ | 数据分类是安全 |
| gdpr.rs | governance/ | security/ | GDPR 是安全合规 |
| api_tracking.rs | cost/ | analytics/ | 成本追踪是分析 |
| health_check.rs | deployment/ | reliability/ | 健康检查是可靠性 |

---

## 五、执行计划

### Phase 1: 冗余清理
- 合并 circuit_breaker → circuit_breaker_unified
- 合并 media_cache + engine/cache → media_cache
- 标记冗余模块为 deprecated

### Phase 2: 扁平化合并
- ecosystem/ → evolution/
- ml/ → ai/
- edge/ + serverless/ + cloud/ → deployment/
- compliance/ → governance/
- simulation/ → testing/

### Phase 3: 跨域对齐
- governance/classification → security/
- governance/gdpr → security/
- cost/api_tracking → analytics/
- deployment/health_check → reliability/

### Phase 4: 验证
- 编译检查
- 依赖图验证
- 边界规则检查
