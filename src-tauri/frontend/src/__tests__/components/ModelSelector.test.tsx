import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ModelSelector } from "../../components/neocodex/ModelSelector";
import { mockInvoke, resetInvokeMocks } from "../tauriMock";

const providerConfig = {
  provider_count: 2,
  resolvable: true,
  active_model: "claude-sonnet",
  providers: [
    { name: "anthropic", model: "claude-sonnet", resolvable: true },
    { name: "openai", model: "gpt-4o", resolvable: false },
  ],
};

beforeEach(() => {
  resetInvokeMocks();
  vi.restoreAllMocks();
});

describe("ModelSelector", () => {
  it("renders active model after config loads", async () => {
    mockInvoke("neocodex_provider_config", () => providerConfig);
    render(<ModelSelector />);
    await screen.findByText("claude-sonnet");
    expect(screen.getByText("可用")).toBeInTheDocument();
  });

  it("shows skeleton while loading", () => {
    mockInvoke("neocodex_provider_config", () => new Promise(() => {}));
    const { container } = render(<ModelSelector />);
    expect(container.querySelector("[class*='skeleton']")).toBeInTheDocument();
  });

  it("renders null when config is null", async () => {
    mockInvoke("neocodex_provider_config", () => {
      throw new Error("boom");
    });
    const { container } = render(<ModelSelector />);
    await waitFor(() => expect(container.innerHTML).toBe(""));
  });

  it("expands dropdown listing providers", async () => {
    mockInvoke("neocodex_provider_config", () => providerConfig);
    render(<ModelSelector />);
    fireEvent.click(await screen.findByText("claude-sonnet"));
    expect(screen.getByText("anthropic")).toBeInTheDocument();
    expect(screen.getByText("openai")).toBeInTheDocument();
    expect(screen.getByText("可用 Providers (2)")).toBeInTheDocument();
  });

  it("switches provider via neocodex_set_provider", async () => {
    const setSpy = vi.fn(() => null);
    mockInvoke("neocodex_provider_config", () => providerConfig);
    mockInvoke("neocodex_set_provider", setSpy);
    render(<ModelSelector />);
    fireEvent.click(await screen.findByText("claude-sonnet"));
    fireEvent.click(screen.getByText("openai"));
    await waitFor(() => expect(setSpy).toHaveBeenCalledWith({ name: "openai" }));
  });

  it("refresh button reloads config", async () => {
    const configSpy = vi.fn(() => providerConfig);
    mockInvoke("neocodex_provider_config", configSpy);
    render(<ModelSelector />);
    fireEvent.click(await screen.findByText("claude-sonnet"));
    fireEvent.click(screen.getByText("刷新 Provider"));
    await waitFor(() => expect(configSpy.mock.calls.length).toBeGreaterThanOrEqual(2));
  });
});
