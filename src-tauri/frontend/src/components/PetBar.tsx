import React, { useEffect, useRef } from "react";
import { useStore } from "../stores";
import styles from "./PetBar.module.css";

const H = 130;
const HC = 48;
const GROUND_Y = 95;
const PET_W = 18;

type Action = "idle" | "walk" | "excited" | "look" | "sleep" | "scratch";

interface PetState {
  x: number; y: number; targetX: number;
  action: Action; step: number; timer: number;
  earAngle: number; tailPhase: number; lookX: number;
  jumpY: number; jumpVy: number; blink: number; dir: 1 | -1; sitTimer: number;
}

interface Particle {
  x: number; y: number; vx: number; vy: number; life: number; maxLife: number;
  type: "leaf" | "firefly" | "ripple" | "bird" | "sparkle" | "petal" | "glow";
  size: number; hue?: number; phase?: number; alpha?: number;
}

function rand(a: number, b: number) { return a + Math.random() * (b - a); }
function clamp(v: number, lo: number, hi: number) { return Math.max(lo, Math.min(hi, v)); }

function lerp(a: number, b: number, t: number) { return a + (b - a) * t; }

// ── Simplified 1D noise ──
function noise(x: number): number {
  const n = Math.sin(x * 12.9898 + 78.233) * 43758.5453;
  return n - Math.floor(n);
}

function fbm(x: number, octaves: number): number {
  let v = 0, amp = 0.5, freq = 1;
  for (let i = 0; i < octaves; i++) { v += amp * noise(x * freq); freq *= 2; amp *= 0.5; }
  return v;
}

function mountainH(x: number, W: number, seed: number): number {
  const t = x / W;
  return fbm(t * 3 + seed, 5) * 28 + fbm(t * 6 + seed * 2, 3) * 8 + 6;
}

function sCurve(t: number): number { return t * t * (3 - 2 * t); }

// ── Draw mountain with shading ──
function drawMountain(ctx: CanvasRenderingContext2D, W: number, baseY: number, hScale: number, seed: number, color: string, shadowColor: string, snowColor: string, snowLine: number) {
  const pts: { x: number; y: number }[] = [];
  ctx.beginPath(); ctx.moveTo(0, baseY + hScale);
  for (let x = 0; x <= W; x += 2) {
    const y = baseY + hScale - mountainH(x, W, seed) * (hScale / 40);
    ctx.lineTo(x, y); pts.push({ x, y });
  }
  ctx.lineTo(W, baseY + hScale); ctx.closePath();
  ctx.fillStyle = color; ctx.fill();

  // Shadow side (right-facing)
  ctx.beginPath();
  for (let i = Math.floor(pts.length / 3); i < pts.length; i++) {
    const p = pts[i];
    i === Math.floor(pts.length / 3) ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y);
  }
  ctx.lineTo(W, baseY + hScale); ctx.lineTo(pts[Math.floor(pts.length / 3)].x, baseY + hScale); ctx.closePath();
  ctx.fillStyle = shadowColor; ctx.fill();

  // Snow caps
  const snowH = snowLine + 5 + fbm(seed * 3, 2) * 6;
  ctx.beginPath(); let started = false;
  for (const p of pts) {
    if (p.y < baseY + snowH) {
      if (!started) { ctx.moveTo(p.x, p.y); started = true; } else ctx.lineTo(p.x, p.y);
    } else started = false;
  }
  ctx.strokeStyle = snowColor; ctx.lineWidth = 2.5; ctx.lineCap = "round"; ctx.stroke();

  // Snow fill (small patches near peaks)
  ctx.beginPath(); started = false;
  for (const p of pts) {
    if (p.y < baseY + snowH - 3) {
      if (!started) { ctx.moveTo(p.x, p.y); started = true; } else ctx.lineTo(p.x, p.y);
    } else started = false;
  }
  ctx.strokeStyle = snowColor; ctx.lineWidth = 4; ctx.globalAlpha = 0.3; ctx.stroke(); ctx.globalAlpha = 1;
}

// ── Draw a beautiful tree ──
function drawTree(ctx: CanvasRenderingContext2D, x: number, baseY: number, h: number, w: number, isDark: boolean, isPine: boolean) {
  const trunkH = h * 0.2;
  const crownH = h - trunkH;
  // Trunk
  ctx.strokeStyle = isDark ? "#3a2a1a" : "#5a3a28";
  ctx.lineWidth = Math.max(1.5, w * 0.18);
  ctx.lineCap = "round";
  ctx.beginPath(); ctx.moveTo(x, baseY); ctx.lineTo(x, baseY - trunkH); ctx.stroke();

  // Shadow
  ctx.save(); ctx.globalAlpha = 0.06;
  ctx.fillStyle = "#000";
  ctx.beginPath(); ctx.ellipse(x + 1.5, baseY, w * 0.3, crownH * 0.08, 0, 0, Math.PI * 2); ctx.fill();
  ctx.restore();

  if (isPine) {
    // Layered pine
    const layers = 4;
    for (let i = 0; i < layers; i++) {
      const ty = baseY - trunkH - (crownH * i) / layers;
      const tw = w * (1 - i * 0.2);
      const th = crownH / layers * 0.9;
      const shade = isDark ? 12 + i * 4 : 30 + i * 8;
      const hue = isDark ? 130 + i * 5 : 115 + i * 3;
      ctx.fillStyle = `hsl(${hue}, ${isDark ? 30 : 45}%, ${shade}%)`;
      ctx.beginPath();
      ctx.moveTo(x, ty);
      ctx.quadraticCurveTo(x - tw / 2, ty + th * 0.4, x - tw * 0.35, ty + th);
      ctx.lineTo(x + tw * 0.35, ty + th);
      ctx.quadraticCurveTo(x + tw / 2, ty + th * 0.4, x, ty);
      ctx.closePath(); ctx.fill();
    }
    // Highlight on top
    ctx.fillStyle = isDark ? "rgba(100,180,100,0.08)" : "rgba(160,220,120,0.12)";
    ctx.beginPath(); ctx.ellipse(x, baseY - trunkH - crownH * 0.15, w * 0.2, crownH * 0.08, 0, 0, Math.PI * 2); ctx.fill();
  } else {
    // Deciduous: rounded canopy
    const hue = isDark ? 45 : 80;
    const sat = isDark ? 25 : 50;
    const light = isDark ? 20 : 38;
    const colors = [
      `hsl(${hue}, ${sat}%, ${light + 8}%)`,
      `hsl(${hue - 5}, ${sat + 5}%, ${light}%)`,
      `hsl(${hue - 10}, ${sat + 10}%, ${light - 5}%)`,
    ];
    // Bottom layer (darkest)
    ctx.fillStyle = colors[2];
    ctx.beginPath(); ctx.ellipse(x, baseY - trunkH - crownH * 0.5, w * 0.5, crownH * 0.45, 0, 0, Math.PI * 2); ctx.fill();
    // Middle
    ctx.fillStyle = colors[1];
    ctx.beginPath(); ctx.ellipse(x - w * 0.12, baseY - trunkH - crownH * 0.55, w * 0.38, crownH * 0.38, 0, 0, Math.PI * 2); ctx.fill();
    ctx.beginPath(); ctx.ellipse(x + w * 0.14, baseY - trunkH - crownH * 0.55, w * 0.35, crownH * 0.35, 0, 0, Math.PI * 2); ctx.fill();
    // Top (lightest)
    ctx.fillStyle = colors[0];
    ctx.beginPath(); ctx.ellipse(x, baseY - trunkH - crownH * 0.65, w * 0.3, crownH * 0.3, 0, 0, Math.PI * 2); ctx.fill();
    // Highlight
    ctx.fillStyle = isDark ? "rgba(180,200,120,0.06)" : "rgba(200,230,160,0.1)";
    ctx.beginPath(); ctx.ellipse(x - w * 0.08, baseY - trunkH - crownH * 0.7, w * 0.12, crownH * 0.08, -0.3, 0, Math.PI * 2); ctx.fill();
  }
}

