import React, { useState } from "react";

interface ImageGenRequest {
  prompt: string;
  size: "256x256" | "512x512" | "1024x1024" | "1792x1024";
  n: number;
}

interface ImageGenResult {
  url: string;
  revisedPrompt: string;
  seed: number;
}

const MOCK_IMAGES: Record<string, string> = {
  "256x256": "https://placehold.co/256x256/png?text=IMG",
  "512x512": "https://placehold.co/512x512/png?text=IMG",
  "1024x1024": "https://placehold.co/1024x1024/png?text=IMG",
  "1792x1024": "https://placehold.co/1792x1024/png?text=IMG",
};

const ImageGenPanel: React.FC<{ onInsert?: (url: string) => void; onClose: () => void }> = ({ onInsert, onClose }) => {
  const [prompt, setPrompt] = useState("");
  const [size, setSize] = useState<"256x256" | "512x512" | "1024x1024" | "1792x1024">("1024x1024");
  const [n, setN] = useState(1);
  const [results, setResults] = useState<ImageGenResult[]>([]);
  const [generating, setGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const generate = () => {
    if (!prompt.trim()) {
      setError("Please enter a prompt");
      return;
    }
    setGenerating(true);
    setError(null);
    setResults([]);

    setTimeout(() => {
      const generated: ImageGenResult[] = [];
      for (let i = 0; i < n; i++) {
        generated.push({
          url: MOCK_IMAGES[size] || MOCK_IMAGES["1024x1024"],
          revisedPrompt: prompt,
          seed: Math.floor(Math.random() * 100000),
        });
      }
      setResults(generated);
      setGenerating(false);
    }, 800);
  };

  const handleInsert = (url: string) => {
    if (onInsert) onInsert(url);
  };

  return (
    <div style={{ padding: 12, background: "var(--bg-primary, #ffffff)", maxHeight: "400px", overflowY: "auto" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
        <h3 style={{ fontSize: 13, fontWeight: 600, margin: 0, color: "var(--text-primary, #1a1a2e)" }}>🎨 Image Generation</h3>
        <button onClick={onClose} style={{ border: "none", background: "none", cursor: "pointer", fontSize: 16, color: "var(--text-muted, #8b949e)" }}>✕</button>
      </div>
      <textarea value={prompt} onChange={(e) => setPrompt(e.target.value)} placeholder="Describe the image you want to generate..." style={{ width: "100%", height: 60, padding: 8, border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, fontSize: 12, resize: "vertical", background: "var(--bg-primary, #ffffff)", color: "var(--text-primary, #1a1a2e)", outline: "none", fontFamily: "inherit" }} />
      {error && <div style={{ color: "var(--error, #d73a49)", fontSize: 11, marginTop: 4 }}>{error}</div>}
      <div style={{ display: "flex", gap: 8, marginTop: 6, alignItems: "center" }}>
        <label style={{ fontSize: 11, color: "var(--text-muted, #8b949e)" }}>Size:</label>
        <select value={size} onChange={(e) => setSize(e.target.value as any)} style={{ padding: "2px 4px", fontSize: 11, border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: "var(--bg-primary, #ffffff)" }}>
          <option value="256x256">256×256</option>
          <option value="512x512">512×512</option>
          <option value="1024x1024">1024×1024</option>
          <option value="1792x1024">1792×1024</option>
        </select>
        <label style={{ fontSize: 11, color: "var(--text-muted, #8b949e)" }}>Count:</label>
        <select value={n} onChange={(e) => setN(parseInt(e.target.value))} style={{ padding: "2px 4px", fontSize: 11, border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: "var(--bg-primary, #ffffff)" }}>
          {[1, 2, 3, 4].map((v) => <option key={v} value={v}>{v}</option>)}
        </select>
      </div>
      <button onClick={generate} disabled={generating || !prompt.trim()} style={{ marginTop: 6, padding: "4px 12px", cursor: generating ? "default" : "pointer", border: "1px solid var(--accent, #007aff)", borderRadius: 4, background: generating ? "var(--border-color, #e1e4e8)" : "var(--accent, #007aff)", color: "#fff", fontSize: 11, fontWeight: 500 }}>
        {generating ? "Generating..." : "Generate"}
      </button>
      {results.length > 0 && (
        <div style={{ marginTop: 10 }}>
          <div style={{ fontSize: 11, color: "var(--text-muted, #8b949e)", marginBottom: 6 }}>Results — click to insert into chat</div>
          <div style={{ display: "grid", gridTemplateColumns: `repeat(${results.length}, 1fr)`, gap: 6 }}>
            {results.map((result, i) => (
              <div key={i} style={{ cursor: "pointer", borderRadius: 4, overflow: "hidden", border: "1px solid var(--border-color, #e1e4e8)", transition: "border-color 0.15s" }} onMouseEnter={(e) => (e.currentTarget.style.borderColor = "var(--accent, #007aff)")} onMouseLeave={(e) => (e.currentTarget.style.borderColor = "var(--border-color, #e1e4e8)")} onClick={() => handleInsert(result.url)}>
                <img src={result.url} alt={result.revisedPrompt} style={{ width: "100%", aspectRatio: "1", objectFit: "cover" }} loading="lazy" />
                <div style={{ padding: 4, fontSize: 9, color: "var(--text-muted, #8b949e)", background: "var(--bg-secondary, #f6f8fa)" }}>seed: {result.seed}</div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default ImageGenPanel;
