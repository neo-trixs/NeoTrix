// ─── NeoTrix Animation Engine ────────────────────────────────────────────────
// Self-built: easing curves, spring physics, timeline orchestration, frame loop
// Zero external dependencies — all math is native

// ─── Easing Functions ────────────────────────────────────────────────────────

export type EasingFn = (t: number) => number;

export const ease: Record<string, EasingFn> = {
  linear: (t) => t,

  // Quad
  inQuad: (t) => t * t,
  outQuad: (t) => t * (2 - t),
  inOutQuad: (t) => t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t,

  // Cubic
  inCubic: (t) => t * t * t,
  outCubic: (t) => --t * t * t + 1,
  inOutCubic: (t) => t < 0.5 ? 4 * t * t * t : (t - 1) * (2 * t - 2) * (2 * t - 2) + 1,

  // Quart
  inQuart: (t) => t * t * t * t,
  outQuart: (t) => 1 - --t * t * t * t,
  inOutQuart: (t) => t < 0.5 ? 8 * t * t * t * t : 1 - 8 * --t * t * t * t,

  // Quint
  inQuint: (t) => t * t * t * t * t,
  outQuint: (t) => 1 + --t * t * t * t * t,
  inOutQuint: (t) => t < 0.5 ? 16 * t * t * t * t * t : 1 + 16 * --t * t * t * t * t,

  // Sine
  inSine: (t) => 1 - Math.cos(t * Math.PI / 2),
  outSine: (t) => Math.sin(t * Math.PI / 2),
  inOutSine: (t) => -(Math.cos(Math.PI * t) - 1) / 2,

  // Expo
  inExpo: (t) => t === 0 ? 0 : Math.pow(2, 10 * (t - 1)),
  outExpo: (t) => t === 1 ? 1 : 1 - Math.pow(2, -10 * t),
  inOutExpo: (t) => {
    if (t === 0 || t === 1) return t;
    return t < 0.5
      ? Math.pow(2, 10 * (2 * t - 1)) / 2
      : (2 - Math.pow(2, -10 * (2 * t - 1))) / 2;
  },

  // Elastic
  inElastic: (t) => {
    if (t === 0 || t === 1) return t;
    return -Math.pow(2, 10 * (t - 1)) * Math.sin((t - 1.1) * 5 * Math.PI);
  },
  outElastic: (t) => {
    if (t === 0 || t === 1) return t;
    return Math.pow(2, -10 * t) * Math.sin((t - 0.1) * 5 * Math.PI) + 1;
  },
  inOutElastic: (t) => {
    if (t === 0 || t === 1) return t;
    t *= 2;
    if (t < 1) return -0.5 * Math.pow(2, 10 * (t - 1)) * Math.sin((t - 1.1) * 5 * Math.PI);
    return 0.5 * Math.pow(2, -10 * (t - 1)) * Math.sin((t - 1.1) * 5 * Math.PI) + 1;
  },

  // Back
  inBack: (t) => t * t * (2.70158 * t - 1.70158),
  outBack: (t) => 1 + --t * t * (2.70158 * t + 1.70158),
  inOutBack: (t) => {
    const s = 1.70158 * 1.525;
    t *= 2;
    if (t < 1) return 0.5 * (t * t * ((s + 1) * t - s));
    return 0.5 * ((t -= 2) * t * ((s + 1) * t + s) + 2);
  },

  // Bounce
  outBounce: (t) => {
    if (t < 1 / 2.75) return 7.5625 * t * t;
    if (t < 2 / 2.75) return 7.5625 * (t -= 1.5 / 2.75) * t + 0.75;
    if (t < 2.5 / 2.75) return 7.5625 * (t -= 2.25 / 2.75) * t + 0.9375;
    return 7.5625 * (t -= 2.625 / 2.75) * t + 0.984375;
  },
  inBounce: (t) => 1 - ease.outBounce(1 - t),
  inOutBounce: (t) => t < 0.5
    ? (1 - ease.outBounce(1 - 2 * t)) / 2
    : (1 + ease.outBounce(2 * t - 1)) / 2,
};

// ─── Spring Physics (Damped Harmonic Oscillator) ──────────────────────────

export interface SpringConfig {
  stiffness: number;
  damping: number;
  mass: number;
}

export function springValue(
  from: number,
  to: number,
  config: SpringConfig,
  t: number,
): number {
  const { stiffness, damping, mass } = config;
  const delta = to - from;
  const omega = Math.sqrt(stiffness / mass);
  const zeta = damping / (2 * Math.sqrt(stiffness * mass));

  if (zeta < 1) {
    const wd = omega * Math.sqrt(1 - zeta * zeta);
    const A = -delta;
    const B = (zeta * omega * A) / wd;
    return to + Math.exp(-zeta * omega * t) * (A * Math.cos(wd * t) + B * Math.sin(wd * t));
  } else {
    const r = -omega * zeta;
    const c1 = -delta;
    const c2 = -r * c1;
    return to + (c1 + c2 * t) * Math.exp(r * t);
  }
}

export function springDuration(config: SpringConfig, threshold = 0.001): number {
  const { stiffness, damping, mass } = config;
  const omega = Math.sqrt(stiffness / mass);
  const zeta = damping / (2 * Math.sqrt(stiffness * mass));
  const decay = zeta * omega;
  if (decay <= 0) return 2;
  return Math.log(threshold) / -decay;
}

// ─── Interpolation ───────────────────────────────────────────────────────────

export type LerpFn = (from: number, to: number, t: number) => number;

export function lerp(from: number, to: number, t: number): number {
  return from + (to - from) * t;
}

