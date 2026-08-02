import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { CapabilityHealthPane } from "../../components/neocodex/CapabilityHealthPane";

const healthyData = {
  consciousness_attached: true,
  brain_attached: true,
  event_bus_attached: true,
  evolution_iterations: 3,
  tool_grounding_degraded: false,
  subagent_results: 5,
  context_usage: 0.42,
  provider_resolvable: true,
  goals_active: true,
  tool_call_count: 12,
  turn_count: 8,
  session_writable: true,
};

describe("CapabilityHealthPane", () => {
  it("shows the health badge and core chain status", () => {
    render(<CapabilityHealthPane data={healthyData} />);
    expect(screen.getByTestId("capability-health")).toHaveAttribute("data-loading", "false");
    expect(screen.getByText("能力网健康")).toBeInTheDocument();
    expect(screen.getByText("核心链路通畅")).toBeInTheDocument();
    expect(screen.getByText("系统已自我进化 3 次，能力网持续自愈。")).toBeInTheDocument();
  });

  it("renders all seven domain nodes", () => {
    render(<CapabilityHealthPane data={healthyData} />);
    const domains = ["NT-CORE", "NT-MIND", "NT-MEMORY", "NT-WORLD", "NT-ACT", "NT-SHIELD", "NT-IO"];
    for (const d of domains) {
      expect(document.querySelector(`[data-domain="${d}"]`)).not.toBeNull();
    }
  });

  it("flags a broken chain when consciousness attachments are missing", () => {
    const degraded = { ...healthyData, consciousness_attached: false, brain_attached: false };
    render(<CapabilityHealthPane data={degraded} />);
    expect(screen.getAllByText("存在断链").length).toBeGreaterThan(0);
    const core = document.querySelector('[data-domain="NT-CORE"]');
    expect(core?.getAttribute("data-ok")).toBe("false");
  });

  it("shows the waiting state when no data is provided", () => {
    render(<CapabilityHealthPane data={null} />);
    expect(screen.getByTestId("capability-health")).toHaveAttribute("data-loading", "true");
    expect(screen.getByText("正在等待健康报告…")).toBeInTheDocument();
  });
});