// ── Draw scene ──
function drawScene(ctx: CanvasRenderingContext2D, W: number, t: number, isDark: boolean, particles: Particle[], scaleY: number) {
  const S = scaleY;
  const GY = GROUND_Y * S;

  // Sky
  const sky = ctx.createLinearGradient(0, 0, 0, GY + 10 * S);
  if (isDark) { sky.addColorStop(0, "#0d0d1a"); sky.addColorStop(0.3, "#1a1a30"); sky.addColorStop(0.6, "#2a2740"); sky.addColorStop(1, "#3a3550"); }
  else { sky.addColorStop(0, "#f7e8d0"); sky.addColorStop(0.3, "#f0dcc0"); sky.addColorStop(0.6, "#e4cdad"); sky.addColorStop(1, "#d4bfa0"); }
  ctx.fillStyle = sky; ctx.fillRect(0, 0, W, GY + 10 * S);

  // Stars
  if (isDark) for (let i = 0; i < 35; i++) {
    const sx = (i * 131.7 + 23) % W, sy = ((i * 89.3 + 11) % 60) * S;
    const tw = 0.3 + Math.sin(t * 1.8 + i * 3.1) * 0.3;
    ctx.globalAlpha = Math.max(0, tw * 0.6); ctx.fillStyle = i % 5 === 0 ? "#ffe8b0" : "#fff";
    const r = 0.3 + (i % 3) * 0.15;
    ctx.beginPath(); ctx.arc(sx, sy, r * Math.min(1, S + 0.3), 0, Math.PI * 2); ctx.fill();
  }
  ctx.globalAlpha = 1;

  // Celestial
  const cx = W * 0.8, cy = (isDark ? 14 : 18) * S;
  if (isDark) {
    const g = ctx.createRadialGradient(cx, cy, 0, cx, cy, 22 * S);
    g.addColorStop(0, "rgba(230,220,200,1)"); g.addColorStop(0.2, "rgba(230,220,200,0.35)"); g.addColorStop(1, "rgba(230,220,200,0)");
    ctx.fillStyle = g; ctx.beginPath(); ctx.arc(cx, cy, 22 * S, 0, Math.PI * 2); ctx.fill();
    ctx.fillStyle = "#e6dcc8"; ctx.beginPath(); ctx.arc(cx, cy, 5 * S, 0, Math.PI * 2); ctx.fill();
    ctx.fillStyle = "rgba(200,190,170,0.25)"; ctx.beginPath(); ctx.arc(cx - 1.3 * S, cy - 0.8 * S, 0.8 * S, 0, Math.PI * 2); ctx.fill();
    // Moon glow on surrounding sky
    const mg = ctx.createRadialGradient(cx, cy, 0, cx, cy, 40 * S);
    mg.addColorStop(0, "rgba(200,200,255,0.03)"); mg.addColorStop(1, "rgba(200,200,255,0)");
    ctx.fillStyle = mg; ctx.beginPath(); ctx.arc(cx, cy, 40 * S, 0, Math.PI * 2); ctx.fill();
  } else {
    const g = ctx.createRadialGradient(cx, cy, 0, cx, cy, 28 * S);
    g.addColorStop(0, "rgba(255,210,120,0.6)"); g.addColorStop(0.4, "rgba(255,200,100,0.15)"); g.addColorStop(1, "rgba(255,200,100,0)");
    ctx.fillStyle = g; ctx.beginPath(); ctx.arc(cx, cy, 28 * S, 0, Math.PI * 2); ctx.fill();
    ctx.fillStyle = "#FFD59A"; ctx.beginPath(); ctx.arc(cx, cy, 6.5 * S, 0, Math.PI * 2); ctx.fill();
    // Sun rays
    ctx.save(); ctx.globalAlpha = 0.06;
    for (let i = 0; i < 8; i++) {
      const a = t * 0.0005 + (i / 8) * Math.PI * 2;
      ctx.fillStyle = "#FFD59A";
      ctx.beginPath(); ctx.ellipse(cx + Math.cos(a) * 14 * S, cy + Math.sin(a) * 14 * S, 4 * S, 1.5 * S, a, 0, Math.PI * 2); ctx.fill();
    }
    ctx.restore();
  }

  // Mist
  ctx.save(); ctx.globalAlpha = isDark ? 0.06 : 0.08;
  for (let i = 0; i < 3; i++) {
    const my = (28 + i * 14 + Math.sin(t * 0.15 + i * 1.7) * 3) * S;
    ctx.fillStyle = isDark ? "#4a4560" : "#e8ddd0";
    ctx.beginPath(); ctx.ellipse(W * 0.3 + Math.sin(t * 0.08 + i) * 35, my, W * 0.45, (3 + i * 1.5) * S, 0, 0, Math.PI * 2); ctx.fill();
    ctx.beginPath(); ctx.ellipse(W * 0.7 + Math.cos(t * 0.06 + i * 2) * 30, my + 1 * S, W * 0.35, (2.5 + i) * S, 0, 0, Math.PI * 2); ctx.fill();
  }
  ctx.restore();

  // Clouds
  ctx.save(); ctx.globalAlpha = isDark ? 0.1 : 0.22;
  for (let i = 0; i < 4; i++) {
    const cxx = ((i * 180 + 80 + t * (6 + i * 3)) % (W + 100)) - 50;
    const cyy = (10 + i * 6 + Math.sin(i * 2.1) * 3) * S;
    const cw = (35 + i * 12);
    ctx.fillStyle = isDark ? "#555" : "#fff";
    ctx.beginPath(); ctx.ellipse(cxx, cyy, cw * 0.45, 3.5 * S, 0, 0, Math.PI * 2); ctx.fill();
    ctx.beginPath(); ctx.ellipse(cxx - cw * 0.2, cyy + 0.8 * S, cw * 0.28, 2.5 * S, 0, 0, Math.PI * 2); ctx.fill();
    ctx.beginPath(); ctx.ellipse(cxx + cw * 0.22, cyy + 1.2 * S, cw * 0.32, 3 * S, 0, 0, Math.PI * 2); ctx.fill();
  }
  ctx.restore();

  // Mountains
  drawMountain(ctx, W, 16 * S, 36 * S, 1.3, isDark ? "#3a3548" : "#c8bca8", isDark ? "#2e2a3a" : "#b8a898", isDark ? "rgba(200,200,220,0.3)" : "rgba(255,255,255,0.35)", 20 * S);
  drawMountain(ctx, W, 26 * S, 32 * S, 2.7, isDark ? "#2e2a3a" : "#b4a894", isDark ? "#221f2c" : "#a49884", isDark ? "rgba(200,200,220,0.25)" : "rgba(255,255,255,0.3)", 26 * S);
  drawMountain(ctx, W, 36 * S, 26 * S, 4.1, isDark ? "#221f2c" : "#a09484", isDark ? "#1a1722" : "#908478", isDark ? "rgba(180,180,200,0.2)" : "rgba(255,255,255,0.25)", 30 * S);

  // Ridge detail
  ctx.save(); ctx.globalAlpha = isDark ? 0.1 : 0.08;
  for (let r = 0; r < 2; r++) {
    ctx.strokeStyle = isDark ? "#4a4560" : "#b8a898"; ctx.lineWidth = 0.5;
    ctx.beginPath(); const by = (36 + r * 12) * S;
    for (let x = 0; x <= W; x += 4) {
      const y = by + mountainH(x, W, 3.3 + r * 0.7) * 0.35 * S;
      x === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
    }
    ctx.stroke();
  }
  ctx.restore();

  // Background forest (deciduous)
  ctx.save();
  for (let i = 0; i < 18; i++) {
    const tx = (i * 73 + 19) % W, th = (14 + (i * 11) % 10) * S, tw = (8 + (i * 7) % 6) * S;
    ctx.globalAlpha = isDark ? 0.5 : 0.45;
    drawTree(ctx, tx, (58 - th * 0.25) * S, th, tw, isDark, false);
  }
  ctx.restore();

  // Mid forest (pine)
  ctx.save();
  for (let i = 0; i < 16; i++) {
    const tx = (i * 97 + 37) % W, th = (12 + (i * 13) % 12) * S, tw = (5 + (i * 5) % 5) * S;
    ctx.globalAlpha = isDark ? 0.6 : 0.55;
    drawTree(ctx, tx, (62 - th * 0.2) * S, th, tw, isDark, true);
  }
  ctx.restore();

  // River
  const riverYs = [70, 72, 74, 76, 78];
  const riverColors = isDark ? ["#1a2a40", "#1e3050", "#223860", "#1e3050", "#1a2a40"] : ["#6a98b8", "#7aa8c8", "#86b4d0", "#7aa8c8", "#6a98b8"];
  for (let i = 0; i < riverYs.length; i++) {
    ctx.save(); ctx.globalAlpha = 0.3 + i * 0.07; ctx.fillStyle = riverColors[i]; ctx.beginPath();
    ctx.moveTo(0, riverYs[i] * S);
    for (let x = 0; x <= W; x += 3) ctx.lineTo(x, riverYs[i] * S + Math.sin(x * 0.018 + t * 0.4 + i * 1.3) * 1.8 * S);
    ctx.lineTo(W, (riverYs[i] + 2) * S); ctx.lineTo(0, (riverYs[i] + 2) * S); ctx.closePath(); ctx.fill(); ctx.restore();
  }

  // River foam/shore edge
  ctx.save(); ctx.globalAlpha = isDark ? 0.08 : 0.12;
  for (let side of [-1, 1]) {
    ctx.fillStyle = isDark ? "#3a4a60" : "#b8d0d8";
    ctx.beginPath(); ctx.moveTo(0, 73 * S + side * 3 * S);
    for (let x = 0; x <= W; x += 4) ctx.lineTo(x, 73 * S + side * 3 * S + Math.sin(x * 0.015 + t * 0.3) * 1.2 * S);
    ctx.lineTo(W, 73 * S); ctx.lineTo(0, 73 * S); ctx.closePath(); ctx.fill();
  }
  ctx.restore();

  // River shimmer
  ctx.save(); ctx.globalAlpha = 0.2;
  for (let i = 0; i < 8; i++) {
    const sx = ((t * 20 + i * 70 + Math.sin(i * 2.3) * 15) % (W + 40)) - 20;
    const sy = (72 + Math.sin(i * 1.7 + t * 0.35) * 2.5) * S;
    ctx.fillStyle = isDark ? "rgba(180,200,240,0.2)" : "rgba(255,255,255,0.35)";
    ctx.beginPath(); ctx.ellipse(sx, sy, (1.5 + Math.sin(t * 0.4 + i) * 0.8) * S, 0.6 * S, 0.3, 0, Math.PI * 2); ctx.fill();
  }
  ctx.restore();

  // Reflection
  ctx.save();
  for (let i = 0; i < 4; i++) {
    const ry = (75 + i * 3) * S;
    ctx.globalAlpha = ((isDark ? 0.1 : 0.12) - i * 0.02) * (S > 0.5 ? 1 : 1.5);
    ctx.fillStyle = isDark ? "#e6dcc8" : "#FFD59A";
    ctx.beginPath(); ctx.ellipse(W * 0.8, ry + Math.sin(t + i * 2) * 1.2 * S, (10 - i * 2) * S, 1 * S, 0, 0, Math.PI * 2); ctx.fill();
  }
  ctx.restore();

  // Ground
  const gg = ctx.createLinearGradient(0, GY, 0, H * S);
  if (isDark) { gg.addColorStop(0, "#2a3520"); gg.addColorStop(0.35, "#1e2818"); gg.addColorStop(0.7, "#182214"); gg.addColorStop(1, "#141c10"); }
  else { gg.addColorStop(0, "#b8c8a0"); gg.addColorStop(0.35, "#a0b088"); gg.addColorStop(0.7, "#88a070"); gg.addColorStop(1, "#74905c"); }
  ctx.fillStyle = gg; ctx.fillRect(0, GY, W, H * S - GY);

  // Grass patches
  for (let i = 0; i < 25; i++) {
    const gx = (i * 43 + 7) % W, gy = GY + (3 + (i * 13) % 14) * S;
    const gh = (2 + (i * 11) % 4) * S;
    const a = 0.25 + (i % 4) * 0.1;
    ctx.save(); ctx.globalAlpha = a * (S > 0.5 ? 1 : 0.8);
    ctx.strokeStyle = isDark ? "#3a5530" : "#6a8a50";
    ctx.lineWidth = Math.max(0.5, 0.7 * S);
    ctx.beginPath(); ctx.moveTo(gx, gy); ctx.lineTo(gx - 0.7 * S, gy - gh);
    ctx.moveTo(gx, gy); ctx.lineTo(gx + 0.8 * S, gy - gh * 0.65);
    ctx.moveTo(gx, gy); ctx.lineTo(gx + 0.3 * S, gy - gh * 1.1);
    ctx.stroke(); ctx.restore();
  }

  // Flowers
  for (let i = 0; i < 8; i++) {
    const fx = (i * 97 + 29) % W, fy = GY + (6 + (i * 17) % 18) * S;
    const fc = isDark ? ["#d07080", "#80a0d0", "#d0b070"][i % 3] : ["#e8a0b0", "#b0d0f0", "#f0d8a0"][i % 3];
    const fs = Math.min(1, S + 0.4);
    ctx.save(); ctx.globalAlpha = 0.5 * fs;
    for (let p = 0; p < 5; p++) {
      const a = (p / 5) * Math.PI * 2 + 0.3;
      ctx.beginPath(); ctx.arc(fx + Math.cos(a) * 1.2 * S, fy + Math.sin(a) * 1.2 * S, 0.6 * S, 0, Math.PI * 2); ctx.fillStyle = fc; ctx.fill();
    }
    ctx.fillStyle = isDark ? "#d0c878" : "#f0e888";
    ctx.beginPath(); ctx.arc(fx, fy, 0.4 * S, 0, Math.PI * 2); ctx.fill();
    ctx.restore();
  }

  // Rocks
  for (let i = 0; i < 5; i++) {
    const rx = (i * 89 + 37) % W, ry = GY + (7 + (i * 13) % 12) * S, rs = (2.5 + (i * 5) % 4) * S;
    ctx.beginPath(); ctx.ellipse(rx, ry, rs, rs * 0.5, 0, 0, Math.PI * 2);
    ctx.fillStyle = isDark ? "#444040" : "#8a8070"; ctx.fill();
    ctx.fillStyle = isDark ? "#555050" : "#9a9080";
    ctx.beginPath(); ctx.ellipse(rx - rs * 0.2, ry - rs * 0.1, rs * 0.35, rs * 0.22, -0.3, 0, Math.PI * 2); ctx.fill();
  }

  // Foreground gradient
  const eg = ctx.createLinearGradient(0, H * S - 15 * S, 0, H * S);
  eg.addColorStop(0, "transparent"); eg.addColorStop(1, isDark ? "rgba(0,0,0,0.3)" : "rgba(0,0,0,0.06)");
  ctx.fillStyle = eg; ctx.fillRect(0, H * S - 15 * S, W, 15 * S);

  // ── Particles ──
  for (const p of particles) {
    const lr = clamp(p.life / p.maxLife, 0, 1);
    ctx.save(); ctx.globalAlpha = lr * (p.alpha ?? 1);
    switch (p.type) {
      case "leaf": {
        const c = isDark ? `hsl(35,25%,${20 + lr * 15}%)` : `hsl(30,55%,${40 + lr * 20}%)`;
        ctx.fillStyle = c;
        ctx.save(); ctx.translate(p.x, p.y); ctx.rotate(p.x * 0.1 + t); ctx.scale(S, S);
        ctx.beginPath(); ctx.ellipse(0, 0, p.size, p.size * 0.45, 0, 0, Math.PI * 2); ctx.fill(); ctx.restore();
        break;
      }
      case "petal": {
        const c = isDark ? `hsl(330,40%,${35 + lr * 20}%)` : `hsl(330,60%,${55 + lr * 20}%)`;
        ctx.fillStyle = c; ctx.globalAlpha = lr * 0.5;
        ctx.save(); ctx.translate(p.x, p.y); ctx.rotate(t + p.x); ctx.scale(S, S);
        ctx.beginPath(); ctx.ellipse(0, 0, p.size, p.size * 0.3, 0, 0, Math.PI * 2); ctx.fill(); ctx.restore();
        break;
      }
      case "firefly": {
        const g = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, p.size * 4 * S);
        g.addColorStop(0, `rgba(255,255,200,${0.5 * lr})`); g.addColorStop(1, "rgba(255,255,200,0)");
        ctx.fillStyle = g; ctx.beginPath(); ctx.arc(p.x, p.y, p.size * 4 * S, 0, Math.PI * 2); ctx.fill();
        ctx.fillStyle = "#ffffd0"; ctx.beginPath(); ctx.arc(p.x, p.y, p.size * S, 0, Math.PI * 2); ctx.fill();
        break;
      }
      case "ripple":
        ctx.strokeStyle = isDark ? "rgba(200,220,255,0.3)" : "rgba(255,255,255,0.4)";
        ctx.lineWidth = 0.5 * S; ctx.globalAlpha = (1 - lr) * 0.35;
        ctx.beginPath(); ctx.ellipse(p.x, p.y, p.size * (1 + (1 - lr) * 2.5) * S, p.size * 0.25 * (1 + (1 - lr) * 2.5) * S, 0, 0, Math.PI * 2); ctx.stroke();
        break;
      case "bird":
        ctx.strokeStyle = isDark ? "#555" : "#666"; ctx.lineWidth = Math.max(0.5, 1 * S); ctx.globalAlpha = lr * 0.6;
        const ww = Math.sin(t * 5 + p.x) * 1.5;
        ctx.beginPath(); ctx.moveTo(p.x - 2.5 * S, p.y); ctx.quadraticCurveTo(p.x - 0.8 * S, p.y - (1.5 + ww) * S, p.x, p.y); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(p.x + 2.5 * S, p.y); ctx.quadraticCurveTo(p.x + 0.8 * S, p.y - (1.5 - ww) * S, p.x, p.y); ctx.stroke();
        break;
      case "sparkle": {
        const sg = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, p.size * 2 * S);
        sg.addColorStop(0, `rgba(255,255,220,${lr})`); sg.addColorStop(1, "rgba(255,255,220,0)");
        ctx.fillStyle = sg; ctx.beginPath(); ctx.arc(p.x, p.y, p.size * 2 * S, 0, Math.PI * 2); ctx.fill();
        break;
      }
      case "glow": {
        const g2 = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, p.size * 3 * S);
        g2.addColorStop(0, `rgba(255,220,180,${0.4 * lr})`); g2.addColorStop(1, "rgba(255,220,180,0)");
        ctx.fillStyle = g2; ctx.beginPath(); ctx.arc(p.x, p.y, p.size * 3 * S, 0, Math.PI * 2); ctx.fill();
        break;
      }
    }
    ctx.restore();
  }
}

