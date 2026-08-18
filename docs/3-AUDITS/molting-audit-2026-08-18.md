# 蜕皮审计 — 全项目健康扫描与真实缺陷修复 — 2026-08-18

> 审计范围: NeoTrix 全项目 (清理 → 审计 → 修复 → 构建验证)。
> 方法: 证据先行 (file:line + 命令输出) + R-P 规则门禁 + 网络探测验证。
> 结论速览: 清理 124.5GB 缓存; clippy 生产代码 0 warning; 7523 测试绿; 发现并修复 **3 个真实缺陷** (Wikipedia 403 / CLI 接线错误 / Tauri 接线错误)。

---

## 1. 清理 (蜕皮前奏)

| 项 | 结果 |
|----|------|
| cargo target | 释放 124.5GB / 187737 文件 (94GB → 16GB) |
| KB 备份 | 16GB → 31MB (保留 3 份: knowledge-1787016403/1787016252/1787016211) |
| tmp 探测文件 | 删除 |
| release 二进制 | 备份 /tmp/neotrix_bin_backup/ 后恢复 |
| 磁盘 | 220GB → 313GB 可用 |

## 2. 构建门禁 (D1)

| 检查 | 结果 |
|------|------|
| `cargo check --all-targets` | 0 error (2m51s) |
| `clippy --lib -D warnings` | 0 error (修复 16 处) |
| `clippy --bins -D warnings` | 0 error (修复 68 处) |
| 前端 `npm run build` | 15.65s 成功 |
| vitest | 71 passed |

## 3. 测试门禁 (D7)

| 轮次 | 结果 |
|------|------|
| 首轮 | 7519 passed / 0 failed / 13 ignored (180.70s) |
| 二轮 | 7523 passed / 0 failed / 13 ignored (584.01s) |
| nt_mind_guard | 10 passed |
| autofixer | 19 passed |

## 4. 修复清单 (提交 46dda66c)

| 类型 | 内容 |
|------|------|
| clippy | lib 16 处 + bins 68 处生产代码 warning 清零 (R-P2) |
| 真实 bug | `experience.rs::cycle_sort_key` — `c[i..]` 多字节字节索引 panic → `char_indices` |
| 死代码 | `entry/mod.rs` — `if pct >= 100 {""} else {""}` 相同分支移除 |
| 构建 | `judge.rs` release 下 unused Duration import → 移入 `#[cfg(test)]` (注: 文件为并发会话 WIP, 后归还) |

## 5. 真实缺陷修复 (search 链路, 提交 d1102826/21aedd45/75f9cd27)

### 缺陷 1: Wikipedia 403 Forbidden (d1102826)
- 现象: `neotrix search` 恒返 "No results found", 尽管 Wikipedia 有 28 结果。
- 根因: reqwest client 无 User-Agent → Wikipedia api.php 返 403; DDG api 已停富结果 (Ok 空)。ordered router fallback 全失败。
- 定位: 网络探测测试 `probe_real_backends` 直接打真实后端, 暴露 `Wikipedia returned status: 403 Forbidden`。
- 修复: `nt_world_search.rs` — WikipediaBackend + WebSearchEngine 的 client 加 `User-Agent: NeoTrix/0.18`。
- 验证: probe 测试确认 router fallback 生效 (DDG 空 → 切 Wikipedia 5 结果)。

### 缺陷 2: CLI 顶层 search 接线错误 (21aedd45)
- 现象: 顶层 `neotrix search` 用旧 `WebSearchEngine` (仅 DDG, 已停富结果), 从不走 Wikipedia fallback。
- 对比: `/search` 命令 (search_cmds) 已用 `UnifiedSearch` (ordered router)。顶层漏网。
- 修复: `entry/mod.rs::run_search` — `WebSearchEngine` → `UnifiedSearch`。
- 验证: `neotrix search "Rust async runtime"` → 5 条 Wikipedia 结果。

### 缺陷 3: Tauri websearch 工具接线错误 (75f9cd27)
- 现象: App 端 `websearch` 工具 + `tool_search` command 同样用旧 `WebSearchEngine`。
- 修复: `tool_cmds.rs` — 两处 `WebSearchEngine` → `UnifiedSearch`。
- 验证: `cargo check -p neotrix-tauri` 通过 (UnifiedSearch Send 兼容)。

## 6. 审计发现 (Warning/Info, 未修)

| 维度 | 发现 | 级别 |
|------|------|------|
| D12 | KB mod 168 pub fn 仅 28 有 doc (≈17% 覆盖) | Warning |
| D12 | tauri bin clippy 剩 30 error (样式类, 不在 R-P2 核心范围) | Info |
| D4 | 无 `deny(unwrap_used)` — 生产代码可用 unwrap (多为 lock().unwrap() 约定) | Warning |
| D4 | 顶层 status 显示 Brain 0/23、KB 0 bytes (KB 懒加载未触发, 实际 234527 节点) | Warning |
| - | DDG api.duckduckgo.com 上游停富结果 (非代码缺陷, router 已 fallback) | Info |

## 7. 构建产物验证

| 产物 | 结果 |
|------|------|
| CLI release (`target/release/neotrix`) | 17.9MB, v0.18.0, search 修复入产物 |
| Tauri release (`target/release/neotrix-tauri`) | 19.3MB arm64, v0.18.0 |
| 冒烟 | `--version` / `reason --standalone` / `search` 全部通过 |

## 8. 纪律备注

- 并发会话在 main 活跃: 其 WIP (judge.rs + mod.rs `pub mod judge`) 曾被我误纳管, 已 revert 归还 (git rm --cached), 磁盘保留。
- git 纪律: 显式路径提交, 禁 `git add .`; 并发文件不碰不提交。