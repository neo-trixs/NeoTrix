/**
 * cn — 统一的 className 合并工具
 *
 * 组合 clsx（条件拼接）+ tailwind-merge（去重合并）
 * 用法：cn('base', condition && 'conditional', overrides)
 */
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs))
}
