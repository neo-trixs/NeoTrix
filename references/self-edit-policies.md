# 自编辑策略参考

> DGM-H / SEAL 自我修改管道的安全策略和门控约定。

## 编辑层级

| 层级 | 范围 | 安全要求 | 审批 |
|------|------|---------|------|
| L0 参数调整 | 数值参数 (阈值/权重/学习率) | SafetyGate 5 检 | 自动 |
| L1 策略修改 | 条件逻辑/分支/调度频率 | + 编译验证 | meta-agent |
| L2 结构修改 | 新增函数/模块/子系统 | + 行为等价测试 | meta + 人类 |
| L3 自举修改 | 修改 self-modify 管道 | + G89 完全验证 | 仅供授权 |

## SafetyGate 5 检

```
Check 1: 自洽性 — Ne 编译器可编译自身                    (强制)
Check 2: 向后兼容 — 所有现有程序输出不变                (强制)
Check 3: 行为保存 — 8 个 VSA 原语输出相同               (强制)
Check 4: 负熵非负 — N_total 不下降                       (强制)
Check 5: 元精度 — |predicted - actual| ≤ 0.05           (强制)
```

参见: `core/nt_core_experience/safety_gate.rs`

## EditSafetyNet 事务保护

```
begin_transaction()    → tx_id
add_operation(tx, op)  → 记录文件变更 (不写磁盘)
commit(tx)             → 写磁盘 + 可选 cargo check
rollback(tx)           → 从备份恢复原始文件
```

参见: `core/nt_core_experience/edit_safety.rs`

## 编辑效果追踪

每次 DGM-H 编辑应当记录:
```
EditRecord {
  target: String,       // 修改目标名称
  new_value: f64,       // 新值
  reason: String,       // 编辑理由
  outcome: EditOutcome, // Success / Failure / NoEffect
  timestamp: u64,       // cycle 编号
}
```

此历史记录用于:
1. 可靠性追踪 — 计算每个 meta-agent 的历史成功率
2. 回滚触发 — 成功率 < 0.5 时自动回退
3. 门控衰减 — 可靠性低的 agent 建议被衰减

## 可靠性门控 (ReliabilityGate)

插入在 `handle_dgmh_edit()` 之前:

```
g = σ(β · (r̄ − 0.5))         // sigmoid 门控
effective_delta = g · raw_delta  // 衰减不可靠建议
```

其中:
- β = 5.0 (sharpness, 同 SDAR)
- r̄ = 滑动窗口平均成功率 (W=20)
- raw_delta = 建议的修改幅度
- g < 0.5 时发出告警但不阻塞

参见: `core/nt_core_experience/reliability_gate.rs` (需实现)

## SEAL 27 阶段 Exit Criteria

| Stage | Exit Criteria |
|-------|---------------|
| S1 定位 | 缺口文档化, 优先级确认 |
| S2 计划 | 实施方案 + 回滚方案 |
| S3 实现 | SafetyGate 5 检全部通过 |
| S4 编译 | cargo check 0 errors 0 warnings |
| S5 测试 | 全部已有测试通过 |
| S6 部署 | EditSafetyNet commit 成功 |
| S7 验证 | 运行 50 cycles 无退化 |
