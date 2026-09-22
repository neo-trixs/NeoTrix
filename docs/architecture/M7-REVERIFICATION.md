# M7 复验记录（SIM-52 收口）

> 2026-09-22，逐项重跑，结果追记。

| 检查项 | 结果 | 备注 |
|---|---|---|
| `check-doc-drift.sh` | 111（基线 111） | 持平 |
| `check-api-surface.sh` | 19 tauri / 267 routes-ceiling / 0 spec | advisory，unchanged |
| `check-fuzz-ready.sh` | 3 targets present / nightly / 需 cargo-fuzz + seed | 新骨架，advisory |
| `check-supply-iocs.sh` | IOC 全绿 / advisory A/B 各 1–2 条 | M2 扩展，实跑通过 |

**结论**：本轮（M1–M7）零回归。fuzz/ 独立 crate，不入 workspace，cargo test 不受影响。
