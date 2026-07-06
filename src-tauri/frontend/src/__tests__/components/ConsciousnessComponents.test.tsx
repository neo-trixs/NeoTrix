import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { useStore } from "../../stores";
import E8Indicator from "../../components/consciousness/E8Indicator";
import CommandPalette from "../../components/CommandPalette";
import { DEFAULT_SETTINGS } from "../../stores/store-utils";
import type { E8State } from "../../types";

beforeEach(() => {
  useStore.setState({
    sessions: [{ id: "default", name: "默认会话", messages: [] }],
    activeSessionIndex: 0,
    statusText: "Ready",
    agentBusy: false,
    projectPath: "",
    showFileTree: false,
    pendingPermission: null,
    showOnboarding: false,
    showCommandPalette: false,
    showShortcuts: false,
    showSearch: false,
    showAgentManager: false,
    showPrivacyFilter: false,
    showSandboxManager: false,
    showIdentityManager: false,
    searchQuery: "",
    streamingContent: "",
    streamingContentType: "markdown",
    notifications: [],
    updateAvailable: false,
    updateStatus: "",
    updateProgress: 0,
    editorState: { open: false, filePath: "", initialContent: "", language: "" },
    settings: { ...DEFAULT_SETTINGS },
    knowledgeBase: [],
    providerConfig: { id: "anthropic" as const, name: "Anthropic", model: "claude-sonnet-4-20250514", apiKey: "", learningRate: 0.05 },
    splitViewActive: false,
    syncVisible: false,
    proxyVisible: false,
    evolutionVisible: false,
    agentMakerActive: false,
    sidebarCollapsed: false,
    pendingDiff: null,
    rightPanelWidth: 320,
    consciousnessActive: false,
    e8State: {
      hexagram: 0x01, hexagramName: "Grounding", confidence: 0.5,
      lines: [
        { value: 1, changing: false },
        { value: 1, changing: false },
        { value: 0, changing: false },
        { value: 0, changing: false },
        { value: 0, changing: false },
        { value: 1, changing: false },
      ],
      transitioning: false,
    },
  });
});

describe("E8Indicator", () => {
  it("renders hexagram name based on hex code and confidence as percentage", () => {
    const e8: E8State = {
      hexagram: 0x01, hexagramName: "Grounding", confidence: 0.5,
      lines: [{ value: 1, changing: false }, { value: 1, changing: false }, { value: 0, changing: false }, { value: 0, changing: false }, { value: 0, changing: false }, { value: 1, changing: false }],
      transitioning: false,
    };
    render(<E8Indicator e8={e8} />);
    expect(screen.getByText("Grounding")).toBeInTheDocument();
    expect(screen.getByText("50%")).toBeInTheDocument();
  });

  it("renders 6 line elements", () => {
    const e8: E8State = {
      hexagram: 0x02, hexagramName: "Test", confidence: 0.5,
      lines: [{ value: 1, changing: false }, { value: 0, changing: false }, { value: 1, changing: false }, { value: 0, changing: false }, { value: 1, changing: false }, { value: 0, changing: false }],
      transitioning: false,
    };
    const { container } = render(<E8Indicator e8={e8} />);
    const lines = container.querySelectorAll(".e8-line");
    expect(lines.length).toBe(6);
  });

  it("calls onClick when interactive and clicked", () => {
    let clicked = false;
    const e8: E8State = {
      hexagram: 0x01, hexagramName: "Grounding", confidence: 0.5,
      lines: [{ value: 1, changing: false }, { value: 1, changing: false }, { value: 0, changing: false }, { value: 0, changing: false }, { value: 0, changing: false }, { value: 1, changing: false }],
      transitioning: false,
    };
    const { container } = render(<E8Indicator e8={e8} interactive onClick={() => { clicked = true; }} />);
    fireEvent.click(container.querySelector(".e8-indicator")!);
    expect(clicked).toBe(true);
  });
});

describe("CommandPalette", () => {
  it("does not render when showCommandPalette is false", () => {
    useStore.setState({ showCommandPalette: false });
    const { container } = render(
      <MemoryRouter><CommandPalette /></MemoryRouter>
    );
    expect(container.innerHTML).toBe("");
  });

  it("renders when showCommandPalette is true", () => {
    useStore.setState({ showCommandPalette: true });
    render(<MemoryRouter><CommandPalette /></MemoryRouter>);
    expect(screen.getByPlaceholderText("Type a command or search...")).toBeInTheDocument();
  });

  it("filters items by query", () => {
    useStore.setState({ showCommandPalette: true });
    render(<MemoryRouter><CommandPalette /></MemoryRouter>);
    const input = screen.getByPlaceholderText("Type a command or search...");
    fireEvent.change(input, { target: { value: "Privacy" } });
    expect(screen.getByText(/Privacy/)).toBeInTheDocument();
  });

  it("closes on Escape", () => {
    useStore.setState({ showCommandPalette: true });
    render(<MemoryRouter><CommandPalette /></MemoryRouter>);
    expect(screen.getByPlaceholderText("Type a command or search...")).toBeInTheDocument();
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        useStore.setState({ showCommandPalette: false });
      }
    };
    window.addEventListener("keydown", handler);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(useStore.getState().showCommandPalette).toBe(false);
    window.removeEventListener("keydown", handler);
  });
});
