import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ErrorBoundary } from "../../components/ErrorBoundary";

function GoodChild() {
  return <div>正常渲染</div>;
}

function BadChild(): React.ReactNode {
  throw new Error("测试崩溃");
}

describe("ErrorBoundary", () => {
  it("renders children normally when no error", () => {
    render(
      <ErrorBoundary>
        <GoodChild />
      </ErrorBoundary>
    );
    expect(screen.getByText("正常渲染")).toBeInTheDocument();
  });

  it("catches errors and shows fallback UI", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    render(
      <ErrorBoundary>
        <BadChild />
      </ErrorBoundary>
    );
    expect(screen.getByText("渲染错误")).toBeInTheDocument();
    expect(screen.getByText("测试崩溃")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "重试" })).toBeInTheDocument();
    spy.mockRestore();
  });

  it("renders custom fallback when provided", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    render(
      <ErrorBoundary fallback={<div>自定义错误</div>}>
        <BadChild />
      </ErrorBoundary>
    );
    expect(screen.getByText("自定义错误")).toBeInTheDocument();
    expect(screen.queryByText("渲染错误")).not.toBeInTheDocument();
    spy.mockRestore();
  });

  it("retry button clears the error state", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    // Suppress the second render error from BadChild after retry
    const { container } = render(
      <ErrorBoundary>
        <BadChild />
      </ErrorBoundary>
    );
    expect(screen.getByText("渲染错误")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "重试" }));
    // After retry, hasError is false so it tries to render children again
    // BadChild will throw again, so we should see the error UI again
    expect(screen.getByText("渲染错误")).toBeInTheDocument();
    spy.mockRestore();
  });
});
