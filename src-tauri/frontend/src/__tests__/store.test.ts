import { describe, it, expect, beforeEach } from "vitest";
import { useStore } from "../stores";
import { DEFAULT_SETTINGS } from "../stores/store-utils";

beforeEach(() => {
  useStore.setState({
    settings: { ...DEFAULT_SETTINGS },
    neocodexMode: "Agent",
    neocodexMessages: [],
    neocodexStreaming: null,
    neocodexSessions: [],
    neocodexActiveSessionId: null,
    updateAvailable: false,
    updateStatus: "",
    notifications: [],
  });
});

describe("useStore", () => {
  it("should expose default settings", () => {
    const { settings } = useStore.getState();
    expect(settings.theme).toBe("light");
  });

  it("should update settings", () => {
    useStore.getState().setSettings({ ...DEFAULT_SETTINGS, theme: "dark" });
    expect(useStore.getState().settings.theme).toBe("dark");
  });

  it("should start in Agent mode with empty messages", () => {
    const { neocodexMode, neocodexMessages } = useStore.getState();
    expect(neocodexMode).toBe("Agent");
    expect(neocodexMessages).toHaveLength(0);
  });

  it("should switch neocodex mode", () => {
    useStore.getState().setNeoCodexMode("Plan");
    expect(useStore.getState().neocodexMode).toBe("Plan");
  });

  it("should add a neocodex message", () => {
    useStore.getState().addNeoCodexMessage({ role: "user", content: "hello", timestamp: Date.now() });
    const messages = useStore.getState().neocodexMessages;
    expect(messages).toHaveLength(1);
    expect(messages[0].content).toBe("hello");
  });

  it("should set streaming content", () => {
    useStore.getState().setNeoCodexStreaming({ content: "hi", role: "assistant" });
    expect(useStore.getState().neocodexStreaming?.content).toBe("hi");
  });

  it("should set neocodex sessions", () => {
    useStore.getState().setNeoCodexSessions([
      { id: "s1", name: "测试", mode: "Agent", messages: [], wire_path: "", created_at: 0, updated_at: 0 },
    ]);
    expect(useStore.getState().neocodexSessions).toHaveLength(1);
    expect(useStore.getState().neocodexSessions[0].name).toBe("测试");
  });

  it("should set active session", () => {
    useStore.getState().setNeoCodexActiveSession("s1");
    expect(useStore.getState().neocodexActiveSessionId).toBe("s1");
  });

  it("should manage notifications", () => {
    useStore.getState().addNotification({ type: "info", message: "hi", duration: 3000 });
    expect(useStore.getState().notifications).toHaveLength(1);
    const id = useStore.getState().notifications[0].id;
    useStore.getState().removeNotification(id);
    expect(useStore.getState().notifications).toHaveLength(0);
  });
});
