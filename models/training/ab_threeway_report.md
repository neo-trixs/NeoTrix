# Three-way head-to-head（同 80 题 seed=13，同度量）

- time=2026-09-24 21:45:26 elapsed=28s
- n: jev=80 laya=80 kev(hist)=80

| model | acc | brier | ece | conf | lat |
| agentjev | 0.725 | 0.387 | 0.122 | 0.707 | sidecar |
| laya-ft | 0.425 | 0.261 | 0.246 | 0.671 | 0.09s |
| kev(hist) | 0.512 | 0.307 | 0.160 | 0.673 | hist |

**结论：AgentJev 更准 → 门不动，吸 Laya 方法论（RLCD/temperature重fit/router）** (Δacc laya-jev=-0.300)

