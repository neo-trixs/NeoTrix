import React, { useRef, useEffect, useState, useCallback } from "react";
import type { Attachment } from "../types";
import styles from "./InputPanel.module.css";

interface Props {
  value: string;
  onChange: (v: string) => void;
  onSubmit: (v: string, attachments?: Attachment[]) => void;
  onStop?: () => void;
  multiLine: boolean;
  onMultiLineToggle: () => void;
  disabled: boolean;
  agentBusy?: boolean;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${(bytes / 1048576).toFixed(1)}MB`;
}

function fileToAttachment(file: File): Promise<Attachment> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const data = reader.result as string;
      resolve({
        id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        name: file.name,
        size: file.size,
        mimeType: file.type,
        data: data,
      });
    };
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

const ACCEPTED_TYPES = [".txt", ".md", ".pdf", ".json", ".yaml", ".yml", ".toml", ".csv", ".rs", ".py", ".js", ".ts", ".tsx", ".jsx", ".html", ".css", ".scss", ".sh", ".sql", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp", ".ico"];

const AttachmentChip: React.FC<{ attachment: Attachment; onRemove: (id: string) => void }> = ({ attachment, onRemove }) => {
  const isImage = attachment.mimeType.startsWith("image/");
  return (
    <div className={styles.chip}>
      {isImage && <img src={attachment.data} alt="" className={styles.chipThumb} />}
      <div className={styles.chipInfo}>
        <span className={styles.chipName}>{attachment.name}</span>
        <span className={styles.chipSize}>{formatSize(attachment.size)}</span>
      </div>
      <button className={styles.chipRemove} onClick={() => onRemove(attachment.id)} aria-label="Remove attachment">
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none"><path d="M2 2l6 6M8 2l-6 6" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/></svg>
      </button>
    </div>
  );
};

const InputPanel: React.FC<Props> = ({ value, onChange, onSubmit, onStop, multiLine, onMultiLineToggle, disabled, agentBusy }) => {
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const dropRef = useRef<HTMLDivElement>(null);
  const [attachments, setAttachments] = useState<Attachment[]>([]);
  const [dragOver, setDragOver] = useState(false);
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    inputRef.current?.focus();
  }, [disabled]);

  useEffect(() => {
    const el = inputRef.current;
    if (el) {
      el.style.height = "auto";
      const newHeight = Math.min(Math.max(el.scrollHeight, 64), 200);
      el.style.height = `${newHeight}px`;
    }
  }, [value]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.altKey && !e.shiftKey) {
      e.preventDefault();
      if (value.trim() && !disabled) {
        onSubmit(value, attachments.length > 0 ? attachments : undefined);
        onChange("");
        setAttachments([]);
      }
    }
    if (e.key === "Enter" && e.altKey) {
      onChange(value + "\n");
    }
  };

  const handleSubmitClick = () => {
    if (value.trim() && !disabled) {
      onSubmit(value, attachments.length > 0 ? attachments : undefined);
      onChange("");
      setAttachments([]);
    }
  };

  const handleStopClick = () => {
    onStop?.();
  };

  const handleFilePick = useCallback(async (files: FileList | null) => {
    if (!files) return;
    const newAttachments: Attachment[] = [];
    for (let i = 0; i < files.length; i++) {
      const at = await fileToAttachment(files[i]);
      newAttachments.push(at);
    }
    setAttachments((prev) => [...prev, ...newAttachments]);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    handleFilePick(e.dataTransfer.files);
  }, [handleFilePick]);

  const handleDragOver = (e: React.DragEvent) => { e.preventDefault(); setDragOver(true); };
  const handleDragLeave = () => setDragOver(false);

  const removeAttachment = (id: string) => {
    setAttachments((prev) => prev.filter((a) => a.id !== id));
  };

  const handlePaste = useCallback((e: React.ClipboardEvent) => {
    const items = e.clipboardData.items;
    const imageFiles: File[] = [];
    for (let i = 0; i < items.length; i++) {
      if (items[i].type.startsWith("image/")) {
        const file = items[i].getAsFile();
        if (file) imageFiles.push(file);
      }
    }
    if (imageFiles.length > 0) {
      e.preventDefault();
      Promise.all(imageFiles.map(fileToAttachment)).then((atts) => {
        setAttachments((prev) => [...prev, ...atts]);
      });
    }
  }, []);

  const showAsBusy = agentBusy || disabled;

  return (
    <div
      ref={dropRef}
      className={`${styles.cic} ${dragOver ? styles.dragOver : ""}`}
      role="form"
      aria-label="Chat input form"
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
    >
      {dragOver && (
        <div className={styles.dropOverlay}>
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none"><path d="M12 16V4m0 12l-4-4m4 4l4-4M4 16v2a2 2 0 002 2h12a2 2 0 002-2v-2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/></svg>
          <span>Drop to attach files</span>
        </div>
      )}

      {attachments.length > 0 && (
        <div className={styles.chipBar}>
          {attachments.map((a) => (
            <AttachmentChip key={a.id} attachment={a} onRemove={removeAttachment} />
          ))}
        </div>
      )}

      <textarea
        ref={inputRef}
        className={styles.field}
        aria-label="Chat input"
        placeholder="How can I help?"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        onPaste={handlePaste}
        disabled={showAsBusy}
      />

      <div className={styles.cicActions}>
        <div className={styles.cicLeft}>
          <button
            className={styles.cicAttach}
            onClick={() => fileInputRef.current?.click()}
            aria-label="Attach file"
            title="Attach file"
          >
            <svg viewBox="0 0 18 18">
              <path d="M9 2v10a3 3 0 006 0V6a5.5 5.5 0 00-11 0v8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
          {!showAsBusy && (
            <button
              className={`${styles.cicMulti} ${multiLine ? styles.active : ""}`}
              onClick={onMultiLineToggle}
              aria-label="Toggle multi-line mode"
              title="Multi-line mode"
            >
              <svg viewBox="0 0 14 14">
                <rect x="2" y="2" width="10" height="10" rx="2" stroke="currentColor" strokeWidth="1.3" />
                <path d="M5 7l1.5 1.5L9 5.5" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
              </svg>
            </button>
          )}
          <input
            ref={fileInputRef}
            type="file"
            hidden
            multiple
            accept={ACCEPTED_TYPES.join(",")}
            onChange={(e) => handleFilePick(e.target.files)}
          />
        </div>
        <div className={styles.cicRight}>
          {showAsBusy ? (
            <button
              className={styles.stopBtn}
              onClick={handleStopClick}
              aria-label="Stop generation"
              title="Stop"
            >
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                <rect x="2" y="2" width="8" height="8" rx="1.5" fill="currentColor"/>
              </svg>
            </button>
          ) : (
            <>
              <button
                className={`${styles.vcBtn} ${styles.vcLang} ${isRecording ? styles.recording : ""}`}
                onClick={() => setIsRecording(!isRecording)}
                aria-label="Voice input"
                title="Voice input"
              >
                <svg viewBox="0 0 18 18">
                  <rect x="6.5" y="2" width="5" height="10" rx="2.5" stroke="currentColor" strokeWidth="1.8"/>
                  <path d="M4 8a5 5 0 0010 0" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round"/>
                  <path d="M9 14v3" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round"/>
                </svg>
              </button>
              <button
                className={`${styles.vcBtn} ${styles.vcSend}`}
                onClick={handleSubmitClick}
                disabled={!value.trim()}
                aria-label="Send message"
                title="Send"
              >
                <svg className="s-mic" viewBox="0 0 18 14">
                  <rect x="6.5" y="1" width="5" height="10" rx="2.5" stroke="currentColor" strokeWidth="2"/>
                  <path d="M4 6a5 5 0 0010 0" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
                  <path d="M9 13v2" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
                </svg>
                <svg className="s-wav" viewBox="0 0 18 14">
                  <path d="M3 6v2M6 4v6M9 2v10M12 4v6M15 6v2" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
                </svg>
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default InputPanel;
