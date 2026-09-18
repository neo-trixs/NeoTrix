# CLI Reference

NeoTrix 的交互式 TUI 通过 `/命令` 提供功能。运行 `neotrix --help` 查看 CLI 子命令；在 TUI 中输入 `/help` 查看全部斜杠命令。

## Basic Usage

```sh
neotrix <command> [options]
```

## Slash Commands

### 会话与蒸馏

| 命令 | 用途 |
|------|------|
| `/session list` | 列出已保存会话 |
| `/session save <name>` | 保存当前会话 (真实落盘到 KB `session_logs` + `~/.neotrix/session-logs/<name>.md`) |
| `/session load/delete/fork/export/import/share` | 会话管理子命令 |
| `/distill` | 扫描 session-logs 提取行为模式与建议 (SessionDistiller) |
| `/resume` | 恢复上次会话 |

### 任务契约与权限 (Phase 2)

| 命令 | 用途 |
|------|------|
| `/contract list` | 查看 agent-loop 任务契约队列 (C1-C6 映射) |
| `/contract define <desc>` | 定义新任务契约 |
| `/contract done <id>` | 标记契约完成 |
| `/contract fail <id> / cancel <id>` | 标记失败/取消 |
| `/contract stats` | 契约完成率统计 (KB `agent_loop` namespace 持久化) |
| `/perm status` | 三轴权限快照 (approval/perm-chain/policy + autonomy level) |
| `/perm check <action>` | 查询指定动作的权限决策 |
| `/perm set-approval <mode>` | 切换审批模式 |
| `/perm set-chain <chain>` | 设置权限链 |
| `/redact <text>` | 隐私脱敏 (secrets + PII → [REDACTED]) |
| `/redact check <text>` | 风险分级分析 (Safe/Suspicious/Dangerous) |
| `/redact secrets-only <text>` | 仅脱敏 secrets (保留 email 等 PII) |

### 网络与故障转移

| 命令 | 用途 |
|------|------|
| `/route` | 查看路由池状态 |
| `/route failover` | 查看故障转移历史 (FailoverHistory) |
| `/route failover --json` | JSON 格式历史 |
| `/route failover clear` | 清空历史 |

### 其他

`/help` `/stats` `/exit` `/clear` `/config` `/doctor` `/e8` `/brain` `/goal` `/plan` `/schedule` `/search` `/board` `/kb` `/wiki` `/git` `/commit` `/cost` `/budget` `/wallet` `/provider` `/model` `/free` `/skill` `/sandbox` `/review` `/plugin` `/profile` `/osint` `/selfaudit` `/comm` `/connector`

## 完整清单

在 TUI 中输入：

```sh
/help
```

获取最新注册的全部命令、别名与用法说明。
