import "@testing-library/jest-dom/vitest";

Object.defineProperty(globalThis, "localStorage", {
  value: (() => {
    const store: Record<string, string> = {};
    return {
      getItem: (key: string) => store[key] ?? null,
      setItem: (key: string, value: string) => { store[key] = value; },
      removeItem: (key: string) => { delete store[key]; },
      clear: () => { Object.keys(store).forEach(k => delete store[k]); },
      length: 0,
      key: (_: number) => null,
    };
  })(),
  writable: true,
  configurable: true,
});

Element.prototype.scrollIntoView = () => {};
Element.prototype.scrollBy = () => {};
