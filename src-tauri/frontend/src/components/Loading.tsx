import React from "react";

interface LoadingProps {
  message?: string;
  type?: "panel" | "overlay";
}

const Loading: React.FC<LoadingProps> = ({ message, type = "panel" }) => (
  <div className={`loading-container loading-${type}`}>
    <div className="loading-spinner" />
    {message && <span className="loading-message">{message}</span>}
  </div>
);

export default Loading;
