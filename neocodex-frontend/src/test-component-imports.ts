/**
 * Component Import Test — 组件导入测试
 * 
 * 验证所有设置标签页组件的导入和导出是否正确
 */

// 测试所有设置标签页组件导入
import { GeneralSection } from './components/settings/GeneralSection'
import { ModelsSection } from './components/settings/ModelsSection'
import { NetworkSection } from './components/settings/NetworkSection'
import { ImSection } from './components/settings/ImSection'
import { AppearanceSection } from './components/settings/AppearanceSection'
import { DataSection } from './components/settings/DataSection'
import { TagsSection } from './components/settings/TagsSection'
import { AboutSection } from './components/settings/AboutSection'
import { CapabilitiesSection } from './components/settings/CapabilitiesSection'
import { SettingsHealthPanel } from './components/settings/SettingsHealthPanel'
import { ProviderHealthPanel } from './components/settings/ProviderHealthPanel'

// 测试 API 模块导入
import { domain } from './api'
import * as modelPool from './api/model-pool'
import * as proxyPool from './api/proxy-pool'
import * as im from './api/im'
import * as market from './api/market'

// 测试工具模块导入
import { checkAllApis } from './lib/apiHealthCheck'

// 测试画板相关导入
import { canvasStore } from './stores/canvas'
import { ArtifactPreview } from './components/ArtifactPreview'
import { SmartCanvas } from './canvas/SmartCanvas'

console.log('✅ All component imports successful!')

// 测试组件是否为函数
const components = [
  GeneralSection,
  ModelsSection,
  NetworkSection,
  ImSection,
  AppearanceSection,
  DataSection,
  TagsSection,
  AboutSection,
  CapabilitiesSection,
  SettingsHealthPanel,
  ProviderHealthPanel,
  ArtifactPreview,
  SmartCanvas,
]

const invalidComponents = components.filter(c => typeof c !== 'function')
if (invalidComponents.length > 0) {
  console.error('❌ Invalid components:', invalidComponents)
} else {
  console.log('✅ All components are valid functions')
}

// 测试 API 模块
const apiModules = [
  { name: 'domain', module: domain },
  { name: 'modelPool', module: modelPool },
  { name: 'proxyPool', module: proxyPool },
  { name: 'im', module: im },
  { name: 'market', module: market },
]

const invalidApis = apiModules.filter(m => typeof m.module !== 'object')
if (invalidApis.length > 0) {
  console.error('❌ Invalid API modules:', invalidApis.map(m => m.name))
} else {
  console.log('✅ All API modules are valid')
}

// 测试工具函数
if (typeof checkAllApis !== 'function') {
  console.error('❌ checkAllApis is not a function')
} else {
  console.log('✅ Utility functions are valid')
}

console.log('🎉 Component import test completed!')
