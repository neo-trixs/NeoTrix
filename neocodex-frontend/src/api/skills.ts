/* ════════════════════════════════════════════
   api/skills.ts — 技能中心命令入口
   契约镜像 src-tauri/commands/skill_cmds.rs:
   SkillInfo { name, path, description, line_count, domain }
   skill_list / skill_get / skill_search
   ════════════════════════════════════════════ */
import { call } from './client'

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
  return call('skill_list')
}

export function skillGet(name: string): Promise<SkillInfo> {
  return call('skill_get', { name })
}

export function skillSearch(query: string): Promise<SkillInfo[]> {
  return call('skill_search', { query })
}
