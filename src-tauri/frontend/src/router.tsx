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
import SandboxManagerPage from "./pages/SandboxManagerPage";
import IdentityManagerPage from "./pages/IdentityManagerPage";
import ProxyPage from "./pages/ProxyPage";
import ChatPage from "./pages/ChatPage";
import ManagePage from "./pages/ManagePage";
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
      { path: "chat", element: <ChatPage /> },
    ],
  },
  { path: "/settings", element: <SettingsPage /> },
  { path: "/manage", element: <ManagePage /> },
]);

export default router;
