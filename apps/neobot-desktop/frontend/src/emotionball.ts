/**
 * NeoBot 自研表情泡（clean-room 实现，不含任何第三方代码）。
 *
 * 技术原理（公开设计依据，不是抄代码）：
 * - Kindchenschema（Lorenz；PNAS 2009 / ACM 2024 验证）：眼睛位置偏下、
 *   圆脸、小嘴、腮红——本球眼心 y=53（中线以下）、嘴宽 12、常驻腮红；
 * - 眼睛尺寸吸收 grok-ball 实测参数（100 盒归一化：rx≈5.8、ry≈7.9、
 *   眼距≈19.3，本球取整 rx6/ry8/眼心 40.5/59.5）——只取数值比例，
 *   无一句代码拷贝；本轮追加：机身三段径向渐变（圆心 38%/32%、半径 75%，
 *   0% 提亮 22 / 62% 本色 / 100% 压暗 12）、眨眼区间（idle 6-14s、
 *   活跃 2.5-5.5s、sleep 不眨）、sleep 开合度 0.08、三枚 z 睡眠粒子、
 *   9~18s 即兴扫视——均为参数/数值，同无代码拷贝；
 *   果冻迭代：stops 半透明 + 白 rim + 底部内阴影 + 高光增强；
 * - 天气拟物：体色向各态天空色偏（idle 晴蓝 / working 风蓝 / done 暖阳 /
 *   error slate 雷雨灰蓝 / waiting 灰蓝阴 / sleep 深海军蓝夜）+ 天气饰件组
 *   （小太阳云 / 风旋线 / 日芒 / 雷云闪电雨 / 漂移双云 / 弯月星芒），
 *   脸保留做表情（眼嘴位置不动，饰件占上半区）；
 * - Disney 12 律：squash & stretch（切表情先压后弹、done 拉伸）、
 *   anticipation（撒花前 0.3s 下蹲）、secondary action（腮红/眼光/呼吸）、
 *   exaggeration（表情推过一点）、appeal（眼内高光 catchlight）；
 * - HRI（Kismet/MiRAE/Cosmo）：眼+嘴两组特征即够；脸永远在动
 *   （呼吸浮动）；注视平滑跟随。
 * 表情 = 目标参数集，每帧 `p += (t - p) * k` 趋近，切换天然平滑；
 * autostart:false 只渲染一帧（缩略图零成本）。
 */
export type MoodName = "idle" | "working" | "done" | "error" | "waiting" | "sleep";

import { BALL } from "./theme";

export interface Ball {
  setMood(m: MoodName): void;
  setGaze(nx: number, ny: number): void;
  /** 主题跟随：换体色/眼色（渐变三段按新体色重算）。 */
  setColors(body: string, eye: string): void;
  destroy(): void;
}

export interface BallOpts {
  mood?: MoodName;
  color?: string;
  eyeColor?: string;
  /** 眼睛放大（小尺寸保可读）。 */
  eyeScale?: number;
  /** false = 只画一帧静态图。 */
  autostart?: boolean;
  /** 云团模式（默认开）：无背景随机云簇＋动态表情，无实心球体。 */
  cloud?: boolean;
}

interface Params {
  eyeRx: number;
  eyeRy: number;
  eyeDy: number;
  happy: boolean;
  mouthSmile: number;
  scanAmp: number;
  gazeAmp: number;
  /** 腮红不透明度。 */
  blush: number;
  /** 担忧眉不透明度（error 用）。 */
  worry: number;
}

const POSES: Record<MoodName, Params> = {
  idle: { eyeRx: 6, eyeRy: 8, eyeDy: 0, happy: false, mouthSmile: 0.18, scanAmp: 0, gazeAmp: 1, blush: 0.35, worry: 0 },
  working: { eyeRx: 6, eyeRy: 7.5, eyeDy: -4, happy: false, mouthSmile: 0.12, scanAmp: 0, gazeAmp: 0.6, blush: 0.25, worry: 0 },
  done: { eyeRx: 6, eyeRy: 8, eyeDy: 0, happy: true, mouthSmile: 1, scanAmp: 0, gazeAmp: 1, blush: 0.85, worry: 0 },
  error: { eyeRx: 6.5, eyeRy: 9.5, eyeDy: 0, happy: false, mouthSmile: -0.55, scanAmp: 0, gazeAmp: 0.3, blush: 0.2, worry: 1 },
  waiting: { eyeRx: 6, eyeRy: 8, eyeDy: 0, happy: false, mouthSmile: 0.22, scanAmp: 5, gazeAmp: 0.4, blush: 0.4, worry: 0 },
  sleep: { eyeRx: 6, eyeRy: 0.8, eyeDy: 2, happy: false, mouthSmile: 0, scanAmp: 0, gazeAmp: 0, blush: 0.15, worry: 0 },
};

