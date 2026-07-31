import { createBrowserRouter } from "react-router-dom";
import App from "./App";
import NeoCodexPage from "./pages/NeoCodexPage";
import TerminalPage from "./pages/TerminalPage";
import SettingsPage from "./pages/SettingsPage";
import ProjectsPage from "./pages/ProjectsPage";
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
      { index: true, element: <NeoCodexPage /> },
      { path: "terminal", element: <TerminalPage /> },
      { path: "settings", element: <SettingsPage /> },
      { path: "projects", element: <ProjectsPage /> },
    ],
  },
]);

export default router;
