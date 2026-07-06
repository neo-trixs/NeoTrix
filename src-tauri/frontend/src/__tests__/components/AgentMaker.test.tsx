import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { useStore } from "../../stores";
import AgentMaker from "../../components/AgentMaker";

beforeEach(() => {
  useStore.setState({
    customPresets: [],
    addCustomPreset: useStore.getState().addCustomPreset,
    removeCustomPreset: useStore.getState().removeCustomPreset,
  });
});

describe("AgentMaker", () => {
  it("renders gallery mode with title", () => {
    render(<AgentMaker />);
    expect(screen.getByText("Agent Maker")).toBeInTheDocument();
  });

  it("shows built-in template presets", () => {
    render(<AgentMaker />);
    expect(screen.getByText("Built-in Templates")).toBeInTheDocument();
    expect(screen.getByText("Personal Assistant")).toBeInTheDocument();
    expect(screen.getByText("Deep Researcher")).toBeInTheDocument();
    expect(screen.getByText("HashCoder")).toBeInTheDocument();
  });

  it("shows create buttons for each preset", () => {
    render(<AgentMaker />);
    const createButtons = screen.getAllByText("Create");
    expect(createButtons.length).toBeGreaterThan(0);
  });

  it("opens editor when a preset template is selected", () => {
    render(<AgentMaker />);
    fireEvent.click(screen.getByText("HashCoder"));
    expect(screen.getByText("Create Agent")).toBeInTheDocument();
    expect(screen.getByText("Preview")).toBeInTheDocument();
  });

  it("opens editor with + New Agent button", () => {
    render(<AgentMaker />);
    fireEvent.click(screen.getByText("+ New Agent"));
    expect(screen.getByText("Create Agent")).toBeInTheDocument();
  });

  it("renders editor mode with form fields", () => {
    render(<AgentMaker />);
    fireEvent.click(screen.getByText("+ New Agent"));
    expect(screen.getByPlaceholderText("My Agent")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("What does this agent do?")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("You are a...")).toBeInTheDocument();
  });

  it("renders tool checkboxes in editor", () => {
    render(<AgentMaker />);
    fireEvent.click(screen.getByText("HashCoder"));
    expect(screen.getByText("web search")).toBeInTheDocument();
    expect(screen.getByText("code exec")).toBeInTheDocument();
    expect(screen.getByText("git ops")).toBeInTheDocument();
  });

  it("renders knowledge source checkboxes in editor", () => {
    render(<AgentMaker />);
    fireEvent.click(screen.getByText("HashCoder"));
    expect(screen.getByText("project context")).toBeInTheDocument();
    expect(screen.getByText("system docs")).toBeInTheDocument();
  });

  it("back button returns to gallery", () => {
    render(<AgentMaker />);
    fireEvent.click(screen.getByText("HashCoder"));
    fireEvent.click(screen.getByTitle("Back to gallery"));
    expect(screen.getByText("Agent Maker")).toBeInTheDocument();
  });

  it("shows Export button in editor", () => {
    render(<AgentMaker />);
    fireEvent.click(screen.getByText("+ New Agent"));
    expect(screen.getByText("Export")).toBeInTheDocument();
  });

  it("shows Save Agent button in editor", () => {
    render(<AgentMaker />);
    fireEvent.click(screen.getByText("+ New Agent"));
    expect(screen.getByText("Save Agent")).toBeInTheDocument();
  });

  it("shows Import JSON button in editor", () => {
    render(<AgentMaker />);
    fireEvent.click(screen.getByText("+ New Agent"));
    expect(screen.getByText("Import JSON")).toBeInTheDocument();
  });

  it("shows Import button in gallery", () => {
    render(<AgentMaker />);
    expect(screen.getByText("Import")).toBeInTheDocument();
  });

  it("renders category labels for presets", () => {
    render(<AgentMaker />);
    const generalLabels = screen.getAllByText("General");
    expect(generalLabels.length).toBeGreaterThanOrEqual(1);
    const researchLabels = screen.getAllByText("Research");
    expect(researchLabels.length).toBeGreaterThanOrEqual(1);
    const devLabels = screen.getAllByText("Development");
    expect(devLabels.length).toBeGreaterThanOrEqual(1);
  });

  it("renders model tier labels", () => {
    render(<AgentMaker />);
    const qualityLabels = screen.getAllByText("High Quality");
    expect(qualityLabels.length).toBeGreaterThanOrEqual(1);
  });
});
