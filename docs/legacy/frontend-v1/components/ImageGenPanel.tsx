import React, { useState } from "react";
import "./ImageGenPanel.css";

interface ImageGenPanelProps {
  onGenerate: (prompt: string, options: ImageOptions) => void;
}

interface ImageOptions {
  width: number;
  height: number;
  style: string;
  quality: string;
}

export function ImageGenPanel({ onGenerate }: ImageGenPanelProps): JSX.Element {
  const [prompt, setPrompt] = useState("");
  const [width, setWidth] = useState(1024);
  const [height, setHeight] = useState(1024);
  const [style, setStyle] = useState("natural");
  const [quality, setQuality] = useState("standard");
  const [generating, setGenerating] = useState(false);
  const [resultUrl, setResultUrl] = useState<string | null>(null);

  const styles = ["natural", "vivid", "anime", "watercolor", "photographic", "cinematic", "digital-art", "pixel-art"];
  const qualities = ["standard", "high", "ultra"];
  const dimensions = [512, 768, 1024, 1536, 2048];

  const handleGenerate = () => {
    if (!prompt.trim()) return;
    setGenerating(true);
    setResultUrl(null);
    onGenerate(prompt, { width, height, style, quality });
    setTimeout(() => {
      setGenerating(false);
      setResultUrl(`data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="' + width + '" height="' + height + '"><rect fill="%231a1a2e" width="100%25" height="100%25"/><text x="50%25" y="50%25" fill="white" text-anchor="middle" dy=".3em" font-size="16">Generated: ' + prompt.slice(0, 40) + '...</text></svg>')}`);
    }, 1500);
  };

  return (
    <div className="image-gen-panel">
      <textarea
        className="gen-prompt"
        value={prompt}
        onChange={e => setPrompt(e.target.value)}
        placeholder="Describe the image you want to generate..."
        rows={4}
      />
      <div className="gen-options">
        <div className="gen-option">
          <label>Width</label>
          <select value={width} onChange={e => setWidth(Number(e.target.value))}>
            {dimensions.map(d => <option key={d} value={d}>{d}px</option>)}
          </select>
        </div>
        <div className="gen-option">
          <label>Height</label>
          <select value={height} onChange={e => setHeight(Number(e.target.value))}>
            {dimensions.map(d => <option key={d} value={d}>{d}px</option>)}
          </select>
        </div>
        <div className="gen-option">
          <label>Style</label>
          <select value={style} onChange={e => setStyle(e.target.value)}>
            {styles.map(s => <option key={s} value={s}>{s}</option>)}
          </select>
        </div>
        <div className="gen-option">
          <label>Quality</label>
          <select value={quality} onChange={e => setQuality(e.target.value)}>
            {qualities.map(q => <option key={q} value={q}>{q}</option>)}
          </select>
        </div>
      </div>
      <button className="gen-btn" onClick={handleGenerate} disabled={generating || !prompt.trim()}>
        {generating ? "Generating..." : "Generate Image"}
      </button>
      {resultUrl && (
        <div className="gen-result">
          <img src={resultUrl} alt="Generated" />
        </div>
      )}
    </div>
  );
}
