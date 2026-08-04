import React from "react";
import styles from "./SideChatPane.module.css";

export interface SideChatPaneProps {
  messages: Array<{ role: string; content: string }>;
  input: string;
  onInputChange: (v: string) => void;
  onSend: () => void;
}

export function SideChatPane({ messages, input, onInputChange, onSend }: SideChatPaneProps) {
  return (
    <div className={styles.container} data-testid="side-chat-pane">
      <div className={styles.messages}>
        {messages.length === 0 && <div className={styles.empty}>侧聊为空，输入消息开始旁路对话</div>}
        {messages.map((m, i) => (
          <div
            key={i}
            className={`${styles.msg} ${m.role === "user" ? styles.msgUser : styles.msgAssistant}`}
          >
            {m.content}
          </div>
        ))}
      </div>
      <div className={styles.inputRow}>
        <input
          className={styles.input}
          value={input}
          onChange={(e) => onInputChange(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter") onSend(); }}
          placeholder="输入内容... (Enter 发送)"
          data-testid="side-chat-input"
        />
        <button className={styles.send} onClick={onSend} data-testid="side-chat-send">发送</button>
      </div>
      <div className={styles.hint}>侧聊不写入主会话 ⌘+; 关闭</div>
    </div>
  );
}

export default SideChatPane;
