// ─── NeoTrix SVG Morphing Engine ─────────────────────────────────────────────
// Path morphing, shape transitions, attribute animation via native Web API
// Zero external dependencies

import { Timeline, ease, lerp, lerpColor } from './animation';

export interface MorphTarget {
  selector: string;
  attrs: Record<string, string | number>;
  duration: number;
  delay?: number;
  easing?: string;
}

export interface SVGPathPoint {
  x: number;
  y: number;
  type: 'M' | 'L' | 'C' | 'Q' | 'Z';
  c1?: { x: number; y: number };
  c2?: { x: number; y: number };
}

// ─── Path parsing (SVG path d attribute → normalized segments) ───────────

export function parsePath(d: string): SVGPathPoint[] {
  const commands = d.match(/[MLQCZmlqcz][^MLQCZmlqcz]*/g) || [];
  const points: SVGPathPoint[] = [];
  let currentX = 0;
  let currentY = 0;

  for (const cmd of commands) {
    const type = cmd[0].toUpperCase() as SVGPathPoint['type'];
    const nums = cmd.slice(1).trim().split(/[\s,]+/).map(Number).filter((n) => !isNaN(n));
    const isLower = cmd[0] === cmd[0].toLowerCase() && cmd[0] !== cmd[0].toUpperCase();
    const relative = isLower;

    if (type === 'M' && nums.length >= 2) {
      currentX = relative ? currentX + nums[0] : nums[0];
      currentY = relative ? currentY + nums[1] : nums[1];
      points.push({ type: 'M', x: currentX, y: currentY });
    } else if (type === 'L' && nums.length >= 2) {
      currentX = relative ? currentX + nums[0] : nums[0];
      currentY = relative ? currentY + nums[1] : nums[1];
      points.push({ type: 'L', x: currentX, y: currentY });
    } else if (type === 'C' && nums.length >= 6) {
      const c1x = relative ? currentX + nums[0] : nums[0];
      const c1y = relative ? currentY + nums[1] : nums[1];
      const c2x = relative ? currentX + nums[2] : nums[2];
      const c2y = relative ? currentY + nums[3] : nums[3];
      currentX = relative ? currentX + nums[4] : nums[4];
      currentY = relative ? currentY + nums[5] : nums[5];
      points.push({ type: 'C', x: currentX, y: currentY, c1: { x: c1x, y: c1y }, c2: { x: c2x, y: c2y } });
    } else if (type === 'Q' && nums.length >= 4) {
      const c1x = relative ? currentX + nums[0] : nums[0];
      const c1y = relative ? currentY + nums[1] : nums[1];
      currentX = relative ? currentX + nums[2] : nums[2];
      currentY = relative ? currentY + nums[3] : nums[3];
      points.push({ type: 'Q', x: currentX, y: currentY, c1: { x: c1x, y: c1y } });
    } else if (type === 'Z') {
      points.push({ type: 'Z', x: currentX, y: currentY });
    }
  }
  return points;
}

export function serializePath(points: SVGPathPoint[]): string {
  return points.map((p) => {
    switch (p.type) {
      case 'M': return `M ${p.x} ${p.y}`;
      case 'L': return `L ${p.x} ${p.y}`;
      case 'C': return `C ${p.c1!.x} ${p.c1!.y} ${p.c2!.x} ${p.c2!.y} ${p.x} ${p.y}`;
      case 'Q': return `Q ${p.c1!.x} ${p.c1!.y} ${p.x} ${p.y}`;
      case 'Z': return 'Z';
    }
  }).join(' ');
}

// Normalize two paths to have the same number of points
function normalizePaths(a: SVGPathPoint[], b: SVGPathPoint[]): [SVGPathPoint[], SVGPathPoint[]] {
  const maxLen = Math.max(a.length, b.length);
  const fill = (arr: SVGPathPoint[], len: number): SVGPathPoint[] => {
    if (arr.length >= len) return arr;
    const last = arr[arr.length - 1] || { type: 'L' as const, x: 0, y: 0 };
    return [...arr, ...Array(len - arr.length).fill(null).map(() => ({ ...last }))];
  };
  return [fill(a, maxLen), fill(b, maxLen)];
}

// ─── SVG Element Morphing ──────────────────────────────────────────────────

