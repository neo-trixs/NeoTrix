import React, { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { Outlet, useNavigate, useLocation } from "react-router-dom";
import { useStore } from "./stores";
import ErrorBoundary from "./components/ErrorBoundary";
import NotificationToast from "./components/NotificationToast";
import { getCurrent } from "@tauri-apps/plugin-deep-link";

const App: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();

  const settings = useStore((s) => s.settings);
  const updateProgress = useStore((s) => s.updateProgress);
  const setUpdateProgress = useStore((s) => s.setUpdateProgress);
  const addNotification = useStore((s) => s.addNotification);

  // ── Apply theme ──
  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      const isDark = settings.theme === "dark" || (settings.theme === "system" && media.matches);
      document.documentElement.setAttribute("data-theme", isDark ? "dark" : "light");
      document.documentElement.style.setProperty("--app-font-size", `${settings.fontSize}px`);
    };
    apply();
    if (settings.theme === "system") {
      media.addEventListener("change", apply);
      return () => media.removeEventListener("change", apply);
    }
  }, [settings.theme, settings.fontSize]);

  // ── Tauri event: task complete ──
  useEffect(() => {
    const unlisten = listen<{ title: string; body: string }>("task-complete", (event) => {
      useStore.getState().addNotification({
        type: "info",
        message: `Task complete: ${event.payload.title}`,
        duration: 5000,
      });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // ── Deep link ──
  useEffect(() => {
    getCurrent().then((urls) => {
      if (urls && urls.length > 0) {
        const decoded = decodeURIComponent(urls.join(","));
        useStore.getState().addNotification({ type: "info", message: `Deep link: ${decoded}`, duration: 5000 });
      }
    });
  }, []);

  // ── Open settings event ──
  useEffect(() => {
    const unlisten = listen("open-settings", () => navigate("/settings"));
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [navigate]);

  // ── Update download progress (from neocodex_download_update) ──
  useEffect(() => {
    const unlisten = listen<{ downloaded: number; total: number | null }>("neocodex_update_progress", (event) => {
      const { downloaded, total } = event.payload;
      if (total && total > 0) {
        setUpdateProgress(Math.min(99, Math.round((downloaded / total) * 100)));
      } else {
        setUpdateProgress(0);
      }
    });
    const unlistenDone = listen("neocodex_update_downloaded", () => {
      setUpdateProgress(100);
      setTimeout(() => setUpdateProgress(0), 2000);
    });
    return () => {
      unlisten.then((fn) => fn());
      unlistenDone.then((fn) => fn());
    };
  }, [setUpdateProgress]);

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

  // ── Global shortcut event from Rust ──
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
      if (mod && e.key === ",") {
        e.preventDefault();
        navigate("/settings");
      } else if (e.key === "Escape" && location.pathname !== "/") {
        navigate("/");
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [navigate, location.pathname]);

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
        <Outlet />
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
