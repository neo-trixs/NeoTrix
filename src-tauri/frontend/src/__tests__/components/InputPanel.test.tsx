import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import InputPanel from "../../components/InputPanel";
import { useStore } from "../../store";

beforeEach(() => {
  const store = useStore.getState();
  useStore.setState({
    ...store,
    contextChips: [],
    agentMode: "chat",
    currentModel: null,
    contextUsage: 0,
  });
});

describe("InputPanel", () => {
  it("renders textarea", () => {
    render(<InputPanel onSubmit={vi.fn()} disabled={false} />);
    const textarea = screen.getByRole("textbox");
    expect(textarea).toBeInTheDocument();
  });

  it("renders send button", () => {
    render(<InputPanel onSubmit={vi.fn()} disabled={false} />);
    expect(screen.getByRole("button", { name: "Send" })).toBeInTheDocument();
  });

  it("submit button triggers onSubmit with value", () => {
    const onSubmit = vi.fn();
    render(<InputPanel onSubmit={onSubmit} disabled={false} />);
    const textarea = screen.getByRole("textbox");
    fireEvent.change(textarea, { target: { value: "test message" } });
    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    expect(onSubmit).toHaveBeenCalledWith("test message");
  });

  it("does not call onSubmit when value is empty", () => {
    const onSubmit = vi.fn();
    render(<InputPanel onSubmit={onSubmit} disabled={false} />);
    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("Enter key submits", () => {
    const onSubmit = vi.fn();
    render(<InputPanel onSubmit={onSubmit} disabled={false} />);
    const textarea = screen.getByRole("textbox");
    fireEvent.change(textarea, { target: { value: "hello" } });
    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(onSubmit).toHaveBeenCalledWith("hello");
  });

  it("Alt+Enter adds newline instead of submitting", () => {
    const onSubmit = vi.fn();
    render(<InputPanel onSubmit={onSubmit} disabled={false} />);
    const textarea = screen.getByRole("textbox");
    fireEvent.change(textarea, { target: { value: "hello" } });
    fireEvent.keyDown(textarea, { key: "Enter", altKey: true });
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("send button is disabled when value is empty", () => {
    render(<InputPanel onSubmit={vi.fn()} disabled={false} />);
    expect(screen.getByRole("button", { name: "Send" })).toBeDisabled();
  });

  it("send button is enabled when value is non-empty", () => {
    render(<InputPanel onSubmit={vi.fn()} disabled={false} />);
    const textarea = screen.getByRole("textbox");
    fireEvent.change(textarea, { target: { value: "hello" } });
    expect(screen.getByRole("button", { name: "Send" })).not.toBeDisabled();
  });

  it("textarea is disabled when disabled prop is true", () => {
    render(<InputPanel onSubmit={vi.fn()} disabled={true} />);
    expect(screen.getByRole("textbox")).toBeDisabled();
  });

  it("does not submit via Enter when disabled", () => {
    const onSubmit = vi.fn();
    render(<InputPanel onSubmit={onSubmit} disabled={true} />);
    const textarea = screen.getByRole("textbox");
    fireEvent.change(textarea, { target: { value: "hello" } });
    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("shows mode indicator for chat mode", () => {
    render(<InputPanel onSubmit={vi.fn()} disabled={false} />);
    expect(screen.getByText("Chat")).toBeInTheDocument();
  });

  it("shows mode indicator for plan mode", () => {
    useStore.setState({ agentMode: "plan" });
    render(<InputPanel onSubmit={vi.fn()} disabled={false} />);
    expect(screen.getByText("Plan")).toBeInTheDocument();
  });

  it("shows mode indicator for agent mode", () => {
    useStore.setState({ agentMode: "agent" });
    render(<InputPanel onSubmit={vi.fn()} disabled={false} />);
    expect(screen.getByText("Agent")).toBeInTheDocument();
  });
});
