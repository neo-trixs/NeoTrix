// ─── NeoTrix Frame Sequence Engine ───────────────────────────────────────────
// Canvas 2D frame rendering, frame interpolation, sprite animation
// Zero external dependencies

import { Timeline, ease, lerp, type EasingFn } from './animation';

// ─── Frame Data Types ────────────────────────────────────────────────────────

export interface Frame {
  data: ImageData | HTMLImageElement | string;
  duration: number;
  easing?: string | EasingFn;
}

export interface FrameSequenceConfig {
  frames: Frame[];
  loop?: boolean;
  width: number;
  height: number;
  onFrame?: (index: number) => void;
  onComplete?: () => void;
}

// ─── Frame Sequence Player ──────────────────────────────────────────────────

export class FrameSequencePlayer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private config: FrameSequenceConfig;
  private timeline: Timeline;
  private currentIndex = 0;
  private loadedImages: Map<string, HTMLImageElement> = new Map();
  private running = false;

  constructor(canvas: HTMLCanvasElement, config: FrameSequenceConfig) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d')!;
    this.config = config;
    this.canvas.width = config.width;
    this.canvas.height = config.height;
    this.timeline = new Timeline();
  }

  async start(): Promise<void> {
    this.running = true;
    await this.preloadImages();
    this.playSequence();
  }

  stop(): void {
    this.running = false;
    this.timeline.stop();
  }

  private async preloadImages(): Promise<void> {
    const promises = this.config.frames.map((frame) => {
      if (typeof frame.data === 'string' && !this.loadedImages.has(frame.data)) {
        return new Promise<void>((resolve) => {
          const img = new Image();
          img.onload = () => {
            this.loadedImages.set(frame.data as string, img);
            resolve();
          };
          img.onerror = () => resolve();
          img.src = frame.data as string;
        });
      }
      return Promise.resolve();
    });
    await Promise.all(promises);
  }

  private playSequence(): void {
    if (!this.running || this.currentIndex >= this.config.frames.length) {
      if (this.config.loop && this.running) {
        this.currentIndex = 0;
        this.playSequence();
      } else {
        this.config.onComplete?.();
      }
      return;
    }

    const frame = this.config.frames[this.currentIndex];
    const nextFrame = this.config.frames[this.currentIndex + 1];
    this.config.onFrame?.(this.currentIndex);

    if (nextFrame && frame.duration > 0) {
      // Crossfade between frames
      this.crossfade(frame, nextFrame, frame.duration, frame.easing);
    } else {
      // Single frame display
      this.renderFrame(frame);
      setTimeout(() => {
        this.currentIndex++;
        this.playSequence();
      }, frame.duration);
    }
  }

  private async crossfade(
    from: Frame,
    to: Frame,
    duration: number,
    easing?: string | EasingFn,
  ): Promise<void> {
    // Render "from" first
    this.renderFrame(from);
    const fromImageData = this.ctx.getImageData(0, 0, this.canvas.width, this.canvas.height);

    // Render "to" for the blend target
    this.renderFrame(to);
    const toImageData = this.ctx.getImageData(0, 0, this.canvas.width, this.canvas.height);

    const easeFn: EasingFn =
      typeof easing === 'string' ? (ease[easing] || ease.outCubic) :
      easing || ease.outCubic;

    const blend = (t: number) => {
      const data = this.ctx.createImageData(this.canvas.width, this.canvas.height);
      const et = easeFn(t);
      for (let i = 0; i < data.data.length; i++) {
        data.data[i] = fromImageData.data[i] * (1 - et) + toImageData.data[i] * et;
      }
      this.ctx.putImageData(data, 0, 0);
    };

    await new Promise<void>((resolve) => {
      const tl = new Timeline();
      tl.onFrame((_id, values) => blend(values.t));
      tl.add({
        id: 'xfade',
        from: { t: 0 },
        to: { t: 1 },
        duration,
        onComplete: () => {
          this.currentIndex++;
          resolve();
        },
      });
      tl.start();
    });

    this.playSequence();
  }

  private renderFrame(frame: Frame): void {
    if (typeof frame.data === 'string') {
      const img = this.loadedImages.get(frame.data);
      if (img) {
        this.ctx.drawImage(img, 0, 0, this.canvas.width, this.canvas.height);
      }
    } else if (frame.data instanceof HTMLImageElement) {
      this.ctx.drawImage(frame.data, 0, 0, this.canvas.width, this.canvas.height);
    } else if (frame.data instanceof ImageData) {
      this.ctx.putImageData(frame.data, 0, 0);
    }
  }
}

