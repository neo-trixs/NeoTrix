/**
 * Hive API — OfficeFloor visualization commands
 *
 * Routes through domain_call since direct IPC commands are not registered.
 */
import { call } from './domain'

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
  return call<FloorState>('hive', 'get_floor_state')
}

export function hiveSendMessage(from: string, to: string, msgType: string, content: string): Promise<string> {
  return call<string>('hive', 'send_message', { from, to, msg_type: msgType, content })
}