// ── Draw pet ──
function drawPet(ctx: CanvasRenderingContext2D, pet: PetState, t: number, isDark: boolean, mouseX: number | null, scaleY: number) {
  const S = scaleY;
  const px = pet.x, py = (pet.y + pet.jumpY) * S;
  ctx.save(); ctx.translate(px, py);

  const sc = Math.min(1, S + 0.15);
  ctx.scale(sc, sc);

  const bodyW = 12, bodyH = 9, headR = 7, earH = 5, earW = 4, legH = 4;
  const bodyColor = isDark ? "#D4A84E" : "#C4944A";
  const bodyLight = isDark ? "#e0c060" : "#d4a858";
  const bellyColor = "#E8D5A3";
  const darkColor = "#8A6A2E";

  // Tail
  ctx.save();
  const tw = Math.sin(t * 0.001 * (pet.action === "excited" ? 8 : 3) + pet.tailPhase);
  ctx.translate(-bodyW * 0.4, -bodyH * 0.2 + Math.sin(t * 0.002) * 0.5);
  ctx.rotate(0.4 + tw * 0.3);
  ctx.beginPath(); ctx.ellipse(0, -legH - 2, 2.5, 4.5, 0.2, 0, Math.PI * 2); ctx.fillStyle = bodyColor; ctx.fill();
  ctx.beginPath(); ctx.ellipse(0, -legH - 4, 1.8, 2.5, 0, 0, Math.PI * 2); ctx.fillStyle = bodyLight; ctx.fill();
  ctx.beginPath(); ctx.ellipse(0, -legH - 5.5, 1, 1.2, 0, 0, Math.PI * 2); ctx.fillStyle = bellyColor; ctx.fill();
  ctx.restore();

  const wc = pet.action === "walk" || pet.action === "excited";
  const sitting = pet.action === "sleep" || pet.action === "scratch";
  const lp = wc ? Math.sin(t * 0.008) : 0;
  const lp2 = wc ? Math.sin(t * 0.008 + Math.PI) : 0.3;

  // Legs
  if (sitting) {
    for (const s of [-1, 1]) { ctx.beginPath(); ctx.ellipse(s * 3, legH * 0.5, 2.5, 1.5, 0, 0, Math.PI * 2); ctx.fillStyle = bodyColor; ctx.fill(); }
  } else {
    for (const s of [-1, 1]) {
      const lh = legH + (wc ? Math.max(0, s > 0 ? lp : lp2) : 0) * 1.5;
      ctx.beginPath(); ctx.moveTo(s * 3.5, 0); ctx.lineTo(s * 3.5, lh); ctx.strokeStyle = darkColor; ctx.lineWidth = 1.8; ctx.lineCap = "round"; ctx.stroke();
      ctx.beginPath(); ctx.arc(s * 3.5, lh, 1.2, 0, Math.PI * 2); ctx.fillStyle = darkColor; ctx.fill();
    }
    for (const s of [-1, 1]) {
      const lh = legH + (wc ? Math.max(0, s > 0 ? lp2 : lp) : 0.5) * 1.5;
      ctx.beginPath(); ctx.moveTo(s * 1.2, 0); ctx.lineTo(s * 1.2, lh); ctx.strokeStyle = darkColor; ctx.lineWidth = 1.8; ctx.lineCap = "round"; ctx.stroke();
      ctx.beginPath(); ctx.arc(s * 1.2, lh, 1.2, 0, Math.PI * 2); ctx.fillStyle = darkColor; ctx.fill();
    }
  }

  // Body
  ctx.beginPath(); ctx.ellipse(0, -bodyH * 0.3, bodyW * 0.5, bodyH * 0.5, 0, 0, Math.PI * 2); ctx.fillStyle = bodyColor; ctx.fill();
  // Belly
  ctx.beginPath(); ctx.ellipse(0, -bodyH * 0.1, bodyW * 0.3, bodyH * 0.3, 0, 0, Math.PI * 2); ctx.fillStyle = bellyColor; ctx.fill();
  // Shoulder highlight
  ctx.fillStyle = bodyLight; ctx.globalAlpha = 0.3;
  ctx.beginPath(); ctx.ellipse(-bodyW * 0.2, -bodyH * 0.45, bodyW * 0.15, bodyH * 0.2, -0.3, 0, Math.PI * 2); ctx.fill();
  ctx.globalAlpha = 1;

  // Head
  const headY = sitting ? -bodyH * 0.3 : -bodyH * 0.5 - headR * 0.5;
  ctx.save(); ctx.translate(0, headY);

  const earAngle = pet.earAngle + (pet.action === "excited" ? 0.3 : (pet.action === "sleep" ? -0.15 : 0));
  for (const s of [-1, 1]) {
    ctx.save(); ctx.translate(s * earW * 0.6, -headR * 0.3); ctx.rotate(s * earAngle);
    ctx.beginPath(); ctx.moveTo(0, 0); ctx.lineTo(-earW * 0.5, -earH); ctx.lineTo(earW * 0.5, -earH); ctx.closePath(); ctx.fillStyle = bodyColor; ctx.fill();
    ctx.beginPath(); ctx.moveTo(0, -2); ctx.lineTo(-earW * 0.3, -earH + 2); ctx.lineTo(earW * 0.3, -earH + 2); ctx.closePath(); ctx.fillStyle = bellyColor; ctx.fill();
    ctx.restore();
  }

  if (pet.action === "sleep") {
    ctx.beginPath(); ctx.ellipse(0, 0, headR, headR * 0.9, 0, 0, Math.PI * 2); ctx.fillStyle = bodyColor; ctx.fill();
    ctx.beginPath(); ctx.ellipse(0, headR * 0.2, headR * 0.6, headR * 0.4, 0, 0, Math.PI * 2); ctx.fillStyle = bellyColor; ctx.fill();
    for (const s of [-1, 1]) { const ex = s * headR * 0.5; ctx.beginPath(); ctx.arc(ex, -headR * 0.15, 1.8, 0, Math.PI); ctx.strokeStyle = darkColor; ctx.lineWidth = 0.8; ctx.stroke(); }
    ctx.beginPath(); ctx.arc(0, headR * 0.25, 1, 0, Math.PI * 2); ctx.fillStyle = darkColor; ctx.fill();
    ctx.save(); ctx.fillStyle = isDark ? "rgba(200,200,255,0.35)" : "rgba(100,100,180,0.25)"; ctx.font = "5px sans-serif";
    for (let i = 0; i < 3; i++) { ctx.globalAlpha = 0.25 - i * 0.06; ctx.fillText("z", 6 + i * 4 + Math.sin(t * 0.001 + i) * 1, -headR - 5 - i * 5); }
    ctx.restore();
  } else {
    ctx.beginPath(); ctx.ellipse(0, 0, headR, headR * 0.9, 0, 0, Math.PI * 2); ctx.fillStyle = bodyColor; ctx.fill();
    ctx.beginPath(); ctx.ellipse(0, headR * 0.2, headR * 0.6, headR * 0.4, 0, 0, Math.PI * 2); ctx.fillStyle = bellyColor; ctx.fill();

    const lookTarget = mouseX !== null ? clamp((mouseX - pet.x) * 0.02, -1.5, 1.5) : pet.lookX * 0.01;
    const blinking = pet.blink > 0;
    for (const s of [-1, 1]) {
      const ex = s * headR * 0.5;
      ctx.beginPath(); ctx.ellipse(ex, -headR * 0.15, 2.8, blinking ? 0.3 : 2.5, 0, 0, Math.PI * 2); ctx.fillStyle = "#f0f0f0"; ctx.fill();
      ctx.strokeStyle = darkColor; ctx.lineWidth = 0.4; ctx.stroke();
      if (!blinking) {
        ctx.beginPath(); ctx.arc(ex + lookTarget * 0.4, -headR * 0.15 + 0.2, 1.4, 0, Math.PI * 2); ctx.fillStyle = isDark ? "#1a1a2e" : "#2a2a3e"; ctx.fill();
        ctx.beginPath(); ctx.arc(ex + lookTarget * 0.4 + 0.6, -headR * 0.15 - 0.4, 0.6, 0, Math.PI * 2); ctx.fillStyle = "rgba(255,255,255,0.7)"; ctx.fill();
        ctx.beginPath(); ctx.arc(ex + lookTarget * 0.4 + 0.4, -headR * 0.15 - 0.5, 0.3, 0, Math.PI * 2); ctx.fillStyle = "rgba(255,255,255,0.4)"; ctx.fill();
      }
    }
    ctx.beginPath(); ctx.ellipse(0, headR * 0.25, 1.2, 0.8, 0, 0, Math.PI * 2); ctx.fillStyle = darkColor; ctx.fill();
    ctx.beginPath(); ctx.arc(0, headR * 0.4, 1.5, 0.1, Math.PI - 0.1); ctx.strokeStyle = darkColor; ctx.lineWidth = 0.5; ctx.stroke();
    ctx.beginPath(); ctx.arc(0, headR * 0.35, 0.4, 0, Math.PI * 2); ctx.fillStyle = darkColor; ctx.fill();
    ctx.save(); ctx.globalAlpha = 0.12; ctx.strokeStyle = darkColor; ctx.lineWidth = 0.3;
    for (const s of [-1, 1]) { ctx.beginPath(); ctx.moveTo(s * 2, headR * 0.25); ctx.lineTo(s * 5, headR * 0.1); ctx.stroke();
      ctx.beginPath(); ctx.moveTo(s * 2, headR * 0.25); ctx.lineTo(s * 5.5, headR * 0.35); ctx.stroke(); }
    ctx.restore();
  }

  // Blush
  if (pet.action === "excited" || pet.action === "walk") {
    ctx.save(); ctx.globalAlpha = 0.18;
    for (const s of [-1, 1]) { ctx.beginPath(); ctx.ellipse(s * headR * 0.6, headR * 0.15, 2.5, 1.5, 0, 0, Math.PI * 2); ctx.fillStyle = "#E8A080"; ctx.fill(); }
    ctx.restore();
  }

  ctx.restore(); ctx.restore(); // head + full pet

  // Shadow
  ctx.save(); ctx.globalAlpha = 0.1;
  ctx.beginPath(); ctx.ellipse(px, (pet.y + pet.jumpY) * S + (sitting ? 2 : legH + 1) * sc, 6 * sc, 2 * sc, 0, 0, Math.PI * 2);
  ctx.fillStyle = "#000"; ctx.fill(); ctx.restore();
}

