import React from "react";

const QRDisplay: React.FC<{ svgContent: string }> = ({ svgContent }) => {
  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 8 }}>
      <div
        style={{
          width: 180,
          height: 180,
          borderRadius: 12,
          overflow: "hidden",
          boxShadow: "0 2px 12px rgba(0,0,0,0.08)",
        }}
        dangerouslySetInnerHTML={{ __html: svgContent }}
      />
      <span style={{ fontSize: 11, color: "var(--mac-text-secondary)", textAlign: "center" }}>
        扫描二维码连接远程会话
      </span>
    </div>
  );
};

export default QRDisplay;
