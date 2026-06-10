import React, { useEffect, useRef } from "react";
import { useStore } from "../store";

interface PetState {
  visual: {
    size: number;
    warmth: number;
    softness: number;
    energy: number;
    brightness: number;
    creature: number;
    complexity: number;
    definition: number;
  };
  behavior: {
    curiosity: number;
    playfulness: number;
    talkativeness: number;
    reactivity: number;
  };
  expression: string;
  level: number;
  energy: number;
}

const DEFAULT_PET: PetState = {
  visual: { size: 0.5, warmth: 0.5, softness: 0.5, energy: 0.5, brightness: 0.5, creature: 0.5, complexity: 0.3, definition: 0.5 },
  behavior: { curiosity: 0.5, playfulness: 0.5, talkativeness: 0.5, reactivity: 0.5 },
  expression: "neutral",
  level: 0,
  energy: 0.5,
};

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

function creatureLabel(c: number): string {
  if (c < 0.2) return "feline";
  if (c < 0.4) return "mixed-feline";
  if (c < 0.6) return "mixed";
  if (c < 0.8) return "mixed-avian";
  return "avian";
}

function bodyShape(creature: number, softness: number): { rx: number; ry: number } {
  const feline = Math.max(0, 1 - creature * 2);
  const avian = Math.max(0, (creature - 0.5) * 2);
  const rx = lerp(18, 14, feline) + avian * 6;
  const ry = lerp(16, 20, feline) - avian * 4 + (1 - softness) * 3;
  return { rx, ry };
}

function warmthColor(warmth: number, brightness: number): string {
  const r = Math.round(lerp(100, 255, warmth * 0.7 + brightness * 0.3));
  const g = Math.round(lerp(120, 220, brightness));
  const b = Math.round(lerp(200, 180, warmth) + brightness * 40);
  return `rgb(${r},${g},${Math.min(255, b)})`;
}

function eyeStyle(brightness: number, expression: string): { pupilSize: number; glow: string } {
  const pupilSize = 3 + brightness * 3;
  let glow = brightness > 0.6 ? `0 0 ${4 + brightness * 6}px rgba(255,255,200,${brightness * 0.5})` : "none";
  return { pupilSize, glow };
}

function mouthPath(expression: string): string {
  switch (expression) {
    case "excited": return "M 12 28 Q 18 34 24 28";
    case "curious": return "M 12 28 Q 18 30 24 28";
    case "content": return "M 14 29 Q 18 31 22 29";
    case "frustrated": return "M 12 32 Q 18 28 24 32";
    case "confused": return "M 12 27 Q 18 29 24 27 M 16 33 L 20 33";
    case "sleepy": return "M 14 30 Q 18 30 22 30";
    default: return "M 13 29 Q 18 32 23 29";
  }
}

function earPath(creature: number, side: "left" | "right"): string {
  const isFeline = creature < 0.4;
  const isAvian = creature > 0.7;
  const xBase = side === "left" ? 8 : 28;
  const xDir = side === "left" ? -1 : 1;

  if (isAvian) {
    return "";
  }
  if (isFeline) {
    const tipX = xBase + xDir * 4;
    const tipY = 4;
    return `M ${xBase - 3} 14 Q ${xBase} 6 ${tipX} ${tipY} Q ${xBase + xDir * 2} 8 ${xBase + 3} 14 Z`;
  }
  const tipX = xBase + xDir * 2;
  const tipY = 6;
  return `M ${xBase - 2} 14 Q ${xBase} 8 ${tipX} ${tipY} Q ${xBase + xDir * 1} 10 ${xBase + 2} 14 Z`;
}

function tailPath(creature: number, energy: number): string {
  const isFeline = creature < 0.4;
  const isAvian = creature > 0.7;
  if (isAvian) {
    return "M 36 30 Q 42 22 40 14 Q 38 10 42 8";
  }
  if (isFeline) {
    const curl = 6 + energy * 8;
    return `M 36 24 Q 44 18 42 ${14 - curl * 0.3} Q 40 ${10 - curl * 0.5} 44 ${6 - curl * 0.3}`;
  }
  return "M 36 26 Q 42 20 40 14";
}

