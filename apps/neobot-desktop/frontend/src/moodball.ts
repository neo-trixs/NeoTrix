/**
 * NeoBot 表情球接线层（状态 → 自研 `emotionball` 引擎）。
 *
 * 映射只用 6 个语义状态（idle/思考/完成/出错/等待/离线），不开放全表；
 * 活球唯一（chat 头），列表头像也是活球（pane 重建前统一销毁防泄漏）。
 */
import { createEmotion, type Ball, type MoodName } from "./emotionball";
import { BALL } from "./theme";

/** 在线态 → 表情（列表缩略图用）。 */
export function moodForPresence(presence: string): "idle" | "sleep" {
  return presence === "off" ? "sleep" : "idle";
}

// 向后兼容导出（颜色以 theme.ts 为准）。
export const NEOBOT_BALL = BALL.NEOBOT_BALL;
export const NEOBOT_EYE = BALL.NEOBOT_EYE;

let live: Ball | null = null;
let rail: Ball | null = null;
let liveState: MoodName = "idle";
let doneTimer: ReturnType<typeof setTimeout> | null = null;
/** 当前球体色（主题跟随；缩略球下次挂载即用新色）。 */
let ballColor: string = BALL.body;
/** 主题切换时调：活球当场换色 + 后续挂载用新色（CSS 变量由主窗口同步）。 */
export function setBallColor(body: string): void {
  if (/^#[0-9a-fA-F]{6}$/.test(body)) ballColor = body;
  for (const b of [live, rail, ...thumbs]) {
    try { b?.setColors(ballColor, BALL.eye); } catch { /* 忽略 */ }
  }
}
/** 列表缩略球注册表（活球；pane 重建前统一 destroy，防 rAF 泄漏）。 */
const thumbs = new Set<Ball>();
/** 销毁全部缩略活球（renderPanes 整 pane 重建前调）。 */
export function clearThumbs(): void {
  for (const b of thumbs) {
    try { b.destroy(); } catch { /* 忽略 */ }
  }
  thumbs.clear();
}

/** 挂活球（chat 头，注视跟随由调用方接 pointermove）。 */
export function mountLiveBall(el: Element): void {
  try {
    live?.destroy();
    live = createEmotion(el, {
      mood: liveState,
      color: ballColor,
      eyeColor: NEOBOT_EYE,
      eyeScale: 1.1,
    });
  } catch { live = null; }
}

/** 挂左轨活球（与头球同状态；小尺寸放大眼睛保可读）。 */
export function mountLiveRail(el: Element): void {
  try {
    rail?.destroy();
    rail = createEmotion(el, {
      mood: liveState,
      color: ballColor,
      eyeColor: NEOBOT_EYE,
      eyeScale: 1.4,
    });
  } catch { rail = null; }
}

/** 列表缩略球（活球：眨眼/呼吸/注视都动；失败回 false，调用方画渐变圆兜底）。 */
export function mountThumb(el: Element, mood: MoodName = "idle"): boolean {
  try {
    const b = createEmotion(el, {
      mood,
      color: ballColor,
      eyeColor: NEOBOT_EYE,
      eyeScale: 1.4,
      autostart: true,
    });
    thumbs.add(b);
    return true;
  } catch { return false; }
}

/** 切换活球状态（头球 + 左轨球同步）；done 会 3s 后自动回 idle（撒花不常驻）。 */
export function setMood(m: MoodName): void {
  liveState = m;
  if (doneTimer) { clearTimeout(doneTimer); doneTimer = null; }
  for (const b of [live, rail]) {
    try { b?.setMood(m); } catch { /* 球没起来不挡主流程 */ }
  }
  if (m === "done") {
    doneTimer = setTimeout(() => {
      liveState = "idle";
      for (const b of [live, rail]) {
        try { b?.setMood("idle"); } catch { /* 忽略 */ }
      }
    }, 3000);
  }
}

/** 注视跟随（相对球心归一化坐标）。 */
export function gazeAt(el: Element, clientX: number, clientY: number): void {
  if (!live) return;
  const r = el.getBoundingClientRect();
  if (r.width === 0) return;
  try {
    live.setGaze(
      (clientX - (r.left + r.width / 2)) / (r.width / 2),
      (clientY - (r.top + r.height / 2)) / (r.height / 2),
    );
  } catch { /* 忽略 */ }
}
