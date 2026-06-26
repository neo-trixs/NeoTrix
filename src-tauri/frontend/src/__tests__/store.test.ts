import { describe, it, expect, beforeEach } from "vitest";
import { useStore } from "../store";

beforeEach(() => {
  useStore.setState({
    sessions: [{ id: "default", name: "默认会话", messages: [] }],
    activeSessionIndex: 0,
    statusText: "就绪",
    agentBusy: false,
    showSettings: false,
    projectPath: "",
    showFileTree: false,
    pendingPermission: null,
    providerConfig: {
      id: "anthropic",
      name: "Anthropic Claude",
      model: "claude-sonnet-4-20250514",
      apiKey: "",
      learningRate: 0.05,
    },
    knowledgeBase: [],
    settings: {
      theme: "light",
      fontSize: 13,
      autoSave: true,
      language: "zh-CN",
      terminalPath: "",
      maxSessions: 20,
    },
    streamingContent: "",
    streamingContentType: "markdown",
    showOnboarding: false,
    updateAvailable: false,
    updateStatus: "",
    notifications: [],
  });
});

describe("useStore", () => {
  it("should start with default session", () => {
    const { sessions } = useStore.getState();
    expect(sessions).toHaveLength(1);
    expect(sessions[0].name).toBe("默认会话");
  });

  it("should add a new session", () => {
    useStore.getState().addSession();
    const { sessions, activeSessionIndex } = useStore.getState();
    expect(sessions).toHaveLength(2);
    expect(activeSessionIndex).toBe(1);
  });

  it("should push a user message", () => {
    useStore.getState().pushMessage("user", "hello");
    const messages = useStore.getState().sessions[0].messages;
    expect(messages).toHaveLength(1);
    expect(messages[0].role).toBe("user");
    expect(messages[0].content).toBe("hello");
  });

  it("should push an assistant message", () => {
    useStore.getState().pushMessage("assistant", "hi there", "markdown");
    const messages = useStore.getState().sessions[0].messages;
    expect(messages).toHaveLength(1);
    expect(messages[0].contentType).toBe("markdown");
  });

  it("should set agent busy state", () => {
    useStore.getState().setAgentBusy(true);
    expect(useStore.getState().agentBusy).toBe(true);
  });

  it("should set streaming content", () => {
    useStore.getState().setStreamingContent("test", "text");
    const state = useStore.getState();
    expect(state.streamingContent).toBe("test");
    expect(state.streamingContentType).toBe("text");
  });

  it("should append streaming content", () => {
    useStore.getState().setStreamingContent("hello ");
    useStore.getState().appendStreamingContent("world");
    expect(useStore.getState().streamingContent).toBe("hello world");
  });

  it("should commit streaming content as message", () => {
    useStore.getState().setStreamingContent("committed content");
    useStore.getState().commitStreamingContent("assistant", "text");
    const messages = useStore.getState().sessions[0].messages;
    expect(messages).toHaveLength(1);
    expect(messages[0].content).toBe("committed content");
    expect(useStore.getState().streamingContent).toBe("");
  });

  it("should update provider config", () => {
    useStore.getState().setProviderConfig({
      id: "openai",
      name: "OpenAI",
      model: "gpt-4",
      apiKey: "sk-test",
      learningRate: 0.1,
    });
    const { providerConfig } = useStore.getState();
    expect(providerConfig.id).toBe("openai");
    expect(providerConfig.model).toBe("gpt-4");
  });

  it("should set status text", () => {
    useStore.getState().setStatusText("思考中...");
    expect(useStore.getState().statusText).toBe("思考中...");
  });
});
