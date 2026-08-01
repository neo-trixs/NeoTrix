import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { PreviewPane } from "../../components/neocodex/PreviewPane";
import { mockInvoke, resetInvokeMocks } from "../../__tests__/tauriMock";

describe("PreviewPane", () => {
  beforeEach(() => {
    resetInvokeMocks();
  });

  it("renders URL input and open button with default hint", async () => {
    render(<PreviewPane />);
    expect(screen.getByTestId("preview-pane")).toBeInTheDocument();
    expect(screen.getByTestId("preview-url")).toHaveValue("http://localhost:5173");
    expect(screen.getByText(/输入 dev server 地址/)).toBeInTheDocument();
  });

  it("opens the URL and shows the title on success", async () => {
    const openSpy = vi.fn(() => ({ title: "Dev Server", url: "http://localhost:5173" }));
    mockInvoke("browser_open", openSpy);
    render(<PreviewPane />);
    fireEvent.click(screen.getByTestId("preview-open"));
    await waitFor(() => expect(screen.getByText(/已打开: Dev Server/)).toBeInTheDocument());
    expect(openSpy).toHaveBeenCalledWith({ url: "http://localhost:5173" });
  });

  it("shows an error state instead of success when the backend rejects", async () => {
    mockInvoke("browser_open", () => {
      throw new Error("browser unavailable");
    });
    render(<PreviewPane />);
    fireEvent.click(screen.getByTestId("preview-open"));
    await waitFor(() => expect(screen.getByText(/打开失败: Error: browser unavailable/)).toBeInTheDocument());
    expect(screen.queryByText(/已打开:/)).not.toBeInTheDocument();
  });

  it("survives a null backend response without crashing", async () => {
    mockInvoke("browser_open", () => null);
    render(<PreviewPane />);
    fireEvent.click(screen.getByTestId("preview-open"));
    await waitFor(() => expect(screen.getByText(/输入 dev server 地址/)).toBeInTheDocument());
  });

  it("opens on Enter in the URL field", async () => {
    const openSpy = vi.fn(() => ({ title: "Page", url: "http://localhost:5173" }));
    mockInvoke("browser_open", openSpy);
    render(<PreviewPane />);
    fireEvent.change(screen.getByTestId("preview-url"), { target: { value: "http://localhost:8000" } });
    fireEvent.keyDown(screen.getByTestId("preview-url"), { key: "Enter" });
    await waitFor(() => expect(openSpy).toHaveBeenCalledWith({ url: "http://localhost:8000" }));
  });
});
