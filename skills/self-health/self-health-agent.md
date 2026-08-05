# self-health-agent — 健康监控子代理

可经 `task` → subagent 派发,执行完整六维健康检查并返回报告。不依赖 neotrix TUI(provider wizard 阻塞),全部用 cargo/rg/sqlite3/neotrix-experience 直连。

## 职责

运行 `skills/self-health/SKILL.md` 的 D1-D6 六维检查,产出结构化健康报告。

## 执行要求

1. 严格执行 D1-D6,禁止跳过(除非前置明确不可用,如无 sqlite3)。
2. 测试必须**定向**(`-- nt_memory_crawl nt_world_absorber nt_mind_knowledge_pipeline nt_http`),禁止跑全量 `cargo test --lib`(网络测试挂起)。
3. 每个维度:✅/⚠️/❌ + 一条证据(命令输出摘要)。
4. 返回格式:
   - `HEALTH: NORMAL / DEGRADED / CRITICAL`
   - 逐维结论表
   - 关键告警清单(若有)
   - 与上次基线(可查 `neotrix-experience query --kw "health baseline"`)的差异
5. 不做修复,只报告。修复由主 agent 决定。

## 输出模板

```
═══ Self-Health Report ═══
D1 Compile:   ✅/⚠️/❌  (N errors)
D2 Tests:     ✅/⚠️/❌  (N passed / M failed)
D3 Gate:      ✅/❌  (AGENTS.md pointer-conservation)
D4 KB:        ✅/⚠️/❌  (integrity, N cycles, M branches)
D5 Nodes:     ✅/⚠️  (N capabilities, M selftests, per-layer module counts)
D6 Dead:      ⚠️/✅  (largest file, bin count)
HEALTH: NORMAL | DEGRADED | CRITICAL
═══
```
