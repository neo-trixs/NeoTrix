import React, { Suspense, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { Outlet, useNavigate, useLocation } from "react-router-dom";
import { useStore } from "./stores";
import ErrorBoundary from "./components/ErrorBoundary";
import NotificationToast from "./components/NotificationToast";
import * as api from "./lib/api";
import { getCurrent } from "@tauri-apps/plugin-deep-link";
import { check } from "@tauri-apps/plugin-updater";

function Lazy({ children }: { children: React.ReactNode }) {
  return <Suspense fallback={<div className="panel-loading" />}>{children}</Suspense>;
}

const App: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const locationRef = useRef(location);
  locationRef.current = location;

  const pendingPermission = useStore((s) => s.pendingPermission);
  const settings = useStore((s) => s.settings);
  const setPendingPermission = useStore((s) => s.setPendingPermission);
  const setSettings = useStore((s) => s.setSettings);
  const updateProgress = useStore((s) => s.updateProgress);
  const setUpdateStatus = useStore((s) => s.setUpdateStatus);
  const addNotification = useStore((s) => s.addNotification);
  const pushMessage = useStore((s) => s.pushMessage);

  const [input, setInput] = React.useState("");
  const [multiLine, setMultiLine] = React.useState(false);
  const terminalSessionId = useRef(`term-${Date.now()}`);
  const [terminalStatus, setTerminalStatus] = React.useState("");

  // ── Poll permissions ──
  useEffect(() => {
    const timer = setInterval(async () => {
      try {
        const perms = await api.getPendingPermissions();
        if (perms.length > 0 && !useStore.getState().pendingPermission) {
          setPendingPermission(perms[0]);
        }
      } catch {}
    }, 3000);
    return () => clearInterval(timer);
  }, [setPendingPermission]);

  // ── Apply theme ──
  useEffect(() => {
    const isDark =
      settings.theme === "dark" ||
      (settings.theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.setAttribute("data-theme", isDark ? "dark" : "light");
  }, [settings.theme]);

  // ── Tauri event listeners ──
  useEffect(() => {
    const unlistenTask = listen<{ title: string; body: string }>("task-complete", (event) => {
      useStore.getState().addNotification({ type: "info", message: `Task complete: ${event.payload.title}`, duration: 5000 });
    });
    return () => {
      unlistenTask.then((fn) => fn());
    };
  }, []);

  // ── Deep link ──
  useEffect(() => {
    getCurrent().then((urls) => {
      if (urls && urls.length > 0) {
        const decoded = decodeURIComponent(urls.join(","));
        useStore.getState().pushMessage("system", `Deep link: ${decoded}`);
      }
    });
  }, []);

  // ── Settings/proxy/sync listeners ──
  useEffect(() => {
    const unlistenSettings = listen("open-settings", () => navigate("/settings"));
    return () => {
      unlistenSettings.then((fn) => fn());
    };
  }, [navigate]);

  // ── Update check ──
  useEffect(() => {
    const checkUpdate = async () => {
      try {
        const update = await check();
        if (update?.available) {
          setUpdateStatus(true, `v${update.version}`);
          pushMessage("system", `Update available: v${update.version}`);
          addNotification({ type: "info", message: `Update v${update.version} available`, duration: 10000 });
        }
      } catch {}
    };
    checkUpdate();
  }, [setUpdateStatus, pushMessage, addNotification]);

  // ── Window state persistence ──
  useEffect(() => {
    let unload: Array<() => void> = [];
    const initWindowState = async () => {
      try {
        const mod = await import("@tauri-apps/api/window");
        const win = (mod as any).getCurrentWindow();
        const saved = (() => { try { return JSON.parse(localStorage.getItem("neotrix_window_state") || "null"); } catch { return null; } })();
        if (saved) {
          try {
            const Pos = (mod as any).PhysicalPosition;
            const Size = (mod as any).PhysicalSize;
            await win.setPosition(new Pos(saved.x, saved.y));
            await win.setSize(new Size(saved.w, saved.h));
          } catch {}
        }
        const saveState = async () => {
          try {
            const pos = await win.getPosition();
            const size = await win.getSize();
            localStorage.setItem("neotrix_window_state", JSON.stringify({ x: pos.x, y: pos.y, w: size.width, h: size.height }));
          } catch {}
        };
        const un1 = await win.onResized(saveState);
        const un2 = await win.onMoved(saveState);
        unload = [un1, un2];
      } catch {}
    };
    initWindowState();
    return () => unload.forEach((fn) => fn());
  }, []);

  // ── Global shortcut event from Rust (CmdOrCtrl+Shift+Space) ──
  useEffect(() => {
    const unlisten = listen("neotrix-global-shortcut", () => {
      useStore.getState().addNotification({ type: "info", message: "NeoTrix 全局快捷键", duration: 3000 });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // ── Keyboard shortcuts ──
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      const st = useStore.getState();
      if (mod && e.key === ",") {
        e.preventDefault();
        navigate("/settings");
      } else if (e.key === "Escape") {
        if (locationRef.current.pathname !== "/") navigate("/");
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [navigate]);

  // ── Submit (legacy ReAct entry kept for outlet context compatibility) ──
  const handleSubmit = React.useCallback(async (text: string) => {
    if (!text.trim()) return;
    pushMessage("user", text, "markdown");
    try {
      const result = await api.agentReason(text);
      if (result.success && result.output) pushMessage("assistant", result.output);
      else pushMessage("error", result.output || "Agent returned empty response");
    } catch (e) {
      pushMessage("error", `Request failed: ${e}`);
    }
  }, [pushMessage]);

  return (
    <div className="app-container">
      <ErrorBoundary
        fallback={
          <div className="main-panel-error">
            <h3>Panel render error</h3>
            <button className="nt-btn-primary" onClick={() => window.location.reload()}>Restore</button>
          </div>
        }
      >
        <Outlet
          context={{
            input,
            setInput,
            multiLine,
            setMultiLine,
            handleSubmit,
            terminalSessionId,
            terminalStatus,
            setTerminalStatus,
          } as import("./router").AppOutletContext}
        />
      </ErrorBoundary>

      <NotificationToast />

      {updateProgress > 0 && updateProgress < 100 && (
        <div className="update-progress-bar">
          <div className="update-progress-fill" style={{ width: `${updateProgress}%` }} />
          <span className="update-progress-text">{updateProgress}%</span>
        </div>
      )}
    </div>
  );
};

export default App;
