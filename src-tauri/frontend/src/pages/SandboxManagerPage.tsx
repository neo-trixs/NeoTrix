import React from "react";
import SandboxManager from "../components/SandboxManager";

const SandboxManagerPage: React.FC = () => {
  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}>
      <SandboxManager />
    </div>
  );
};

export default SandboxManagerPage;