// ─── Sprite Animation ────────────────────────────────────────────────────────

export interface SpriteSheetConfig {
  image: HTMLImageElement | string;
  frameWidth: number;
  frameHeight: number;
  columns: number;
  rows: number;
  totalFrames: number;
  fps: number;
  loop?: boolean;
}

export class SpriteAnimator {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private config: SpriteSheetConfig;
  private image: HTMLImageElement | null = null;
  private currentFrame = 0;
  private rafId: number | null = null;
  private lastTime = 0;
  private running = false;

  constructor(canvas: HTMLCanvasElement, config: SpriteSheetConfig) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d')!;
    this.config = config;
    this.canvas.width = config.frameWidth;
    this.canvas.height = config.frameHeight;
  }

  async load(): Promise<void> {
    if (typeof this.config.image === 'string') {
      const img = new Image();
      img.src = this.config.image;
      await new Promise((resolve, reject) => {
        img.onload = resolve;
        img.onerror = reject;
      });
      this.image = img;
    } else {
      this.image = this.config.image;
    }
  }

  start(): void {
    if (!this.image) return;
    this.running = true;
    this.lastTime = performance.now();
    const loop = (now: number) => {
      if (!this.running) return;
      const dt = now - this.lastTime;
      this.lastTime = now;

      const frameDuration = 1000 / this.config.fps;
      const framesToAdvance = Math.floor(dt / frameDuration);
      if (framesToAdvance > 0) {
        this.currentFrame += framesToAdvance;
        if (this.currentFrame >= this.config.totalFrames) {
          if (this.config.loop) {
            this.currentFrame = this.currentFrame % this.config.totalFrames;
          } else {
            this.currentFrame = this.config.totalFrames - 1;
          }
        }
      }

      this.render();
      this.rafId = requestAnimationFrame(loop);
    };
    this.rafId = requestAnimationFrame(loop);
  }

  stop(): void {
    this.running = false;
    if (this.rafId !== null) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
  }

  private render(): void {
    if (!this.image) return;
    const col = this.currentFrame % this.config.columns;
    const row = Math.floor(this.currentFrame / this.config.columns);
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    this.ctx.drawImage(
      this.image,
      col * this.config.frameWidth,
      row * this.config.frameHeight,
      this.config.frameWidth,
      this.config.frameHeight,
      0, 0,
      this.canvas.width,
      this.canvas.height,
    );
  }
}

// ─── Frame Interpolation (between two ImageData) ────────────────────────────

export function interpolateFrames(
  from: ImageData,
  to: ImageData,
  t: number,
): ImageData {
  const w = from.width;
  const h = from.height;
  const result = new ImageData(w, h);
  for (let i = 0; i < result.data.length; i++) {
    result.data[i] = from.data[i] * (1 - t) + to.data[i] * t;
  }
  return result;
}

// ─── Canvas to ImageData capture ────────────────────────────────────────────

export function captureFrame(canvas: HTMLCanvasElement): ImageData {
  const ctx = canvas.getContext('2d')!;
  return ctx.getImageData(0, 0, canvas.width, canvas.height);
}

// ─── Frame Sequence Generator (procedural) ──────────────────────────────────

export interface ProceduralFrameGenerator {
  (t: number, ctx: CanvasRenderingContext2D, w: number, h: number): void;
}

export function generateFrameSequence(
  canvas: HTMLCanvasElement,
  generator: ProceduralFrameGenerator,
  totalFrames: number,
  fps: number,
  loop?: boolean,
): void {
  const ctx = canvas.getContext('2d')!;
  const w = canvas.width;
  const h = canvas.height;
  let frame = 0;
  let lastTime = performance.now();
  const frameDuration = 1000 / fps;

  const render = (now: number) => {
    const dt = now - lastTime;
    lastTime = now;

    const advance = Math.floor(dt / frameDuration);
    if (advance > 0) {
      frame += advance;
      if (frame >= totalFrames) {
        if (loop) frame = frame % totalFrames;
        else frame = totalFrames - 1;
      }
    }

    ctx.clearRect(0, 0, w, h);
    generator(frame / totalFrames, ctx, w, h);
    requestAnimationFrame(render);
  };

  requestAnimationFrame(render);
}
