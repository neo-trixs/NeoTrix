# 点火清单（SIM-49；机器侧已清空，人侧 6 项）

> 实测：`git ls-remote` 读认证 OK；写 main 禁区，无 `gh`，CI（push-main/PR 触发）只能人点。
> 状态机：⬜待人 → 🟩关。每一项含：动作＋证据＋关项标准。

| # | 项 | 动作（精确） | 关项标准 |
|---|---|---|---|
| 1 | CI 点火 | 向 main 建 PR（含 973d7719＋1f7ea5b4 两批代码）；读 check×3OS＋test＋layer-deps 全部门 | 门绿＋基线入库；红则单 commit 还原，当事人文件先查（SIM-47/48） |
| 2 | entry 4 行合入 | 属主在其重写分支内重放 hunk（SIM-48 内附精确文本）；或通知代理重放 | hunk 入属主分支，CI 同验 |
| 3 | ADR-0004 签字 | L5 审 `docs/adr/0004-sdb-fail-closed-flip.md`，proposed→accepted | 状态行改 accepted＋签字人日期 |
| 4 | EQ-11 签字 | L5 审 `docs/architecture/FITNESS-THRESHOLDS-SIGNOFF.md` 六行 | 签字栏落笔，EQ-11 关 |
| 5 | EQ-18/19 认主 | 定 Desktop 主（审计报告）＋Architect 09-30 前认领 FULL（同步/归档/重写三选一） | EQ 行改 🟩（有主＋日期＋产物） |
| 6 | Phase 3 SIM | Architect 基于 SIM-49 修订开 P1-02 下沉 SIM（E8 留守／CRT 整体搬／CoT 拆分） | 新 SIM 号＋allowlist 退役计划（2026-10-31 前） |

P2~P4 不在表内（顺序锁，到站自开，非点火事项）。