function updatePet(pet: PetState, dt: number, W: number, agentBusy: boolean) {
  pet.timer += dt;
  pet.blink = Math.max(0, pet.blink - dt * 0.003);
  if (pet.blink === 0 && Math.random() < 0.0015) pet.blink = 1;

  if (pet.jumpY > 0 || pet.jumpVy !== 0) {
    pet.jumpVy -= 0.0018 * dt;
    pet.jumpY += pet.jumpVy * dt * 0.06;
    if (pet.jumpY <= 0) { pet.jumpY = 0; pet.jumpVy = 0; }
  }

  pet.earAngle = Math.sin(pet.timer * 0.001) * 0.1;

  if (agentBusy && pet.action !== "excited" && pet.action !== "sleep" && Math.random() < 0.003) {
    pet.action = "excited"; pet.jumpVy = 3; pet.timer = 0;
  }

  switch (pet.action) {
    case "sleep":
      if (pet.timer > 3000 + Math.random() * 3000) { pet.action = "idle"; pet.timer = 0; }
      break;
    case "scratch":
      if (pet.timer > 1200 + Math.random() * 800) { pet.action = "idle"; pet.timer = 0; }
      break;
    case "idle":
      if (pet.timer > 1500 + Math.random() * 2000) {
        const r = Math.random();
        if (r < 0.35) { pet.action = "walk"; pet.targetX = clamp(pet.x + (Math.random() > 0.5 ? 1 : -1) * rand(40, 120), PET_W, W - PET_W); pet.timer = 0; pet.dir = pet.targetX > pet.x ? 1 : -1; }
        else if (r < 0.5) { pet.action = "look"; pet.timer = 0; }
        else if (r < 0.56) { pet.action = "scratch"; pet.timer = 0; }
        else { pet.action = "walk"; pet.targetX = clamp(pet.x + (Math.random() > 0.5 ? 1 : -1) * rand(40, 120), PET_W, W - PET_W); pet.timer = 0; pet.dir = pet.targetX > pet.x ? 1 : -1; }
      }
      break;
    case "look":
      if (pet.timer > 800 + Math.random() * 600) { pet.action = "idle"; pet.timer = 0; }
      pet.lookX = Math.sin(pet.timer * 0.004) * 5;
      break;
    case "walk": {
      const speed = agentBusy ? 0.08 : 0.04;
      const dx = pet.targetX - pet.x;
      if (Math.abs(dx) < 2) { pet.action = "idle"; pet.timer = 0; }
      else { pet.x += Math.sign(dx) * speed * dt; pet.dir = dx > 0 ? 1 : -1; }
      if (agentBusy && Math.random() < 0.002) pet.jumpVy = 2.5;
      break;
    }
    case "excited":
      if (pet.timer > 600) { pet.action = "idle"; pet.timer = 0; }
      else if (pet.timer > 200 && pet.timer % 300 < 20) pet.jumpVy = 2;
      pet.lookX = Math.sin(pet.timer * 0.01) * 3;
      pet.x = clamp(pet.x + Math.sin(pet.timer * 0.006) * 0.008 * dt, PET_W, W - PET_W);
      break;
  }
  pet.tailPhase += agentBusy ? 0.06 * dt * 0.06 : 0.02 * dt * 0.06;
}

