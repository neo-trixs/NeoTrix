/* ════════════════════════════════════════════
   api/skills.ts — 技能中心命令入口
   Routes through domain_call (no direct IPC registered).
   ════════════════════════════════════════════ */
import { call } from './domain'

export interface SkillInfo {
  name: string
  path: string
  description: string
  line_count: number
  domain: string
}

export interface SkillListResult {
  skills: SkillInfo[]
  total: number
}

export function skillList(): Promise<SkillListResult> {
  return call<SkillListResult>('skill', 'list')
}

export function skillGet(name: string): Promise<SkillInfo> {
  return call<SkillInfo>('skill', 'get', { name })
}

export function skillSearch(query: string): Promise<SkillInfo[]> {
  return call<SkillInfo[]>('skill', 'search', { query })
}
