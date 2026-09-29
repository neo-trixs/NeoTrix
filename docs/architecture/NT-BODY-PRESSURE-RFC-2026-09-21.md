# NT-BODY-PRESSURE RFC — 身体感：把机器负载接进意识闭环

> 日期： 2026-09-21
> 来源： 真实事故复盘（多 opencode 窗口卡顿治理）
> 状态： RFC / 待实现
> 关联正典： `docs/architecture/ARCHITECTURE.md`（L0–L6 分层）、`R-P1`（零 unsafe）、`R-P79`（同会话接到生产）

## 1. 事故快照（证据）

| 指标 | 事故值 | 治理后 |
|---|---|---|
| 机型 | 10 核 / 16G（`hw.ncpu=10, hw.memsize=16G`） | — |
| `uptime load` | 5.68 / 3.90 / 3.52 | 2.8 左右 |
| `rustc` 全量编译 | 121.6% CPU / ~3G RSS（`--all-targets --test`，100+ rlib） | 已结束 |
| `opencode` 进程 | 4 个，各 30–70% CPU、各 300–780M RSS | 保留 1 个，其余手动关闭 |
| `target/` | 41G（`deps 22G + incremental 18G/167 份`） | 23G（删 `incremental`） |
| 内存 | free ~2G，`Swapout 42万次 / Compress 1400万次` | 待僵尸窗口关闭后回升 |
| 落地动作 | `.cargo/config.toml` 加 `[build] jobs = 5` | 已落盘验证 |

根因一句话：**4 个 watcher 盯同一 41G 目录 + 全量编译抢光 16G，多智能体无准入控制，内存一爆就是断崖式变慢。**

## 2. 公理（设计约束）

- A1 身体感是第一公民：`load / free / swapout / target体积` 与 `phi / coherence` 同级，进入 `consciousness_tick`。
- A2 感知去重：同一工作区只允许 1 个 watcher，感知一次、分发 N 次；并行隔离走 `.worktrees/`，不允许多 watcher 挤同一目录。
- A3 遗忘是功能：任何增量缓存 / 经验 / WAL 必须带 TTL + 定时 sweep（对标 `cargo sweep --time 14`）。
- A4 快慢双环：日常快反射（`check -p neotrix --lib` 级），提交前才慢深思（`clippy --all-targets` 级）。意识同理，不做无脑全量推理。
- A5 并发设限：`N核` 机器编译/推理并发恒留余量（如 `jobs = N/2`），压力超阈直接限流而非排队堆积。

## 3. 落点（对标现有模块）

| 层 | 现有模块 | RFC 改动 |
|---|---|---|
| L0 | `l0_substrate/nt_core_telemetry.rs`（`TelemetryEvent::ConsciousnessTick { phi, coherence, quality }`） | 新增 `BodyPressure` 只读采集（`loadavg / mem_free / swapio / target_bytes`，纯 std，无 unsafe，R-P1 合规），`ConsciousnessTick` 增加 `body_pressure: f64` 字段 |
| L5/L6 | `l6_meta/nt_core_self_model.rs`（`value_weights: coherence/safety/growth`） | 新增第 4 维 `body` 权重；`body_pressure > 阈值` 时自动降 `growth` 权重、升 `safety` |
| L6 | `l6_meta/healing/nt_repair_self_heal.rs`（检测→诊断→自愈→复测） | 把 `body_pressure` 作为一类可自愈项：超阈 → 限并发 → 停全量任务 → sweep 旧产物 → 复测 |
| L1/L2 | 任务分发 / 感知 | 单 watcher 总线 + `worktree` 隔离；`--all-targets` 类重任务串行化、禁多窗口并发 |

## 4. 最小实现（C0→C1，三步）

1. **T1 采集**：`nt_core_telemetry.rs` 加 `BodyPressure::sample()`（macOS `sysctl/vm_stat`，Linux `/proc`，失败即降级为 `None`，永不 panic，错误用 `?` 传播）。
2. **T2 接线**：`ConsciousnessTick` 携带 `body_pressure`；`SelfModel::value_function` 读到超阈即触发限流决策（降并发、禁全量）。
3. **T3 生产接地（R-P79）**：`nt_repair_self_heal` 注册 `body_pressure` 检测器，自愈动作真实生效（`jobs` 降档 / 杀全量编译 / 清 `incremental`），`self_test()` 即闭环本身。

## 5. 验收标准

- 压力注入（模拟 `load > N`）：10s 内限流生效，全量任务被拦截或串行化。
- 恢复验证：压力解除后并发自动回升，`self_test()` 全绿。
- 常态开销：采样本身 CPU < 0.1%，无 unsafe，无 `unwrap/expect/panic`。
- 本次事故不复发：`target > 30G` 或 `swapout` 暴涨时有 telemetry 记录 + 自愈动作记录。

## 6. 本次已落地（非代码）

- `.cargo/config.toml`：`[build] jobs = 5`（10 核留半）。
- `target/debug/incremental`（18G/167 份）已删，`target` 41G → 23G。
- 待办：关闭 3 个僵尸 opencode 窗口（`s000/s001/s002`），人工确认，避免误杀丢数据。
