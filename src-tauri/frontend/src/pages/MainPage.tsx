import React, { Suspense } from "react";
import { useOutletContext, useNavigate } from "react-router-dom";
import { useStore } from "../stores";
import type { AppOutletContext } from "../router";
import WelcomeState from "../components/consciousness/WelcomeState";

const EvolutionPanel = React.lazy(() => import("../components/EvolutionPanel"));
const SyncPanel = React.lazy(() => import("../components/SyncPanel"));
const AgentMaker = React.lazy(() => import("../components/AgentMaker"));
const SplitView = React.lazy(() => import("../components/SplitView"));
const ChatPanel = React.lazy(() => import("../components/ChatPanel"));
const InputPanel = React.lazy(() => import("../components/InputPanel"));
const AgentManagerPageContent = React.lazy(() => import("../pages/AgentManagerPage"));
const PrivacyFilterPageContent = React.lazy(() => import("../pages/PrivacyFilterPage"));
const SandboxManagerPageContent = React.lazy(() => import("../pages/SandboxManagerPage"));
const IdentityManagerPageContent = React.lazy(() => import("../pages/IdentityManagerPage"));

function Lazy({ children }: { children: React.ReactNode }) {
  return <Suspense fallback={<div className="panel-loading" />}>{children}</Suspense>;
}

const MainPage: React.FC = () => {
  const { input, setInput, multiLine, setMultiLine, handleSubmit } = useOutletContext<AppOutletContext>();
  const navigate = useNavigate();

  const sessions = useStore((s) => s.sessions);
  const activeSessionIndex = useStore((s) => s.activeSessionIndex);
  const agentBusy = useStore((s) => s.agentBusy);
  const streamingContent = useStore((s) => s.streamingContent);
  const streamingContentType = useStore((s) => s.streamingContentType);
  const splitViewActive = useStore((s) => s.splitViewActive);
  const setSplitViewActive = useStore((s) => s.setSplitViewActive);
  const agentMakerActive = useStore((s) => s.agentMakerActive);
  const setAgentMakerActive = useStore((s) => s.setAgentMakerActive);
  const evolutionVisible = useStore((s) => s.evolutionVisible);
  const setEvolutionVisible = useStore((s) => s.setEvolutionVisible);
  const syncVisible = useStore((s) => s.syncVisible);
  const setSyncVisible = useStore((s) => s.setSyncVisible);
  const showAgentManager = useStore((s) => s.showAgentManager);
  const setShowAgentManager = useStore((s) => s.setShowAgentManager);
  const showPrivacyFilter = useStore((s) => s.showPrivacyFilter);
  const setShowPrivacyFilter = useStore((s) => s.setShowPrivacyFilter);
  const showSandboxManager = useStore((s) => s.showSandboxManager);
  const setShowSandboxManager = useStore((s) => s.setShowSandboxManager);
  const showIdentityManager = useStore((s) => s.showIdentityManager);
  const setShowIdentityManager = useStore((s) => s.setShowIdentityManager);
  const setShowSearch = useStore((s) => s.setShowSearch);
  const setShowShortcuts = useStore((s) => s.setShowShortcuts);

  const activeMessages = sessions[activeSessionIndex]?.messages || [];

  return (
    <>
      {evolutionVisible ? (
        <Lazy><EvolutionPanel /></Lazy>
      ) : syncVisible ? (
        <Lazy><SyncPanel /></Lazy>
      ) : agentMakerActive ? (
        <Lazy><AgentMaker /></Lazy>
      ) : splitViewActive ? (
        <Lazy><SplitView /></Lazy>
      ) : showAgentManager ? (
        <Lazy><AgentManagerPageContent /></Lazy>
      ) : showPrivacyFilter ? (
        <Lazy><PrivacyFilterPageContent /></Lazy>
      ) : showSandboxManager ? (
        <Lazy><SandboxManagerPageContent /></Lazy>
      ) : showIdentityManager ? (
        <Lazy><IdentityManagerPageContent /></Lazy>
      ) : activeMessages.length === 0 && !agentBusy ? (
        <div className="vw-chat">
          <div className="wc-inner">
            <WelcomeState onSuggestionClick={(text) => { setInput(text); handleSubmit(text); }} />
            <InputPanel
              value={input}
              onChange={setInput}
              onSubmit={handleSubmit}
              multiLine={multiLine}
              onMultiLineToggle={() => setMultiLine(!multiLine)}
              disabled={agentBusy}
              agentBusy={agentBusy}
            />
          </div>
        </div>
      ) : (
        <>
          <ChatPanel
            messages={activeMessages}
            agentBusy={agentBusy}
            streamingContent={streamingContent}
            streamingContentType={streamingContentType}
          />
          <InputPanel
            value={input}
            onChange={setInput}
            onSubmit={handleSubmit}
            multiLine={multiLine}
            onMultiLineToggle={() => setMultiLine(!multiLine)}
            disabled={agentBusy}
            agentBusy={agentBusy}
          />
        </>
      )}
    </>
  );
};

export default MainPage;
