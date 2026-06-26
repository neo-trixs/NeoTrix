import React, { memo } from "react";
import { Handle, Position, type Node, type NodeProps } from "@xyflow/react";
import type { AgentFlowNodeData, AgentStatus, AgentNodeType } from "../types";

type AgentFlowNodeType = Node<AgentFlowNodeData>;

const STATUS_COLORS: Record<AgentStatus, string> = {
  running: "#ff9500",
  completed: "#34c759",
  failed: "#ff3b30",
  pending: "#aeaeb2",
  idle: "#86868b",
};

const TYPE_LABELS: Record<AgentNodeType, string> = {
  orchestrator: "Orchestrator",
  planner: "Planner",
  "sub-agent": "Sub-Agent",
  critic: "Critic",
  aggregator: "Aggregator",
  input: "Input",
  output: "Output",
};

const TYPE_COLORS: Record<AgentNodeType, string> = {
  orchestrator: "#007aff",
  planner: "#5856d6",
  "sub-agent": "#34c759",
  critic: "#ff9500",
  aggregator: "#ff2d55",
  input: "#86868b",
  output: "#86868b",
};

const AgentFlowNode: React.FC<NodeProps<AgentFlowNodeType>> = ({ data, selected }) => {
  const { label, agentType, status, description, progress, duration, steps } = data;
  const statusColor = STATUS_COLORS[status];
  const typeColor = TYPE_COLORS[agentType];

  return (
    <div className={`agent-flow-node${selected ? " selected" : ""}`}>
      <div className="afn-header" style={{ borderBottomColor: statusColor }}>
        <span className="afn-status-dot" style={{ backgroundColor: statusColor }}>
          {status === "running" && <span className="afn-pulse" />}
        </span>
        <span className="afn-title">{label}</span>
        {duration && <span className="afn-duration">{duration}</span>}
      </div>

      <div className="afn-body">
        <div className="afn-type-badge" style={{ backgroundColor: `${typeColor}18`, color: typeColor }}>
          {TYPE_LABELS[agentType]}
        </div>

        <div className="afn-desc">{description}</div>

        {steps && (
          <div className="afn-progress">
            <div className="afn-progress-label">
              步骤 {steps.done}/{steps.total}
            </div>
            <div className="afn-progress-track">
              <div
                className="afn-progress-fill"
                style={{
                  width: `${Math.round((steps.done / steps.total) * 100)}%`,
                  backgroundColor: statusColor,
                }}
              />
            </div>
          </div>
        )}

        {progress !== undefined && !steps && (
          <div className="afn-progress">
            <div className="afn-progress-track">
              <div
                className="afn-progress-fill"
                style={{
                  width: `${Math.round(progress * 100)}%`,
                  backgroundColor: statusColor,
                }}
              />
            </div>
          </div>
        )}
      </div>

      <Handle type="target" position={Position.Left} className="afn-handle" />
      <Handle type="source" position={Position.Right} className="afn-handle" />
    </div>
  );
};

export default memo(AgentFlowNode);
