import React from "react";
import { useStore } from "../store";

const POS: Record<string, string> = { pos: "#34c759", neg: "#ff3b30", neut: "#ff9f0a" };

const SmartCanvas: React.FC = () => {
  const canvasNodes = useStore((s) => s.canvasNodes);
  const setCanvasActive = useStore((s) => s.setCanvasActive);

  const node = canvasNodes[0];
  const hasData = !!node;

  return (
    <div className="smart-canvas" data-empty={!hasData}>
      <div className="smart-canvas-header">
        <span className="smart-canvas-title">E8 Reasoning</span>
        {hasData && (
          <span className="smart-canvas-stats">
            {node.label}
          </span>
        )}
        <button className="btn-icon smart-canvas-close" onClick={() => setCanvasActive(false)} title="Close">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
            <path d="M2 2l8 8M10 2l-8 8" />
          </svg>
        </button>
      </div>
      <div className="smart-canvas-body">
        {hasData ? (
          <div
            className="canvas-settle-card"
            style={{
              background: POS[node.color] || node.color || "#aeaeb2",
            }}
          >
            <div className="canvas-settle-label">{node.label}</div>
            {node.content && <div className="canvas-settle-content">{node.content}</div>}
          </div>
        ) : (
          <div className="smart-canvas-empty">等待推理...</div>
        )}
      </div>
    </div>
  );
};

export default SmartCanvas;
