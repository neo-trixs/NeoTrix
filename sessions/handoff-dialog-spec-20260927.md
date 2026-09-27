# 对话面修复＋格式 Spec（2026-09-27，D/Z/W 三 lane）

## D lane 落地（5 文件，tsc 零错，未提交Список后见§4）

1. “运行中·今日头条”卡死：根因＝`tasks` 表 stale running（崩溃落在 Running-save 与终态-save 之间；
   `recover_stale_running` 只在 turn 起点调，读路径 15s 轮询永不回收）。修：新 `nt_stale_guard.rs`
   （STALE 600s＝LEASE 同值，本地流进行中豁免）＋前端 hero 超 10min 降级`运行超时`＋取消键。
2. 列表清理：软隐藏空/重名（置顶优先留最新）＋有效时间倒序＋`已隐藏N·显示全部`开关；不删数据。
3. 精简：删 `model_used` 内联前缀／`attempts`／`visibility` 三冗余（tn-meta 仅剩状态·认领）。
4. 复制＋重跑：修 `.msg-actions` 写在 `.msg` 外致 hover 永不命中真 bug；用户行新增重跑（同 `runStreamTurn` 道）；复制剪贴板优先原文。
5. 读路径自愈（一行，未动，留主线程）：`neobot_tasks` 首行调 `recover_stale_best_effort`。

## Z lane Spec（知乎 403→最佳实践综合，对话流二期）

- P0：流式 caret＋Stop／复制重跑 ActionBar／streaming-stopped-error-done 四态／长文折叠／代码块一键复制。
- P1：虚拟滚动＋跟随／分支对比／引用悬停卡／追问 chips／工具折叠。
- 本期已含：P0-2（复制重跑）部分；余入队。

## W lane 裁决（三件全 park＋1 嫁接）

- nt-act：整批已落地（6 文件逐字节 0 差），Cargo 陈旧禁合 → park。
- resilience：breaker/self_healing 两处回归（HalfOpen/deadline 丢）→ park。
- typed：分叉／fmt 超集／幻影 → park；唯一真值 `kb get_by_estate ?1` 由主线程嫁接（本文件 §4）。
- acl-manifests 161KB：sha 一致，幻影不动。

## §4 本轮提交内容

- 后端：`nt_stale_guard.rs`（新）＋`lib.rs` 1 行注册（hunk 过滤）。
- 嫁接：`kb.rs get_by_estate ?1`（2 行，W 归属注明；既有 `test_store_get_by_estate` 回归）。
- 前端（board/thread/main）：park 在 worktree（待 rebuild/repack 窗上线；dist 已旧）。
- 本文件（Z＋D＋W 证据）。