export function lerpColor(from: [number, number, number], to: [number, number, number], t: number): string {
  const r = Math.round(lerp(from[0], to[0], t));
  const g = Math.round(lerp(from[1], to[1], t));
  const b = Math.round(lerp(from[2], to[2], t));
  return `rgb(${r},${g},${b})`;
}

export function lerpPath(from: string, to: string, t: number): string {
  const parsePoints = (s: string): number[] =>
    s.trim().split(/[\s,]+/).map(Number).filter((n) => !isNaN(n));
  const a = parsePoints(from);
  const b = parsePoints(to);
  const len = Math.max(a.length, b.length);
  const result: number[] = [];
  for (let i = 0; i < len; i++) {
    result.push(lerp(a[i] ?? a[a.length - 1] ?? 0, b[i] ?? b[b.length - 1] ?? 0, t));
  }
  return result.join(' ');
}

// ─── Timeline ────────────────────────────────────────────────────────────────

export interface Keyframe {
  target: string;
  props: Record<string, number | string>;
  duration: number;
  delay?: number;
  easing?: EasingFn | string;
  onStart?: () => void;
  onComplete?: () => void;
}

export interface AnimState {
  id: string;
  from: Record<string, number>;
  to: Record<string, number>;
  duration: number;
  delay: number;
  elapsed: number;
  easing: EasingFn;
  running: boolean;
  onComplete?: () => void;
}

export type AnimTarget = Record<string, number | string>;

export class Timeline {
  private anims: AnimState[] = [];
  private rafId: number | null = null;
  private lastTime = 0;
  private onUpdate: ((id: string, values: Record<string, number>) => void) | null = null;

  onFrame(cb: (id: string, values: Record<string, number>) => void): void {
    this.onUpdate = cb;
  }

  add(config: {
    id: string;
    from: Record<string, number>;
    to: Record<string, number>;
    duration: number;
    delay?: number;
    easing?: EasingFn | string;
    onComplete?: () => void;
  }): void {
    const easingFn: EasingFn =
      typeof config.easing === 'string'
        ? ease[config.easing] || ease.outCubic
        : config.easing || ease.outCubic;

    this.anims.push({
      id: config.id,
      from: { ...config.from },
      to: { ...config.to },
      duration: config.duration,
      delay: config.delay || 0,
      elapsed: -(config.delay || 0),
      easing: easingFn,
      running: true,
      onComplete: config.onComplete,
    });
  }

  start(): void {
    if (this.rafId !== null) return;
    this.lastTime = performance.now();
    const loop = (now: number) => {
      const dt = (now - this.lastTime) / 1000;
      this.lastTime = now;
      this.tick(dt);
      if (this.anims.some((a) => a.running)) {
        this.rafId = requestAnimationFrame(loop);
      } else {
        this.rafId = null;
      }
    };
    this.rafId = requestAnimationFrame(loop);
  }

  stop(): void {
    if (this.rafId !== null) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
    this.anims = [];
  }

  pause(): void {
    if (this.rafId !== null) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
  }

  resume(): void {
    if (this.anims.some((a) => a.running)) {
      this.lastTime = performance.now();
      this.start();
    }
  }

  private tick(dt: number): void {
    const active = this.anims.filter((a) => a.running);
    for (const a of active) {
      a.elapsed += dt * 1000;
      if (a.elapsed < 0) continue;

      const t = Math.min(a.elapsed / a.duration, 1);
      const eased = a.easing(t);

      if (this.onUpdate) {
        const values: Record<string, number> = {};
        for (const key of Object.keys(a.from)) {
          values[key] = lerp(a.from[key], a.to[key], eased);
        }
        this.onUpdate(a.id, values);
      }

      if (t >= 1) {
        a.running = false;
        a.onComplete?.();
      }
    }
  }

  get running(): boolean {
    return this.rafId !== null;
  }
}

// ─── Stagger ─────────────────────────────────────────────────────────────────

export function stagger(
  baseDelay: number,
  options?: { from?: 'start' | 'center' | 'end'; grid?: [number, number] },
): (index: number) => number {
  const { from = 'start', grid } = options || {};
  return (index: number) => {
    if (grid) {
      const col = index % grid[0];
      const row = Math.floor(index / grid[0]);
      const cx = (grid[0] - 1) / 2;
      const cy = (grid[1] - 1) / 2;
      const dist = Math.sqrt((col - cx) ** 2 + (row - cy) ** 2);
      return baseDelay * dist;
    }
    if (from === 'center') {
      const mid = index / 2;
      return baseDelay * Math.abs(index - mid);
    }
    if (from === 'end') return baseDelay * (1 - index);
    return baseDelay * index;
  };
}

// ─── Frame Loop (Canvas/requestAnimationFrame) ───────────────────────────────

export interface FrameRenderer {
  (ctx: CanvasRenderingContext2D, t: number, dt: number): void;
}

export class FrameLoop {
  private rafId: number | null = null;
  private lastTime = 0;
  private elapsed = 0;
  private renderer: FrameRenderer;
  private canvas: HTMLCanvasElement;

  constructor(canvas: HTMLCanvasElement, renderer: FrameRenderer) {
    this.canvas = canvas;
    this.renderer = renderer;
  }

  start(): void {
    if (this.rafId !== null) return;
    this.lastTime = performance.now();
    this.elapsed = 0;
    const loop = (now: number) => {
      const dt = (now - this.lastTime) / 1000;
      this.lastTime = now;
      this.elapsed += dt;
      const ctx = this.canvas.getContext('2d');
      if (ctx) {
        ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        this.renderer(ctx, this.elapsed, dt);
      }
      this.rafId = requestAnimationFrame(loop);
    };
    this.rafId = requestAnimationFrame(loop);
  }

  stop(): void {
    if (this.rafId !== null) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
  }
}
