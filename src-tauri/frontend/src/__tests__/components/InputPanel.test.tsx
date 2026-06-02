import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import InputPanel from "../../components/InputPanel";

const defaultProps = {
  value: "",
  onChange: vi.fn(),
  onSubmit: vi.fn(),
  multiLine: false,
  onMultiLineToggle: vi.fn(),
  disabled: false,
};

describe("InputPanel", () => {
  it("renders textarea", () => {
    render(<InputPanel {...defaultProps} />);
    const textarea = screen.getByPlaceholderText(/输入消息/);
    expect(textarea).toBeInTheDocument();
  });

  it("renders send button", () => {
    render(<InputPanel {...defaultProps} />);
    expect(screen.getByRole("button", { name: "发送" })).toBeInTheDocument();
  });

  it("submit button triggers onSubmit with value", () => {
    const onSubmit = vi.fn();
    const onChange = vi.fn();
    render(
      <InputPanel
        {...defaultProps}
        value="测试消息"
        onSubmit={onSubmit}
        onChange={onChange}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: "发送" }));
    expect(onSubmit).toHaveBeenCalledWith("测试消息");
    expect(onChange).toHaveBeenCalledWith("");
  });

  it("does not call onSubmit when value is empty", () => {
    const onSubmit = vi.fn();
    render(<InputPanel {...defaultProps} onSubmit={onSubmit} />);
    fireEvent.click(screen.getByRole("button", { name: "发送" }));
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("Enter key submits when not multiLine", () => {
    const onSubmit = vi.fn();
    const onChange = vi.fn();
    render(
      <InputPanel
        {...defaultProps}
        value="hello"
        onSubmit={onSubmit}
        onChange={onChange}
      />
    );
    fireEvent.keyDown(screen.getByRole("textbox"), { key: "Enter" });
    expect(onSubmit).toHaveBeenCalledWith("hello");
    expect(onChange).toHaveBeenCalledWith("");
  });

  it("Alt+Enter adds newline instead of submitting", () => {
    const onSubmit = vi.fn();
    const onChange = vi.fn();
    render(
      <InputPanel
        {...defaultProps}
        value="hello"
        onSubmit={onSubmit}
        onChange={onChange}
      />
    );
    fireEvent.keyDown(screen.getByRole("textbox"), { key: "Enter", altKey: true });
    expect(onSubmit).not.toHaveBeenCalled();
    expect(onChange).toHaveBeenCalledWith("hello\n");
  });

  it("send button is disabled when value is empty", () => {
    render(<InputPanel {...defaultProps} value="" />);
    expect(screen.getByRole("button", { name: "发送" })).toBeDisabled();
  });

  it("send button is enabled when value is non-empty", () => {
    render(<InputPanel {...defaultProps} value="hello" />);
    expect(screen.getByRole("button", { name: "发送" })).not.toBeDisabled();
  });

  it("textarea is disabled when disabled prop is true", () => {
    render(<InputPanel {...defaultProps} disabled={true} />);
    expect(screen.getByRole("textbox")).toBeDisabled();
  });

  it("does not submit via Enter when disabled", () => {
    const onSubmit = vi.fn();
    const onChange = vi.fn();
    render(
      <InputPanel
        {...defaultProps}
        value="hello"
        disabled={true}
        onSubmit={onSubmit}
        onChange={onChange}
      />
    );
    fireEvent.keyDown(screen.getByRole("textbox"), { key: "Enter" });
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("calls onMultiLineToggle when multiline button clicked", () => {
    const onMultiLineToggle = vi.fn();
    render(
      <InputPanel
        {...defaultProps}
        onMultiLineToggle={onMultiLineToggle}
      />
    );
    const multiLineBtn = screen.getByTitle("多行模式");
    fireEvent.click(multiLineBtn);
    expect(onMultiLineToggle).toHaveBeenCalledOnce();
  });

  it("shows multiline placeholder in multiline mode", () => {
    render(<InputPanel {...defaultProps} multiLine={true} />);
    expect(screen.getByPlaceholderText(/多行模式/)).toBeInTheDocument();
  });
});
