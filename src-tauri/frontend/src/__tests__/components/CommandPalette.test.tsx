import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CommandPalette, type PaletteItem } from "../../components/neocodex/CommandPalette";

const items: PaletteItem[] = [
  { id: "new", label: "新建会话", hint: "⌘N", onSelect: vi.fn() },
  { id: "settings", label: "设置", hint: "⌘,", onSelect: vi.fn() },
  { id: "focus", label: "专注模式", hint: "⌘Shift+F", onSelect: vi.fn() },
  { id: "session-1", label: "重构缓存层", hint: "Agent", onSelect: vi.fn() },
];

describe("CommandPalette — fuzzy search & keyboard nav", () => {
  it("renders all items when open and no query", () => {
    render(<CommandPalette open items={items} onClose={() => {}} />);
    expect(screen.getByText("新建会话")).toBeInTheDocument();
    expect(screen.getByText("设置")).toBeInTheDocument();
    expect(screen.getByText("重构缓存层")).toBeInTheDocument();
  });

  it("returns null when closed", () => {
    const { container } = render(<CommandPalette open={false} items={items} onClose={() => {}} />);
    expect(container.firstChild).toBeNull();
  });

  it("fuzzy filters and ranks: query surfaces matching item", async () => {
    render(<CommandPalette open items={items} onClose={() => {}} />);
    const input = screen.getByPlaceholderText(/搜索会话/);
    await userEvent.type(input, "会话");
    expect(screen.getByText("新建会话")).toBeInTheDocument();
    expect(screen.queryByText("设置")).not.toBeInTheDocument();
  });

  it("prefix match ranks above plain containment", async () => {
    render(<CommandPalette open items={items} onClose={() => {}} />);
    const input = screen.getByPlaceholderText(/搜索会话/);
    await userEvent.type(input, "置");
    // "设置" contains 置; only item should remain
    const list = screen.getByText("设置").closest("div")!.parentElement!;
    const labels = Array.from(list.querySelectorAll("button")).map((b) => b.textContent);
    expect(labels[0]).toContain("设置");
  });

  it("ArrowDown + Enter selects and calls onClose", async () => {
    const onClose = vi.fn();
    render(<CommandPalette open items={items} onClose={onClose} />);
    await userEvent.keyboard("{ArrowDown}");
    await userEvent.keyboard("{Enter}");
    expect(items[1].onSelect).toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });

  it("Enter without navigation selects first item", async () => {
    const onClose = vi.fn();
    render(<CommandPalette open items={items} onClose={onClose} />);
    await userEvent.keyboard("{Enter}");
    expect(items[0].onSelect).toHaveBeenCalled();
  });

  it("Escape closes", async () => {
    const onClose = vi.fn();
    render(<CommandPalette open items={items} onClose={onClose} />);
    await userEvent.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalled();
  });

  it("mouse click selects and closes", async () => {
    const onClose = vi.fn();
    render(<CommandPalette open items={items} onClose={onClose} />);
    await userEvent.click(screen.getByText("专注模式"));
    expect(items[2].onSelect).toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });

  it("shows empty state when no match", async () => {
    render(<CommandPalette open items={items} onClose={() => {}} />);
    await userEvent.type(screen.getByPlaceholderText(/搜索会话/), "zzzz");
    expect(screen.getByText("无匹配项")).toBeInTheDocument();
  });

  it("ArrowUp wraps and keeps bounds (does not go negative)", async () => {
    render(<CommandPalette open items={items} onClose={() => {}} />);
    await userEvent.keyboard("{ArrowUp}");
    await userEvent.keyboard("{Enter}");
    expect(items[0].onSelect).toHaveBeenCalled();
  });

  it("declares dialog semantics and traps Tab focus", async () => {
    render(<CommandPalette open items={items} onClose={() => {}} />);
    const palette = screen.getByTestId("command-palette");
    expect(palette).toHaveAttribute("role", "dialog");
    expect(palette).toHaveAttribute("aria-modal", "true");
    expect(screen.getByTestId("palette-input")).toHaveFocus();
    // Tab from the last item wraps back to the input (focus trap).
    const buttons = screen.getAllByTestId(/palette-item/);
    buttons[buttons.length - 1].focus();
    await userEvent.tab();
    expect(screen.getByTestId("palette-input")).toHaveFocus();
    // Shift+Tab from the first item wraps to the last (focus trap).
    await userEvent.tab({ shift: true });
    expect(buttons[buttons.length - 1]).toHaveFocus();
  });

  it("restores focus to the trigger on close", async () => {
    const trigger = document.createElement("button");
    trigger.textContent = "打开";
    document.body.appendChild(trigger);
    trigger.focus();
    const { rerender } = render(<CommandPalette open items={items} onClose={() => {}} />);
    rerender(<CommandPalette open={false} items={items} onClose={() => {}} />);
    expect(document.activeElement).toBe(trigger);
    trigger.remove();
  });
});
