import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { useStore } from "../../stores";
import NotificationToast from "../../components/NotificationToast";

beforeEach(() => {
  useStore.setState({
    notifications: [],
    addNotification: useStore.getState().addNotification,
    removeNotification: useStore.getState().removeNotification,
  });
});

describe("NotificationToast", () => {
  it("renders nothing when no notifications", () => {
    const { container } = render(<NotificationToast />);
    expect(container.innerHTML).toBe("");
  });

  it("renders notification items", () => {
    useStore.setState({
      notifications: [
        { id: "n1", type: "info", message: "Hello info" },
        { id: "n2", type: "success", message: "Success msg" },
      ],
    });
    render(<NotificationToast />);
    expect(screen.getByText("Hello info")).toBeInTheDocument();
    expect(screen.getByText("Success msg")).toBeInTheDocument();
  });

  it("renders error notification", () => {
    useStore.setState({
      notifications: [{ id: "n1", type: "error", message: "Error occurred" }],
    });
    const { container } = render(<NotificationToast />);
    expect(screen.getByText("Error occurred")).toBeInTheDocument();
    const icons = container.querySelectorAll("[class*='icon']");
    expect(icons.length).toBe(1);
  });

  it("renders warning notification", () => {
    useStore.setState({
      notifications: [{ id: "n1", type: "warning", message: "Warning message" }],
    });
    render(<NotificationToast />);
    expect(screen.getByText("Warning message")).toBeInTheDocument();
    expect(screen.getByText("⚠")).toBeInTheDocument();
  });

  it("renders success notification with checkmark", () => {
    useStore.setState({
      notifications: [{ id: "n1", type: "success", message: "All good" }],
    });
    render(<NotificationToast />);
    expect(screen.getByText("All good")).toBeInTheDocument();
    expect(screen.getByText("✓")).toBeInTheDocument();
  });

  it("renders info notification with info icon", () => {
    useStore.setState({
      notifications: [{ id: "n1", type: "info", message: "Info here" }],
    });
    render(<NotificationToast />);
    expect(screen.getByText("Info here")).toBeInTheDocument();
    expect(screen.getByText("ℹ")).toBeInTheDocument();
  });

  it("has role alert on each notification", () => {
    useStore.setState({
      notifications: [{ id: "n1", type: "info", message: "Test alert" }],
    });
    render(<NotificationToast />);
    expect(screen.getByRole("alert")).toBeInTheDocument();
  });

  it("has dismiss button on each notification", () => {
    useStore.setState({
      notifications: [{ id: "n1", type: "info", message: "Dismiss me" }],
    });
    render(<NotificationToast />);
    expect(screen.getByLabelText("Dismiss")).toBeInTheDocument();
  });

  it("calls removeNotification on dismiss click", () => {
    useStore.setState({
      notifications: [{ id: "n1", type: "info", message: "Dismiss me" }],
    });
    render(<NotificationToast />);
    fireEvent.click(screen.getByLabelText("Dismiss"));
    expect(useStore.getState().notifications).toHaveLength(0);
  });

  it("has aria-live polite region", () => {
    useStore.setState({
      notifications: [{ id: "n1", type: "info", message: "Live" }],
    });
    const { container } = render(<NotificationToast />);
    expect(container.querySelector("[aria-live='polite']")).toBeInTheDocument();
  });

  it("renders an action button and fires onClick then dismisses", () => {
    let clicked = false;
    useStore.setState({
      notifications: [
        { id: "n1", type: "info", message: "发现新版本", action: { label: "立即更新", onClick: () => { clicked = true; } } },
      ],
    });
    render(<NotificationToast />);
    fireEvent.click(screen.getByText("立即更新"));
    expect(clicked).toBe(true);
    expect(useStore.getState().notifications).toHaveLength(0);
  });
});
