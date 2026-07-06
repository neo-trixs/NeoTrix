import type { DesktopWindow } from "../types";

export interface DesktopSlice {
  desktopWindows: DesktopWindow[];

  openWindow: (win: DesktopWindow) => void;
  closeWindow: (id: string) => void;
  focusWindow: (id: string) => void;
  moveWindow: (id: string, x: number, y: number) => void;
  toggleMinimizeWindow: (id: string) => void;
}

export const createDesktopSlice = (set: any) => ({
  desktopWindows: [],

  openWindow: (win: DesktopWindow) => set((state: any) => ({ desktopWindows: [...state.desktopWindows, win] })),

  closeWindow: (id: string) => set((state: any) => ({ desktopWindows: state.desktopWindows.filter((w: DesktopWindow) => w.id !== id) })),

  focusWindow: (id: string) => set((state: any) => {
    const maxZ = Math.max(...state.desktopWindows.map((w: DesktopWindow) => w.zIndex), 0);
    return { desktopWindows: state.desktopWindows.map((w: DesktopWindow) => w.id === id ? { ...w, zIndex: maxZ + 1 } : w) };
  }),

  moveWindow: (id: string, x: number, y: number) => set((state: any) => ({
    desktopWindows: state.desktopWindows.map((w: DesktopWindow) => w.id === id ? { ...w, x, y } : w),
  })),

  toggleMinimizeWindow: (id: string) => set((state: any) => ({
    desktopWindows: state.desktopWindows.map((w: DesktopWindow) => w.id === id ? { ...w, minimized: !w.minimized } : w),
  })),
});
