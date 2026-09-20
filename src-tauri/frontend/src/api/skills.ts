/* ════════════════════════════════════════════
   api/skills.ts — 技能中心命令入口
   契约镜像 src-tauri/commands/skill_cmds.rs:
   SkillInfo { name, path, description, line_count, domain }
   skill_list / skill_get / skill_search
   ════════════════════════════════════════════ */
import { tauriInvoke } from './tauri-bridge'

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
  return tauriInvoke('skill_list')
}

export function skillGet(name: string): Promise<SkillInfo> {
  return tauriInvoke('skill_get', { name })
}

export function skillSearch(query: string): Promise<SkillInfo[]> {
  return tauriInvoke('skill_search', { query })
}
