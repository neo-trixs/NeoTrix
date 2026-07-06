import React from "react";
import PrivacyFilter from "../components/PrivacyFilter";

const PrivacyFilterPage: React.FC = () => {
  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}>
      <PrivacyFilter />
    </div>
  );
};

export default PrivacyFilterPage;