// ── Spawn helpers ──
function spawnBird(particles: Particle[], W: number, S: number) {
  const side = Math.random() > 0.5;
  particles.push({ x: side ? -10 : W + 10, y: rand(8, 22) * S, vx: side ? rand(0.4, 0.9) : rand(-0.9, -0.4), vy: rand(-0.04, 0.04), life: 1, maxLife: 300 + rand(0, 200), type: "bird", size: 1 });
}
function spawnLeaf(particles: Particle[], W: number) {
  particles.push({ x: rand(0, W), y: rand(0.35, 0.55) * H, vx: rand(-0.12, 0.12), vy: rand(0.08, 0.2), life: 1, maxLife: 400 + rand(0, 300), type: "leaf", size: 1.2 + rand(0, 0.8) });
}
function spawnPetal(particles: Particle[], x: number, y: number) {
  for (let i = 0; i < 5; i++) particles.push({ x: x + rand(-3, 3), y, vx: rand(-0.3, 0.3), vy: rand(-0.2, -0.05), life: 1, maxLife: 60 + rand(0, 40), type: "petal", size: 1.5 + rand(0, 1) });
}
function spawnFirefly(particles: Particle[], W: number) {
  particles.push({ x: rand(0, W), y: rand(0.45, 0.75) * H, vx: rand(-0.08, 0.08), vy: rand(-0.08, 0.08), life: 1, maxLife: 500 + rand(0, 400), type: "firefly", size: 1 + rand(0, 0.5) });
}
function spawnRipple(particles: Particle[], x: number, y: number) {
  for (let i = 0; i < 4; i++) particles.push({ x, y: y + rand(-0.5, 0.5), vx: 0, vy: 0, life: 1, maxLife: 35 + i * 10, type: "ripple", size: 2 + i * 1.5 });
}
function spawnSparkles(particles: Particle[], x: number, y: number, count: number) {
  for (let i = 0; i < count; i++) particles.push({ x: x + rand(-4, 4), y: y + rand(-4, 4), vx: rand(-0.15, 0.15), vy: rand(-0.4, -0.08), life: 1, maxLife: 25 + rand(0, 25), type: "sparkle", size: 1.5 + rand(0, 1) });
}
function spawnGlow(particles: Particle[], x: number, y: number, count: number) {
  for (let i = 0; i < count; i++) particles.push({ x: x + rand(-6, 6), y: y + rand(-6, 6), vx: rand(-0.05, 0.05), vy: rand(-0.08, -0.02), life: 1, maxLife: 40 + rand(0, 30), type: "glow", size: 1.5 + rand(0, 1) });
}

