import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ShortcutHelp } from "../../components/neocodex/ShortcutHelp";

describe("ShortcutHelp", () => {
  it("renders nothing when closed", () => {
    const { container } = render(<ShortcutHelp open={false} onClose={() => {}} />);
    expect(container.innerHTML).toBe("");
  });

  it("renders shortcut list when open", () => {
    render(<ShortcutHelp open onClose={() => {}} />);
    expect(screen.getByText("快捷键")).toBeInTheDocument();
    expect(screen.getByText("⌘N")).toBeInTheDocument();
    expect(screen.getByText("新建会话")).toBeInTheDocument();
    expect(screen.getByText("命令面板")).toBeInTheDocument();
  });

  it("has dialog role and aria-label", () => {
    render(<ShortcutHelp open onClose={() => {}} />);
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.getByLabelText("快捷键")).toBeInTheDocument();
  });

  it("closes on Escape", () => {
    let closed = false;
    render(<ShortcutHelp open onClose={() => { closed = true; }} />);
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(closed).toBe(true);
  });

  it("closes on overlay click", () => {
    let closed = false;
    const { container } = render(<ShortcutHelp open onClose={() => { closed = true; }} />);
    fireEvent.click(container.querySelector("[class*='overlay']")!);
    expect(closed).toBe(true);
  });
});