const NS = "http://www.w3.org/2000/svg";
function el(tag: string, attrs: Record<string, string>): SVGElement {
  const n = document.createElementNS(NS, tag);
  for (const k of Object.keys(attrs)) n.setAttribute(k, attrs[k]);
  return n as unknown as SVGElement;
}
const clamp1 = (v: number): number => Math.max(-1, Math.min(1, v));
/** 明暗罩（amt>0 向白、<0 向黑；纯数学）。 */
function shade(hex: string, amt: number): string {  let h = hex.replace("#", "");
  if (h.length === 3) h = h[0] + h[0] + h[1] + h[1] + h[2] + h[2];
  const n = parseInt(h, 16);
  const t = amt < 0 ? 0 : 255;
  const a = Math.abs(amt);
  const mix = (c: number): number => Math.round(c + (t - c) * a);
  const r = mix((n >> 16) & 255);
  const g = mix((n >> 8) & 255);
  const b = mix(n & 255);
  return "#" + ((1 << 24) | (r << 16) | (g << 8) | b).toString(16).slice(1);
}
/** 双色混合（天气天空 tint 用；t=0 全 c1）。 */
function mixc(c1: string, c2: string, t: number): string {
  const p = (h: string): [number, number, number] => {
    let s = h.replace("#", "");
    if (s.length === 3) s = s[0] + s[0] + s[1] + s[1] + s[2] + s[2];
    const n = parseInt(s, 16);
    return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
  };
  const [r1, g1, b1] = p(c1);
  const [r2, g2, b2] = p(c2);
  const m = (a: number, b: number): number => Math.round(a + (b - a) * t);
  return "#" + ((1 << 24) | (m(r1, r2) << 16) | (m(g1, g2) << 8) | m(b1, b2)).toString(16).slice(1);
}
/** 表情 → 天气（简约拟物：体色向天空色偏 + 天气饰件；脸保留做表情）。 */
const SKY: Record<MoodName, { sky: string; mix: number }> = {
  idle: { sky: "#8fd0f5", mix: 0.45 }, // 晴：浅天蓝
  working: { sky: "#a8d8f0", mix: 0.4 }, // 风：淡蓝 + 旋线
  done: { sky: "#ffd66e", mix: 0.5 }, // 大晴：暖阳
  error: { sky: "#5a6b85", mix: 0.6 }, // 雷阵雨： slate 灰蓝
  waiting: { sky: "#b8cfe0", mix: 0.5 }, // 阴：灰蓝
  sleep: { sky: "#1e3468", mix: 0.65 }, // 夜：深海军蓝
};
/** 眨眼区间（idle 4.5-9s，活跃 2.5-5.5s，done 4-8s，sleep 不眨）。 */
const BLINK_MS: Record<MoodName, [number, number] | null> = {
  idle: [4500, 9000],
  working: [2500, 5500],
  done: [4000, 8000],
  error: [2500, 5500],
  waiting: [2500, 5500],
  sleep: null,
};

