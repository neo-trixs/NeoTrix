/**
 * NeoBot 极简图标系统（对齐主 App settingsIcons 的 open 线条语言，再砍一刀）：
 * 16 网格 · 1.3px 单线 · round 端点 · currentColor 单色 · 零渐变零节点。
 * 用法：icon("chat", 22)。新增图标只加 path，不加语义色。
 */
export type IconName =
  | "chat" | "clock" | "puzzle" | "chart" | "gear" | "search"
  | "plus" | "x" | "check" | "send" | "download" | "upload"
  | "user" | "users" | "refresh" | "play" | "sliders" | "info"
  | "globe" | "edit" | "doc" | "arrow-up" | "stop"
  // 侧边栏工作台用（Rust 侧 `nt_sidebar::builtin_tabs` 按名字引这三个，两边要对齐）
  | "folder" | "branch" | "layers" | "save" | "file" | "git" | "trash" | "chevron";

const PATHS: Record<IconName, string> = {
  chat: `<path d="M2.5 3h11a1 1 0 011 1v6a1 1 0 01-1 1H6.5l-3 3v-10a1 1 0 01.5-1z"/>`,
  clock: `<circle cx="8" cy="8" r="5.8"/><path d="M8 5v3.2l2.2 1.4"/>`,
  puzzle: `<path d="M5.5 2.5h3v2.2a1.4 1.4 0 010 2.6v2.2h-3V7.3a1.4 1.4 0 010-2.6V2.5zM8.5 5.5h2.5v3H8.5zM5.5 9.5h3V13a1 1 0 01-1 1H5.5a1 1 0 01-1-1v-2.5a1 1 0 011-1z"/>`,
  chart: `<path d="M2.5 13.5h11"/><path d="M4.5 13.5V9M8 13.5V5.5M11.5 13.5V7.5"/>`,
  gear: `<circle cx="8" cy="8" r="2.4"/><path d="M8 1.8v1.8M8 12.4v1.8M1.8 8h1.8M12.4 8h1.8M3.6 3.6l1.3 1.3M11.1 11.1l1.3 1.3M12.4 3.6l-1.3 1.3M4.9 11.1l-1.3 1.3"/>`,
  search: `<circle cx="7" cy="7" r="4.3"/><path d="M10.2 10.2L14 14"/>`,
  plus: `<path d="M8 3.5v9M3.5 8h9"/>`,
  x: `<path d="M4 4l8 8M12 4l-8 8"/>`,
  check: `<path d="M2.8 8.4l3.2 3.2 7.2-8"/>`,
  send: `<path d="M14 2L7.5 8.5M14 2l-4.5 12-2-5.5L2 6.5 14 2z"/>`,
  download: `<path d="M8 2v8M5 7l3 3 3-3M3 13.5h10"/>`,
  upload: `<path d="M8 10V2M5 5l3-3 3 3M3 13.5h10"/>`,
  user: `<circle cx="8" cy="5.2" r="2.6"/><path d="M2.8 13.8c.6-2.6 2.7-4 5.2-4s4.6 1.4 5.2 4"/>`,
  users: `<circle cx="6" cy="5.5" r="2.3"/><path d="M1.5 13.5c.5-2.3 2.3-3.5 4.5-3.5s4 1.2 4.5 3.5"/><circle cx="11" cy="6" r="1.9"/><path d="M11.5 10.2c1.7.2 2.9 1.2 3.3 3"/>`,
  refresh: `<path d="M13.5 8a5.5 5.5 0 11-1.6-3.9M13.5 1.8v2.6h-2.6"/>`,
  play: `<path d="M5 3.2v9.6L12.5 8 5 3.2z"/>`,
  sliders: `<path d="M2.5 5h11M2.5 11h11"/><circle cx="6" cy="5" r="1.6"/><circle cx="10.5" cy="11" r="1.6"/>`,
  info: `<circle cx="8" cy="8" r="5.8"/><path d="M8 7.4V11"/><circle cx="8" cy="4.9" r="0.2"/>`,
  globe: `<circle cx="8" cy="8" r="5.8"/><path d="M2.2 8h11.6M8 2.2c-3.6 3.4-3.6 8.2 0 11.6 3.6-3.4 3.6-8.2 0-11.6z"/>`,
  edit: `<path d="M11.5 2.8l1.7 1.7L5.4 12.3 2.5 13.5l1.2-2.9 7.8-7.8z"/>`,
  doc: `<path d="M4 1.8h5.5L12.5 5v9.2H4V1.8z"/><path d="M9.3 1.8V5H12.5M6 8h4M6 10.5h4"/>`,
  "arrow-up": `<path d="M8 13.5V2.8M4.3 6.3L8 2.6l3.7 3.7"/>`,
  "stop": `<rect x="4" y="4" width="8" height="8" rx="2"/>`,
  folder: `<path d="M1.8 4.2h4l1.4 1.6h7V12a1 1 0 01-1 1H2.8a1 1 0 01-1-1V4.2z"/>`,
  file: `<path d="M4 1.8h5.5L12.5 5v9.2H4V1.8z"/><path d="M9.3 1.8V5H12.5"/>`,
  branch: `<circle cx="4.5" cy="3.8" r="1.7"/><circle cx="4.5" cy="12.2" r="1.7"/><circle cx="11.5" cy="6.4" r="1.7"/><path d="M4.5 5.5v5M6.2 6.4h3.6M4.5 8.6c0-1.3 1-2.2 2.3-2.2"/>`,
  layers: `<path d="M8 1.8l6 3-6 3-6-3 6-3z"/><path d="M2 8.2l6 3 6-3M2 11.2l6 3 6-3"/>`,
  save: `<path d="M2.5 3.2a1 1 0 011-1h7.2L13.8 5v7.8a1 1 0 01-1 1h-9.3a1 1 0 01-1-1V3.2z"/><path d="M5 2.2v3.6h5V2.6M5 13.8V9.4h6v4.4"/>`,
  git: `<circle cx="4" cy="4" r="1.8"/><circle cx="4" cy="12" r="1.8"/><circle cx="12" cy="7.4" r="1.8"/><path d="M4 5.8v4.4M5.8 4h2.4a1.8 1.8 0 011.8 1.8v0"/>`,
  trash: `<path d="M2.5 4h11M6 4V2.6h4V4M4 4l.6 9.2h6.8L12 4M6.6 6.4v4.4M9.4 6.4v4.4"/>`,
  chevron: `<path d="M6 3.5L10.5 8 6 12.5"/>`,
};

export function icon(name: IconName, size = 20): string {
  return `<svg viewBox="0 0 16 16" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${PATHS[name]}</svg>`;
}
