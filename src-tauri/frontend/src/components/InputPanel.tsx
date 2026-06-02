import React, { useRef, useEffect } from "react";

interface Props {
  value: string;
  onChange: (v: string) => void;
  onSubmit: (v: string) => void;
  multiLine: boolean;
  onMultiLineToggle: () => void;
  disabled: boolean;
}

const InputPanel: React.FC<Props> = ({ value, onChange, onSubmit, multiLine, onMultiLineToggle, disabled }) => {
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, [disabled]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.altKey && !e.shiftKey) {
      e.preventDefault();
      if (value.trim() && !disabled) {
        onSubmit(value);
        onChange("");
      }
    }
    if (e.key === "Enter" && e.altKey) {
      onChange(value + "\n");
    }
  };

  return (
    <div className="input-panel glass-panel">
      <textarea
        ref={inputRef}
        className="input-field"
        placeholder={multiLine ? "多行模式 · Alt+Enter=换行 Enter=发送" : "输入消息... (Alt+Enter=多行, Tab=补全)"}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        disabled={disabled}
        rows={multiLine ? 4 : 2}
      />
      <div className="input-actions">
        <button
          className={`btn-icon ${multiLine ? "active" : ""}`}
          onClick={onMultiLineToggle}
          title="多行模式"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="2" y="2" width="10" height="10" rx="2" stroke="currentColor" strokeWidth="1.3" />
            <path d="M5 7l1.5 1.5L9 5.5" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
        </button>
        <button
          className="btn-send"
          onClick={() => { if (value.trim() && !disabled) { onSubmit(value); onChange(""); } }}
          disabled={disabled || !value.trim()}
        >
          发送
        </button>
      </div>
    </div>
  );
};

export default InputPanel;
