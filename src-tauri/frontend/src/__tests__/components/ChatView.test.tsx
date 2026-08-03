import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ChatView } from "../../components/neocodex/ChatView";
import { useStore } from "../../stores";
import { mockInvoke, resetInvokeMocks } from "../tauriMock";
import type { Message } from "../../types";

function makeMessages(): Message[] {
  return [
    { id: 0, role: "user", content: "帮我写个函数", timestamp: Date.now() - 2000 },
    { id: 1, role: "assistant", content: "```ts\nfunction add(a: number, b: number) { return a + b; }\n```", timestamp: Date.now() - 1000 },
  ];
}

beforeEach(() => {
  resetInvokeMocks();
  useStore.setState({ notifications: [] });
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("ChatView — composer & steer mode", () => {
  it("sends the typed message on Enter via onSend", async () => {
    const onSend = vi.fn();
    render(<ChatView messages={[]} agentBusy={false} onSend={onSend} />);
    const textarea = screen.getByPlaceholderText(/Enter 发送/);
    await userEvent.type(textarea, "你好，NeoCodex");
    await userEvent.keyboard("{Enter}");
    expect(onSend).toHaveBeenCalledWith("你好，NeoCodex", []);
  });

  it("queues input when agent is busy (steer mode: Enter queues)", async () => {
    const onSend = vi.fn();
    render(<ChatView messages={makeMessages()} agentBusy onSend={onSend} />);
    const textarea = screen.getByPlaceholderText(/Enter\/Tab 排队/);
    await userEvent.type(textarea, "排队中的问题");
    await userEvent.keyboard("{Enter}");
    expect(onSend).not.toHaveBeenCalled();
    expect(screen.getByText(/已排队 1 条/)).toBeInTheDocument();
  });

  it("Tab also queues while busy", async () => {
    const onSend = vi.fn();
    render(<ChatView messages={makeMessages()} agentBusy onSend={onSend} />);
    const textarea = screen.getByPlaceholderText(/Enter\/Tab 排队/);
    await userEvent.type(textarea, "第二条问题");
    await userEvent.tab();
    expect(screen.getByText(/已排队 1 条/)).toBeInTheDocument();
  });

  it("queue bar cancel clears queued inputs", async () => {
    const onSend = vi.fn();
    render(<ChatView messages={makeMessages()} agentBusy onSend={onSend} />);
    const textarea = screen.getByPlaceholderText(/Enter\/Tab 排队/);
    await userEvent.type(textarea, "待取消问题");
    await userEvent.keyboard("{Enter}");
    expect(screen.getByText(/已排队 1 条/)).toBeInTheDocument();
    await userEvent.click(screen.getByTitle("清空队列"));
    expect(screen.queryByText(/已排队/)).not.toBeInTheDocument();
  });

  it("shows stop button while busy and invokes onStop", async () => {
    const onStop = vi.fn();
    render(<ChatView messages={makeMessages()} agentBusy onSend={() => {}} onStop={onStop} />);
    const stop = screen.getByTitle(/停止生成/);
    await userEvent.click(stop);
    expect(onStop).toHaveBeenCalled();
  });

  it("renders Codex-parity composer control: plan toggle", async () => {
    const onModeChange = vi.fn();
    render(<ChatView messages={[]} agentBusy={false} onSend={() => {}} mode="Agent" onModeChange={onModeChange} />);
    expect(screen.getByTestId("composer-plan-toggle")).toBeInTheDocument();
  });

  it("plan toggle calls onModeChange('Plan') and reflects active Plan state", async () => {
    const onModeChange = vi.fn();
    const { unmount } = render(<ChatView messages={[]} agentBusy={false} onSend={() => {}} mode="Agent" onModeChange={onModeChange} />);
    const toggle = screen.getByTestId("composer-plan-toggle");
    await userEvent.click(toggle);
    expect(onModeChange).toHaveBeenCalledWith("Plan");
    unmount();
    render(<ChatView messages={[]} agentBusy={false} onSend={() => {}} mode="Plan" onModeChange={onModeChange} />);
    await userEvent.click(screen.getByTestId("composer-plan-toggle"));
    expect(onModeChange).toHaveBeenLastCalledWith("Agent");
  });
});

describe("ChatView — slash commands", () => {
  it("shows slash menu when typing / and selects via Enter", async () => {
    const onSend = vi.fn();
    render(<ChatView messages={[]} agentBusy={false} onSend={onSend} />);
    const textarea = screen.getByPlaceholderText(/Enter 发送/);
    await userEvent.type(textarea, "/comp");
    await waitFor(() => expect(screen.getByText("压缩会话")).toBeInTheDocument());
    await userEvent.keyboard("{Enter}");
    expect(textarea).toHaveValue("/compact ");
    await userEvent.keyboard("{Enter}");
    expect(onSend).toHaveBeenCalledWith("/compact", []);
  });

  it("Escape closes the slash menu", async () => {
    render(<ChatView messages={[]} agentBusy={false} onSend={() => {}} />);
    const textarea = screen.getByPlaceholderText(/Enter 发送/);
    await userEvent.type(textarea, "/status");
    await waitFor(() => expect(screen.getByText("查看状态")).toBeInTheDocument());
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByText("查看状态")).not.toBeInTheDocument();
  });
});

describe("ChatView — @mention recursive search", () => {
  it("opens mention menu and inserts a file", async () => {
    mockInvoke("neocodex_search_files", () => ["src/core/cache.rs", "src/main.rs"]);
    render(<ChatView messages={[]} agentBusy={false} onSend={() => {}} />);
    const textarea = screen.getByPlaceholderText(/Enter 发送/);
    await userEvent.type(textarea, "看看 @cache");
    await waitFor(() => expect(screen.getByText("src/core/cache.rs")).toBeInTheDocument());
    await userEvent.keyboard("{Enter}");
    await waitFor(() => expect(screen.getByText(/@src\/core\/cache\.rs/)).toBeInTheDocument());
    expect((textarea as HTMLTextAreaElement).value).toContain("@src/core/cache.rs");
  });
});

describe("ChatView — message actions (编辑/删除/重新生成/复制)", () => {
  it("edits a user message in place and commits via onEdit", async () => {
    const onEdit = vi.fn();
    render(<ChatView messages={makeMessages()} agentBusy={false} onSend={() => {}} onEdit={onEdit} />);
    const editBtns = screen.getAllByTitle("编辑");
    await userEvent.click(editBtns[0]);
    const editTextarea = screen.getByDisplayValue("帮我写个函数");
    await userEvent.clear(editTextarea);
    await userEvent.type(editTextarea, "帮我写个带缓存的函数");
    await userEvent.keyboard("{Enter}");
    expect(onEdit).toHaveBeenCalledWith(0, "帮我写个带缓存的函数");
  });

  it("deletes a user message via onDelete", async () => {
    const onDelete = vi.fn();
    render(<ChatView messages={makeMessages()} agentBusy={false} onSend={() => {}} onDelete={onDelete} />);
    const deleteBtns = screen.getAllByTitle("删除");
    await userEvent.click(deleteBtns[0]);
    expect(onDelete).toHaveBeenCalledWith(0);
  });

  it("regenerates an assistant reply via onRegenerate", async () => {
    const onRegenerate = vi.fn();
    render(<ChatView messages={makeMessages()} agentBusy={false} onSend={() => {}} onRegenerate={onRegenerate} />);
    const regen = screen.getByTitle("重新生成");
    await userEvent.click(regen);
    expect(onRegenerate).toHaveBeenCalledWith(1);
  });

  it("copies a message to the clipboard", async () => {
    Object.assign(navigator, { clipboard: { writeText: vi.fn().mockResolvedValue(undefined) } });
    render(<ChatView messages={makeMessages()} agentBusy={false} onSend={() => {}} />);
    const copy = screen.getAllByTitle("复制");
    await userEvent.click(copy[0]);
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith("帮我写个函数");
  });
});

describe("ChatView — rendering details", () => {
  it("renders markdown code blocks with code indicator", () => {
    render(<ChatView messages={makeMessages()} agentBusy={false} onSend={() => {}} />);
    expect(screen.getByText("1 代码块")).toBeInTheDocument();
    expect(document.querySelector("pre code")).not.toBeNull();
  });

  it("shows thinking indicator while busy before any streamed content", () => {
    render(<ChatView messages={makeMessages()} agentBusy onSend={() => {}} />);
    expect(screen.getByText("思考中…")).toBeInTheDocument();
  });

  it("renders inline image attachment preview", () => {
    const withImage: Message[] = [{
      id: 5,
      role: "user",
      content: "看看这张图",
      attachments: [{ id: "a1", name: "shot.png", size: 2048, mimeType: "image/png", data: "iVBORw0KGgo=" }],
    }];
    render(<ChatView messages={withImage} agentBusy={false} onSend={() => {}} />);
    const img = document.querySelector("img[alt='shot.png']") as HTMLImageElement;
    expect(img).not.toBeNull();
    expect(img.src).toContain("data:image/png;base64");
  });

  it("renders tool card with success status dot", () => {
    const toolMsg: Message[] = [{ id: 9, role: "tool", content: "**read_file**\n```\nfn main() {}\n```", timestamp: Date.now() }];
    render(<ChatView messages={toolMsg} agentBusy={false} onSend={() => {}} />);
    expect(screen.getByText("read_file")).toBeInTheDocument();
    const dot = document.querySelector("[title='成功']");
    expect(dot).not.toBeNull();
  });

  it("summary view hides tool messages", () => {
    const msgs = [...makeMessages(), { id: 9, role: "tool" as const, content: "**ls**\n```\nfile.txt\n```" }];
    render(<ChatView messages={msgs} agentBusy={false} onSend={() => {}} viewMode="summary" />);
    expect(screen.queryByText("ls")).not.toBeInTheDocument();
  });
});

describe("ChatView — welcome empty state (recent sessions)", () => {
  const recent = [
    { id: "s-r1", name: "上次的会话", mode: "Agent" as const, message_count: 12, updated_at: Date.now() - 60_000 },
    { id: "s-r2", name: "修 bug 的会话", mode: "Code" as const, message_count: 3, updated_at: Date.now() - 3600_000 },
  ];

  it("renders recent sessions when no messages and shows usage stats", () => {
    render(<ChatView messages={[]} agentBusy={false} onSend={() => {}} recentSessions={recent} />);
    expect(screen.getByText("最近会话")).toBeInTheDocument();
    expect(screen.getByText("上次的会话")).toBeInTheDocument();
    expect(screen.getByText("修 bug 的会话")).toBeInTheDocument();
    expect(screen.getByTestId("recent-sessions")).toBeInTheDocument();
  });

  it("clicking a recent session invokes onRecentSessionSelect", async () => {
    const onSelect = vi.fn();
    render(<ChatView messages={[]} agentBusy={false} onSend={() => {}} recentSessions={recent} onRecentSessionSelect={onSelect} />);
    await userEvent.click(screen.getByText("上次的会话"));
    expect(onSelect).toHaveBeenCalledWith("s-r1");
  });

  it("hides the recent block once the conversation has messages", () => {
    render(<ChatView messages={makeMessages()} agentBusy={false} onSend={() => {}} recentSessions={recent} />);
    expect(screen.queryByTestId("recent-sessions")).not.toBeInTheDocument();
    expect(screen.queryByText("最近会话")).not.toBeInTheDocument();
  });
});

describe("ChatView — quick action chips (welcome empty state)", () => {
  it("renders quick action chips in the empty state", () => {
    render(<ChatView messages={[]} agentBusy={false} onSend={() => {}} />);
    expect(screen.getByTestId("quick-actions")).toBeInTheDocument();
    expect(screen.getByText("分析项目结构")).toBeInTheDocument();
    expect(screen.getByText("排查 Bug")).toBeInTheDocument();
  });

  it("clicking a chip fills the composer and focuses it", async () => {
    render(<ChatView messages={[]} agentBusy={false} onSend={() => {}} />);
    const chip = screen.getByTestId("quick-action-排查 Bug");
    await userEvent.click(chip);
    const textarea = screen.getByPlaceholderText(/Enter 发送/) as HTMLTextAreaElement;
    expect(textarea.value).toContain("排查");
    expect(document.activeElement).toBe(textarea);
  });

  it("hides quick actions once messages exist", () => {
    render(<ChatView messages={makeMessages()} agentBusy={false} onSend={() => {}} />);
    expect(screen.queryByTestId("quick-actions")).not.toBeInTheDocument();
  });
});

describe("ChatView — per-code-block copy button", () => {
  it("renders a copy button for each code block", () => {
    Object.assign(navigator, { clipboard: { writeText: vi.fn().mockResolvedValue(undefined) } });
    render(<ChatView messages={makeMessages()} agentBusy={false} onSend={() => {}} />);
    const btns = document.querySelectorAll(".codeblock-copy");
    expect(btns.length).toBeGreaterThan(0);
    expect(screen.getAllByTitle("复制代码").length).toBeGreaterThan(0);
  });

  it("clicking the code-block copy button copies the code text", async () => {
    Object.assign(navigator, { clipboard: { writeText: vi.fn().mockResolvedValue(undefined) } });
    render(<ChatView messages={makeMessages()} agentBusy={false} onSend={() => {}} />);
    const btn = document.querySelector(".codeblock-copy") as HTMLButtonElement;
    expect(btn).not.toBeNull();
    await userEvent.click(btn);
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith("function add(a: number, b: number) { return a + b; }");
  });
});

describe("ChatView — scroll-to-bottom button", () => {
  it("does not show by default and appears when scrolled away from the bottom", () => {
    render(<ChatView messages={makeMessages()} agentBusy={false} onSend={() => {}} />);
    expect(screen.queryByTestId("scroll-to-bottom")).not.toBeInTheDocument();
    const main = document.querySelector("main") as HTMLElement;
    expect(main).not.toBeNull();
    // Simulate scroll position far from bottom.
    Object.defineProperty(main, "scrollHeight", { value: 1000, configurable: true });
    Object.defineProperty(main, "clientHeight", { value: 500, configurable: true });
    fireEvent.scroll(main, { target: { scrollTop: 0 } });
    expect(screen.getByTestId("scroll-to-bottom")).toBeInTheDocument();
  });

  it("clicking scroll-to-bottom scrolls the messages container to the end", async () => {
    render(<ChatView messages={makeMessages()} agentBusy={false} onSend={() => {}} />);
    const main = document.querySelector("main") as HTMLElement;
    Object.defineProperty(main, "scrollHeight", { value: 1000, configurable: true });
    Object.defineProperty(main, "clientHeight", { value: 500, configurable: true });
    const scrollSpy = vi.fn();
    main.scrollTo = scrollSpy as typeof main.scrollTo;
    fireEvent.scroll(main, { target: { scrollTop: 0 } });
    await userEvent.click(screen.getByTestId("scroll-to-bottom"));
    expect(scrollSpy).toHaveBeenCalledWith({ top: 1000, behavior: "smooth" });
  });
});
