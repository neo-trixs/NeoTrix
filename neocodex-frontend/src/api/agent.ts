/**
 * Agent API — 意识核心与模型管理
 * 所有调用走 domain.ts typed helpers
 */

import * as domain from './domain'

export async function getAgentStatus() {
  return domain.agent.status()
}

export async function startAgent(task: string) {
  return domain.agent.start(task)
}

export async function stopAgent() {
  return domain.agent.stop()
}

export async function setProvider(name: string) {
  return domain.agent.setProvider(name)
}

export async function testProvider(name: string) {
  return domain.agent.testProvider(name)
}

export async function fetchModels(baseUrl: string, apiKey: string) {
  return domain.agent.fetchModels(baseUrl, apiKey)
}

export async function getProviderConfig() {
  return domain.agent.providerConfig()
}

export async function addCustomProvider(config: Record<string, unknown>) {
  return domain.agent.addCustomProvider(config)
}

export async function getAppVersion() {
  return domain.agent.appVersion()
}

export async function getAgentHealth() {
  return domain.agent.health()
}
