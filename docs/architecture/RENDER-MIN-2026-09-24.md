# 最小资源渲染算法设计（2026-09-24）

> Explanation 象限：每像素花多少、为什么这么花。目标：60fps @ 16.6ms，
> 内存峰值 <300MB，二进制增量 <5MB。先给成本模型，再给 7 个子系统最优解。

---

## 0. 成本模型（1280×720，60fps）

| 开销 | 当前实测（debug） | 瓶颈 | 预算 |
|------|-------------------|------|------|
| CPU 逻辑（战斗/AI/粒子） | ~3ms | 敌人逐个 AABB | <5ms |
| CPU 提交（draw call） | ~2ms | 状态切换打断合批 | <3ms，draw call <120 |
| GPU 光栅 | ~2ms | 全屏 alpha 叠加 | <6ms，overdraw <3× |
| 内存 | ~180MB | 字体 atlas 失控增长 | <300MB，atlas 定额 |
| 显存上传/帧 | ~0 | 每帧新纹理 | 0（除粒子 atlas 一次） |

**第一性原理**：2D 游戏每帧真正变化的 <5% 像素。最优解 = **静态烘焙 + 脏区重绘 +
定额缓存 + 合批提交**。所有算法按「零每帧分配」设计。

---

## 1. 字形：预烘焙 Alpha 图集（P0，已落地本轮）

**问题**：macroquad 运行时 atlas 按需 doubling（512→…→8192），
`TextureFormat::size` u32 溢出 abort（D3），高负载下 Metal OOM。

**最优解**：charset 已知（2593 字）→ 离线一次烘焙，运行时零光栅化。

- 格式：单通道 Alpha（白字，运行时 `color` 相乘着色），
  2048² × 1B = **4MB 定额**（RGBA 方案 16MB，省 4×）。
- 格子：cell 40px @ 字号 32（留 4px overhang），51×51 格。
- 产物：`glyph_atlas.png` + `glyph_metrics.ron`
  （`{ch, x, y, w, h, adv}`，serde 直读，复用 abilities RON 链）。
- 复杂度：烘焙 O(N) 离线；运行时取字 O(1) hash，绘制 O(1)/字。
- 回退：atlas 缺字 → 运行时单字光栅进 128² 溢出区（LRU，永不 doubling）。
- 烘焙器：`assets/fonts/bake_atlas.py`（fontTools+Pillow，管线已热）。

## 2. 静态层烘焙（P0）

- 背景/平台/装饰 → 每波一次 `RenderTarget` 烘焙（1280×720 RGBA ≈ 3.5MB，
  双缓冲 7MB），每帧 1 个 textured quad。
- 脏标记：仅波次切换/暂停遮罩时重烘；战斗数字/血条走动态层。
- 节省：平台 N 个 rect（~40 draw call）→ 1 call。

## 3. 提交合批纪律（P0，零代码量，纯规范）

```
排序键 = (layer, texture_id, depth)
同 texture 的 quad macroquad 自动合批；状态切换（blend/color）打断批。
```
- 规则：动态实体按纹理分组绘制；文字全部走图集单纹理；
  粒子单纹理点精灵；每帧 `draw call` 上限 120（计数器断言，超了即告警）。
- 复杂度：排序 O(E log E)，E<200 可忽略；提交 O(1)/批。

## 4. 粒子：定额 SoA 环形池（P1）

- 布局：`struct Pool { pos:[Vec2;N], vel:[Vec2;N], life:[f32;N], head:usize }`，
  N=1024 定额，spawn O(1) 覆盖最老，update 线性扫描 O(N) 无分支预测失败
  （life<=0 跳过，分支可预测）。
- 内存：1024×(8+8+4+4) ≈ **24KB**，零分配。
- overdraw 预算：粒子总面积 < 0.5 屏，超了按 life 排序裁剪（O(N) 选择）。
- 现有 `particles.rs::Pool` 已是对象池 → 改造为 SoA 即可（API 不变，加单测）。

## 5. 文本布局缓存（P1）

- HUD 字符串高频相同（`第X波`、`Lv`）：`HashMap<u64,(w,h)>` 缓存 measure，
  脏标记仅数值变化时重排。measure O(1) 均摊。
- 叙事文本：整段一次排版，多帧复用顶点（静态文本烘焙进小 RT 可选）。

## 6. 后处理：零 FBO（P0，现状已达标，固化）

- 震屏 = 相机 offset（免费）；受击闪 = 1 个全屏 alpha rect（1 quad）；
  顿帧 = timestep 置 0（免费）。**禁止**全屏后处理 pass（bloom/blur），
  保持 GPU <6ms。

## 7. 动态分辨率（P2，保险丝）

- 帧时 EMA >15ms 持续 30 帧 → render_scale 1.0→0.75→0.5（世界层 RT 缩放，
  HUD 保持原生）；恢复阈值 <9ms。O(1)/帧，3 行代码。

---

## 内存总账（定额）

| 项 | 大小 | 备注 |
|----|------|------|
| 字形图集 Alpha 2048² | 4MB | 定额，永不增长 |
| 世界 RT 双缓冲 | 7MB | 1280×720 RGBA×2 |
| 粒子池 | 24KB | 定额 |
| 音频缓冲（WAV 合成） | <2MB | 已有 |
| 纹理/精灵（未来美术） | <20MB | 单图集 2048² 上限 |
| **合计增量** | **<35MB** | 峰值 180→~215MB，远离 300MB 线 |

## 落地顺序

1. ✅ 预烘焙图集（本轮：脚本+产物；Rust 加载器进 M2-D，`atlas.rs` + 单测）
2. 静态层烘焙（M2-C 顺手，`render_cache.rs`）
3. 合批计数器断言（M2-C，`debug_drawcalls()`）
4. 粒子 SoA（M2 空闲）
5. 文本缓存/动态分辨率（M3/M4）
