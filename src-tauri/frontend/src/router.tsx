import { createBrowserRouter } from "react-router-dom";
import App from "./App";
import NeoCodexPage from "./pages/NeoCodexPage";

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    children: [
      { index: true, element: <NeoCodexPage /> },
    ],
  },
]);

export default router;
