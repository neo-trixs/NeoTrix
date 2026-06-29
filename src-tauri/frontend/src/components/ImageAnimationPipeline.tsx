import { useEffect, useRef, useState, useCallback } from 'react';
import { Timeline, type EasingFn, ease, FrameLoop, type FrameRenderer } from '../engine/animation';
import { morphPath, morphAttrs, drawPath, parsePath, serializePath } from '../engine/svg-morph';
import { FrameSequencePlayer, type Frame, type ProceduralFrameGenerator, generateFrameSequence } from '../engine/frame-sequence';

// ─── Types ───────────────────────────────────────────────────────────────────

export interface T2IInput {
  prompt: string;
  width?: number;
  height?: number;
  seed?: number;
  style?: 'fractal' | 'geometric' | 'perlin' | 'combined';
}

export interface AnimationPreset {
  type: 'morph' | 'crossfade' | 'draw' | 'particle' | 'procedural' | 'scroll';
  duration: number;
  easing?: string;
}

export interface I2VConfig {
  sourceImages: string[];
  animationType: AnimationPreset['type'];
  duration: number;
  loop?: boolean;
  width: number;
  height: number;
}

// ─── SVG Template Generator (from T2I output) ────────────────────────────────

function generateSVGFromT2I(config: T2IInput): string {
  const { prompt, width = 512, height = 512 } = config;
  const lower = prompt.toLowerCase();
  const w = width;
  const h = height;

  // Map T2I prompt to SVG primitives
  if (lower.includes('circle') || lower.includes('ball') || lower.includes('orb')) {
    return `<svg width="${w}" height="${h}" xmlns="http://www.w3.org/2000/svg">
      <defs>
        <radialGradient id="g1" cx="40%" cy="40%">
          <stop offset="0%" stop-color="#ff6b6b"/>
          <stop offset="100%" stop-color="#c0392b"/>
        </radialGradient>
      </defs>
      <rect width="${w}" height="${h}" fill="#1a1a2e"/>
      <circle id="morph-target" cx="${w/2}" cy="${h/2}" r="${Math.min(w,h)*0.3}" fill="url(#g1)"/>
      <circle cx="${w*0.3}" cy="${h*0.3}" r="4" fill="white" opacity="0.8"/>
    </svg>`;
  }

  if (lower.includes('wave') || lower.includes('wave') || lower.includes('ocean') || lower.includes('water')) {
    return `<svg width="${w}" height="${h}" xmlns="http://www.w3.org/2000/svg">
      <defs>
        <linearGradient id="ocean" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color="#006994"/>
          <stop offset="100%" stop-color="#003d5c"/>
        </linearGradient>
      </defs>
      <rect width="${w}" height="${h}" fill="url(#ocean)"/>
      <path id="morph-target" d="M0 ${h*0.6} Q${w*0.25} ${h*0.4} ${w*0.5} ${h*0.6} T${w} ${h*0.6}" fill="none" stroke="#00bfff" stroke-width="3" opacity="0.8"/>
      <path d="M0 ${h*0.7} Q${w*0.25} ${h*0.5} ${w*0.5} ${h*0.7} T${w} ${h*0.7}" fill="none" stroke="#0099cc" stroke-width="2" opacity="0.5"/>
    </svg>`;
  }

  if (lower.includes('star') || lower.includes('space') || lower.includes('galaxy') || lower.includes('cosmos')) {
    return `<svg width="${w}" height="${h}" xmlns="http://www.w3.org/2000/svg">
      <rect width="${w}" height="${h}" fill="#0a0a1a"/>
      <g id="morph-target">
        <polygon points="${w/2},${h*0.2} ${w/2+20},${h*0.4} ${w/2+50},${h*0.45} ${w/2+30},${h*0.65} ${w/2+35},${h*0.9} ${w/2},${h*0.75} ${w/2-35},${h*0.9} ${w/2-30},${h*0.65} ${w/2-50},${h*0.45} ${w/2-20},${h*0.4}" fill="#ffd700" opacity="0.9"/>
      </g>
      <circle cx="${w*0.2}" cy="${h*0.3}" r="2" fill="white" opacity="0.7"/>
      <circle cx="${w*0.8}" cy="${h*0.2}" r="1.5" fill="white" opacity="0.5"/>
      <circle cx="${w*0.5}" cy="${h*0.1}" r="1" fill="white" opacity="0.6"/>
    </svg>`;
  }

  // Default: geometric abstract
  return `<svg width="${w}" height="${h}" xmlns="http://www.w3.org/2000/svg">
    <rect width="${w}" height="${h}" fill="#1a1a2e"/>
    <polygon id="morph-target" points="${w/2},${h*0.2} ${w*0.8},${h*0.5} ${w/2},${h*0.8} ${w*0.2},${h*0.5}" fill="#e94560" opacity="0.8"/>
  </svg>`;
}

