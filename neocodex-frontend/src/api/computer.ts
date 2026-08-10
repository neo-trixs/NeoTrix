import { call } from './client'
import type { DisplayInfo, FrontmostApp, MousePosition, ScreenCapture, WindowInfo } from './types'

/* ════════════════════════════════════════════
   api/computer.ts — 电脑操作（屏幕/鼠标/键盘/窗口）
   对应 computer_cmds.rs / computer_interactive_cmds.rs
   ════════════════════════════════════════════ */

export function screenshotAndSave(path: string): Promise<ScreenCapture> {
  return call('computer_screenshot_and_save', { path })
}

export function screenCapture(path?: string): Promise<ScreenCapture> {
  return call('computer_screen_capture', { path: path ?? null })
}

export function screenList(): Promise<DisplayInfo[]> {
  return call('computer_screen_list', {})
}

export function getWindowList(): Promise<WindowInfo[]> {
  return call('computer_get_window_list', {})
}

export function getFrontmostApp(): Promise<FrontmostApp> {
  return call('computer_get_frontmost_app', {})
}

export function mousePosition(): Promise<MousePosition> {
  return call('computer_mouse_position', {})
}

export function mouseMove(x: number, y: number): Promise<void> {
  return call('computer_mouse_move', { x, y })
}

export function mouseClick(button?: string | null): Promise<void> {
  return call('computer_mouse_click', { button: button ?? null })
}

export function keyboardType(text: string): Promise<void> {
  return call('computer_keyboard_type', { text })
}

export function keyboardPress(key: string, modifiers?: string[]): Promise<void> {
  return call('computer_keyboard_press', { key, modifiers: modifiers ?? null })
}