export function morphPath(
  element: SVGPathElement,
  fromD: string,
  toD: string,
  duration: number,
  easing?: string,
): Promise<void> {
  const fromPoints = parsePath(fromD);
  const toPoints = parsePath(toD);
  const [normFrom, normTo] = normalizePaths(fromPoints, toPoints);

  return new Promise((resolve) => {
    const timeline = new Timeline();
    timeline.onFrame((_id, values) => {
      const t = values.t;
      const current: SVGPathPoint[] = normFrom.map((fp, i) => {
        const tp = normTo[i];
        return {
          type: tp.type,
          x: lerp(fp.x, tp.x, t),
          y: lerp(fp.y, tp.y, t),
          ...(fp.c1 && tp.c1 ? { c1: { x: lerp(fp.c1.x, tp.c1.x, t), y: lerp(fp.c1.y, tp.c1.y, t) } } : {}),
          ...(fp.c2 && tp.c2 ? { c2: { x: lerp(fp.c2.x, tp.c2.x, t), y: lerp(fp.c2.y, tp.c2.y, t) } } : {}),
        };
      });
      element.setAttribute('d', serializePath(current));
    });

    timeline.add({
      id: 'morph',
      from: { t: 0 },
      to: { t: 1 },
      duration,
      easing: easing || 'outCubic',
      onComplete: () => resolve(),
    });
    timeline.start();
  });
}

// ─── SVG Attribute Morphing ─────────────────────────────────────────────────

export function morphAttrs(
  element: SVGElement,
  fromAttrs: Record<string, string | number>,
  toAttrs: Record<string, string | number>,
  duration: number,
  easing?: string,
): Promise<void> {
  // Set initial attributes
  for (const [key, val] of Object.entries(fromAttrs)) {
    element.setAttribute(key, String(val));
  }

  // For color attributes, use color lerping
  const colorKeys = ['fill', 'stroke', 'color', 'stop-color'];
  type MorphKey = { key: string; fromVal: number; toVal: number; isColor: boolean; colorFrom?: [number, number, number]; colorTo?: [number, number, number] };

  const parseColor = (s: string): [number, number, number] => {
    const rgb = s.match(/\d+/g);
    if (rgb && rgb.length >= 3) return [parseInt(rgb[0]), parseInt(rgb[1]), parseInt(rgb[2])];
    return [128, 128, 128];
  };

  const keys: MorphKey[] = [];
  for (const [key, toVal] of Object.entries(toAttrs)) {
    const fromVal = fromAttrs[key];
    if (colorKeys.includes(key)) {
      keys.push({
        key,
        fromVal: 0,
        toVal: 0,
        isColor: true,
        colorFrom: parseColor(String(fromVal)),
        colorTo: parseColor(String(toVal)),
      });
    } else {
      keys.push({
        key,
        fromVal: Number(fromVal) || 0,
        toVal: Number(toVal) || 0,
        isColor: false,
      });
    }
  }

  return new Promise((resolve) => {
    const timeline = new Timeline();
    timeline.onFrame((_id, values) => {
      const t = values.t;
      for (const k of keys) {
        if (k.isColor && k.colorFrom && k.colorTo) {
          const r = Math.round(lerp(k.colorFrom[0], k.colorTo[0], t));
          const g = Math.round(lerp(k.colorFrom[1], k.colorTo[1], t));
          const b = Math.round(lerp(k.colorFrom[2], k.colorTo[2], t));
          element.setAttribute(k.key, `rgb(${r},${g},${b})`);
        } else {
          const val = lerp(k.fromVal, k.toVal, t);
          element.setAttribute(k.key, String(val));
        }
      }
    });

    timeline.add({
      id: 'attr_morph',
      from: { t: 0 },
      to: { t: 1 },
      duration,
      easing: easing || 'outCubic',
      onComplete: () => resolve(),
    });
    timeline.start();
  });
}

// ─── CSS Transition via Web Animations API ──────────────────────────────────

export function animateElement(
  element: Element,
  keyframes: Keyframe[],
  options: KeyframeAnimationOptions,
): Animation {
  return element.animate(keyframes, options);
}

// ─── DOM element tween (generic) ────────────────────────────────────────────

export function tweenValue(
  from: number,
  to: number,
  duration: number,
  onUpdate: (value: number) => void,
  easing?: string,
): Promise<void> {
  return new Promise((resolve) => {
    const timeline = new Timeline();
    timeline.onFrame((_id, values) => {
      onUpdate(values.v);
    });
    timeline.add({
      id: 'tween',
      from: { v: from },
      to: { v: to },
      duration,
      easing,
      onComplete: () => resolve(),
    });
    timeline.start();
  });
}

// ─── SVG Drawing Animation (stroke-dashoffset reveal) ──────────────────────

export function drawPath(
  element: SVGPathElement,
  duration: number,
  easing?: string,
): Promise<void> {
  const length = element.getTotalLength();
  element.setAttribute('stroke-dasharray', String(length));
  element.setAttribute('stroke-dashoffset', String(length));

  return tweenValue(length, 0, duration, (v) => {
    element.setAttribute('stroke-dashoffset', String(v));
  }, easing);
}
