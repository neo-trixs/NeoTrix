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

  it("opens the URL in iframe mode and shows the title on success", async () => {
    const openSpy = vi.fn(() => ({ title: "Dev Server", url: "http://localhost:5173" }));
    const previewStartSpy = vi.fn(() => "pv-test-123");
    mockInvoke("browser_open", openSpy);
    mockInvoke("preview_start", previewStartSpy);
    mockInvoke("preview_list", () => []);
    mockInvoke("chrome_debug_get_console_logs", () => []);
    render(<PreviewPane />);
    fireEvent.click(screen.getByTestId("preview-open"));
    await waitFor(() => expect(screen.getByText(/已打开: Dev Server/)).toBeInTheDocument());
    expect(openSpy).toHaveBeenCalledWith({ url: "http://localhost:5173" });
    expect(previewStartSpy).toHaveBeenCalled();
  });

  it("opens the URL in external mode", async () => {
    const openSpy = vi.fn(() => ({ title: "External", url: "http://localhost:5173" }));
    mockInvoke("browser_open", openSpy);
    mockInvoke("preview_list", () => []);
    mockInvoke("chrome_debug_get_console_logs", () => []);
    render(<PreviewPane />);
    fireEvent.change(screen.getByTestId("preview-mode"), { target: { value: "external" } });
    fireEvent.click(screen.getByTestId("preview-open"));
    await waitFor(() => expect(screen.getByText(/已打开: External/)).toBeInTheDocument());
    expect(openSpy).toHaveBeenCalledWith({ url: "http://localhost:5173" });
  });

  it("shows an error state when the backend rejects", async () => {
    mockInvoke("browser_open", () => {
      throw new Error("browser unavailable");
    });
    mockInvoke("preview_list", () => []);
    mockInvoke("chrome_debug_get_console_logs", () => []);
    render(<PreviewPane />);
    fireEvent.click(screen.getByTestId("preview-open"));
    await waitFor(() => expect(screen.getByText(/打开失败: Error: browser unavailable/)).toBeInTheDocument());
    expect(screen.queryByText(/已打开:/)).not.toBeInTheDocument();
  });

  it("survives a null backend response without crashing", async () => {
    mockInvoke("browser_open", () => null);
    mockInvoke("preview_list", () => []);
    mockInvoke("chrome_debug_get_console_logs", () => []);
    render(<PreviewPane />);
    fireEvent.click(screen.getByTestId("preview-open"));
    await waitFor(() => expect(screen.getByText(/输入 dev server 地址/)).toBeInTheDocument());
  });

  it("opens on Enter in the URL field", async () => {
    const openSpy = vi.fn(() => ({ title: "Page", url: "http://localhost:8000" }));
    const previewStartSpy = vi.fn(() => "pv-test-456");
    mockInvoke("browser_open", openSpy);
    mockInvoke("preview_start", previewStartSpy);
    mockInvoke("preview_list", () => []);
    mockInvoke("chrome_debug_get_console_logs", () => []);
    render(<PreviewPane />);
    fireEvent.change(screen.getByTestId("preview-url"), { target: { value: "http://localhost:8000" } });
    fireEvent.keyDown(screen.getByTestId("preview-url"), { key: "Enter" });
    await waitFor(() => expect(openSpy).toHaveBeenCalledWith({ url: "http://localhost:8000" }));
  });

  it("renders device toolbar in iframe mode", async () => {
    mockInvoke("browser_open", () => ({ title: "Test", url: "http://localhost:5173" }));
    mockInvoke("preview_start", () => "pv-1");
    mockInvoke("preview_list", () => []);
    mockInvoke("chrome_debug_get_console_logs", () => []);
    render(<PreviewPane />);
    fireEvent.click(screen.getByTestId("preview-open"));
    await waitFor(() => {
      expect(screen.getByTestId("preview-device-desktop")).toBeInTheDocument();
      expect(screen.getByTestId("preview-device-tablet")).toBeInTheDocument();
      expect(screen.getByTestId("preview-device-mobile")).toBeInTheDocument();
    });
  });

  it("has reload button in iframe mode", async () => {
    const reloadSpy = vi.fn();
    mockInvoke("browser_open", () => ({ title: "Test", url: "http://localhost:5173" }));
    mockInvoke("preview_start", () => "pv-1");
    mockInvoke("preview_list", () => [{ id: "pv-1", url: "http://localhost:5173", title: "Test", width: 1280, height: 720, status: "ready", started_at: Date.now() }]);
    mockInvoke("chrome_debug_get_console_logs", () => []);
    mockInvoke("preview_reload", reloadSpy);
    render(<PreviewPane />);
    fireEvent.click(screen.getByTestId("preview-open"));
    await waitFor(() => expect(screen.getByTestId("preview-reload")).toBeInTheDocument());
    fireEvent.click(screen.getByTestId("preview-reload"));
    expect(reloadSpy).toHaveBeenCalledWith({ session_id: "pv-1" });
  });

  it("has open in external browser button", async () => {
    const openSpy = vi.fn(() => ({ title: "Test", url: "http://localhost:5173" }));
    mockInvoke("browser_open", openSpy);
    mockInvoke("preview_list", () => []);
    mockInvoke("chrome_debug_get_console_logs", () => []);
    render(<PreviewPane />);
    fireEvent.click(screen.getByTestId("preview-external"));
    expect(openSpy).toHaveBeenCalledWith({ url: "http://localhost:5173" });
  });

  it("toggles console panel", async () => {
    mockInvoke("browser_open", () => ({ title: "Test", url: "http://localhost:5173" }));
    mockInvoke("preview_start", () => "pv-1");
    mockInvoke("preview_list", () => []);
    mockInvoke("chrome_debug_get_console_logs", () => [{ level: "info", message: "test log", timestamp: Date.now() / 1000, source: "console" }]);
    render(<PreviewPane />);
    fireEvent.click(screen.getByTestId("preview-open"));
    await waitFor(() => expect(screen.getByTestId("preview-console-toggle")).toBeInTheDocument());
    fireEvent.click(screen.getByTestId("preview-console-toggle"));
    await waitFor(() => expect(screen.getByTestId("preview-console")).toBeInTheDocument());
    expect(screen.getByText("test log")).toBeInTheDocument();
  });

  it("clears console logs", async () => {
    const clearSpy = vi.fn();
    mockInvoke("browser_open", () => ({ title: "Test", url: "http://localhost:5173" }));
    mockInvoke("preview_start", () => "pv-1");
    mockInvoke("preview_list", () => []);
    mockInvoke("chrome_debug_get_console_logs", () => [{ level: "info", message: "test log", timestamp: Date.now() / 1000, source: "console" }]);
    mockInvoke("chrome_debug_clear_console_logs", clearSpy);
    render(<PreviewPane />);
    fireEvent.click(screen.getByTestId("preview-open"));
    await waitFor(() => expect(screen.getByTestId("preview-console-toggle")).toBeInTheDocument());
    fireEvent.click(screen.getByTestId("preview-console-toggle"));
    await waitFor(() => expect(screen.getByTestId("preview-console-clear")).toBeInTheDocument());
    fireEvent.click(screen.getByTestId("preview-console-clear"));
    expect(clearSpy).toHaveBeenCalled();
  });
});