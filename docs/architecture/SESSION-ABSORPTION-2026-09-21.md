# Session Absorption 2026-09-21: 12 Lessons (SIM-01–SIM-21)

> 把本会话交过的学费变成文字。下条按时间序：现象 → 代价 → 固化到何处。
> 违反其中任何一条导致返工的，在对应 SIM 里记一笔。

| # | 教训 | 代价 | 固化 |
|---|------|------|------|
| L-01 | 归档/搬家前先查运行时消费者 | SIM-14 把宪法运行时读成桩（回归） | SIM-16；NTS-G04 吸收协议"探针含运行时" |
| L-02 | 登记表只许末尾追加＋排序机验 | 4 次错序（SIM-13/14/15/17） | SIM-PROTOCOL §5 登记铁律 |
| L-03 | 版本头尾标记同升 | v1.2.x 尾标滞后；SIM-19 误写不存在的 v1.2.9 | SIM-18 tripwire；每次发布机验 |
| L-04 | 禁 SIGTERM 杀 cargo | E0689 幽灵错，误导半轮 | SIM-17；清缓存走脚本 |
| L-05 | `.rs` 后缀即触发 Rust 门（含 docs 模板） | 30min 门禁空转＋OOM Kill | 模板命名评审项；门禁按 staged 后缀触发是已知行为 |
| L-06 | 脚本只写 bash-3.2 安全子集 | `$( )` 内 case 报 syntax error | SIM-08；`bash -n` 必跑 |
| L-07 | `git mv` 前先查跟踪态 | untracked 文件 mv 失败阻塞一批 | 探针标配 `git ls-files` |
| L-08 | macOS 下 gitignore 大小写不敏感 | `neoTrix-*` 误杀 6 个正典文档 | 删行；新 ignore 规则必须跑 `check-ignore` 验证 |
| L-09 | heredoc 内断言先数行号 | 两次 assert 打错行（SIM-16/20） |  assert 前 `rg -n` 确认锚点唯一 |
| L-10 | 他人进行中代码不动 | 4 错越修越错的风险 | P0-01 收编；本轮只归属不定罪 |
| L-11 | 隔离资产不删，零删除可为正确结论 | .disabled 1590 行差点被当占位符 | SIM-19："无可删"是探针结论 |
| L-12 | Override 三件套才放行 | —（首战告捷 SIM-20） | NT-STD §0.1：SIM＋ADR＋批复＋trailer＋tripwire，缺一不可 |

适用边界：L-05/L-06/L-08 是 macOS＋本仓库特异的，不要外推为通用真理；
L-01/L-02/L-03/L-12 是通用纪律，可进任何仓库的 CONTRIBUTING。
