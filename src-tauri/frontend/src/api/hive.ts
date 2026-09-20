/**
 * Hive API — OfficeFloor visualization commands
 */
import { invoke } from '@tauri-apps/api/core'

export interface AgentFloorData {
  id: string
  name: string
  avatar: string
  status: string
  specialty: string
  x: number
  y: number
  inbox_depth: number
  last_output: string | null
}

export interface AgentFloorMessage {
  from: string
  to: string
  msg_type: string
  timestamp: number
}

export interface HiveStatsData {
  total_messages: number
  total_agents: number
  blackboard_entries: number
  events_count: number
}

export interface FloorState {
  agents: AgentFloorData[]
  messages: AgentFloorMessage[]
  stats: HiveStatsData
}

export function getFloorState(): Promise<FloorState> {
  return invoke('hive_get_floor_state')
}

export function hiveSendMessage(from: string, to: string, msgType: string, content: string): Promise<string> {
  return invoke('hive_send_message', { from, to, msgType, content })
}
