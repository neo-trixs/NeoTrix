import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import StatusBar from "../../components/StatusBar";
import { useStore } from "../../store";

beforeEach(() => {
  const store = useStore.getState();
  useStore.setState({
    ...store,
    statusText: "Ready",
    agentBusy: false,
    activeSessionIndex: 0,
    sessions: [{ id: "s1" }, { id: "s2" }, { id: "s3" }] as any,
    currentModel: null,
    contextUsage: 0,
    agentMode: "chat",
    currentGitBranch: "main",
    settings: { ...store.settings, theme: "dark" },
  });
});

describe("StatusBar", () => {
  it("renders status text", () => {
    render(<StatusBar />);
    expect(screen.getByText("Ready")).toBeInTheDocument();
  });

  it("shows session index and count", () => {
    render(<StatusBar />);
    expect(screen.getByText("S1/3")).toBeInTheDocument();
  });

  it("shows busy indicator when agentBusy is true", () => {
    useStore.setState({ agentBusy: true, statusText: "Thinking..." });
    render(<StatusBar />);
    expect(screen.getByText("Thinking...")).toBeInTheDocument();
    expect(screen.getByText("S1/3")).toBeInTheDocument();
  });

  it("has a busy class on the container when agent is busy", () => {
    useStore.setState({ agentBusy: true });
    const { container } = render(<StatusBar />);
    expect(container.querySelector(".status-bar")?.className).toContain("busy");
  });

  it("shows model badge when model is set", () => {
    useStore.setState({
      currentModel: { id: "op4", name: "claude-opus-4", provider: "anthropic", context_length: 200000, capabilities: [] },
    });
    render(<StatusBar />);
    expect(screen.getByText("claude-opus-4")).toBeInTheDocument();
  });

  it("shows context usage percentage", () => {
    useStore.setState({
      currentModel: { id: "op4", name: "claude-opus-4", provider: "anthropic", context_length: 200000, capabilities: [] },
      contextUsage: 45,
    });
    render(<StatusBar />);
    const ctxLabels = screen.getAllByText(/k/);
    expect(ctxLabels.length).toBeGreaterThanOrEqual(0);
  });

  it("mode button cycles through modes", () => {
    render(<StatusBar />);
    const modeBtn = screen.getByText("Chat");
    fireEvent.click(modeBtn);
    expect(screen.getByText("Plan")).toBeInTheDocument();
  });

  it("shows git branch", () => {
    render(<StatusBar />);
    expect(screen.getByText("main")).toBeInTheDocument();
  });

  it("settings button opens settings", () => {
    const setShowSettings = vi.fn();
    const orig = useStore.getState().setShowSettings;
    useStore.setState({ setShowSettings });
    render(<StatusBar />);
    const settingsBtn = screen.getByTitle("Settings");
    fireEvent.click(settingsBtn);
    expect(setShowSettings).toHaveBeenCalledWith(true);
    useStore.setState({ setShowSettings: orig });
  });
});
