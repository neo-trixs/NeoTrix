import React, { useMemo, useCallback, useState, useEffect, useRef } from "react";
import {
  ReactFlow,
  Background,
  BackgroundVariant,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  addEdge,
  MarkerType,
  type Node,
  type Edge,
  type Connection,
  type NodeTypes,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { listen } from "@tauri-apps/api/event";
import { useStore } from "../store";
import * as api from "../lib/api";
import AgentFlowNode from "./AgentFlowNode";
import type { AgentFlowNodeData, AgentStatus, AgentNodeType } from "../types";
import type { DistillationNode, DistillationEdge, DistillationFlowEvent, UserAvatar } from "../lib/api";

type FlowPreset = "example" | "simple" | "empty" | "live";

interface PresetConfig {
  nodes: Node<AgentFlowNodeData>[];
  edges: Edge[];
}

const PRESETS: Record<string, PresetConfig> = {
  example: {
    nodes: [
      {
        id: "user-input", type: "agentFlowNode", position: { x: 50, y: 200 },
        data: { label: "用户请求", agentType: "input", status: "completed", description: "来自用户的原始请求输入", duration: "0.1s" },
      },
      {
        id: "orchestrator", type: "agentFlowNode", position: { x: 280, y: 200 },
        data: { label: "Orchestrator", agentType: "orchestrator", status: "running", description: "任务分解与调度", duration: "2.3s", progress: 0.6 },
      },
      {
        id: "planner", type: "agentFlowNode", position: { x: 510, y: 200 },
        data: { label: "Planner", agentType: "planner", status: "completed", description: "将任务拆解为可执行的子步骤", duration: "1.8s", steps: { done: 5, total: 5 } },
      },
      {
        id: "code-writer", type: "agentFlowNode", position: { x: 740, y: 30 },
        data: { label: "Code Writer", agentType: "sub-agent", status: "running", description: "根据计划编写代码", duration: "4.5s", steps: { done: 3, total: 5 } },
      },
      {
        id: "reviewer", type: "agentFlowNode", position: { x: 740, y: 200 },
        data: { label: "Code Reviewer", agentType: "critic", status: "pending", description: "审查代码质量", duration: "0s", steps: { done: 0, total: 3 } },
      },
      {
        id: "tester", type: "agentFlowNode", position: { x: 740, y: 370 },
        data: { label: "Test Runner", agentType: "sub-agent", status: "pending", description: "运行测试验证", duration: "0s" },
      },
      {
        id: "aggregator", type: "agentFlowNode", position: { x: 1000, y: 200 },
        data: { label: "Aggregator", agentType: "aggregator", status: "idle", description: "汇总所有子 Agent 输出", duration: "0s" },
      },
      {
        id: "final-output", type: "agentFlowNode", position: { x: 1230, y: 200 },
        data: { label: "最终输出", agentType: "output", status: "idle", description: "整合后的完整响应", duration: "0s" },
      },
    ],
    edges: [
      { id: "e-user-orch", source: "user-input", target: "orchestrator", style: { stroke: "#86868b", strokeWidth: 1.5 }, markerEnd: { type: MarkerType.ArrowClosed, color: "#86868b" } },
      { id: "e-orch-plan", source: "orchestrator", target: "planner", animated: true, style: { stroke: "#007aff", strokeWidth: 2 }, markerEnd: { type: MarkerType.ArrowClosed, color: "#007aff" } },
      { id: "e-plan-code", source: "planner", target: "code-writer", animated: true, style: { stroke: "#5856d6", strokeWidth: 1.5 }, markerEnd: { type: MarkerType.ArrowClosed, color: "#5856d6" } },
      { id: "e-plan-review", source: "planner", target: "reviewer", style: { stroke: "#5856d6", strokeWidth: 1.5 }, markerEnd: { type: MarkerType.ArrowClosed, color: "#5856d6" } },
      { id: "e-plan-test", source: "planner", target: "tester", style: { stroke: "#5856d6", strokeWidth: 1.5 }, markerEnd: { type: MarkerType.ArrowClosed, color: "#5856d6" } },
      { id: "e-code-agg", source: "code-writer", target: "aggregator", style: { stroke: "#34c759", strokeWidth: 1.5 }, markerEnd: { type: MarkerType.ArrowClosed, color: "#34c759" } },
      { id: "e-review-agg", source: "reviewer", target: "aggregator", style: { stroke: "#ff9500", strokeWidth: 1.5 }, markerEnd: { type: MarkerType.ArrowClosed, color: "#ff9500" } },
      { id: "e-test-agg", source: "tester", target: "aggregator", style: { stroke: "#86868b", strokeWidth: 1.5 }, markerEnd: { type: MarkerType.ArrowClosed, color: "#86868b" } },
      { id: "e-agg-out", source: "aggregator", target: "final-output", style: { stroke: "#86868b", strokeWidth: 1.5 }, markerEnd: { type: MarkerType.ArrowClosed, color: "#86868b" } },
    ],
  },
  simple: {
    nodes: [
      { id: "s-input", type: "agentFlowNode", position: { x: 50, y: 150 }, data: { label: "请求", agentType: "input", status: "completed", description: "用户输入", duration: "0.1s" } },
      { id: "s-agent", type: "agentFlowNode", position: { x: 300, y: 150 }, data: { label: "Agent", agentType: "orchestrator", status: "completed", description: "单 Agent 执行", duration: "3.2s", progress: 1 } },
      { id: "s-output", type: "agentFlowNode", position: { x: 550, y: 150 }, data: { label: "响应", agentType: "output", status: "completed", description: "最终输出", duration: "0s" } },
    ],
    edges: [
      { id: "e-si-sa", source: "s-input", target: "s-agent", style: { stroke: "#86868b", strokeWidth: 1.5 }, markerEnd: { type: MarkerType.ArrowClosed } },
      { id: "e-sa-so", source: "s-agent", target: "s-output", style: { stroke: "#86868b", strokeWidth: 1.5 }, markerEnd: { type: MarkerType.ArrowClosed } },
    ],
  },
  empty: { nodes: [], edges: [] },
};

function distillationNodeToFlowNode(dn: DistillationNode, index: number): Node<AgentFlowNodeData> {
  const baseX = 60;
  const baseY = 40;
  return {
    id: dn.id,
    type: "agentFlowNode",
    position: { x: baseX + (index % 3) * 260, y: baseY + Math.floor(index / 3) * 170 },
    data: {
      label: dn.label,
      agentType: (dn.type === "orchestrator" ? "orchestrator" : dn.type === "planner" ? "planner" : dn.type === "aggregator" ? "aggregator" : dn.type === "critic" ? "critic" : "sub-agent") as AgentNodeType,
      status: (dn.status === "running" ? "running" : dn.status === "completed" ? "completed" : dn.status === "fading" ? "idle" : "pending") as AgentStatus,
      description: dn.description,
      progress: dn.progress,
      duration: dn.ttl_seconds > 0 ? `${dn.ttl_seconds.toFixed(0)}s` : undefined,
    },
  };
}

function distillationEdgeToFlowEdge(de: DistillationEdge): Edge {
  return {
    id: `e-${de.source}-${de.target}`,
    source: de.source,
    target: de.target,
    style: { stroke: "#5856d6", strokeWidth: 1.5 },
    markerEnd: { type: MarkerType.ArrowClosed, color: "#5856d6" },
  };
}

const AgentFlow: React.FC = () => {
  const setAgentFlowActive = useStore((s) => s.setAgentFlowActive);
  const [preset, setPreset] = useState<string>("example");
  const [showLive, setShowLive] = useState(false);
  const [avatarSummary, setAvatarSummary] = useState("");
  const [avatarConfidence, setAvatarConfidence] = useState(0);
  const [avatarEdition, setAvatarEdition] = useState(0);
  const [messageInCount, setMessageInCount] = useState(0);

  const [nodes, setNodes, onNodesChange] = useNodesState(PRESETS.example.nodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(PRESETS.example.edges);

  const fadeTimersRef = useRef<Map<string, number>>(new Map());
  const showLiveRef = useRef(false);
  showLiveRef.current = showLive;

  const nodeTypes: NodeTypes = useMemo(() => ({ agentFlowNode: AgentFlowNode }), []);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge({ ...params, markerEnd: { type: MarkerType.ArrowClosed } }, eds)),
    [setEdges],
  );

  const handlePresetChange = useCallback((newPreset: string) => {
    setPreset(newPreset);
    setShowLive(false);
    const p = PRESETS[newPreset];
    if (p) {
      setNodes(p.nodes);
      setEdges(p.edges);
    }
  }, [setNodes, setEdges]);

  const handleRefresh = useCallback(() => {
    if (showLive) {
      api.getDistillationFlow().then((flow) => {
        const flowNodes = flow.nodes.map((dn, i) => distillationNodeToFlowNode(dn, i));
        const flowEdges = flow.edges.map(distillationEdgeToFlowEdge);
        setNodes(flowNodes);
        setEdges(flowEdges);
        setAvatarSummary(flow.avatar_summary);
        setAvatarConfidence(Math.round(flow.avatar_confidence * 100));
      }).catch(() => {});
      api.getChainStats().then((s) => setAvatarEdition(s.identity_edition)).catch(() => {});
    } else {
      const p = PRESETS[preset];
      if (p) { setNodes(p.nodes); setEdges(p.edges); }
    }
  }, [preset, showLive, setNodes, setEdges]);

  const handleLiveDistillation = useCallback(() => {
    setPreset("live");
    setShowLive(true);
    api.getDistillationFlow().then((flow) => {
      const flowNodes = flow.nodes.map((dn, i) => distillationNodeToFlowNode(dn, i));
      const flowEdges = flow.edges.map(distillationEdgeToFlowEdge);
      setNodes(flowNodes);
      setEdges(flowEdges);
      setAvatarSummary(flow.avatar_summary);
      setAvatarConfidence(Math.round(flow.avatar_confidence * 100));
    }).catch(() => {});
    api.getChainStats().then((s) => setAvatarEdition(s.identity_edition)).catch(() => {});
  }, [setNodes, setEdges]);

  const handleAutoLayout = useCallback(() => {
    setNodes((nds) =>
      nds.map((n, i) => ({
        ...n,
        position: { x: 50 + Math.floor(i / 4) * 280, y: 50 + (i % 4) * 170 },
      }))
    );
  }, [setNodes]);

  useEffect(() => {
    handleLiveDistillation();
    api.getChainStats().then((s) => {
      setAvatarEdition(s.identity_edition);
    }).catch(() => {});
  }, []);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    listen<DistillationFlowEvent>("distillation-update", (event) => {
      const flow = event.payload;
      if (showLive) {
        const flowNodes = flow.nodes.map((dn, i) => distillationNodeToFlowNode(dn, i));
        const flowEdges = flow.edges.map(distillationEdgeToFlowEdge);
        setNodes(flowNodes);
        setEdges(flowEdges);
      }
      setAvatarSummary(flow.avatar_summary);
      setAvatarConfidence(Math.round(flow.avatar_confidence * 100));
      setMessageInCount((c) => c + 1);
      api.getChainStats().then((s) => setAvatarEdition(s.identity_edition)).catch(() => {});
    }).then((fn) => unlisteners.push(fn));

    listen<UserAvatar>("avatar-updated", () => {
      if (!showLiveRef.current) {
        setShowLive(true);
        setPreset("live");
      }
    }).then((fn) => unlisteners.push(fn));

    return () => {
      for (const unlisten of unlisteners) {
        unlisten();
      }
      for (const [, timer] of fadeTimersRef.current) {
        clearTimeout(timer);
      }
    };
  }, [showLive, setNodes, setEdges]);

  return (
    <div className="agent-flow">
      <div className="agent-flow-toolbar">
        <div className="agent-flow-toolbar-left">
          <h2>Agent Flow</h2>
          {showLive && avatarSummary && (
            <span className="agent-flow-avatar-badge" title={avatarSummary}>
              分身 v{avatarEdition} · {avatarConfidence}%
            </span>
          )}
          <select
            className="agent-flow-select"
            value={preset}
            onChange={(e) => handlePresetChange(e.target.value)}
          >
            <option value="example">示例: 多 Agent 协作</option>
            <option value="simple">简单: 单一 Agent</option>
            <option value="empty">空白画布</option>
            {showLive && <option value="live">🟢 实时蒸馏</option>}
          </select>
        </div>
        <div className="agent-flow-toolbar-right">
          <button className={`btn-icon${showLive ? " active" : ""}`} onClick={handleLiveDistillation} title="实时蒸馏 (自动)">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round">
              <circle cx="7" cy="7" r="5" />
              <path d="M7 3v8M3 7h8" />
              <circle cx="7" cy="7" r="1.5" fill="currentColor" stroke="none" />
            </svg>
          </button>
          <button className="btn-icon" onClick={handleRefresh} title="重置视图">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
              <path d="M1 7a6 6 0 0111.5-2M13 7a6 6 0 01-11.5 2" />
              <path d="M12.5 1v4h-4M1.5 13V9h4" />
            </svg>
          </button>
          <button className="btn-icon" onClick={handleAutoLayout} title="自动布局">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round">
              <rect x="1" y="1" width="5" height="4" rx="1" />
              <rect x="8" y="1" width="5" height="4" rx="1" />
              <rect x="1" y="8" width="5" height="5" rx="1" />
              <rect x="8" y="8" width="5" height="5" rx="1" />
            </svg>
          </button>
          <div className="agent-flow-divider" />
          <button className="btn-icon" onClick={() => setAgentFlowActive(false)} title="关闭">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
              <path d="M3 3l8 8M11 3l-8 8" />
            </svg>
          </button>
        </div>
      </div>

      <div className="agent-flow-canvas">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          nodeTypes={nodeTypes}
          fitView
          fitViewOptions={{ padding: 0.3 }}
          minZoom={0.1}
          maxZoom={2.5}
          defaultEdgeOptions={{
            style: { stroke: "#86868b", strokeWidth: 1.5 },
            markerEnd: { type: MarkerType.ArrowClosed, color: "#86868b" },
          }}
        >
          <Background variant={BackgroundVariant.Dots} gap={24} size={1} color="rgba(0,0,0,0.06)" />
          <Controls showInteractive={false} className="agent-flow-controls" />
          <MiniMap
            nodeStrokeColor="#86868b"
            nodeColor={(node) => {
              const d = node.data as AgentFlowNodeData;
              if (!d || !d.status) return "#e8e8ed";
              const MAP: Record<string, string> = { running: "#ff9500", completed: "#34c759", failed: "#ff3b30", pending: "#aeaeb2", idle: "#86868b" };
              return MAP[d.status] || "#e8e8ed";
            }}
            maskColor="rgba(0,0,0,0.08)"
            className="agent-flow-minimap"
          />
        </ReactFlow>
      </div>

      <div className="agent-flow-footer">
        <span className="agent-flow-footer-text">
          {showLive ? (
            <>
              🟢 实时蒸馏 · {nodes.length} 节点 · {edges.length} 连接 · {avatarConfidence}% 信心度 · {avatarSummary}
            </>
          ) : (
            <>{nodes.length} 节点 · {edges.length} 连接 · 无限画布 · 拖拽节点 · 滚轮缩放</>
          )}
        </span>
      </div>
    </div>
  );
};

export default AgentFlow;
