import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { FileTreePanel } from "../../components/neocodex/FileTreePanel";
import * as fs from "@tauri-apps/plugin-fs";

vi.mock("@tauri-apps/plugin-fs", () => ({
  readDir: vi.fn(),
  readTextFile: vi.fn(),
  writeTextFile: vi.fn(),
}));

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(fs.readDir).mockResolvedValue([] as any);
});

describe("FileTreePanel", () => {
  it("renders empty state when no files", async () => {
    render(<FileTreePanel projectRoot="." />);
    await screen.findByText("无文件");
    expect(screen.getByText("文件")).toBeInTheDocument();
  });

  it("renders tree with directories and files", async () => {
    vi.mocked(fs.readDir).mockResolvedValue([
      { name: "src", isDirectory: true, path: "./src" },
      { name: "main.rs", isDirectory: false, path: "./main.rs" },
    ] as any);
    render(<FileTreePanel projectRoot="." />);
    await screen.findByText("src");
    expect(screen.getByText("main.rs")).toBeInTheDocument();
  });

  it("opens file preview via readTextFile", async () => {
    vi.mocked(fs.readDir).mockResolvedValue([
      { name: "main.rs", isDirectory: false, path: "./main.rs" },
    ] as any);
    vi.mocked(fs.readTextFile).mockResolvedValue("fn main() {}");
    render(<FileTreePanel projectRoot="." />);
    fireEvent.click(await screen.findByText("main.rs"));
    await screen.findByText("fn main() {}");
    expect(screen.getByText("编辑")).toBeInTheDocument();
  });

  it("edit + save writes file via writeTextFile", async () => {
    vi.mocked(fs.readDir).mockResolvedValue([
      { name: "a.txt", isDirectory: false, path: "./a.txt" },
    ] as any);
    vi.mocked(fs.readTextFile).mockResolvedValue("old");
    vi.mocked(fs.writeTextFile).mockResolvedValue(undefined as any);
    render(<FileTreePanel projectRoot="." />);
    fireEvent.click(await screen.findByText("a.txt"));
    fireEvent.click(await screen.findByText("编辑"));
    const editor = document.querySelector("textarea")!;
    fireEvent.change(editor, { target: { value: "new" } });
    fireEvent.click(screen.getByText("保存"));
    await waitFor(() => expect(fs.writeTextFile).toHaveBeenCalledWith("./a.txt", "new"));
    expect(screen.getByText("已保存")).toBeInTheDocument();
  });

  it("onPick inserts selected file path", async () => {
    let picked = "";
    vi.mocked(fs.readDir).mockResolvedValue([
      { name: "b.txt", isDirectory: false, path: "./b.txt" },
    ] as any);
    vi.mocked(fs.readTextFile).mockResolvedValue("x");
    render(<FileTreePanel projectRoot="." onPick={(p) => { picked = p; }} />);
    fireEvent.click(await screen.findByText("b.txt"));
    fireEvent.click(screen.getByText("插入到输入"));
    expect(picked).toBe("./b.txt");
  });
});
