import { describe, it, expect, vi } from "vitest";
import * as api from "../lib/api";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-deep-link", () => ({
  getCurrent: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-fs", () => ({
  writeTextFile: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

describe("api.getBrainStats", () => {
  it("should invoke brain_stats command", async () => {
    const mockStats = {
      iteration: 5,
      absorb_count: 12,
      capability_sum: 3.14,
      memory_count: 100,
      engine_active: true,
      capability_vector: [0.5, 0.3, 0.2],
      dimension_names: ["dim_0", "dim_1", "dim_2"],
    };
    vi.mocked(invoke).mockResolvedValue(mockStats);

    const result = await api.getBrainStats();
    expect(invoke).toHaveBeenCalledWith("brain_stats");
    expect(result.iteration).toBe(5);
    expect(result.absorb_count).toBe(12);
  });
});

describe("api.testProviderConnection", () => {
  it("should return true when invoke returns ok", async () => {
    vi.mocked(invoke).mockResolvedValue("ok");
    const config = {
      id: "anthropic" as const,
      name: "Anthropic",
      model: "claude-3",
      apiKey: "sk-test",
      learningRate: 0.05,
    };
    const result = await api.testProviderConnection(config);
    expect(result).toBe(true);
    expect(invoke).toHaveBeenCalledWith("test_provider", { config });
  });

  it("should return false when invoke throws", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("connection failed"));
    const config = {
      id: "openai" as const,
      name: "OpenAI",
      model: "gpt-4",
      apiKey: "",
      learningRate: 0.01,
    };
    const result = await api.testProviderConnection(config);
    expect(result).toBe(false);
  });
});

describe("api.agentReason", () => {
  it("should invoke agent_reason with prompt", async () => {
    vi.mocked(invoke).mockResolvedValue({ output: "response text", success: true });
    const result = await api.agentReason("hello");
    expect(invoke).toHaveBeenCalledWith("agent_reason", { req: { prompt: "hello" } });
    expect(result.success).toBe(true);
    expect(result.output).toBe("response text");
  });
});

describe("api.searchKnowledge", () => {
  it("should parse JSON result", async () => {
    const entries = [
      { id: "1", title: "Rust", content: "A systems language", relevance: 0.9 },
    ];
    vi.mocked(invoke).mockResolvedValue(JSON.stringify(entries));
    const result = await api.searchKnowledge("rust");
    expect(result).toHaveLength(1);
    expect(result[0].title).toBe("Rust");
  });

  it("should return empty array on error", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("parse error"));
    const result = await api.searchKnowledge("fail");
    expect(result).toEqual([]);
  });
});

describe("api.getConsciousnessMetrics", () => {
  it("should derive metrics from brain stats", async () => {
    vi.mocked(invoke).mockResolvedValue({
      iteration: 10,
      absorb_count: 20,
      capability_sum: 5.0,
      memory_count: 200,
      engine_active: true,
      capability_vector: [0.1, 0.2, 0.3],
      dimension_names: ["a", "b", "c"],
    });
    const metrics = await api.getConsciousnessMetrics();
    expect(metrics.phi).toBe(5.0);
    expect(metrics.fcs).toBe(3);
    expect(metrics.usk).toBe(200);
  });
});

describe("api.loadSessions", () => {
  it("should invoke session_list", async () => {
    vi.mocked(invoke).mockResolvedValue([{ id: "default", name: "默认会话", message_count: 0 }]);
    const sessions = await api.loadSessions();
    expect(invoke).toHaveBeenCalledWith("session_list");
    expect(sessions).toHaveLength(1);
  });
});

describe("api.getDeepLinkUrl", () => {
  it("should join URLs from getCurrent", async () => {
    const { getCurrent } = await import("@tauri-apps/plugin-deep-link");
    vi.mocked(getCurrent).mockResolvedValue(["neotrix://session/abc", "neotrix://action"]);
    const result = await api.getDeepLinkUrl();
    expect(result).toBe("neotrix://session/abc,neotrix://action");
  });

  it("should return null on error", async () => {
    const { getCurrent } = await import("@tauri-apps/plugin-deep-link");
    vi.mocked(getCurrent).mockRejectedValue(new Error("not available"));
    const result = await api.getDeepLinkUrl();
    expect(result).toBeNull();
  });
});
