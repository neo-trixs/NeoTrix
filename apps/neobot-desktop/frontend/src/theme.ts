/**
 * NeoBot 主题单一源（TS 侧）。
 *
 * 架构：`styles.css :root` 是 CSS 侧 token 表；本文件是 TS 侧 token 表，
 * 两边取值一一对应（改色只改这两处，且值必须同步）。JS 直接画的
 * 东西（表情球 SVG、头像渐变）从这里取色，不再散落硬编码。
 *
 * 当前主题：极简雪域（冰蓝品牌 + 雪青底 + Mac 系统状态色；gold 只做置顶/高亮）。
 */
export const THEME = {
  brand: "#38a1db",
  brandDeep: "#1f7ab5",
  brandInk: "#16324a",
  coral: "#f0756a",
  coralDeep: "#c84e3f",
  gold: "#f4b740",
  goldDeep: "#ba8418",
  whisper: "#7b6cb0",
  whisperLight: "#b8a9e0",
  avail: "#34c759",
  working: "#ff9f0a",
  resting: "#d1d1d6",
  ink900: "#101c2c",
  ink700: "#2c4258",
  ink500: "#64788d",
} as const;

/** 表情球主题（果球纳入整体主题架构：体色=品牌蓝，眼色=纸白，腮红=珊瑚浅）。 */
export const BALL = {
  body: THEME.brand,
  eye: "#ffffff",
  blush: "#ff8fa8",
  // 向后兼容导出（moodball 历史名）。
  NEOBOT_BALL: THEME.brand,
  NEOBOT_EYE: "#ffffff",
} as const;

/** 主题 token（品牌色七件套；设置窗口模板/导入导出与主窗口事件同构）。 */
export type ThemeTokens = {
  skype: string; deep: string; ink: string; coral: string;
  gold: string; whisper: string; avail: string;
};

/** 内置主题（值与 THEME 默认一致；改色先改这里，settings 只消费）。 */
export const BUILTIN_THEMES: Record<string, { label: string; tokens: ThemeTokens }> = {
  default: { label: "雪域", tokens: { skype: "#38a1db", deep: "#1f7ab5", ink: "#16324a", coral: "#f0756a", gold: "#f4b740", whisper: "#7b6cb0", avail: "#34c759" } },
  ink: { label: "墨青", tokens: { skype: "#0e9f8a", deep: "#0b7c6c", ink: "#073f36", coral: "#ff7a6b", gold: "#f4b740", whisper: "#7b6cb0", avail: "#6ec56a" } },
  sun: { label: "暖阳", tokens: { skype: "#f0913a", deep: "#c96a1b", ink: "#5c3408", coral: "#ff7a6b", gold: "#f4b740", whisper: "#7b6cb0", avail: "#6ec56a" } },
  sakura: { label: "樱粉", tokens: { skype: "#f06292", deep: "#c2185b", ink: "#5c1a2e", coral: "#ff7a6b", gold: "#f4b740", whisper: "#b388b0", avail: "#6ec56a" } },
  violet: { label: "雾紫", tokens: { skype: "#8b7cf0", deep: "#5f4fd0", ink: "#2c2560", coral: "#ff7a6b", gold: "#f4b740", whisper: "#7b6cb0", avail: "#6ec56a" } },
  night: { label: "夜蓝", tokens: { skype: "#4f9cf0", deep: "#2b5fb8", ink: "#0d1b3e", coral: "#ff7a6b", gold: "#f4b740", whisper: "#7b6cb0", avail: "#6ec56a" } },
  matcha: { label: "抹茶", tokens: { skype: "#58a05c", deep: "#3d7a40", ink: "#1d3a24", coral: "#ff7a6b", gold: "#f4b740", whisper: "#7b6cb0", avail: "#6ec56a" } },
};

/** 字体（id/label/CSS 栈；设置窗口选，主窗口 dataset 应用）。 */
export const FONT_FACES: ReadonlyArray<{ id: string; label: string; stack: string }> = [
  { id: "system", label: "系统", stack: `"Manrope", system-ui, -apple-system, "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", sans-serif` },
  { id: "pingfang", label: "黑体", stack: `"PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", system-ui, sans-serif` },
  { id: "song", label: "宋体", stack: `"Songti SC", "STSong", "SimSun", serif` },
  { id: "round", label: "圆体", stack: `"Hiragino Maru Gothic ProN", "PingFang SC", "Microsoft YaHei", system-ui, sans-serif` },
];

/** 头像渐变（按名哈希取； hues 全部来自 THEME）。 */
export const AVATAR_BG: readonly string[] = [
  `linear-gradient(135deg,${THEME.coral},${THEME.gold})`,
  `linear-gradient(135deg,${THEME.brand},${THEME.whisper})`,
  `linear-gradient(135deg,${THEME.whisper},${THEME.whisperLight})`,
  `linear-gradient(135deg,${THEME.avail},${THEME.brand})`,
];

/** 按名取头像背景（稳定哈希）。 */
export function avatarBg(name: string): string {
  let h = 0;
  for (const c of name) h = (h * 31 + (c.codePointAt(0) ?? 0)) >>> 0;
  return AVATAR_BG[h % AVATAR_BG.length];
}
