import { useState } from "react";

interface ReasoningTraceProps {
  content: string;
}

const THINKING_RE = /﹅﹅\s*thinking\s*﹅﹅([\s\S]*?)﹅﹅\s*\/thinking\s*﹅﹅/gi;

export default function ReasoningTrace({ content }: ReasoningTraceProps) {
  const [expanded, setExpanded] = useState(false);

  const match = THINKING_RE.exec(content);
  if (!match) return null;

  const reasoning = match[1].trim();
  const before = content.slice(0, match.index);
  const after = content.slice(match.index + match[0].length);

  return (
    <div className="reasoning-trace">
      <div
        className="reasoning-header"
        onClick={() => setExpanded(!expanded)}
      >
        <span className="reasoning-icon">⟡</span>
        <span className="reasoning-label">NeoTrix 正在思考...</span>
        <span className="reasoning-toggle">{expanded ? "▾" : "▸"}</span>
      </div>
      {expanded && (
        <div className="reasoning-body">
          {reasoning.split("\n").map((line, i) => (
            <p key={i} className="reasoning-line">{line}</p>
          ))}
        </div>
      )}
      <div className="reasoning-after" dangerouslySetInnerHTML={{ __html: before + after }} />
    </div>
  );
}
