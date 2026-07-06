import React, { Suspense } from "react";
import { useOutletContext } from "react-router-dom";
import type { AppOutletContext } from "../router";

const Terminal = React.lazy(() => import("../components/Terminal"));

function Lazy({ children }: { children: React.ReactNode }) {
  return <Suspense fallback={<div className="panel-loading" />}>{children}</Suspense>;
}

const TerminalPage: React.FC = () => {
  const { terminalSessionId, terminalStatus, setTerminalStatus } = useOutletContext<AppOutletContext>();

  return (
    <Lazy><Terminal
      sessionId={terminalSessionId.current}
      onClose={() => {}}
      onStatusChange={setTerminalStatus}
    /></Lazy>
  );
};

export default TerminalPage;