const PetPanel: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animRef = useRef<number>(0);
  const petRef = useRef<PetState>({ ...DEFAULT_PET });
  const timeRef = useRef(0);

  const brainHealth = useStore((s) => s.brainHealth);
  const agentBusy = useStore((s) => s.agentBusy);
  const brainEvents = useStore((s) => s.brainEvents);

  useEffect(() => {
    const pet = petRef.current;
    pet.energy = (brainHealth.curiosity_bonus ?? 0.5) * 0.7 + (agentBusy ? 0.3 : 0);

    const lastEvents = brainEvents.slice(-3);
    for (const ev of lastEvents) {
      if (ev.kind === "stage" && ev.status === "done") {
        pet.visual.energy = Math.min(1, pet.visual.energy + 0.02);
      }
      if (ev.kind === "stage" && ev.status === "error") {
        pet.visual.energy = Math.max(0, pet.visual.energy - 0.05);
      }
    }
  }, [brainHealth, agentBusy, brainEvents]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const W = 80;
    const H = 80;
    canvas.width = W;
    canvas.height = H;

    function draw() {
      if (!ctx) return;
      ctx.clearRect(0, 0, W, H);

      const pet = petRef.current;
      const v = pet.visual;
      const time = timeRef.current;

      const cx = W / 2;
      const cy = H / 2 + 6;

      const body = bodyShape(v.creature, v.softness);
      const color = warmthColor(v.warmth, v.brightness);
      const { pupilSize, glow } = eyeStyle(v.brightness, pet.expression);

      const bobY = Math.sin(time * 0.04 + (1 - v.energy) * 2) * (1 + v.energy * 2);
      const scale = 0.6 + v.size * 0.6;
      const bobSway = Math.sin(time * 0.03) * v.energy * 2;

      ctx.save();
      ctx.translate(cx + bobSway, cy + bobY);
      ctx.scale(scale, scale);

      if (glow !== "none") {
        ctx.save();
        ctx.shadowColor = `rgba(255,255,200,${v.brightness * 0.3})`;
        ctx.shadowBlur = 8 + v.brightness * 12;
        ctx.beginPath();
        ctx.ellipse(0, 0, body.rx + 4, body.ry + 4, 0, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(255,255,200,${v.brightness * 0.08})`;
        ctx.fill();
        ctx.restore();
      }

      const earL = earPath(v.creature, "left");
      if (earL) {
        ctx.beginPath();
        const earPath2 = new Path2D(earL);
        ctx.fillStyle = color;
        ctx.fill(earPath2);
      }
      const earR = earPath(v.creature, "right");
      if (earR) {
        ctx.beginPath();
        const earPath3 = new Path2D(earR);
        ctx.fillStyle = color;
        ctx.fill(earPath3);
      }

      ctx.beginPath();
      ctx.ellipse(0, 0, body.rx, body.ry, 0, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();
      ctx.strokeStyle = `rgba(0,0,0,${0.1 + v.definition * 0.3})`;
      ctx.lineWidth = 1 + v.definition;
      ctx.stroke();

      const tail = tailPath(v.creature, v.energy);
      if (tail) {
        ctx.save();
        ctx.translate(-body.rx + 4, 0);
        ctx.beginPath();
        const tailPath2 = new Path2D(tail);
        ctx.strokeStyle = color;
        ctx.lineWidth = 2 + v.softness * 2;
        ctx.lineCap = "round";
        ctx.stroke(tailPath2);
        ctx.restore();
      }

      const eyeY = -3;
      const eyeSpacing = 6 + (1 - v.softness) * 2;
      for (const side of [-1, 1]) {
        const ex = side * eyeSpacing;
        const ey = eyeY;

        ctx.beginPath();
        ctx.ellipse(ex, ey, 3.5, 4 + (1 - v.energy) * 1, 0, 0, Math.PI * 2);
        ctx.fillStyle = `rgb(${Math.round(240 - v.brightness * 40)}, ${Math.round(240 - v.brightness * 40)}, ${Math.round(255 - v.brightness * 40)})`;
        ctx.fill();

        if (v.brightness > 0.5) {
          ctx.save();
          ctx.shadowColor = `rgba(255,255,200,${v.brightness * 0.4})`;
          ctx.shadowBlur = 4 + v.brightness * 6;
          ctx.beginPath();
          ctx.arc(ex, ey, pupilSize * 0.4, 0, Math.PI * 2);
          ctx.fillStyle = `rgba(255,255,200,${v.brightness * 0.3})`;
          ctx.fill();
          ctx.restore();
        }

        ctx.beginPath();
        ctx.arc(ex, ey, pupilSize * 0.6, 0, Math.PI * 2);
        ctx.fillStyle = "#1a1a2e";
        ctx.fill();

        const blinkCycle = Math.sin(time * 0.05) > 0.95;
        if (blinkCycle) {
          ctx.beginPath();
          ctx.ellipse(ex, ey, 4, 1, 0, 0, Math.PI * 2);
          ctx.fillStyle = color;
          ctx.fill();
        }
      }

      const mouth = mouthPath(pet.expression);
      ctx.beginPath();
      const mouthP = new Path2D(mouth);
      ctx.strokeStyle = `rgba(0,0,0,${0.3 + v.definition * 0.3})`;
      ctx.lineWidth = 1.2;
      ctx.stroke(mouthP);

      ctx.restore();

      animRef.current = requestAnimationFrame(draw);
    }

    const interval = setInterval(() => { timeRef.current += 1; }, 50);
    animRef.current = requestAnimationFrame(draw);

    return () => {
      cancelAnimationFrame(animRef.current);
      clearInterval(interval);
    };
  }, []);

  const v = petRef.current.visual;
  const label = creatureLabel(v.creature);

  return (
    <div className="pet-panel">
      <div className="pet-canvas-wrapper">
        <canvas ref={canvasRef} className="pet-canvas" />
      </div>
      <div className="pet-info">
        <span className="pet-level">Lv.{petRef.current.level}</span>
        <span className="pet-creature">{label}</span>
        <span className="pet-expression">{petRef.current.expression}</span>
      </div>
      <div className="pet-traits">
        {(["size", "warmth", "softness", "energy", "brightness"] as const).map((t) => (
          <div key={t} className="pet-trait-row">
            <span className="pet-trait-label">{t[0].toUpperCase() + t.slice(1)}</span>
            <div className="pet-trait-bar">
              <div
                className="pet-trait-fill"
                style={{ width: `${v[t] * 100}%` }}
              />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default PetPanel;