// ─── Generate morph targets from prompt ──────────────────────────────────────

function generateMorphTargets(prompt: string): string[] {
  const lower = prompt.toLowerCase();
  if (lower.includes('circle') || lower.includes('ball')) {
    return [
      'M 100,100 Q 150,50 200,100 Q 250,150 200,200 Q 150,250 100,200 Q 50,150 100,100',
      'M 150,100 L 200,150 L 150,200 L 100,150 Z',
      'M 100,100 Q 150,200 200,100 Q 250,200 200,300 Q 150,400 100,300 Q 50,200 100,100',
    ];
  }
  if (lower.includes('star')) {
    return [
      'M 150,50 L 180,130 L 260,130 L 195,180 L 220,260 L 150,210 L 80,260 L 105,180 L 40,130 L 120,130 Z',
      'M 150,50 Q 200,100 260,130 Q 200,160 220,260 Q 150,200 80,260 Q 100,160 40,130 Q 100,100 150,50',
      'M 150,30 Q 200,130 260,140 Q 210,180 230,270 Q 150,220 70,270 Q 90,180 40,140 Q 100,130 150,30',
    ];
  }
  // Default morph sequence
  return [
    'M 100,100 L 200,100 L 200,200 L 100,200 Z',
    'M 150,50 L 250,100 L 200,250 L 100,200 Z',
    'M 100,100 Q 200,50 250,150 Q 200,250 100,200 Q 50,150 100,100',
    'M 100,100 L 200,100 Q 250,150 200,200 L 100,200 Q 50,150 100,100',
  ];
}

// ─── Pipeline States ─────────────────────────────────────────────────────────

type PipelinePhase = 'idle' | 'generating' | 'svg-render' | 'animating' | 'complete' | 'error';

interface PipelineState {
  phase: PipelinePhase;
  message: string;
  progress: number;
}

// ─── Main Component ──────────────────────────────────────────────────────────

interface ImageAnimationPipelineProps {
  prompt?: string;
  width?: number;
  height?: number;
  autoStart?: boolean;
  animationPreset?: AnimationPreset;
  onPhaseChange?: (phase: PipelinePhase) => void;
  svgContent?: string;
  generatedBase64?: string;
}

