import { createBrowserRouter } from "react-router-dom";
import App from "./App";
import NeoCodexPage from "./pages/NeoCodexPage";
import SettingsPage from "./pages/SettingsPage";

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    children: [
      { index: true, element: <NeoCodexPage /> },
      { path: "settings", element: <SettingsPage /> },
    ],
  },
]);

export default router;
