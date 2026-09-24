# NeoTrix STYLE FORMULA（higgsfield 式：所有视觉逐字节复用本公式）

> 无 FORMULA 不存在视觉。新增视觉前先更新本文件 + `design/assets.csv` 加行。

## 字体栈（唯一事实源）

- 基底：fusion-pixel 12px 像素字体 + Noto CJK 移植补字 → `assets/fonts/pixel-game.ttf`
- 构建：`assets/fonts/build_font_pixel.py`（语料 2502 字 → `charset.txt` 过滤 1823 字，ASCII 字母数字补全）
- 光栅三档：14 / 20 / 26（`populate_font_cache` 预热，见 main.rs 启动段）
- 许可：OFL（`assets/fonts/OFL.txt`）；emoji 不可用 → 一律用汉字替代（如意图 pill 用汉字）

## 色板（语义色，render.rs 常量名即契约）

`YELLOW` 标题/货币 · `GREEN` 增益/休息 · `RED` 伤害/战斗 · `ORANGE` 精英
`PURPLE` 事件/能力 · `SKYBLUE` 技能牌 · `WHITE` 正文 · `TEXT_DIM` 次要
`BORDER_DIM` 面板边框。Color 值为 f32 0–1；新语义色必须先在此登记再使用。

## 场景分层（render_scene / render_screen 两阶段）

背景（水墨天空/远山/地形植被/区域染色）→ 实体 → 粒子 → HUD → 面板（图鉴/塔图/事件/怪鉴）
→ 转场。世界层与屏幕层分相机：震屏只动世界层（`nt_juice`），HUD 零位移。

## 图标

液态玻璃 squircle + 透明背景，`tools/make_icon.py` v2 生成 → `icon.icns` 部署；
Finder 缓存需 `killall Finder Dock` 刷新。

## 面板

居中 `panel_centered` + `BORDER_DIM` 边框 + 0.92 暗底罩；标题 36 / 正文 20 / 小字 14；
超宽按字截断（`truncate`），富文本按 BBCode 色键逐段推进（`draw_rich`，键定义见 `neotrix_abilities::rich`）。
