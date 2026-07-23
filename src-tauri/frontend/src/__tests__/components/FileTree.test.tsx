import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import FileTree from "../../components/FileTree";
import type { FileNode } from "../../types";

const mockReadDirRecursive = vi.fn();
const mockReadFile = vi.fn();

vi.mock("../../lib/api", () => ({
  readDirRecursive: (...args: any[]) => mockReadDirRecursive(...args),
  readFile: (...args: any[]) => mockReadFile(...args),
}));

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: vi.fn(),
}));

const fileNodes: FileNode[] = [
  {
    name: "src",
    path: "/project/src",
    is_dir: true,
    size: undefined,
    depth: 0,
  },
  {
    name: "main.rs",
    path: "/project/src/main.rs",
    is_dir: false,
    size: 512,
    depth: 1,
  },
  {
    name: "lib.rs",
    path: "/project/src/lib.rs",
    is_dir: false,
    size: 128,
    depth: 1,
  },
  {
    name: "Cargo.toml",
    path: "/project/Cargo.toml",
    is_dir: false,
    size: 256,
    depth: 0,
  },
];

const defaultProps = {
  rootPath: "/project",
  onClose: vi.fn(),
  onStatusChange: vi.fn(),
};

describe("FileTree", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows loading state initially", () => {
    mockReadDirRecursive.mockReturnValue(new Promise(() => {}));
    render(<FileTree {...defaultProps} />);
    expect(screen.getByText("加载中...")).toBeInTheDocument();
  });

  it("renders top-level files after loading", async () => {
    mockReadDirRecursive.mockResolvedValue(fileNodes);
    render(<FileTree {...defaultProps} />);
    expect(await screen.findByText("Cargo.toml")).toBeInTheDocument();
    expect(await screen.findByText("src")).toBeInTheDocument();
  });

  it("shows file size for top-level files", async () => {
    mockReadDirRecursive.mockResolvedValue(fileNodes);
    render(<FileTree {...defaultProps} />);
    expect(await screen.findByText("256 B")).toBeInTheDocument();
  });

  it("expands directory on click and shows children", async () => {
    mockReadDirRecursive.mockResolvedValue(fileNodes);
    render(<FileTree {...defaultProps} />);
    const dirNode = await screen.findByText("src");
    fireEvent.click(dirNode);
    expect(await screen.findByText("main.rs")).toBeInTheDocument();
    expect(await screen.findByText("lib.rs")).toBeInTheDocument();
  });

  it("shows file sizes inside expanded directories", async () => {
    mockReadDirRecursive.mockResolvedValue(fileNodes);
    render(<FileTree {...defaultProps} />);
    const dirNode = await screen.findByText("src");
    fireEvent.click(dirNode);
    expect(await screen.findByText("512 B")).toBeInTheDocument();
    expect(await screen.findByText("128 B")).toBeInTheDocument();
  });

  it("shows empty state when no files", async () => {
    mockReadDirRecursive.mockResolvedValue([]);
    render(<FileTree {...defaultProps} />);
    expect(await screen.findByText("空目录")).toBeInTheDocument();
  });

  it("shows error status on load failure", async () => {
    const onStatusChange = vi.fn();
    mockReadDirRecursive.mockRejectedValue(new Error("permission denied"));
    render(<FileTree {...defaultProps} onStatusChange={onStatusChange} />);
    await waitFor(() => {
      expect(onStatusChange).toHaveBeenCalledWith(expect.stringContaining("permission denied"));
    });
  });
});
