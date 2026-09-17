/**
 * shared/api/ — 共享 API 工具
 *
 * 职责：统一 API 客户端、共享类型
 * 依赖：shared/lib
 */
export { call as domainCall, DomainError } from '../../api/domain'
export type { DomainCall, DomainResponse, DomainInfo, ActionSpec, ParamSpec } from '../../api/domain'
