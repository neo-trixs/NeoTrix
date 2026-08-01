import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SettingsView } from "../../components/neocodex/SettingsView";
import { useStore } from "../../stores";
import { mockInvoke, resetInvokeMocks } from "../tauriMock";

const defaultSettings = {
  theme: "system",
  fontSize: 14,
  autoSave: true,
  language: "zh-CN",
  terminalPath: "",
  maxSessions: 10,
  voiceInput: false,
  voiceLang: "zh-CN",
  voiceAutoSend: false,
  privacyStoreMessages: true,
  privacyTelemetry: false,
  privacyLocalFirst: true,
  privacyPreflightCheck: false,
  notifyOnComplete: true,
  defaultModel: "",
  temperature: 0.7,
  maxTokens: 4096,
};

beforeEach(() => {
  resetInvokeMocks();
  mockInvoke("neocodex_provider_config", () => ({
    provider_count: 2,
    resolvable: true,
    active_model: "gpt-4o",
    providers: [
      { name: "openai", model: "gpt-4o", resolvable: true },
      { name: "ollama", model: "llama3", resolvable: false },
    ],
  }));
  mockInvoke("neocodex_app_version", () => "0.18.0");
  mockInvoke("neocodex_check_update", () => ({ current: "0.18.0", available: false, latest: "0.18.0", error: null }));
  useStore.setState({ settings: defaultSettings as any });
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("SettingsView — tab clicking", () => {
  it("renders Providers tab by default", async () => {
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    expect(screen.getByText("openai")).toBeInTheDocument();
    expect(screen.getByText("已配置")).toBeInTheDocument();
  });

  it("switches to 外观 tab on click", async () => {
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    await userEvent.click(screen.getByText("外观"));
    expect(screen.getByText("主题模式")).toBeInTheDocument();
  });

  it("switches to 高级 tab on click", async () => {
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    await userEvent.click(screen.getByText("高级"));
    expect(screen.getByText("高级设置")).toBeInTheDocument();
    expect(screen.getByText("进化循环")).toBeInTheDocument();
  });

  it("switches to 关于 tab on click and shows version + update check", async () => {
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    await userEvent.click(screen.getByText("关于"));
    await waitFor(() => expect(screen.getByText(/v0\.18\.0/)).toBeInTheDocument());
    await userEvent.click(screen.getByText("检查更新"));
    await waitFor(() => expect(screen.getByText("已是最新版本。")).toBeInTheDocument());
  });
});

describe("SettingsView — theme & font size (标签点击)", () => {
  it("switches theme to 深色 via option click", async () => {
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    await userEvent.click(screen.getByText("外观"));
    await userEvent.click(screen.getByText("深色"));
    const state = useStore.getState();
    expect(state.settings.theme).toBe("dark");
  });

  it("A+ increases font size and persists to store", async () => {
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    await userEvent.click(screen.getByText("外观"));
    expect(screen.getByText("14px")).toBeInTheDocument();
    await userEvent.click(screen.getByTitle("增大"));
    expect(screen.getByText("15px")).toBeInTheDocument();
    expect(useStore.getState().settings.fontSize).toBe(15);
  });

  it("A- decreases font size with floor at 11", async () => {
    useStore.setState({ settings: { ...defaultSettings, fontSize: 11 } as any });
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    await userEvent.click(screen.getByText("外观"));
    await userEvent.click(screen.getByTitle("减小"));
    expect(screen.getByText("11px")).toBeInTheDocument();
    expect(useStore.getState().settings.fontSize).toBe(11);
  });
});

describe("SettingsView — advanced toggles (标签点击)", () => {
  it("toggles a checkbox and persists via onChange", async () => {
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    await userEvent.click(screen.getByText("高级"));
    const autoFix = screen.getByText("自动修复");
    const cb = autoFix.closest("label")!.querySelector("input") as HTMLInputElement;
    expect(cb.checked).toBe(false);
    await userEvent.click(cb);
    expect(useStore.getState().settings.privacyPreflightCheck).toBe(true);
  });

  it("renders grouped advanced cards", async () => {
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    await userEvent.click(screen.getByText("高级"));
    expect(screen.getByText("上下文管理")).toBeInTheDocument();
    expect(screen.getByText("开发者")).toBeInTheDocument();
    expect(screen.getByText("保留完整历史")).toBeInTheDocument();
  });

  it("shows 通知 group with the completion-notification toggle checked by default", async () => {
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    await userEvent.click(screen.getByText("高级"));
    expect(screen.getByText("通知")).toBeInTheDocument();
    const label = screen.getByText("任务完成时发送系统通知").closest("label")!;
    const cb = label.querySelector("input") as HTMLInputElement;
    expect(cb.checked).toBe(true);
    await userEvent.click(cb);
    expect(useStore.getState().settings.notifyOnComplete).toBe(false);
  });

  it("surfaces model/context numeric fields on 高级 and persists them", async () => {
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    await userEvent.click(screen.getByText("高级"));
    expect(screen.getByTestId("settings-defaultModel")).toBeInTheDocument();
    const temp = screen.getByTestId("settings-temperature") as HTMLInputElement;
    await userEvent.clear(temp);
    await userEvent.type(temp, "0.5");
    expect(useStore.getState().settings.temperature).toBe(0.5);
    const tokens = screen.getByTestId("settings-maxTokens") as HTMLInputElement;
    await userEvent.clear(tokens);
    await userEvent.type(tokens, "16384");
    expect(useStore.getState().settings.maxTokens).toBe(16384);
    const sessions = screen.getByTestId("settings-maxSessions") as HTMLInputElement;
    await userEvent.clear(sessions);
    await userEvent.type(sessions, "30");
    expect(useStore.getState().settings.maxSessions).toBe(30);
  });

  it("language selector on 外观 persists to store", async () => {
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    await userEvent.click(screen.getByText("外观"));
    await userEvent.selectOptions(screen.getByTestId("settings-language"), "en-US");
    expect(useStore.getState().settings.language).toBe("en-US");
  });
});

describe("SettingsView — About download", () => {
  it("renders 立即下载并重启 when an update is available and triggers neocodex_download_update", async () => {
    mockInvoke("neocodex_check_update", () => ({ current: "0.18.0", available: true, latest: "0.19.0", error: null }));
    const downloadSpy = vi.fn(() => null);
    mockInvoke("neocodex_download_update", downloadSpy);
    render(<SettingsView />);
    await waitFor(() => expect(screen.getByText("API Providers")).toBeInTheDocument());
    await userEvent.click(screen.getByText("关于"));
    await userEvent.click(screen.getByText("检查更新"));
    await waitFor(() => expect(screen.getByText("立即下载并重启")).toBeInTheDocument());
    await userEvent.click(screen.getByText("立即下载并重启"));
    await waitFor(() => expect(downloadSpy).toHaveBeenCalled());
  });
});
