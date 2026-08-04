import React from "react";
import { TaskPane, DiffPane, PreviewPane, TerminalPane, FileTreePanel, SideChatPane } from "./index";
import styles from "./ContextPanel.module.css";

export interface ContextPanelProps {
  activeTab: "review" | "terminal" | "browser" | "file" | "tasks" | "chat";
  taskSteps: Array<{ id: string; name: string; args: string; startedAt: number; status: "running" | "done"; success?: boolean }>;
  taskStartedAt: number | null;
  onFilePick?: (path: string) => void;
  sideChatMessages?: Array<{ role: string; content: string }>;
  sideChatInput?: string;
  onSideChatInputChange?: (v: string) => void;
  onSideChatSend?: () => void;
}

export function ContextPanel({
  activeTab,
  taskSteps,
  taskStartedAt,
  onFilePick,
  sideChatMessages,
  sideChatInput,
  onSideChatInputChange,
  onSideChatSend,
}: ContextPanelProps) {
  return (
    <div className={styles.panel}>
      <div className={styles.content}>
        {activeTab === "tasks" && <TaskPane steps={taskSteps} startedAt={taskStartedAt} />}
        {activeTab === "review" && <DiffPane />}
        {activeTab === "browser" && <PreviewPane />}
        {activeTab === "terminal" && <TerminalPane />}
        {activeTab === "file" && <FileTreePanel onPick={onFilePick} />}
        {activeTab === "chat" && (
          <SideChatPane
            messages={sideChatMessages || []}
            input={sideChatInput || ""}
            onInputChange={(v) => onSideChatInputChange?.(v)}
            onSend={() => onSideChatSend?.()}
          />
        )}
      </div>
    </div>
  );
}