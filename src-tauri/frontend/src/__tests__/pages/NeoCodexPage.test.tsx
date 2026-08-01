import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import NeoCodexPage from "../../pages/NeoCodexPage";
import { useStore } from "../../stores";
import { mockInvoke, resetInvokeMocks } from "../../__tests__/tauriMock";

describe("NeoCodexPage new-session handling", () => {
  beforeEach(() => {
    resetInvokeMocks();
    useStore.setState({ neocodexSessions: [], neocodexActiveSessionId: null });
    mockInvoke("neocodex_create_session", () => ({
      id: "s-cmdn",
      name: "新会话",
      mode: "Agent",
      message_count: 0,
      wire_path: "",
      updated_at: 0,
    }));
  });

  it("creates a session via Cmd+N even when the sidebar is collapsed", async () => {
    render(<NeoCodexPage />);

    const collapseBtn = await screen.findByTitle("收起侧栏");
    fireEvent.click(collapseBtn);
    expect(screen.queryByTitle("新建会话")).toBeNull();

    const createSpy = vi.fn(() => ({
      id: "s-cmdn",
      name: "新会话",
      mode: "Agent",
      message_count: 0,
      wire_path: "",
      updated_at: 0,
    }));
    mockInvoke("neocodex_create_session", createSpy);
    fireEvent.keyDown(window, { key: "n", metaKey: true });

    await waitFor(() => {
      expect(createSpy).toHaveBeenCalled();
      expect(useStore.getState().neocodexActiveSessionId).toBe("s-cmdn");
    });
  });

  it("creates a session when the neotrix:new-session event fires with the sidebar collapsed", async () => {
    render(<NeoCodexPage />);

    fireEvent.click(await screen.findByTitle("收起侧栏"));
    expect(screen.queryByTitle("新建会话")).toBeNull();

    const createSpy = vi.fn(() => ({
      id: "s-event",
      name: "新会话",
      mode: "Agent",
      message_count: 0,
      wire_path: "",
      updated_at: 0,
    }));
    mockInvoke("neocodex_create_session", createSpy);
    window.dispatchEvent(new CustomEvent("neotrix:new-session"));

    await waitFor(() => {
      expect(createSpy).toHaveBeenCalled();
      expect(useStore.getState().neocodexActiveSessionId).toBe("s-event");
    });
  });
});