const PetBar: React.FC = () => {
  const hasMessages = useStore(s => (s.sessions[s.activeSessionIndex]?.messages?.length ?? 0) > 0);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animRef = useRef(0);
  const timeRef = useRef(0);
  const sceneRef = useRef({ W: 0, collapsed: false });
  const mouseRef = useRef<number | null>(null);
  const mouseYRef = useRef<number | null>(null);
  const clickCd = useRef(0);
  const particlesRef = useRef<Particle[]>([]);
  const petRef = useRef<PetState>({ x: 60, y: GROUND_Y - 2, targetX: 100, action: "idle", step: 0, timer: 0, earAngle: 0, tailPhase: 0, lookX: 0, jumpY: 0, jumpVy: 0, blink: 0, dir: 1, sitTimer: 0 });
  const nightToggle = useRef(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const birdTimer = useRef(0);
  const leafTimer = useRef(0);
  const fireflyTimer = useRef(0);

  function hitTest(x: number, y: number, W: number): string {
    const p = petRef.current;
    const petY = (p.y + p.jumpY) * (sceneRef.current.collapsed ? HC / H : 1);
    if (Math.abs(x - p.x) < 14 && Math.abs(y - petY) < 14) return "pet";
    const cx = W * 0.8, cy = (nightToggle.current ? 14 : 18) * (sceneRef.current.collapsed ? HC / H : 1);
    if (Math.abs(x - cx) < 18 && Math.abs(y - cy) < 18) return "celestial";
    if (y >= 68 && y <= 82) return "river";
    if (y >= 40 && y <= 68) return "trees";
    return "ground";
  }

  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    // Always render at H for full quality; CSS scaleY handles collapsed
    const resize = () => {
      const w = container.clientWidth;
      sceneRef.current.W = w;
      canvas.width = w * devicePixelRatio;
      canvas.height = H * devicePixelRatio;
      canvas.style.width = `${w}px`;
      canvas.style.height = `${H}px`;
    };
    resize();
    const ro = new ResizeObserver(resize);
    ro.observe(container);

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const scaleY = () => container.dataset.collapsed === "true" ? HC / H : 1;

    const getCanvasY = (clientY: number) => { const r = canvas.getBoundingClientRect(); return (clientY - r.top) / scaleY(); };

    const mouseMove = (e: MouseEvent) => {
      const r = canvas.getBoundingClientRect();
      mouseRef.current = e.clientX - r.left;
      mouseYRef.current = getCanvasY(e.clientY);
    };
    const mouseLeave = () => { mouseRef.current = null; mouseYRef.current = null; };
    canvas.addEventListener("mousemove", mouseMove);
    canvas.addEventListener("mouseleave", mouseLeave);

    canvas.addEventListener("click", (e: MouseEvent) => {
      const now = performance.now();
      if (now - clickCd.current < 200) return;
      clickCd.current = now;
      const r = canvas.getBoundingClientRect();
      const cx = e.clientX - r.left;
      const cy = getCanvasY(e.clientY);
      const W = sceneRef.current.W;
      const hit = hitTest(cx, cy, W);
      const particles = particlesRef.current;
      const pet = petRef.current;
      switch (hit) {
        case "pet":
          if (pet.jumpY < 1) { pet.jumpVy = 5; pet.action = "excited"; pet.timer = 0; }
          spawnSparkles(particles, pet.x, (pet.y - 10) * 1, 10);
          spawnGlow(particles, pet.x, pet.y * 1, 6);
          break;
        case "celestial":
          nightToggle.current = !nightToggle.current;
          spawnGlow(particles, W * 0.8, (nightToggle.current ? 14 : 18) * 1, 15);
          spawnSparkles(particles, W * 0.8, (nightToggle.current ? 14 : 18) * 1, 8);
          break;
        case "river":
          spawnRipple(particles, cx, cy);
          spawnSparkles(particles, cx, cy, 4);
          break;
        case "trees":
          for (let i = 0; i < 3 + Math.floor(Math.random() * 2); i++) spawnBird(particles, W, 1);
          spawnPetal(particles, cx, cy);
          break;
        case "ground":
          spawnSparkles(particles, cx, cy, 6);
          spawnGlow(particles, cx, cy, 4);
          spawnPetal(particles, cx, cy);
          break;
      }
    });

    let running = true, lastTime = performance.now();

    const loop = (now: number) => {
      if (!running) return;
      const dt = Math.min(now - lastTime, 50);
      lastTime = now;
      timeRef.current += dt;
      const t = timeRef.current;
      const { W } = sceneRef.current;
      if (W === 0) { animRef.current = requestAnimationFrame(loop); return; }

      const store = useStore.getState();
      const sysDark = store.settings.theme === "dark" || (store.settings.theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
      const isDark = nightToggle.current ? !sysDark : sysDark;
      const agentBusy = store.agentBusy;
      const collapsed = (store.sessions[store.activeSessionIndex]?.messages?.length ?? 0) > 0;
      sceneRef.current.collapsed = collapsed;
      container.dataset.collapsed = collapsed ? "true" : "false";
      const S = collapsed ? HC / H : 1;

      updatePet(petRef.current, dt, W, agentBusy);

      const particles = particlesRef.current;
      birdTimer.current += dt;
      if (birdTimer.current > 4000 + Math.random() * 3000) { birdTimer.current = 0; if (Math.random() < 0.25) spawnBird(particles, W, S); }
      leafTimer.current += dt;
      if (leafTimer.current > 700 + Math.random() * 500) { leafTimer.current = 0; spawnLeaf(particles, W); }
      if (isDark) { fireflyTimer.current += dt; if (fireflyTimer.current > 500 + Math.random() * 400) { fireflyTimer.current = 0; if (particles.filter(p => p.type === "firefly").length < 5) spawnFirefly(particles, W); } }

      for (let i = particles.length - 1; i >= 0; i--) {
        const p = particles[i]; p.life += dt * 0.002; p.x += p.vx; p.y += p.vy;
        if (p.type === "bird") p.vy += Math.sin(t * 0.01 + p.x) * 0.005;
        if (p.type === "leaf" || p.type === "petal") p.vx += Math.sin(t * 0.003 + p.x) * 0.003;
        if (p.life >= p.maxLife || p.x < -30 || p.x > W + 30) { particles.splice(i, 1); }
      }

      ctx.save();
      ctx.scale(devicePixelRatio, devicePixelRatio);
      drawScene(ctx, W, t, isDark, particles, S);
      drawPet(ctx, petRef.current, t, isDark, mouseRef.current, S);
      ctx.restore();

      animRef.current = requestAnimationFrame(loop);
    };
    animRef.current = requestAnimationFrame(loop);
    return () => { running = false; cancelAnimationFrame(animRef.current); ro.disconnect(); canvas.removeEventListener("mousemove", mouseMove); canvas.removeEventListener("mouseleave", mouseLeave); };
  }, []);

  return (
    <div ref={containerRef} className={styles.container} data-collapsed={hasMessages ? "true" : "false"}>
      <canvas ref={canvasRef} className={styles.canvas} />
    </div>
  );
};

export default PetBar;
