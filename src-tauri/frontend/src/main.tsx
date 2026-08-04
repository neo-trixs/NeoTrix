import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "react-router-dom";
import router from "./router";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { captureConsoleErrors } from "./lib/error-reporter";
import "./styles/global.css";
import "highlight.js/styles/github-dark.min.css";

captureConsoleErrors();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <RouterProvider router={router} />
    </ErrorBoundary>
  </React.StrictMode>,
);
