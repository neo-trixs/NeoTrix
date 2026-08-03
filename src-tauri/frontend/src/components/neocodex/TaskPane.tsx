import React, { useState, useRef, useEffect, useCallback, useMemo } from "react";
import styles from "./TaskPane.module.css";

export interface TaskStep {
  id: string;
  name: string;
  args: string;
  startedAt: number;
  doneAt?: number;
  status: "running" | "done" | "failed";
  success?: boolean;
  parentId?: string;
  depth?: number;
}

interface TaskPaneProps {
  steps: TaskStep[];
  startedAt: number | null;
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function prettyPrintJson(jsonStr: string): string {
  try {
    const parsed = JSON.parse(jsonStr);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return jsonStr;
  }
}

function TaskStepItem({
  step,
  isExpanded,
  onToggleExpand,
  onCopyArgs,
  now,
  depth = 0,
}: {
  step: TaskStep;
  isExpanded: boolean;
  onToggleExpand: () => void;
  onCopyArgs: () => void;
  now: number;
  depth?: number;
}) {
  const duration = step.doneAt
    ? step.doneAt - step.startedAt
    : step.status === "running"
    ? now - step.startedAt
    : 0;

  const statusClass = step.status === "running" ? "running" : step.success === false ? "failed" : "done";

  return (
    <li
      key={step.id}
      className={`${styles.item} ${styles[`depth-${Math.min(depth, 3)}`]}`}
      style={{ marginLeft: `${depth * 16}px` }}
      data-testid={`task-step-${step.id}`}
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onToggleExpand();
        } else if (e.key === "c" || e.key === "C") {
          e.preventDefault();
          onCopyArgs();
        }
      }}
    >
      <span
        className={`${styles.status} ${styles[statusClass]}`}
        data-status={step.status}
        title={step.status === "running" ? "运行中" : step.success === false ? "失败" : "成功"}
      >
        {step.status === "running" && <span className={styles.spinner} />}
        {step.status === "running" ? "◌" : step.success === false ? "✕" : "✓"}
      </span>
      <div className={styles.body}>
        <div className={styles.row}>
          <span className={styles.name}>{step.name}</span>
          <span className={`${styles.duration} ${styles[statusClass]}`}>
            {duration > 0 && formatDuration(duration)}
          </span>
        </div>
        {step.args && (
          <div className={styles.argsContainer}>
            <span
              className={styles.args}
              title={step.args}
              onClick={onToggleExpand}
            >
              {isExpanded ? prettyPrintJson(step.args) : step.args.slice(0, 80)}
            </span>
            {step.args.length > 80 && (
              <button
                className={styles.copyBtn}
                onClick={(e) => {
                  e.stopPropagation();
                  onCopyArgs();
                }}
                title="复制参数"
                aria-label="复制参数"
              >
                📋
              </button>
            )}
          </div>
        )}
        {isExpanded && step.args && (
          <pre className={styles.argsJson}>{prettyPrintJson(step.args)}</pre>
        )}
      </div>
    </li>
  );
}

export function TaskPane({ steps, startedAt }: TaskPaneProps) {
  const listRef = useRef<HTMLUListElement>(null);
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const now = startedAt ? Date.now() : null;

  const runningSteps = useMemo(
    () => steps.filter((s) => s.status === "running"),
    [steps]
  );
  const doneSteps = useMemo(
    () => steps.filter((s) => s.status === "done" && s.success !== false),
    [steps]
  );
  const failedSteps = useMemo(
    () => steps.filter((s) => s.status === "done" && s.success === false),
    [steps]
  );

  const totalElapsed = now && startedAt ? now - startedAt : 0;

  const toggleExpand = useCallback((id: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const copyArgs = useCallback((id: string, args: string) => {
    navigator.clipboard.writeText(args);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 1500);
  }, []);

  useEffect(() => {
    if (runningSteps.length > 0 && listRef.current) {
      const lastRunningItem = listRef.current.querySelector(
        `[data-testid="task-step-${runningSteps[runningSteps.length - 1].id}"]`
      );
      if (lastRunningItem) {
        lastRunningItem.scrollIntoView({ behavior: "smooth", block: "nearest" });
      }
    }
  }, [runningSteps]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      const focusedItem = document.activeElement as HTMLElement | null;
      if (!focusedItem?.matches(`[data-testid^="task-step-"]`)) return;

      const stepId = focusedItem.dataset.testid?.replace("task-step-", "");
      if (!stepId) return;

      const step = steps.find((s) => s.id === stepId);
      if (!step) return;

      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        toggleExpand(stepId);
      } else if (e.key === "c" || e.key === "C") {
        e.preventDefault();
        copyArgs(stepId, step.args);
      }
    },
    [steps, toggleExpand, copyArgs]
  );

  return (
    <div className={styles.pane} data-testid="task-pane" onKeyDown={handleKeyDown}>
      <div className={styles.header}>
        <span className={styles.title}>任务</span>
        {now != null && <span className={styles.elapsed}>耗时 {(totalElapsed / 1000).toFixed(1)}s</span>}
      </div>
      <div className={styles.summary}>
        <span className={styles.statTotal}>总计: {steps.length}</span>
        <span className={`${styles.stat} ${styles.statRunning}`}>运行中: {runningSteps.length}</span>
        <span className={`${styles.stat} ${styles.statDone}`}>完成: {doneSteps.length}</span>
        <span className={`${styles.stat} ${styles.statFailed}`}>失败: {failedSteps.length}</span>
        {totalElapsed > 0 && <span className={styles.statElapsed}>总耗时: {formatDuration(totalElapsed)}</span>}
      </div>
      {steps.length === 0 ? (
        <div className={styles.empty} data-testid="task-pane-empty">
          发送消息后，这里会实时展示 agent 的工具调用步骤与耗时。
        </div>
      ) : (
        <ul
          ref={listRef}
          className={styles.list}
          data-testid="task-step-list"
          role="tree"
          aria-label="任务步骤列表"
        >
          {steps.map((step) => (
            <TaskStepItem
              key={step.id}
              step={step}
              isExpanded={expandedIds.has(step.id)}
              onToggleExpand={() => toggleExpand(step.id)}
              onCopyArgs={() => copyArgs(step.id, step.args)}
              now={now ?? 0}
              depth={step.depth ?? 0}
            />
          ))}
        </ul>
      )}
    </div>
  );
}