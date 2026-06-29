import React from "react";

interface AgentResult {
  nodeId: string;
  agentName: string;
  status: string;
  output: string;
  duration: string;
  startedAt: string;
  completedAt: string;
}

interface AgentResultModalProps {
  result: AgentResult | null;
  onClose: () => void;
}

const AgentResultModal: React.FC<AgentResultModalProps> = ({ result, onClose }) => {
  if (!result) return null;
  return (
    <div className="agent-result-overlay" onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div className="agent-result-modal glass-panel">
        <div className="agent-result-header">
          <h3>{result.agentName}</h3>
          <button className="btn-icon" onClick={onClose}>
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M2 2l8 8M10 2l-8 8"/></svg>
          </button>
        </div>
        <div className="agent-result-meta">
          <span>状态: {result.status}</span>
          <span>耗时: {result.duration}</span>
          <span>开始: {result.startedAt}</span>
          <span>完成: {result.completedAt}</span>
        </div>
        <div className="agent-result-output">
          <pre>{result.output}</pre>
        </div>
      </div>
    </div>
  );
};

export default AgentResultModal;
export type { AgentResult };
