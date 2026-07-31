import { createBrowserRouter } from "react-router-dom";
import App from "./App";
import MainPage from "./pages/MainPage";
import TerminalPage from "./pages/TerminalPage";
import AgentFlowPage from "./pages/AgentFlowPage";
import VirtualDesktopPage from "./pages/VirtualDesktopPage";
import SettingsPage from "./pages/SettingsPage";
import AgentManagerPage from "./pages/AgentManagerPage";
import KnowledgeGraphPage from "./pages/KnowledgeGraphPage";
import ExplorePage from "./pages/ExplorePage";
import PrivacyFilterPage from "./pages/PrivacyFilterPage";
import SecurityScanPanel from "./components/SecurityScanPanel";
import SandboxManagerPage from "./pages/SandboxManagerPage";
import IdentityManagerPage from "./pages/IdentityManagerPage";
import MemoryManager from "./components/MemoryManager";
import ProxyPage from "./pages/ProxyPage";
import ChatPage from "./pages/ChatPage";
import ManagePage from "./pages/ManagePage";
import ProjectsPage from "./pages/ProjectsPage";
import PlanPage from "./pages/PlanPage";
import LoopPanel from "./components/LoopPanel";
import ProfileManager from "./components/ProfileManager";
import InsightsDashboard from "./components/InsightsDashboard";
import UnifiedSessionPanel from "./components/UnifiedSessionPanel";
import AnnotationsPanel from "./components/AnnotationsPanel";
import MarketplacePanel from "./components/MarketplacePanel";
import VoiceModePanel from "./components/VoiceModePanel";
import CoworkPanel from "./components/CoworkPanel";
import BackgroundCUPanel from "./components/BackgroundCUPanel";
import TerminalTabsPanel from "./components/TerminalTabsPanel";
import type { Attachment } from "./types";
export interface AppOutletContext {
  input: string;
  setInput: (v: string) => void;
  multiLine: boolean;
  setMultiLine: (v: boolean) => void;
  handleSubmit: (text: string, attachments?: Attachment[]) => Promise<void>;
  terminalSessionId: React.MutableRefObject<string>;
  terminalStatus: string;
  setTerminalStatus: (v: string) => void;
}

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    children: [
      { index: true, element: <MainPage /> },
      { path: "terminal", element: <TerminalPage /> },
      { path: "agent-flow", element: <AgentFlowPage /> },
      { path: "desktop", element: <VirtualDesktopPage /> },
      { path: "agents", element: <AgentManagerPage /> },
      { path: "knowledge-graph", element: <KnowledgeGraphPage /> },
      { path: "explore", element: <ExplorePage /> },
      { path: "privacy", element: <PrivacyFilterPage /> },
      { path: "sandbox", element: <SandboxManagerPage /> },
      { path: "identity", element: <IdentityManagerPage /> },
      { path: "proxy", element: <ProxyPage /> },
      { path: "security", element: <SecurityScanPanel /> },
      { path: "profiles", element: <ProfileManager /> },
      { path: "chat", element: <ChatPage /> },
      { path: "memory", element: <MemoryManager /> },
      { path: "projects", element: <ProjectsPage /> },
      { path: "loop", element: <LoopPanel /> },
      { path: "annotations", element: <AnnotationsPanel /> },
      { path: "marketplace", element: <MarketplacePanel /> },
      { path: "voice", element: <VoiceModePanel /> },
      { path: "cowork", element: <CoworkPanel /> },
      { path: "insights", element: <InsightsDashboard /> },
      { path: "sessions", element: <UnifiedSessionPanel /> },
      { path: "bg-computer", element: <BackgroundCUPanel /> },
      { path: "terminal-tabs", element: <TerminalTabsPanel /> },
    ],
  },
  { path: "/settings", element: <SettingsPage /> },
  { path: "/manage", element: <ManagePage /> },
  { path: "/plan", element: <PlanPage /> },
]);

export default router;
