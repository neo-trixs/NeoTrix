import { useState, useRef, useEffect } from "react";

interface ArtifactRendererProps {
  language: string;
  code: string;
}

type ArtifactType = "mermaid" | "svg" | "html" | "table" | null;

function detectArtifact(language: string, code: string): ArtifactType {
  const trimmed = code.trim();
  if (language === "mermaid") return "mermaid";
  if (language === "svg" || (language === "html" && /<svg[\s>]/i.test(trimmed))) return "svg";
  if (language === "html" && (/^<!DOCTYPE html/i.test(trimmed) || /^<html/i.test(trimmed) || /^<div/i.test(trimmed))) return "html";
  if (language === "table" || (language === "" && /^\|.+\|/.test(trimmed))) return "table";
  return null;
}

function parseMermaid(code: string): string {
  const typeMatch = code.match(/^\s*(\w+)\b/m);
  const type = typeMatch ? typeMatch[1] : "graph";
  const lines = code.trim().split("\n").filter(l => l.trim());
  const nodeCount = lines.filter(l => /-->|===|---|->>/.test(l)).length;
  const labelCount = lines.filter(l => /\[|\(|{/.test(l)).length;

  return `<div class="artifact-mermaid">
    <div class="artifact-mermaid-header">
      <span class="artifact-icon">⬡</span>
      <span>Mermaid ${type} — ${nodeCount} edges, ${labelCount} nodes</span>
    </div>
    <pre class="artifact-mermaid-code"><code>${code.replace(/</g, "&lt;").replace(/>/g, "&gt;")}</code></pre>
  </div>`;
}

function parseSVG(code: string): string {
  const svgMatch = code.match(/<svg[\s\S]*?<\/svg>/i);
  if (!svgMatch) return `<pre><code>${code}</code></pre>`;
  return `<div class="artifact-svg-wrapper">${svgMatch[0]}</div>`;
}

function ParseHtml({ code }: { code: string }) {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const [height, setHeight] = useState(200);

  useEffect(() => {
    const iframe = iframeRef.current;
    if (!iframe) return;
    const doc = iframe.contentDocument || iframe.contentWindow?.document;
    if (!doc) return;
    doc.open();
    doc.write(code);
    doc.close();
    const checkHeight = () => {
      const h = doc.documentElement.scrollHeight || doc.body?.scrollHeight || 200;
      setHeight(Math.min(Math.max(h, 100), 480));
    };
    setTimeout(checkHeight, 100);
  }, [code]);

  return (
    <div className="artifact-html-wrapper">
      <div className="artifact-html-header">
        <span className="artifact-icon">◻</span>
        <span>HTML Preview</span>
      </div>
      <iframe
        ref={iframeRef}
        className="artifact-iframe"
        style={{ height: `${height}px` }}
        sandbox="allow-scripts"
        title="HTML preview"
      />
    </div>
  );
}

function ParseTable({ code }: { code: string }) {
  const lines = code.trim().split("\n");
  const headerMatch = lines[0]?.match(/^\|(.+)\|$/);
  if (!headerMatch) return <pre><code>{code}</code></pre>;

  const headers = headerMatch[1].split("|").map(h => h.trim());
  const alignMatch = lines[1]?.match(/^\|(.+)\|$/);
  const alignments = alignMatch
    ? alignMatch[1].split("|").map(a => {
        const t = a.trim();
        if (t.startsWith(":") && t.endsWith(":")) return "center";
        if (t.endsWith(":")) return "right";
        return "left";
      })
    : headers.map(() => "left");

  const data = lines.slice(2).filter(l => l.trim().startsWith("|")).map(l => {
    const parts = l.match(/^\|(.+)\|$/);
    return parts ? parts[1].split("|").map(c => c.trim()) : [];
  });

  return (
    <div className="artifact-table-wrapper">
      <table className="artifact-table">
        <thead>
          <tr>{headers.map((h, i) => <th key={i} style={{ textAlign: alignments[i] as any }}>{h}</th>)}</tr>
        </thead>
        <tbody>
          {data.map((row, ri) => (
            <tr key={ri}>
              {headers.map((_, ci) => (
                <td key={ci} style={{ textAlign: alignments[ci] as any }}>{row[ci] || ""}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default function ArtifactRenderer({ language, code }: ArtifactRendererProps) {
  const type = detectArtifact(language, code);
  if (!type) return null;

  const renderContent = () => {
    switch (type) {
      case "mermaid":
        return <div dangerouslySetInnerHTML={{ __html: parseMermaid(code) }} />;
      case "svg":
        return <div dangerouslySetInnerHTML={{ __html: parseSVG(code) }} />;
      case "html":
        return <ParseHtml code={code} />;
      case "table":
        return <ParseTable code={code} />;
      default:
        return null;
    }
  };

  return <div className={`artifact-renderer artifact-${type}`}>{renderContent()}</div>;
}
