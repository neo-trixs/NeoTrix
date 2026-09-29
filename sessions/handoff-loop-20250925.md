# Handoff — 采矿循环窗 + 抖音任务（2026-09-25 晚，新对话从此接）

## 1. 会话标识

- 窗口：本对话（loop + douyin 双任务窗）
- 分支：`feat/capability-absorb-20260828`（多窗共建，脏树）
- 交接时间：2026-09-25 21:50（UTC+8）
- 本窗提交：`4a845183 feat(media): 抖音统一管道 + 自研语音转写 + 管道缺陷清零`（23 文件，+1663/-43，已复核零混入）
- 他窗提交（同期）：`40f3098f`（dispatch 拆分）、`8d19b1f1`（ingester 拆分）

## 2. 目标（一句话）

互联网采矿循环向 9000+ 推进（每轮 10 条 + 神经整合）；抖音【蜗牛有点田】62 视频已收官。

## 3. 当前状态（以文件为准，2026-09-25 21:50 实测）

- `loop_state.json`：rounds_completed=**43**，next_seed=**73**，remaining 8957
- 活茧：144 茧 / 7450 记忆；web-mined=**552**，neural-field=53，brace-depth=0
- `axioms.json` pending（含待补）=364；`web_raw_01..55.jsonl` 在位
- 锁：`datasets/mining/loop.lock` owner=**watcher5**（约 21:44 上岗，疑似仍在跑，rounds 停 43——它可能正在 server 端检索，本地无进程属正常）
- neural-field 53 vs 账面 47（+6 对不上 9 轮，疑他窗也写 neural 域或某轮重灌；待下轮对账，不阻塞）

## 4. 已完成（本窗）

- 循环：亲手跑 web13/web16 两轮；watcher 编队跑其余轮次（单驱动锁 + 门控全程有效）
- 安全基建：`nt_web_loop.py`（fresh 抽取/--commit/--lock 30min-stale）；digest 防清空门控（ABORT exit 2）+ pid 唯一茧 ID + 轮转快照
- 茧清空事故响应：他窗 cargo test 清空事件后完成全量重建（66 茧/7021）+ 合并他窗 5 茧 + 修复其坏 JSON（补 5 处冒号），`datasets/_recovery/REBUILD-MANIFEST.md` 存档
- 抖音：【蜗牛有点田】62 视频列表 + 61 mp4（1.0GB）+ 61 转写 + 10 精华 + 豆包/DeepSeek 交叉验证，吸收 29+ 条命中
- Rust：`nt_media/nt_douyin_extract.rs`（5 单测）+ `nt_speech_transcribe.rs`（8 单测，Mel 与 numpy 对标通过，真音逐词命中）+ `neotrix-douyin` bin（含 `--transcribe`）+ ort-1.x→2.x 迁移（onnx check 21 错清零）
- 管道缺陷 7 修：min_disk_space 透传 / format_bytes>= / is_resumable+Pending / RIFF12 / disposition 前缀 17 / 延迟表实装 / gzip 2 字节门 → nt_media 81/81
- 验证口径：xl 绿 + onnx check 绿 + 关键词命中 + brace-depth=0 + 单调增长检查

## 5. 正在进行（勿动）

- watcher5 持有循环锁（owner=watcher5）。**不要另起驱动**，等它 `--unlock` 交棒后再派下一任。若锁超 30min 无进展且无 raw 新增，按 stale 接管（先喊一声）。

## 6. 下一步（按优先级）

1. watcher5 交棒后：验 rounds/next_seed/web-mined/brace → 派下一任（seed 73 起，同一 prompt 模板，见本窗记录）
2. fresh_pending 耗尽（现 254+）后：循环自然收尾，转新原料源
3. 抖音收尾可选：DeepSeek API Key（用户未给）→ 批量蒸馏自动化；mp4→wav 收进管道（经 symphonia）；multilingual 权重替换 tiny.en
4. 转告他窗：测试隔离（夹具禁写真实茧路径）+ 单写者纪律 + 提交前 `git diff --cached --name-only` 自查（本窗曾误删其 dispatch.rs，已回退；另有 index 被并发清空两次，用 hunk 过滤重建）

## 7. 阻塞点

- 共享 index 多窗并发：提交必须用 hunk 过滤 + 提交后 `git show --stat` 复核异物；门禁 BUILD GATE 他窗在建变红时会拦所有人（禁 --no-verify，等绿错峰）
- 抖音账号保护：登录态 profile 在 `datasets/douyin_*/profile`（gitignored，绝不提交）；captcha 出现两次后已停自动化浏览
- disk：曾满（target/incremental 42GB 已清）；模型在 `datasets/_models`（gitignored）；dylib 需放二进制旁或 `ORT_DYLIB_PATH`

## 8. 给接手会话的话

- 恢复：读本文件 → `python3 scripts/ops/nt_web_loop.py`（只读）→ `ls datasets/mining/loop.lock` → 按§5/§6 行动
- 禁止：双驱动（先 --lock）、`cargo check --all-targets`、整文件覆写他人内容、提交前不核对暂存集
- 本窗经验已吸收（sess_git_battle_01、sess_media_fix_01、sess_speech_full_01 等），可 query 回查
