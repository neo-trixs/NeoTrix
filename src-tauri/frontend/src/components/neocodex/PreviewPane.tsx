import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import styles from "./PreviewPane.module.css";

export function PreviewPane() {
  const [url, setUrl] = useState("http://localhost:5173");
  const [state, setState] = useState<{ title?: string; url?: string }>({});

  const open = async () => {
    try {
      const s = await invoke("browser_open", { url });
      setState(s as any);
    } catch (e) {
      setState({ title: String(e), url });
    }
  };

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <span className={styles.title}>预览</span>
        <input
          className={styles.urlInput}
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && open()}
          placeholder="http://localhost:5173"
        />
        <button type="button" className={styles.openBtn} onClick={open}>打开</button>
      </div>
      <div className={styles.hint}>
        {state.title ? `已打开: ${state.title}` : "输入 dev server 地址，在内置浏览器预览应用。也可直接打开 HTML/PDF/图片路径。"}
      </div>
    </div>
  );
}
