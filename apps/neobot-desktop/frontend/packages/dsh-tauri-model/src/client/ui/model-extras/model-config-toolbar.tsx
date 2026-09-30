import type { ReactElement } from 'react'
import type { WorkspacePathApplication } from '../../types/remotes.ts'
import type { Translate } from './types'
import { Action, Select } from 'dsh-tauri-ui/client'
import { useEffect, useState } from 'react'
import { listConfigApplications, openConfigInApp } from '../../service/open-in-app'

export type ModelDraft = Record<string, unknown>

export interface ModelProbeTarget {
  settingsNs: string
  profilePath: readonly string[]
  provider?: string
  baseURL?: string
  api?: string
  apiKey?: string
}

export interface ModelConfigToolbarProps {
  t: Translate
  onOpenConfig?: () => void
  disabled?: boolean
}

const DEFAULT_APPLICATION = ''
const APPLICATION_CHOICE_KEY = 'dsh-tauri-model.open-in-app.choice'

/**
 * 模型页工具条：左侧用哪个应用打开（复用 ui 的 `Select`），右侧「打开配置文件」。
 * 候选来自官方 open-in-app 的主机目录，低版本核心或非桌面环境为空，此时只剩右侧按钮。
 */
export function ModelConfigToolbar({
  t,
  onOpenConfig,
  disabled,
}: ModelConfigToolbarProps): ReactElement {
  const [applications, setApplications] = useState<readonly WorkspacePathApplication[]>([])
  const [choice, setChoice] = useState<string>(readChoice)

  useEffect(() => {
    let active = true
    void listConfigApplications().then((next) => {
      if (active)
        setApplications(next)
    })
    return () => {
      active = false
    }
  }, [])

  if (onOpenConfig === undefined)
    return <div className="flex items-center flex-wrap gap-[8px] min-w-0" />

  const selected = applications.find(application => application.id === choice)
  const openConfig = (): void => {
    if (selected === undefined)
      onOpenConfig()
    else
      void openConfigInApp(selected.id)
  }

  return (
    <div className="flex items-center flex-wrap gap-[8px] min-w-0">
      {applications.length === 0
        ? null
        : (
            <Select
              className="max-w-[200px]"
              disabled={disabled}
              variant="composerTrigger"
              options={[
                { value: DEFAULT_APPLICATION, label: t('defaultApplication') },
                ...applications.map(application => ({
                  value: application.id,
                  label: application.name,
                  icon: <ApplicationIcon application={application} />,
                })),
              ]}
              value={selected === undefined ? DEFAULT_APPLICATION : choice}
              icon={selected === undefined ? undefined : <ApplicationIcon application={selected} />}
              onChange={(id) => {
                setChoice(id)
                writeChoice(id)
              }}
            />
          )}
      <Action className="text-[13px] text-secondary rounded-[14px]" variant="link" disabled={disabled} onClick={openConfig}>
        {t('openConfigFile')}
      </Action>
    </div>
  )
}

function ApplicationIcon({ application }: { application: WorkspacePathApplication }): ReactElement {
  return <img src={application.icon ?? ''} alt="" className="w-[14px] h-[14px] rounded-[3px] flex-none" />
}

function readChoice(): string {
  try {
    return localStorage.getItem(APPLICATION_CHOICE_KEY) ?? DEFAULT_APPLICATION
  }
  catch {
    return DEFAULT_APPLICATION
  }
}

function writeChoice(id: string): void {
  try {
    localStorage.setItem(APPLICATION_CHOICE_KEY, id)
  }
  catch {

  }
}
