# ABSORPTION-ROUND16 — swordgame.ai 吸收 + 旧游戏清零 + 劍嘯江湖 v1（2026-09-23）

> 用户决断：删光旧游戏代码（只留引擎+能力），重建武侠游戏。本轮即执行。

## 一、swordgame.ai 本体（实测扒取）

- **栈**：Vite SPA + Canvas 渲染 + DOM UI 混合（`#ui` 覆盖层），无商业引擎；
  水墨感 = **SVG 滤镜程序化描边**（`feTurbulence+feDisplacementMap` 四组 ink 参数，
  零美术资产）；LXGW 文楷 + 毛笔展示体；劍俠情緣貳 OST；Cloudflare 托管。
- **系统**：标题→诗体剧情→HUD（血/氣/分/波次/章节横幅/连击+连击教学点）→旁白浮层→
  暫停/劍折；J 攻/K 二段跳/L 轻功/Q 飛笠（帽子投射物）/Shift 疾跑；氣值驱动特殊技；
  波次刷怪。另有方法论对标：《群侠传：幸存者》1 人+AI、3 个月、Godot、94% 好评。
- 取舍：DOM/UI 混合与 OST 版权不跟（栈/法务不匹配）；SVG-ink 思想记入风格，
  本轮用像素 CJK 保交付。

## 二、删除执行（Slice-4b 之后续）

- `games/neotrix-cards/` 整目录备份（`_archive/swords-rebuild/`，2.9M）后删除；
  workspace 成员同步摘除。
- game 侧 `data/`：lib 七文件改道 `games/neotrix-swords/data/`（单拷贝已落位），
  删 game `data/`（备份于 `_archive/strangle-slice4b/data-game/`）。
- game 侧 `assets/`：字体管线文件归位 swords（见三），精灵/图标随旧游戏退役
  （`_archive` 有全量拷贝）；`assetforge::asset_root` 加 swords 候选（探测制，零风险）。

## 三、字体手术（PingFang SC surgical merge，全程可复现）

- 病因：事件库（66 事件）后于字体构建导入 + 新游全繁体 → 缺 706 + 25 字，
  标题"浴" tofu 实锤。
- donor 原件（/tmp 融合底+Noto）已失；改道系统 **PingFang SC Regular**
  （MobileAsset 内，face 3，4400 权重已核），`pyftsubset` 取子集。
- `pyftmerge` 跨轮廓格式（glyf vs CFF）失败 → 手动合并：
  CFF→glyf（Cu2QuPen + TransformPen ×1.2 upm 对齐）+ hmtx/cmap，
  名被规范化为 `uniXXXX`（妆造无碍）。
- 结果：2247→2978 字形，浴及 25 繁体 delta 全入库，FreeType 2530/2530 光栅通过，
  SHOT 双屏 tofu 清零。管线文件（脚本+charset+OFL+merge 程序）已归位 swords，
  `build_font_pixel.py` 路径与注记同步。
- **崩溃翻案**：合并后 cards 一次 abort，证为环境 Metal OOM（同二进制重跑即过；
  swords 同字体双屏全过；16G+并行构建通病）。审计脚本已加 3 次重试。

## 四、新品 劍嘯江湖 v1（`games/neotrix-swords`）

- **引擎** `nt_move`（game lib）：土狼/预输入缓冲/二段跳/可变跳高/冲刺cd，
  7 单测。首个无缝消费者即本游。
- **产品系统**（本 crate 纯模块）：`combo`（1.2s 窗连击+特招表 JJJ/JJK/JJL/JLJ）、
  `qi`（命中回氣/原子花费）、`waves`（预算 3+2w/弱1快2精4/3 波精锐/3 波一章/2 倍封顶）。
- **主循环**：标题→战斗→暂停→劍折；WASD/方向键（引擎表可重绑）+ J/K/L/Q/
  Shift/Esc/R；AABB 单向平台物理；怪追击+分离+接触伤；帽子穿透投射物；
  juice 分级 + 伤害数字 + 粒子；死区前视相机；`NEOTRIX_SWORDS_SHOT` 双场景。
- SHOT 双屏通过（月/山/平台/怪血条/白衣剑线/HUD/横幅全对）。

## 五、wasm 与 PLAN

- wasm32 编译验证通过（先修 PATH：Homebrew cargo 遮挡 rustup，`build_wasm.sh`
  早有备注）；浏览器实测仍需手动。
- `games/neotrix-swords/PLAN.md` 八要素已立；M1（四态一波）✅，M2–M4 待办。

## 六、验证

game lib **141** · spire **76** · swords **7**，全绿；新模块零警告；
新品 SHOT 审计脚本（重试版）2/2 通过。
