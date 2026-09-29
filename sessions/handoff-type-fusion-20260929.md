# Handoff — 类型自动融合器 nt_fuse_types.py（2026-09-29）

## 本会话交付
- 融合 10 组重复类型（真源保留、副本改 re-export，每组独立编译验证）：
  InquiryMetadata / MaterialSpec / Transform / Vector3 / QuoteSheet /
  AuthRequest / BrainCapability / PerformanceMetrics / PressureRating /
  PriceInfo / SizeSpec
- 提交：`67897fca`（MaterialSpec 首组）→ `7653bd4d`→amend 为 `ef7b7468`
  （6 组+工具修复）→ `eb4b7426`（4 组+第⑦项判据）
- 工具：`scripts/ops/nt_fuse_types.py`（七项判据全过才动）+
  `scripts/ops/nt_dup_types.py`（检测端）

## 七项判据（按加入顺序）
① 同名 ② 字段名+类型全同 ③ 无刻意镜像声明 ④ 两侧 trait impl 一致
⑤ 被删侧无固有 impl ⑥ 依赖安全（re-export 可达）⑦ 字段类型闭包安全

## 经验（每次都是先踩坑后立规约）
1. **return 丢值**：`block_bounds` 算好 `start` 却 return `m.start()`，
   修正只过了语法检查、没重跑失败用例 ⇒ 规则：修完必须重跑**原失败用例**。
2. **回滚粒度**：整文件 `git checkout` wipe 掉同文件已成功的 4 组 ⇒
   改为组级快照恢复。规则：回滚范围 = 改动范围。
3. **枚举不完备**：只查 trait impl、漏固有 impl（Verdict/E0592）⇒
   规则：列检查项时默认「一定还有没想到的」，加一条开放兜底。
4. **门要长进工具里**：我在 mem-gate BLOCKED 时启动构建 ⇒ 工具自查
   mem-gate + 基线树状态，不再依赖人记得查。
5. **提交信息照 diff 写**：曾把未融合的组写进 message，已 amend 修正 ⇒
   message 的组列表必须来自 `git diff --stat`，不能来自计划。
6. **名义类型陷阱**：Product.connection_type 两处同名但 4变体 vs 7变体 ⇒
   第⑦项判据。**同名字段背后可能是不同类型**，这是本轮最有价值的一课。
7. **跨窗协作**：kdf.rs / nt_qwen_mm_manifests.rs 他窗活跃编辑中（mtime
   分钟级），一律不碰、等绿。R-SCAN-4（复核现场）两次生效。

## 待处理（需人裁决，不可自动）
- **Product 组**：字段闭包不安全（ConnectionType 4变体 vs 7变体是真语义分叉，
  不是疏忽）。要么统一变体集（改任一侧行为），要么接受两份并存。**这是架构决策。**
- **ConvergenceProof**：真源缺 `Display` impl，另一侧有。需判断 Display 是否该保留。
- **Verdict**：两侧各有 `is_blocked` 等固有方法，需人工迁移方法后才能融合。
- 其余 ~90 组多为跨模块/跨层，需逐一定归属层（工具只处理同模块同层）。

## 恢复指令
```sh
sh scripts/ops/nt_mem_gate.sh  # 必须 OPEN
python3 scripts/ops/nt_fuse_types.py --dry-run --limit 30  # 先看
python3 scripts/ops/nt_fuse_types.py --limit 30            # 执行（含自查门）
```
