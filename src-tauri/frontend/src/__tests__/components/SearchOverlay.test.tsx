import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { useStore } from "../../stores";
import SearchOverlay from "../../components/SearchOverlay";

beforeEach(() => {
  useStore.setState({
    sessions: [
      {
        id: "s1",
        name: "会话一",
        messages: [
          { role: "user", content: "hello world", contentType: "text", timestamp: 1000 },
          { role: "assistant", content: "hi there", contentType: "text", timestamp: 1001 },
        ],
      },
      {
        id: "s2",
        name: "会话二",
        messages: [
          { role: "user", content: "search for rust", contentType: "text", timestamp: 2000 },
        ],
      },
    ],
    searchQuery: "",
    showSearch: true,
    setSearchQuery: useStore.getState().setSearchQuery,
    setShowSearch: useStore.getState().setShowSearch,
    setActiveSessionIndex: useStore.getState().setActiveSessionIndex,
  });
});

describe("SearchOverlay", () => {
  it("renders empty state when no query", () => {
    render(<SearchOverlay />);
    expect(screen.getByText("Type to search across all sessions")).toBeInTheDocument();
  });

  it("renders search input", () => {
    render(<SearchOverlay />);
    expect(screen.getByPlaceholderText("Search messages across all sessions…")).toBeInTheDocument();
  });

  it("shows results when query matches", () => {
    useStore.setState({ searchQuery: "hello" });
    render(<SearchOverlay />);
    const sessionLabels = screen.getAllByText("会话一");
    expect(sessionLabels.length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText(/hello/).length).toBeGreaterThan(0);
  });

  it("shows no results message when query has no matches", () => {
    useStore.setState({ searchQuery: "zzzznotfound" });
    render(<SearchOverlay />);
    expect(screen.getByText(/No results found/)).toBeInTheDocument();
  });

  it("shows result count", () => {
    useStore.setState({ searchQuery: "hello" });
    render(<SearchOverlay />);
    expect(screen.getByText("1 result")).toBeInTheDocument();
  });

  it("shows multiple results across sessions", () => {
    useStore.setState({ searchQuery: "h" });
    render(<SearchOverlay />);
    const youLabels = screen.getAllByText("You");
    expect(youLabels.length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText(/h/).length).toBeGreaterThanOrEqual(1);
  });

  it("includes hint text for keyboard navigation", () => {
    render(<SearchOverlay />);
    expect(screen.getByText(/↑↓/)).toBeInTheDocument();
    expect(screen.getByText(/↵/)).toBeInTheDocument();
    expect(screen.getByText(/Esc/)).toBeInTheDocument();
  });

  it("shows role labels in results", () => {
    useStore.setState({ searchQuery: "hello" });
    render(<SearchOverlay />);
    expect(screen.getByText("You")).toBeInTheDocument();
  });

  it("shows session tag for results", () => {
    useStore.setState({ searchQuery: "hello" });
    render(<SearchOverlay />);
    const sessionLabels = screen.getAllByText("会话一");
    expect(sessionLabels.length).toBeGreaterThanOrEqual(1);
  });

  it("clears search on clear button click", () => {
    useStore.setState({ searchQuery: "hello" });
    render(<SearchOverlay />);
    const clearBtn = document.querySelector("button[tabindex='-1']");
    if (clearBtn) fireEvent.click(clearBtn);
    expect(useStore.getState().searchQuery).toBe("");
  });
});
