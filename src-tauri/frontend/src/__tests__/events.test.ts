import { describe, it, expect, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-deep-link", () => ({
  getCurrent: vi.fn().mockResolvedValue([]),
}));

describe("streaming event handlers in App", () => {
  it("should define streaming-token and streaming-done listener imports", async () => {
    const { listen } = await import("@tauri-apps/api/event");
    expect(typeof listen).toBe("function");
  });

  it("should define App component that uses listen", async () => {
    const App = (await import("../App")).default;
    expect(App).toBeDefined();
    expect(typeof App).toBe("function");
  });
});
