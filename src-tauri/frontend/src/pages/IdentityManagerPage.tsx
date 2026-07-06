import React from "react";
import IdentityManager from "../components/IdentityManager";

const IdentityManagerPage: React.FC = () => {
  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}>
      <IdentityManager />
    </div>
  );
};

export default IdentityManagerPage;