export default function ImageAnimationPipeline({
  prompt = '',
  width = 512,
  height = 512,
  autoStart = false,
  animationPreset = { type: 'morph', duration: 2000, easing: 'outCubic' },
  onPhaseChange,
  svgContent,
  generatedBase64,
}: ImageAnimationPipelineProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const targetRef = useRef<SVGElement | null>(null);
  const timelineRef = useRef<Timeline | null>(null);
  const playerRef = useRef<FrameSequencePlayer | null>(null);
  const [phase, setPhase] = useState<PipelinePhase>('idle');
  const [state, setState] = useState<PipelineState>({ phase: 'idle', message: '', progress: 0 });
  const [svgOutput, setSvgOutput] = useState<string>('');

  const updatePhase = useCallback((p: PipelinePhase, msg = '', progress = 0) => {
    setPhase(p);
    setState({ phase: p, message: msg, progress });
    onPhaseChange?.(p);
  }, [onPhaseChange]);

  // Phase 1: Generate SVG from prompt (T2I → vector representation)
  const generateSVG = useCallback(async () => {
    updatePhase('generating', 'Generating SVG from prompt...', 0.2);
    await new Promise((r) => setTimeout(r, 100));

    const svg = svgContent || generateSVGFromT2I({ prompt, width, height });
    setSvgOutput(svg);
    updatePhase('svg-render', 'SVG rendered', 0.4);
    return svg;
  }, [prompt, width, height, svgContent, updatePhase]);

  // Phase 2: Animate based on preset
  const animate = useCallback(async () => {
    if (!svgRef.current) return;
    updatePhase('animating', `Running ${animationPreset.type} animation...`, 0.6);

    const svg = svgRef.current;
    targetRef.current = svg.querySelector('#morph-target');

    switch (animationPreset.type) {
      case 'morph':
        await runMorphAnimation(svg);
        break;
      case 'draw':
        await runDrawAnimation(svg);
        break;
      case 'crossfade': {
        const frames = generateImageFrames(generatedBase64, width, height);
        await runCrossfadeAnimation(frames);
        break;
      }
      case 'particle':
        runParticleAnimation();
        break;
      case 'procedural':
        runProceduralAnimation();
        break;
      case 'scroll':
        runScrollAnimation(svg);
        break;
    }

    updatePhase('complete', 'Animation complete', 1.0);
  }, [animationPreset, generatedBase64, width, height, updatePhase]);

  // Morph animation (SVG path → path)
  const runMorphAnimation = async (svg: SVGSVGElement) => {
    if (!svg) return;
    const paths = svg.querySelectorAll('path, polygon, polyline');
    if (paths.length === 0) return;

    const allMorphTargets = generateMorphTargets(prompt);
    const timeline = new Timeline();
    timelineRef.current = timeline;

    // Sequence: SVG → morph series → final
    let seqIdx = 0;
    const runNext = () => {
      if (seqIdx >= allMorphTargets.length || !paths[0]) return;
      const currentD = paths[0].getAttribute('d') || '';
      const targetD = allMorphTargets[seqIdx];

      const fromPts = parsePath(currentD);
      const toPts = parsePath(targetD);
      const maxLen = Math.max(fromPts.length, toPts.length);
      const fill = (arr: typeof fromPts, len: number) =>
        arr.length >= len ? arr : [...arr, ...Array(len - arr.length).fill(arr[arr.length - 1] || { type: 'L' as const, x: 0, y: 0 })];
      const [normFrom, normTo] = [fill(fromPts, maxLen), fill(toPts, maxLen)];

      timeline.add({
        id: `morph_${seqIdx}`,
        from: { t: 0 },
        to: { t: 1 },
        duration: animationPreset.duration / allMorphTargets.length,
        easing: animationPreset.easing as string | EasingFn,
        onComplete: () => {
          seqIdx++;
          if (seqIdx < allMorphTargets.length) setTimeout(runNext, 200);
        },
      });

      timeline.onFrame((_id, values) => {
        const t = values.t;
        const current = normFrom.map((fp, i) => {
          const tp = normTo[i] || fp;
          return {
            type: tp.type as 'M' | 'L' | 'C' | 'Q' | 'Z',
            x: fp.x + (tp.x - fp.x) * t,
            y: fp.y + (tp.y - fp.y) * t,
            ...(fp.c1 && tp.c1 ? { c1: { x: fp.c1.x + (tp.c1.x - fp.c1.x) * t, y: fp.c1.y + (tp.c1.y - fp.c1.y) * t } } : {}),
            ...(fp.c2 && tp.c2 ? { c2: { x: fp.c2.x + (tp.c2.x - fp.c2.x) * t, y: fp.c2.y + (tp.c2.y - fp.c2.y) * t } } : {}),
          };
        });
        if (paths[0] instanceof SVGPathElement) {
          paths[0].setAttribute('d', serializePath(current));
        }
      });
    };

    runNext();
    timeline.start();

    // Wait for completion
    await new Promise<void>((resolve) => {
      const check = setInterval(() => {
        if (!timeline.running && seqIdx >= allMorphTargets.length) {
          clearInterval(check);
          resolve();
        }
      }, 100);
    });
  };

  // Draw animation (stroke-dashoffset reveal)
  const runDrawAnimation = async (svg: SVGSVGElement) => {
    const paths = svg.querySelectorAll('path');
    for (const path of Array.from(paths)) {
      await drawPath(path as SVGPathElement, animationPreset.duration / paths.length, animationPreset.easing);
    }
  };

  // Crossfade between generated image frames
  const runCrossfadeAnimation = (frames: Frame[]) => {
    if (!canvasRef.current || frames.length < 2) return;
    const player = new FrameSequencePlayer(canvasRef.current, {
      frames,
      width,
      height,
      loop: false,
      onComplete: () => updatePhase('complete', 'Crossfade complete', 1.0),
    });
    playerRef.current = player;
    player.start();
  };

  // Placeholder: image frames from base64
  const generateImageFrames = (base64?: string, w = 512, h = 512): Frame[] => {
    if (!base64) {
      // Generate placeholder frames (gradient shifts)
      return [
        { data: `data:image/svg+xml,${encodeURIComponent(`<svg width="${w}" height="${h}"><rect width="${w}" height="${h}" fill="#ff6b6b"/></svg>`)}`, duration: 1000 },
        { data: `data:image/svg+xml,${encodeURIComponent(`<svg width="${w}" height="${h}"><rect width="${w}" height="${h}" fill="#4ecdc4"/></svg>`)}`, duration: 1000 },
        { data: `data:image/svg+xml,${encodeURIComponent(`<svg width="${w}" height="${h}"><rect width="${w}" height="${h}" fill="#45b7d1"/></svg>`)}`, duration: 1000 },
      ];
    }
    // Real base64 image + transitions
    const baseFrame: Frame = { data: `data:image/png;base64,${base64}`, duration: animationPreset.duration, easing: animationPreset.easing };
    const nextFrame: Frame = { data: `data:image/png;base64,${base64}`, duration: animationPreset.duration };
    return [baseFrame, nextFrame, baseFrame];
  };

  // Particle animation (Canvas-based)
  const runParticleAnimation = () => {
    if (!canvasRef.current) return;
    const canvas = canvasRef.current;
    canvas.width = width;
    canvas.height = height;

    type Particle = { x: number; y: number; vx: number; vy: number; size: number; hue: number; life: number };
    const particles: Particle[] = [];
    const MAX = 80;

    const renderer: FrameRenderer = (ctx, t) => {
      // Add new particles
      if (particles.length < MAX) {
        for (let i = 0; i < 3; i++) {
          particles.push({
            x: width / 2 + (Math.random() - 0.5) * 100,
            y: height / 2,
            vx: (Math.random() - 0.5) * 3,
            vy: (Math.random() - 0.5) * 3 - 2,
            size: 2 + Math.random() * 4,
            hue: (t * 50 + i * 30) % 360,
            life: 1,
          });
        }
      }

      // Update & draw
      for (let i = particles.length - 1; i >= 0; i--) {
        const p = particles[i];
        p.x += p.vx;
        p.y += p.vy;
        p.vy += 0.05;
        p.life -= 0.008;

        if (p.life <= 0) {
          particles.splice(i, 1);
          continue;
        }

        ctx.beginPath();
        ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
        ctx.fillStyle = `hsla(${p.hue}, 80%, 60%, ${p.life})`;
        ctx.fill();
      }
    };

    const loop = new FrameLoop(canvas, renderer);
    loop.start();
  };

  // Procedural Canvas animation
  const runProceduralAnimation = () => {
    if (!canvasRef.current) return;
    const canvas = canvasRef.current;
    canvas.width = width;
    canvas.height = height;

    const generator: ProceduralFrameGenerator = (t, ctx, w, h) => {
      const cx = w / 2;
      const cy = h / 2;
      const maxR = Math.min(w, h) * 0.4;

      for (let i = 0; i < 20; i++) {
        const angle = (i / 20) * Math.PI * 2 + t * Math.PI * 2;
        const r = maxR * (0.5 + 0.5 * Math.sin(t * Math.PI * 2 + i * 0.5));
        const x = cx + r * Math.cos(angle);
        const y = cy + r * Math.sin(angle);
        const hue = (i * 18 + t * 60) % 360;

        ctx.beginPath();
        ctx.arc(x, y, 4 + 3 * Math.sin(t * 3 + i), 0, Math.PI * 2);
        ctx.fillStyle = `hsl(${hue}, 80%, 60%)`;
        ctx.fill();
      }
    };

    generateFrameSequence(canvas, generator, 60, 30, true);
  };

  // Scroll reveal animation
  const runScrollAnimation = (svg: SVGSVGElement) => {
    svg.style.opacity = '0';
    svg.style.transform = 'translateY(30px)';
    svg.animate([
      { opacity: '0', transform: 'translateY(30px)' },
      { opacity: '1', transform: 'translateY(0)' },
    ], {
      duration: animationPreset.duration,
      easing: animationPreset.easing || 'cubic-bezier(0.25, 0.46, 0.45, 0.94)',
      fill: 'forwards',
    });
  };

  // ─── Main Pipeline ──────────────────────────────────────────────────

  const runPipeline = useCallback(async () => {
    try {
      await generateSVG();
      await animate();
    } catch (e) {
      updatePhase('error', `Pipeline error: ${e}`, 0);
    }
  }, [generateSVG, animate, updatePhase]);

  useEffect(() => {
    if (autoStart && phase === 'idle') {
      runPipeline();
    }
  }, [autoStart]);

  useEffect(() => {
    return () => {
      timelineRef.current?.stop();
      playerRef.current?.stop();
    };
  }, []);

  // ─── Render ─────────────────────────────────────────────────────────

  const canvasVisible = ['crossfade', 'particle', 'procedural'].includes(animationPreset.type) || phase === 'animating';
  const svgVisible = ['morph', 'draw', 'scroll'].includes(animationPreset.type) || phase === 'svg-render';

  // Capture SVG ref from DOM after innerHTML update
  useEffect(() => {
    if (!svgOutput) return;
    const container = document.getElementById('animation-svg-container');
    if (container) {
      const svgEl = container.querySelector('svg');
      if (svgEl) (svgRef as React.MutableRefObject<SVGSVGElement | null>).current = svgEl;
    }
  }, [svgOutput]);

  return (
    <div className="image-animation-pipeline" style={{ position: 'relative' }}>
      {/* Phase indicator */}
      <div style={{
        padding: '4px 8px',
        fontSize: 11,
        fontFamily: 'monospace',
        color: state.phase === 'error' ? '#ff4444' : '#888',
        marginBottom: 4,
      }}>
        [{state.phase}] {state.message}
        {state.progress > 0 && state.progress < 1 && (
          <span style={{ marginLeft: 8, opacity: 0.5 }}>
            {(state.progress * 100).toFixed(0)}%
          </span>
        )}
      </div>

      {/* SVG viewport */}
      {svgVisible && (
        <div
          id="animation-svg-container"
          style={{
            width,
            height,
            border: '1px solid #333',
            borderRadius: 4,
            overflow: 'hidden',
            background: '#0d0d1a',
          }}
          dangerouslySetInnerHTML={{ __html: svgOutput }}
        />
      )}

      {/* Canvas viewport */}
      {canvasVisible && (
        <canvas
          ref={canvasRef}
          width={width}
          height={height}
          style={{
            border: '1px solid #333',
            borderRadius: 4,
            background: '#0d0d1a',
          }}
        />
      )}

      {/* Controls */}
      <div style={{ marginTop: 8, display: 'flex', gap: 8 }}>
        {phase === 'idle' && (
          <button
            onClick={runPipeline}
            style={buttonStyle}
          >
            ▶ Generate & Animate
          </button>
        )}
        {phase === 'complete' && (
          <button
            onClick={() => { updatePhase('idle', '', 0); setSvgOutput(''); }}
            style={buttonStyle}
          >
            ↻ Reset
          </button>
        )}
      </div>
    </div>
  );
}

const buttonStyle: React.CSSProperties = {
  padding: '4px 12px',
  fontSize: 12,
  fontFamily: 'monospace',
  background: '#1a1a2e',
  color: '#e94560',
  border: '1px solid #e94560',
  borderRadius: 4,
  cursor: 'pointer',
};
