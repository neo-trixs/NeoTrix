import type { EditorState } from "../types";

export interface EditorSlice {
  editorState: EditorState;
  openEditor: (filePath: string) => Promise<void>;
  closeEditor: () => void;
}

export const createEditorSlice = (set: any) => ({
  editorState: { open: false, filePath: "", initialContent: "", language: "" },

  openEditor: async (filePath: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const content = await invoke<string>("read_file", { path: filePath });
      const ext = filePath.split(".").pop()?.toLowerCase() || "";
      const langMap: Record<string, string> = {
        rs: "rust", ts: "typescript", tsx: "typescript", js: "javascript", jsx: "javascript",
        py: "python", html: "html", htm: "html", css: "css", json: "json", md: "markdown",
        mdx: "markdown", yaml: "yaml", yml: "yaml", toml: "toml", sh: "bash", bash: "bash",
        zsh: "bash", sql: "sql", go: "go", java: "java", kt: "kotlin", swift: "swift",
      };
      const language = langMap[ext] || "";
      set({ editorState: { open: true, filePath, initialContent: content, language } });
    } catch (e) {
      console.error("Failed to open file:", e);
      set({ statusText: `打开文件失败: ${e}` });
    }
  },

  closeEditor: () => set({ editorState: { open: false, filePath: "", initialContent: "", language: "" } }),
});
