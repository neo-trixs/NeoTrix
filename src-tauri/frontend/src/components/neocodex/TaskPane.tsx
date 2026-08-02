import React from "react";
import styles from "./TaskPane.module.css";

export interface TaskStep {
  id: string;
  name: string;
  args: string;
  startedAt: number;
  status: "running" | "done";
  success?: boolean;
}

export function TaskPane({ steps, startedAt }: { steps: TaskStep[]; startedAt: number | null }) {
  const now = startedAt ? Date.now() : null;
  const elapsed = now && startedAt ? now - startedAt : 0;
  const runningCount = steps.filter((s) => s.status === "running").length;
  const doneCount = steps.filter((s) => s.status === "done").length;

  return (
    <div className={styles.pane} data-testid="task-pane">
      <div className={styles.header}>
        <span className={styles.title}>任务</span>
        {now != null && <span className={styles.elapsed}>耗时 {(elapsed / 1000).toFixed(1)}s</span>}
      </div>
      <div className={styles.summary}>
        {doneCount} 完成 · {runningCount} 运行中
        {now == null && steps.length === 0 ? " · 尚无任务" : ""}
      </div>
      {steps.length === 0 ? (
        <div className={styles.empty} data-testid="task-pane-empty">
          发送消息后，这里会实时展示 agent 的工具调用步骤与耗时。
        </div>
      ) : (
        <ul className={styles.list} data-testid="task-step-list">
          {steps.map((step) => (
            <li key={step.id} className={styles.item} data-testid={`task-step-${step.id}`}>
              <span
                className={styles.status}
                data-status={step.status}
                title={step.status === "done" ? (step.success ? "成功" : "失败") : "运行中"}
              >
                {step.status === "running" ? "◌" : step.success === false ? "✕" : "✓"}
              </span>
              <div className={styles.body}>
                <span className={styles.name}>{step.name}</span>
                {step.args && <span className={styles.args} title={step.args}>{step.args.slice(0, 60)}</span>}
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
