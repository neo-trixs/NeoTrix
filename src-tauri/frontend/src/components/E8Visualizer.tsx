import { useEffect, useRef } from "react";

interface E8AttentionData {
  scores: number[];
  top_roots: [string, number][];
  tick: number;
}

interface E8VisualizerProps {
  data?: E8AttentionData | null;
  cycle?: number;
}

const E8_COLS = 24;
const E8_ROWS = 10;

const E8Visualizer: React.FC<E8VisualizerProps> = ({ data, cycle }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const scores = data?.scores || Array(240).fill(0.01);
  const tick = data?.tick || 0;

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const w = canvas.width;
    const h = canvas.height;
    const cellW = w / E8_COLS;
    const cellH = h / E8_ROWS;

    ctx.clearRect(0, 0, w, h);

    for (let i = 0; i < 240 && i < scores.length; i++) {
      const col = i % E8_COLS;
      const row = Math.floor(i / E8_COLS);
      const v = Math.min(1, Math.max(0, scores[i] || 0.01));

      const r = Math.round(10 + v * 120);
      const g = Math.round(30 + (1 - v) * 180);
      const b = Math.round(200 + v * 55);

      ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
      ctx.fillRect(col * cellW + 1, row * cellH + 1, cellW - 2, cellH - 2);

      const label = `${(v * 100).toFixed(0)}`;
      if (cellW > 14 && cellH > 10) {
        ctx.fillStyle = "rgba(255,255,255,0.7)";
        ctx.font = `${Math.min(8, cellW * 0.5)}px monospace`;
        ctx.textAlign = "center";
        ctx.fillText(label, col * cellW + cellW / 2, row * cellH + cellH / 2 + 3);
      }
    }

    if (data?.top_roots?.length) {
      ctx.fillStyle = "rgba(255,255,255,0.6)";
      ctx.font = "8px monospace";
      ctx.textAlign = "left";
      data.top_roots.slice(0, 4).forEach(([name, _score], i) => {
        ctx.fillText(`★ ${name}`, 4, h - 8 - i * 10);
      });
    }
  }, [scores, tick, data]);

  return (
    <div className="cd-card">
      <div className="cd-card-title">
        E8 Attention Lattice
        {cycle !== undefined && (
          <span style={{ fontSize: 10, opacity: 0.5, marginLeft: 8 }}>
            cycle #{cycle}
          </span>
        )}
      </div>
      <canvas
        ref={canvasRef}
        width={E8_COLS * 18}
        height={E8_ROWS * 16}
        style={{
          width: "100%",
          aspectRatio: `${E8_COLS * 18} / ${E8_ROWS * 16}`,
          borderRadius: 6,
          background: "rgba(0,0,0,0.15)",
        }}
      />
      <div style={{ display: "flex", justifyContent: "space-between", marginTop: 4, fontSize: 10, opacity: 0.4 }}>
        <span>240 E8 roots · cell = attention score</span>
        <span>tick {tick}</span>
      </div>
    </div>
  );
};

export default E8Visualizer;
