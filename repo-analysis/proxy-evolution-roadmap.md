# Proxy 进化路线图

## 当前状态

```
用户 shell (.zshrc)  ─── proxy_on/proxy_off/proxy-run/proxy_status
                              │
NeoTrix 进程          ─── ProxyClient ─→ Unix socket ─→ proxy_daemon (:11080)
                              │
BackgroundLoop        ─── auto_mode() 每30s (Tor/Stealth/Geo)
                              │
Headless REPL         ─── /proxy status | mode <off|geo|stealth|tor>
```

**缺失**: ① CLI 子命令 ② 桌面 UI ③ daemon 开机自启

---

## P0: `neotrix proxy` CLI 子命令

| 命令 | 实现方式 | 文件 |
|------|----------|------|
| `neotrix proxy status` | ProxyClient::status() → 格式化输出 | `entry/proxy_cmd.rs` (新) |
| `neotrix proxy mode` | ProxyClient::status() → mode 字段 | 同上 |
| `neotrix proxy mode stealth` | ProxyClient::set_mode() | 同上 |
| `neotrix proxy start` | ProxyClient::is_reachable() → 否则 spawn daemon | 同上 |
| `neotrix proxy stop` | ProxyClient::shutdown() | 同上 |
| `neotrix proxy install` | `launchctl load plist` | 同上 |

**实现**:
1. 新建 `entry/proxy_cmd.rs` — `pub fn run_proxy_cmd(cmd: &ProxySubcommand)`
2. `main.rs` Commands 枚举新增 `Proxy { sub: ProxySubcommand }`
3. `entry/mod.rs` 匹配路由

**文件**:
- `neotrix-core/src/main.rs` (+3 行 enum + 1 match arm)
- `neotrix-core/src/entry/proxy_cmd.rs` (~120 行, 新文件)
- `neotrix-core/src/entry/mod.rs` (+2 行)

**验证**: `cargo check --features stealth-net`

---

## P1: 桌面 Tauri 集成

### 1. Tauri 命令 (后端)

| 命令 | 作用 |
|------|------|
| `proxy_status` | 返回 JSON { mode, pid, port, uptime, active_count } |
| `proxy_set_mode` | 切换模式 |
| `proxy_start_daemon` | 启动 daemon (后台 nohup) |
| `proxy_stop_daemon` | 关闭 daemon |

**实现**:
- `src-tauri/src/commands/proxy.rs` (新, ~80 行) — 使用 ProxyClient
- `src-tauri/src/commands/mod.rs` — 注册 `pub mod proxy;`
- `src-tauri/src/main.rs` — invoke_handler 注册 `commands::proxy::*`

### 2. 前端面板

```
┌─ 代理控制 ─────────────────┐
│ 状态: ✅ 运行中 (:11080)     │
│ 模式: [Geo ▾]               │
│ 活跃: 42 请求               │
│ 空闲: 3s                    │
│                             │
│ [启动] [停止] [重启]        │
└─────────────────────────────┘
```

### 3. 系统托盘

```
neotrix-proxy
─────────────
  Geo         ← 当前
  Stealth
  Tor
  Off
─────────────
  重启代理
  停止代理
```

**实现**:
- `src-tauri/src/tray.rs` — 当前已有 `setup_tray()`，追加 proxy 子菜单
- 右键点击切换模式，左键点击显示状态通知

### 4. 设置页

- 代理端口 (默认 11080)
- 默认模式 (默认 geo)
- 开机自启 (plist 加载/卸载)

---

## P2: 进化路线

| # | 功能 | 触发条件 | 文件 |
|---|------|----------|------|
| E-01 | **智能模式选择**: NetworkDiagnostics 检测到被封锁 → 自动切 Stealth | 连续 3 次 HTTP 失败 | `background_loop/run.rs` |
| E-02 | **按 App 路由**: Chrome 走 Geo, curl 走 Stealth, Tor 浏览器走 Tor | 进程名匹配规则表 | `proxy_control.rs` |
| E-03 | **健康自恢复**: daemon 崩溃后自动重启 (已有 ProxyDaemonWrapper) | 心跳丢失 | `proxy_daemon_wrapper.rs` |
| E-04 | **多出口**: 每个模式独立 ProxyPool (Geo 池 / Stealth 池 / Tor 固定) | 模式切换时换池 | `proxy_pool.rs` |
| E-05 | **统计面板**: 请求量/成功率/延迟分布 → 桌面图表 | 每 60s 聚合 | `proxy_control.rs` |
| E-06 | **融合 NetworkDiagnostics**: 探测到异常自动降级 + 告警 | CUSUM/EWMA 触发 | `network_diagnostics.rs` |

---

## 文件清单

| 文件 | 新/改 | 行数 | P |
|------|-------|------|---|
| `main.rs` | 改 | +5 | P0 |
| `entry/proxy_cmd.rs` | **新** | ~120 | P0 |
| `entry/mod.rs` | 改 | +2 | P0 |
| `src-tauri/src/commands/proxy.rs` | **新** | ~80 | P1 |
| `src-tauri/src/commands/mod.rs` | 改 | +2 | P1 |
| `src-tauri/src/main.rs` | 改 | +5 | P1 |
| `src-tauri/src/tray.rs` | 改 | ~30 | P1 |
| `src-tauri/proxy-panel.tsx` (或 vue) | **新** | ~100 | P1 |
| `background_loop/run.rs` | 改 | +~40 | P2 |
| `proxy_pool.rs` | 改 | ~+60 | P2 |

---

## 依赖关系

```
P0 (CLI) ──→ P1 (Desktop) ──→ P2 (进化)
   │              │
   └── ProxyClient ── 共享核心
```

P0 无阻塞依赖，可立即开始。P1 依赖 ProxyClient（已完成）。P2 依赖 P0+P1。
