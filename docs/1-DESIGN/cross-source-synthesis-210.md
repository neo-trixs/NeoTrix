# 跨源综合分析 — 架构审计 × 外部研究

## 架构审计核心发现 (5.8/10)

| 问题 | 严重度 | 影响 |
|------|--------|------|
| 116个空壳模块 (mod.rs <20行) | P1 | 架构膨胀, 维护负担 |
| nt_feel 双重归属 (L3+L4) | P1 | 模块职责混乱 |
| 357 TODO/FIXME + 146 panic! | P1 | 运行时崩溃风险 |
| 能力集成器 3个关键 TODO | P0 | 模块无法协作 |
| 资产注册表重复实现 | P2 | 数据不一致 |
| EventBus 跨域连接断裂 | P1 | 模块间不可见 |

## 外部研究 P0 吸收清单

| 来源 | 模式 | NeoTrix 映射 | 吸收动作 |
|------|------|-------------|---------|
| Trace as State | 两遍 trace 重注入 | ConsciousnessTree 反馈循环 | D-new |
| Procedural Graphs | 自进化动作知识图谱 | E8 + SEAL pipeline | D-new |
| NeoHorse-1 | RSI routing loop | SEAL 管道具体实现 | D-new |
| ripwire | confidence-gated routing, 5% token | GWT cost-aware routing | D-new |
| Mistral legacy modernization | parity harness + structured agent | SEAL self-test 对齐 | D-new |
| FlyWire connectome | 全脑接线图, 注意力路由 | GWT 注意力拍卖机制 | D-new |
| OpenHands | 代码执行 agent 架构 | NT-ACT 执行框架参考 | D-new |
| autoresearch | 自主实验循环 | SEAL 探索循环参考 | D-new |
| i-have-adhd | action-first output protocol | CLI 输出格式参考 | D-new |
| robin | 多 agent 编排 | NT-CORE 编排参考 | D-new |

## 架构缺陷 × 外部解法交叉映射

| 架构缺陷 | 外部解法来源 | 建议吸收方案 |
|---------|-------------|-------------|
| 空壳模块膨胀 | OpenHands 模块化架构 | Dark Forest 清理 + 完整性检查 |
| nt_feel 双重归属 | FlyWire 分层注意力 | 统一到 L4, L3 只保留接口 |
| panic! 滥用 | ripwire confidence-gated | 替换为 Result + 降级策略 |
| EventBus 断裂 | Anthropic GWT | 统一事件总线, 跨域订阅 |
| 能力集成器空壳 | NeoHorse-1 RSI loop | 实现完整 pipeline |
| 资产注册表重复 | autoresearch 统一接口 | 单一事实源注册表 |

## 吸收优先级排序

### P0 (立即吸收, 5项)
1. **能力集成器实现** — nt_core_capability/integrator.rs 的 3 个 TODO
2. **nt_feel 统一** — 消除 L3/L4 双重归属
3. **panic! 替换** — 核心路径 panic 改为 Result
4. **EventBus 跨域** — 建立统一事件总线
5. **Dark Forest 清理** — 移除空壳模块

### P1 (本轮吸收, 8项)
6. **Trace as State** — ConsciousnessTree 反馈循环增强
7. **Procedural Graphs** — E8 知识图谱自进化
8. **NeoHorse-1 RSI** — SEAL pipeline 具体化
9. **ripwire routing** — GWT cost-aware 路由
10. **Mistral parity** — SEAL self-test 对齐
11. **FlyWire attention** — 注意力拍卖机制
12. **i-have-adhd protocol** — CLI action-first 输出
13. **统一资产注册表** — 单一事实源

### P2 (后续吸收, 5项)
14. **OpenHands 执行框架** — NT-ACT 参考
15. **autoresearch 循环** — SEAL 探索参考
16. **robin 编排** — 多 agent 参考
17. **Skill Tree 实现** — 架构文档定义
18. **Rune Socketing 实现** — 架构文档定义
