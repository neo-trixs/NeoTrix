import "@testing-library/jest-dom";

Element.prototype.scrollIntoView = () => {};
Element.prototype.scrollBy = () => {};

class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
window.ResizeObserver = MockResizeObserver as any;
