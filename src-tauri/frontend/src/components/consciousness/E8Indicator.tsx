import React from "react";
import type { E8State, E8Line } from "../../types";

interface Props {
  e8: E8State;
  interactive?: boolean;
  onClick?: () => void;
}

const HEXAGRAM_NAMES: Record<number, string> = {
  0x01: "Grounding",
  0x02: "Receptive",
  0x08: "Holding Together",
  0x16: "Enthusiasm",
  0x23: "Splitting Apart",
  0x2B: "Peace",
  0x35: "Progress",
  0x3C: "Disagreement",
  0x42: "Creating",
  0x4B: "Revolution",
  0x5A: "Abundance",
  0x6C: "Adversity",
  0x7B: "Completion",
  0x8F: "Analyzing",
  0xA5: "Deciding",
  0xC3: "Transcending",
  0xE4: "Illumination",
  0xFF: "Transcending",
};

export function hexagramToName(h: number): string {
  return HEXAGRAM_NAMES[h] ?? `0x${h.toString(16).toUpperCase().padStart(2, "0")}`;
}

function E8LineComponent({ line, index }: { line: E8Line; index: number }) {
  const fadeWidth = 100 - index * 8;
  return (
    <div className={`e8-line ${line.value === 1 ? "e8-line-yang" : "e8-line-yin"} ${line.changing ? "e8-line-changing" : ""}`}>
      <div
        className="e8-line-fill"
        style={{
          width: line.value === 1 ? `${fadeWidth}%` : `${Math.min(fadeWidth, 40)}%`,
        }}
      />
    </div>
  );
}

const E8Indicator: React.FC<Props> = ({ e8, interactive, onClick }) => {
  return (
    <div
      className="e8-indicator"
      onClick={onClick}
      role="status"
      aria-label={`E8 state: ${e8.hexagramName}`}
      title={`E8: ${e8.hexagramName} (0x${e8.hexagram.toString(16).toUpperCase().padStart(2, "0")}) — ${Math.round(e8.confidence * 100)}% confidence`}
      style={{ cursor: interactive ? "pointer" : "default" }}
    >
      <div className="e8-lines-container">
        {e8.lines.map((line, i) => (
          <E8LineComponent key={i} line={line} index={i} />
        ))}
      </div>
      <div className="e8-label-group">
        <span className="e8-label">{hexagramToName(e8.hexagram)}</span>
        <span className="e8-confidence">{Math.round(e8.confidence * 100)}%</span>
      </div>
    </div>
  );
};

export default E8Indicator;
