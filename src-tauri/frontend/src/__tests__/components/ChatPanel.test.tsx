import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import ChatPanel from "../../components/ChatPanel";
import type { Message } from "../../types";

describe("ChatPanel", () => {
  it("shows empty state when no messages", () => {
    render(<ChatPanel messages={[]} agentBusy={false} />);
    expect(screen.getByText("输入消息开始对话")).toBeInTheDocument();
  });

  it("does not show empty state when there are messages", () => {
    const messages: Message[] = [
      { role: "user", content: "hello", contentType: "text" },
    ];
    render(<ChatPanel messages={messages} agentBusy={false} />);
    expect(screen.queryByText("输入消息开始对话")).not.toBeInTheDocument();
  });

  it("renders user message", () => {
    const messages: Message[] = [
      { role: "user", content: "用户消息", contentType: "text" },
    ];
    render(<ChatPanel messages={messages} agentBusy={false} />);
    expect(screen.getByText("用户消息")).toBeInTheDocument();
  });

  it("renders assistant message", () => {
    const messages: Message[] = [
      { role: "assistant", content: "助手回复", contentType: "text" },
    ];
    render(<ChatPanel messages={messages} agentBusy={false} />);
    expect(screen.getByText("助手回复")).toBeInTheDocument();
  });

  it("renders system message", () => {
    const messages: Message[] = [
      { role: "system", content: "系统消息", contentType: "text" },
    ];
    render(<ChatPanel messages={messages} agentBusy={false} />);
    expect(screen.getByText("系统消息")).toBeInTheDocument();
  });

  it("renders error message", () => {
    const messages: Message[] = [
      { role: "error", content: "错误消息", contentType: "text" },
    ];
    render(<ChatPanel messages={messages} agentBusy={false} />);
    expect(screen.getByText("错误消息")).toBeInTheDocument();
  });

  it("shows streaming content when present", () => {
    render(
      <ChatPanel
        messages={[]}
        agentBusy={false}
        streamingContent="正在生成..."
        streamingContentType="text"
      />
    );
    expect(screen.getByText("正在生成...")).toBeInTheDocument();
    expect(screen.getByText("▊")).toBeInTheDocument();
  });

  it("shows thinking header when agent is busy", () => {
    render(<ChatPanel messages={[]} agentBusy={true} />);
    expect(screen.getByText("思考中...")).toBeInTheDocument();
  });

  it("shows 对话 header when agent is idle", () => {
    render(<ChatPanel messages={[]} agentBusy={false} />);
    expect(screen.getByText("对话")).toBeInTheDocument();
  });
});