export function createEmotion(target: string | Element, opts: BallOpts = {}): Ball {
  const host = typeof target === "string" ? document.querySelector(target) : target;
  let color = opts.color ?? BALL.body;
  let eyeColor = opts.eyeColor ?? BALL.eye;
  const eyeScale = opts.eyeScale ?? 1;
  const running = opts.autostart !== false;
  let mood: MoodName = opts.mood ?? "idle";
  const cloudMode = opts.cloud !== false;

  const svg = el("svg", { viewBox: "0 0 100 100", width: "100%", height: "100%" });
  // 机身三段径向渐变（圆心 38%/32%、半径 75%；
  // 0% 提亮 22、62% 本色、100% 压暗 12）+ 果冻通透： stops 半透明，
  // 底下渐变/背景透出来 + 白 rim + 底部内阴影。
  const gid = "nbg" + Math.random().toString(36).slice(2, 8);
  const defs = el("defs", {});
  const grad = el("radialGradient", { id: gid, cx: "38%", cy: "32%", r: "75%" });
  const stop = (offset: string, c: string, o: string): SVGElement =>
    el("stop", { offset, "stop-color": c, "stop-opacity": o });
  grad.append(
    stop("0%", shade(color, 0.45), "0.85"),
    stop("55%", color, "0.62"),
    stop("100%", shade(color, -0.12), "0.72"),
  );
  defs.append(grad);
  // 脸组（挤压拉伸以底部为锚点，体积近似守恒）。
  const face = el("g", {});
  const body = el("circle", { cx: "50", cy: "50", r: "44", fill: `url(#${gid})` });
  // 果冻 rim 光 + 底部内阴影。
  const rim = el("circle", { cx: "50", cy: "50", r: "44", fill: "none", stroke: "#ffffff", "stroke-width": "2", opacity: "0.65" });
  const innerShade = el("ellipse", { cx: "50", cy: "78", rx: "26", ry: "12", fill: shade(color, -0.3), opacity: "0.28" });
  const gloss = el("ellipse", { cx: "36", cy: "30", rx: "20", ry: "12", fill: BALL.eye, opacity: "0.3" });
  // 腮红（Kindchenschema 圆颊；色取主题）。
  const blushL = el("ellipse", { cx: "23", cy: "63", rx: "5.5", ry: "3.2", fill: BALL.blush, opacity: "0.35" });
  const blushR = el("ellipse", { cx: "77", cy: "63", rx: "5.5", ry: "3.2", fill: BALL.blush, opacity: "0.35" });
  // 担忧眉（error 用；内端上扬装无辜）。
  const browL = el("path", { d: "M31,37 L43,33", stroke: cloudMode ? shade(eyeColor, -0.35) : eyeColor, "stroke-width": cloudMode ? "3" : "2.6", fill: "none", "stroke-linecap": "round", opacity: "0" });
  const browR = el("path", { d: "M57,33 L69,37", stroke: cloudMode ? shade(eyeColor, -0.35) : eyeColor, "stroke-width": cloudMode ? "3" : "2.6", fill: "none", "stroke-linecap": "round", opacity: "0" });
  const pupils = el("g", {});
  const eyeL = el("ellipse", { cx: "40.5", cy: "53", rx: "6", ry: "8", fill: cloudMode ? shade(eyeColor, -0.35) : eyeColor });
  const eyeR = el("ellipse", { cx: "59.5", cy: "53", rx: "6", ry: "8", fill: cloudMode ? shade(eyeColor, -0.35) : eyeColor });
  // 眼内高光（appeal 关键；随瞳孔组一起动）。
  const glintL = el("circle", { cx: "38.2", cy: "50", r: cloudMode ? "1.4" : "1.8", fill: cloudMode ? "#ffffff" : eyeColor });
  const glintR = el("circle", { cx: "57.2", cy: "50", r: cloudMode ? "1.4" : "1.8", fill: cloudMode ? "#ffffff" : eyeColor });
  const glintL2 = el("circle", { cx: "42.6", cy: "56", r: cloudMode ? "0.7" : "0.9", fill: cloudMode ? "#ffffff" : eyeColor, opacity: "0.8" });
  const glintR2 = el("circle", { cx: "61.6", cy: "56", r: cloudMode ? "0.7" : "0.9", fill: cloudMode ? "#ffffff" : eyeColor, opacity: "0.8" });
  // 笑眼弧（done 用；平时隐藏）。
  const happyL = el("path", { d: "M33,55 Q40.5,48 48,55", stroke: cloudMode ? shade(eyeColor, -0.35) : eyeColor, "stroke-width": "3.4", fill: "none", "stroke-linecap": "round", display: "none" });
  const happyR = el("path", { d: "M52,55 Q59.5,48 67,55", stroke: cloudMode ? shade(eyeColor, -0.35) : eyeColor, "stroke-width": "3.4", fill: "none", "stroke-linecap": "round", display: "none" });
  const mouth = el("path", { d: "", stroke: cloudMode ? shade(eyeColor, -0.35) : eyeColor, "stroke-width": cloudMode ? "3.8" : "3", fill: "none", "stroke-linecap": "round", opacity: "0.9" });
  const ring = el("circle", {
    cx: "50", cy: "50", r: "34", fill: "none", stroke: eyeColor,
    "stroke-width": "2.5", "stroke-dasharray": "10 14", opacity: "0", "stroke-linecap": "round",
  });
  const fx = el("g", {});
  // 睡眠粒子：三枚小写 z，右上循环漂浮（颜色随眼色）。
  const zzz: SVGElement[] = [];
  for (let i = 0; i < 3; i++) {
    const z = el("text", {
      x: String(70 + i * 5), y: "30", fill: eyeColor, "font-size": String(9 + i * 2.5),
      "font-weight": "bold", "font-style": "italic", opacity: "0", "text-anchor": "middle",
    });
    z.textContent = "z";
    zzz.push(z);
  }
  pupils.append(eyeL, eyeR, glintL, glintR, glintL2, glintR2, happyL, happyR);
  face.append(body, innerShade, gloss, blushL, blushR, browL, browR, pupils, mouth, rim);
  // ── 小云团（无背景随机云簇：5 团每挂载抖动，全透明白底；脸画在云上） ──
  const cloudBody = el("g", {});
  {
    const j = (): number => (Math.random() * 2 - 1) * 3;
    const puffs: Array<[number, number, number]> = [
      [35 + j(), 60 + j(), 12], [46 + j(), 50 + j(), 15],
      [59 + j(), 52 + j(), 13], [67 + j(), 63 + j(), 11], [51 + j(), 70 + j(), 12],
    ];
    for (const [x, y, r] of puffs) {
      cloudBody.append(el("circle", { cx: x.toFixed(1), cy: y.toFixed(1), r: String(r), fill: `url(#${gid}cl)` }));
    }
    cloudBody.append(el("rect", { x: "24", y: "58", width: "52", height: "16", rx: "8", fill: `url(#${gid}cl)` }));
  }
  // 动态表情（随 mood 切换的小 emoji，坐云角）。
  const MOOD_EMOJI: Record<MoodName, string> = { idle: "🙂", working: "🤔", done: "🎉", error: "😵", waiting: "💭", sleep: "😴" };
  const moodEmoji = el("text", { x: "76", y: "58", "font-size": "12", "text-anchor": "middle" });
  moodEmoji.textContent = MOOD_EMOJI[mood];
  cloudBody.append(moodEmoji);
  if (cloudMode) {
    // 云团模式：藏实心球体件（渐变/rim/内阴影/高光），脸与天气饰件保留。
    for (const n of [body, rim, innerShade, gloss]) n.style.display = "none";
  } else {
    cloudBody.style.display = "none";
  }

  // ── 天气拟物饰件（简约拟物：渐变 + 柔影；脸保留做表情） ──
  const sunGrad = el("radialGradient", { id: gid + "sun", cx: "38%", cy: "35%", r: "75%" });
  sunGrad.append(
    el("stop", { offset: "0%", "stop-color": "#fff6c8" }),
    el("stop", { offset: "60%", "stop-color": "#ffd66e" }),
    el("stop", { offset: "100%", "stop-color": "#f5a623" }),
  );
  const cloudGrad = el("linearGradient", { id: gid + "cl", x1: "0", y1: "0", x2: "0", y2: "1" });
  cloudGrad.append(
    el("stop", { offset: "0%", "stop-color": "#ffffff" }),
    el("stop", { offset: "100%", "stop-color": "#cfd8e3" }),
  );
  const boltGrad = el("linearGradient", { id: gid + "bo", x1: "0", y1: "0", x2: "0", y2: "1" });
  boltGrad.append(
    el("stop", { offset: "0%", "stop-color": "#ffe27a" }),
    el("stop", { offset: "100%", "stop-color": "#f5a623" }),
  );
  defs.append(sunGrad, cloudGrad, boltGrad);
  /** 云（一团三圆 + 底座 + 柔影）。 */
  function cloud(x: number, y: number, s: number): SVGElement {
    const g = el("g", { transform: `translate(${x} ${y}) scale(${s})` });
    g.append(
      el("ellipse", { cx: "0", cy: "7", rx: "14", ry: "3.5", fill: "#0a1b2e", opacity: "0.12" }),
      el("circle", { cx: "-8", cy: "1", r: "6", fill: `url(#${gid}cl)` }),
      el("circle", { cx: "0", cy: "-3", r: "8", fill: `url(#${gid}cl)` }),
      el("circle", { cx: "8", cy: "1", r: "6", fill: `url(#${gid}cl)` }),
      el("rect", { x: "-13", y: "-1", width: "26", height: "7", rx: "3.5", fill: `url(#${gid}cl)` }),
    );
    return g;
  }
  /** 四角星芒。 */
  function sparkle(x: number, y: number, s: number): SVGElement {
    return el("path", {
      d: `M${x},${y - s} C${x + 1},${y - 1} ${x + 1},${y - 1} ${x + s},${y} C${x + 1},${y + 1} ${x + 1},${y + 1} ${x},${y + s} C${x - 1},${y + 1} ${x - 1},${y + 1} ${x - s},${y} C${x - 1},${y - 1} ${x - 1},${y - 1} ${x},${y - s} Z`,
      fill: "#fff6c8", opacity: "0.9",
    });
  }
  const wIdle = el("g", {});
  wIdle.append(
    el("circle", { cx: "68", cy: "24", r: "6", fill: `url(#${gid}sun)` }),
    cloud(30, 26, 0.55),
  );
  const wWork = el("g", { stroke: "#ffffff", "stroke-width": "2.5", fill: "none", "stroke-linecap": "round", opacity: "0.85" });
  const wind1 = el("path", { d: "M14,30 Q34,21 54,29 T88,27", "stroke-dasharray": "12 9" });
  const wind2 = el("path", { d: "M20,38 Q38,32 56,37 T84,35", "stroke-dasharray": "9 8", opacity: "0.7" });
  wWork.append(wind1, wind2);
  const wDone = el("g", {});
  const rays = el("g", { stroke: "#f5a623", "stroke-width": "2.5", "stroke-linecap": "round", opacity: "0.9" });
  for (let i = 0; i < 8; i++) {
    const a = (i / 8) * Math.PI * 2;
    const x1 = 50 + Math.cos(a) * 13;
    const y1 = 22 + Math.sin(a) * 13;
    const x2 = 50 + Math.cos(a) * 17.5;
    const y2 = 22 + Math.sin(a) * 17.5;
    rays.append(el("line", { x1: String(x1.toFixed(1)), y1: String(y1.toFixed(1)), x2: String(x2.toFixed(1)), y2: String(y2.toFixed(1)) }));
  }
  wDone.append(rays, el("circle", { cx: "50", cy: "22", r: "9", fill: `url(#${gid}sun)` }));
  const wError = el("g", {});
  const bolt = el("polygon", {
    points: "47,32 55,32 51,39 56,39 46,52 48.5,41 43.5,41",
    fill: `url(#${gid}bo)`, stroke: "#ffffff", "stroke-width": "1", "stroke-linejoin": "round",
  });
  const rain: SVGElement[] = [42, 50, 58].map((x) =>
    el("line", { x1: String(x), y1: "40", x2: String(x - 2), y2: "47", stroke: "#9fc4e8", "stroke-width": "2", "stroke-linecap": "round" }),
  );
  wError.append(cloud(50, 26, 1), bolt, ...rain);
  const wWait = el("g", {});
  const cloudA = cloud(33, 26, 0.8);
  const cloudB = cloud(64, 31, 0.62);
  wWait.append(cloudA, cloudB);
  const wSleep = el("g", {});
  const moon = el("path", {
    d: "M70,15 A11,11 0 1,0 70,33 A8.5,8.5 0 1,1 70,15 Z",
    fill: "#ffe9a8", opacity: "0.95",
  });
  const stars = [sparkle(30, 18, 3), sparkle(42, 12, 2.2), sparkle(24, 30, 1.8)];
  wSleep.append(moon, ...stars);
  const wGroups: Record<MoodName, SVGElement> = {
    idle: wIdle, working: wWork, done: wDone, error: wError, waiting: wWait, sleep: wSleep,
  };
  const weather = el("g", {});
  weather.append(wIdle, wWork, wDone, wError, wWait, wSleep);
  svg.append(defs, cloudBody, face, weather, ring, fx, ...zzz);
  if (host) { host.innerHTML = ""; host.append(svg); }

  // 当前插值状态（从 idle 出发，首帧即趋近目标）。
  const cur = { ...POSES.idle };
  const gaze = { x: 0, y: 0, tx: 0, ty: 0 };
  let lastGazeAt = 0; // 外部注视最后时刻（即兴扫视只在无交互时触发）
  let blinkAt = performance.now() + 4000;
  let blinkUntil = 0;
  let anticAt = performance.now() + 9000 + Math.random() * 9000;
  let shakeUntil = 0;
  let squashFrom = 0; // 上次切表情时刻（挤压拉伸计时起点）
  let squashKind: "dip" | "stretch" = "dip";
  let ringAngle = 0;
  let dead = false;
  let raf = 0;
  const born = performance.now();
  let skyKey = "";
  /** 天空 tint + 天气饰件显隐（mood/color 变化时刷；静态帧同调）。 */
  function refreshSky(): void {
    const key = `${mood}|${color}`;
    if (key === skyKey) return;
    skyKey = key;
    const skyBody = mixc(color, SKY[mood].sky, SKY[mood].mix);
    const stops = grad.querySelectorAll("stop");
    if (stops.length >= 3) {
      stops[0].setAttribute("stop-color", shade(skyBody, 0.45));
      stops[0].setAttribute("stop-opacity", "0.85");
      stops[1].setAttribute("stop-color", skyBody);
      stops[1].setAttribute("stop-opacity", "0.62");
      stops[2].setAttribute("stop-color", shade(skyBody, -0.12));
      stops[2].setAttribute("stop-opacity", "0.72");
    }
    innerShade.setAttribute("fill", shade(skyBody, -0.3));
    moodEmoji.textContent = MOOD_EMOJI[mood];
    (Object.keys(wGroups) as MoodName[]).forEach((m) => {
      wGroups[m].style.display = m === mood ? "" : "none";
    });
  }

  /** 进入 done/error 时的一次性饰件。 */
  function burst(kind: MoodName): void {
    if (!running) return;
    while (fx.firstChild) fx.removeChild(fx.firstChild);
    if (kind === "done") {
      for (let i = 0; i < 10; i++) {
        const a = (i / 10) * Math.PI * 2 + Math.random() * 0.5;
        const c = el("circle", { cx: "50", cy: "50", r: String(2 + Math.random() * 2), fill: eyeColor, opacity: "0.95" });
        c.setAttribute("data-vx", String(Math.cos(a) * (26 + Math.random() * 14)));
        c.setAttribute("data-vy", String(Math.sin(a) * (26 + Math.random() * 14) - 8));
        c.setAttribute("data-born", String(performance.now()));
        fx.append(c);
      }
    }
    if (kind === "error") shakeUntil = performance.now() + 600;
  }
  if (mood === "done" || mood === "error") burst(mood);

  function frame(now: number): void {
    if (dead) return;
    refreshSky();
    const t = POSES[mood];
    // 表情参数插值（k 越大跟随越快；happy 做硬切换，眨眼期强制睁眼参数）。
    const k = 0.18;
    cur.eyeRx += (t.eyeRx - cur.eyeRx) * k;
    cur.eyeRy += (t.eyeRy - cur.eyeRy) * k;
    cur.eyeDy += (t.eyeDy - cur.eyeDy) * k;
    cur.mouthSmile += (t.mouthSmile - cur.mouthSmile) * k;
    cur.scanAmp += (t.scanAmp - cur.scanAmp) * k;
    cur.gazeAmp += (t.gazeAmp - cur.gazeAmp) * k;
    cur.blush += (t.blush - cur.blush) * k;
    cur.worry += (t.worry - cur.worry) * k;
    const showHappy = mood === "done";

    // 眨眼（区间按表情取；sleep 不眨）。
    const blinkRange = BLINK_MS[mood];
    if (blinkRange && now >= blinkAt && !showHappy) {
      blinkUntil = now + 120;
      blinkAt = now + blinkRange[0] + Math.random() * (blinkRange[1] - blinkRange[0]);
    }
    const blinking = now < blinkUntil;
    const ry = (blinking ? cur.eyeRy * 0.1 : cur.eyeRy) * eyeScale;
    const rx = cur.eyeRx * eyeScale;

    // 注视跟随 + waiting 扫读 + idle 微 sway + error 抖动。
    gaze.x += (gaze.tx - gaze.x) * 0.2;
    gaze.y += (gaze.ty - gaze.y) * 0.2;
    // 即兴小动作（9~18s 一次；有外部注视时让路）：随机扫视一瞥。
    if (now >= anticAt) {
      anticAt = now + 9000 + Math.random() * 9000;
      if (now - lastGazeAt > 6000 && (mood === "idle" || mood === "waiting")) {
        gaze.tx = (Math.random() * 2 - 1) * 0.7;
        gaze.ty = (Math.random() * 2 - 1) * 0.5;
      }
    }
    let dx = gaze.x * 4 * cur.gazeAmp;
    let dy = gaze.y * 3 * cur.gazeAmp + cur.eyeDy;
    if (mood === "idle") dy += Math.sin((now - born) / 3000) * 0.8;
    if (cur.scanAmp > 0.1) dx += Math.sin((now - born) / 480) * cur.scanAmp;
    if (now < shakeUntil) dx += Math.sin(now * 0.09) * 3 * ((shakeUntil - now) / 600);

    eyeL.setAttribute("rx", rx.toFixed(2));
    eyeL.setAttribute("ry", Math.max(0.6, ry).toFixed(2));
    eyeR.setAttribute("rx", rx.toFixed(2));
    eyeR.setAttribute("ry", Math.max(0.6, ry).toFixed(2));
    const eyeDisp = showHappy ? "none" : "";
    eyeL.style.display = eyeDisp;
    eyeR.style.display = eyeDisp;
    // 笑眼时高光藏进弧里（弧本身够亮）。
    const glintDisp = showHappy || mood === "sleep" ? "none" : "";
    glintL.style.display = glintDisp;
    glintR.style.display = glintDisp;
    glintL2.style.display = glintDisp;
    glintR2.style.display = glintDisp;
    happyL.style.display = showHappy ? "" : "none";
    happyR.style.display = showHappy ? "" : "none";
    pupils.setAttribute("transform", `translate(${dx.toFixed(2)} ${dy.toFixed(2)})`);

    // 腮红 + 担忧眉跟随表情。
    blushL.setAttribute("opacity", cloudMode ? Math.min(cur.blush, 0.5).toFixed(2) : cur.blush.toFixed(2));
    blushR.setAttribute("opacity", cloudMode ? Math.min(cur.blush, 0.5).toFixed(2) : cur.blush.toFixed(2));
    browL.setAttribute("opacity", cur.worry.toFixed(2));
    browR.setAttribute("opacity", cur.worry.toFixed(2));

    // 嘴：小嘴（smile>0 上扬，<0 下撇）。
    const s = cur.mouthSmile;
    mouth.setAttribute("d", `M44,72 Q50,${(72 + 8 * s).toFixed(1)} 56,72`);

    // 挤压拉伸 + 呼吸（sleep 呼吸更深更慢；眨眼带一点跟随下压）。
    const breatheAmp = mood === "sleep" ? 0.02 : 0.013;
    const breatheRate = mood === "sleep" ? 2600 : 900;
    const breathe = Math.sin((now - born) / breatheRate) * breatheAmp;
    const sp = Math.min(1, (now - squashFrom) / 320);
    const env = Math.sin(Math.PI * sp);
    const dip = squashKind === "dip" ? -0.07 * env : 0.05 * env;
    const blinkDip = blinking ? -0.015 : 0;
    const sy = 1 + breathe + dip + blinkDip;
    const sx = 1 - (breathe + dip + blinkDip) * 0.7; // 体积近似守恒
    face.setAttribute("transform", `translate(50 96) scale(${sx.toFixed(3)} ${sy.toFixed(3)}) translate(-50 -96)`);
    // 云团轻浮（与脸同频不同相，簇感）。
    if (cloudMode) cloudBody.setAttribute("transform", `translate(0 ${(Math.sin((now - born) / 1400) * 1.6).toFixed(2)})`);

    // working 转环。
    if (mood === "working") {
      ringAngle = (ringAngle + 3) % 360;
      ring.setAttribute("opacity", "0.55");
      ring.setAttribute("transform", `rotate(${ringAngle} 50 50)`);
    } else {
      ring.setAttribute("opacity", "0");
    }

    // 天气动效（只跑当前组，便宜）。
    if (mood === "done") {
      rays.setAttribute("transform", `rotate(${(now / 60) % 360} 50 22)`);
    } else if (mood === "working") {
      const off = -(now / 40) % 21;
      wind1.setAttribute("stroke-dashoffset", off.toFixed(1));
      wind2.setAttribute("stroke-dashoffset", (-off).toFixed(1));
    } else if (mood === "error") {
      bolt.setAttribute("opacity", (0.65 + 0.35 * Math.sin(now / 85)).toFixed(2));
      rain.forEach((r, i) => {
        const p = ((now / 14 + i * 9) % 18) / 18;
        r.setAttribute("y1", (38 + p * 12).toFixed(1));
        r.setAttribute("y2", (45 + p * 12).toFixed(1));
        r.setAttribute("opacity", (1 - p * 0.6).toFixed(2));
      });
    } else if (mood === "waiting") {
      // drift 拼在 base transform 之前（单位不被 scale 吃掉）。
      cloudA.setAttribute("transform", `translate(${(Math.sin(now / 900) * 3).toFixed(1)} 0) translate(33 26) scale(0.8)`);
      cloudB.setAttribute("transform", `translate(${(Math.sin(now / 1100 + 2) * -3).toFixed(1)} 0) translate(64 31) scale(0.62)`);
    } else if (mood === "sleep") {
      stars.forEach((s, i) => {
        s.setAttribute("opacity", (0.35 + 0.6 * Math.abs(Math.sin(now / 520 + i * 1.7))).toFixed(2));
      });
    }

    // done 撒花粒子（0.9s 生命）。
    for (const c of Array.from(fx.childNodes) as SVGCircleElement[]) {
      const age = (now - Number(c.getAttribute("data-born"))) / 900;
      if (age >= 1) { fx.removeChild(c); continue; }
      const vx = Number(c.getAttribute("data-vx"));
      const vy = Number(c.getAttribute("data-vy"));
      c.setAttribute("cx", (50 + vx * age).toFixed(1));
      c.setAttribute("cy", (50 + vy * age + 14 * age * age).toFixed(1));
      c.setAttribute("opacity", (0.95 * (1 - age)).toFixed(2));
    }

    // sleep zzz 三枚（2.4s 一轮，依次错峰）。
    if (mood === "sleep") {
      zzz.forEach((z, i) => {
        const p = (((now - born) / 2400) + i / 3) % 1;
        z.setAttribute("opacity", (p < 0.15 ? p / 0.15 : 1 - (p - 0.15) / 0.85).toFixed(2));
        z.setAttribute("y", String(32 - p * 14 - i * 2));
      });
    } else {
      for (const z of zzz) z.setAttribute("opacity", "0");
    }

    raf = requestAnimationFrame(frame);
  }

  /** 静态帧（autostart:false）：按目标参数直接画一遍，不起循环。 */
  function paintStatic(): void {
    refreshSky();
    const t = POSES[mood];
    const showHappy = mood === "done";
    eyeL.setAttribute("rx", String(t.eyeRx * eyeScale));
    eyeL.setAttribute("ry", String(Math.max(0.6, t.eyeRy * eyeScale)));
    eyeR.setAttribute("rx", String(t.eyeRx * eyeScale));
    eyeR.setAttribute("ry", String(Math.max(0.6, t.eyeRy * eyeScale)));
    const eyeDisp = showHappy ? "none" : "";
    eyeL.style.display = eyeDisp;
    eyeR.style.display = eyeDisp;
    const glintDisp = showHappy || mood === "sleep" ? "none" : "";
    glintL.style.display = glintDisp;
    glintR.style.display = glintDisp;
    glintL2.style.display = glintDisp;
    glintR2.style.display = glintDisp;
    happyL.style.display = showHappy ? "" : "none";
    happyR.style.display = showHappy ? "" : "none";
    pupils.setAttribute("transform", `translate(0 ${t.eyeDy})`);
    mouth.setAttribute("d", `M44,72 Q50,${72 + 8 * t.mouthSmile} 56,72`);
    blushL.setAttribute("opacity", cloudMode ? String(Math.min(t.blush, 0.5)) : String(t.blush));
    blushR.setAttribute("opacity", cloudMode ? String(Math.min(t.blush, 0.5)) : String(t.blush));
    browL.setAttribute("opacity", String(t.worry));
    browR.setAttribute("opacity", String(t.worry));
    if (mood === "sleep") {
      zzz.forEach((z, i) => {
        z.setAttribute("opacity", i === 0 ? "0.9" : "0");
        z.setAttribute("y", String(26 - i * 2));
      });
    }
  }

  if (running) {
    raf = requestAnimationFrame(frame);
  } else {
    paintStatic();
  }

  return {
    setMood(m: MoodName): void {      if (dead || !(m in POSES)) return;
      const changed = m !== mood;
      mood = m;
      if (changed) {
        // anticipation：切表情先下蹲（done 则上弹），0.32s 回正。
        squashFrom = performance.now();
        squashKind = m === "done" ? "stretch" : "dip";
        if (running) burst(m);
      }
      if (!running) paintStatic();
    },
    setGaze(nx: number, ny: number): void {
      lastGazeAt = performance.now();
      gaze.tx = clamp1(nx);
      gaze.ty = clamp1(ny);
    },
    setColors(bodyColor: string, eyeColorNew: string): void {
      if (dead) return;
      color = bodyColor;
      eyeColor = eyeColorNew;
      const stops = grad.querySelectorAll("stop");
      if (stops.length >= 3) {
        stops[0].setAttribute("stop-color", shade(color, 0.45));
        stops[1].setAttribute("stop-color", color);
        stops[2].setAttribute("stop-color", shade(color, -0.12));
      }
      innerShade.setAttribute("fill", shade(color, -0.3));
      const ink = cloudMode ? shade(eyeColor, -0.35) : eyeColor;
      const glintFill = cloudMode ? "#ffffff" : eyeColor;
      for (const n of [eyeL, eyeR]) n.setAttribute("fill", ink);
      for (const n of [mouth, browL, browR, happyL, happyR]) {
        n.setAttribute("stroke", ink);
      }
      ring.setAttribute("stroke", eyeColor);
      for (const n of [glintL, glintR, glintL2, glintR2]) {
        n.setAttribute("fill", glintFill);
      }
      for (const n of zzz) {
        n.setAttribute("fill", eyeColor);
      }
      if (!running) paintStatic();
    },
    destroy(): void {
      dead = true;
      cancelAnimationFrame(raf);
      svg.remove();
    },
  };
}
