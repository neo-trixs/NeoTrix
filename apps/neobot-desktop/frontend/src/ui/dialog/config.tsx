import type { PropsWithOverlays } from '@overlastic/react'
import { Code, Cpu, LogoWindows, Puzzle } from '@gravity-ui/icons'
import { useEffect } from 'react'
import { loadApiPanel } from '@/api-panel'
import { cn, Modal } from '@heroui/react'
import { useDisclosure } from '@overlastic/react'
import { useListener } from '@reause/core'
import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Case, If, Switch } from 'react-if-lite'
import { hooks } from '@/config/hooks'
import { queryKeys } from '@/config/query-keys'
import { ConfigCore } from '@/ui/config/core'
import { ConfigDebug } from '@/ui/config/debug'
import { ConfigSkills } from '@/ui/skills-panel'

/** 配置面板标识（左侧导航与顶部「配置」菜单共用同一组值） */
/**
 * 配置面板的页签。`api` 与 `skills` 是本仓加的 —— **后端契约的可视化** 与
 * **NeoBot 真正的扩展件（技能）**。
 *
 * 放在这里而不是另开入口：配置面板本来就是「这个应用有什么、是什么状态」的
 * 容器，接口清单与技能属于同一类信息。单独造入口会让「应用到底有哪些接口 /
 * 能装哪些扩展」变成要找两处才能回答的问题。
 *
 * ⛔ 自持形态**不渲染** `profiles` 与 `plugins` 两个页签：
 *    - profiles：`get_profiles` / `create_profile` / `set_active_profile` … 一条都没注册，
 *      面板只会画出一个空列表 + 一排点了报错的按钮 —— 「安静地坏掉」比不显示更糟。
 *    - plugins：DSH 插件体系整条不存在（本仓不加载那个运行时）。
 *    两者都有真实替代或明确不存在，所以是**裁决**，不是欠实现。
 *    门 `nt_check_ui_calls.mjs` 守着这件事：可达模块里不得再调未注册命令。
 */
export type ConfigTab = 'application' | 'profiles' | 'plugins' | 'harness' | 'api' | 'skills'

/**
 * API 契约页。
 *
 * ⛔ 它**读契约而不是硬编码清单** —— 硬编码的清单必然腐化，
 * 而本仓已经吃过一次亏（CAPABILITY-MAP 声称 111 条命令，46 条不存在）。
 * 数据源是 Rust 侧 `src/api.rs` 的 SPECS，经 `neobot_api_specs` 取出。
 */
function ConfigApi() {
  const [host, setHost] = useState<HTMLDivElement | null>(null)
  useEffect(() => {
    if (host) void loadApiPanel(host)
  }, [host])
  return (
    <div className="api-panel" ref={setHost} />
  )
}

export interface ConfigDialogProps extends PropsWithOverlays {
  /** 打开时定位到的面板；缺省为「应用」 */
  tab?: ConfigTab
}

export function ConfigDialog(props: ConfigDialogProps) {
  const disclosure = useDisclosure({ props })
  const { t } = useTranslation()
  // 坏技能数：在「技能」Tab 上给出角标，方便直接感知装坏的技能。
  // ⛔ 数据源是技能清单，**不是** `get_dsh_plugins` —— 那条命令本仓不存在，
  //    而它在 react-query 里 reject 后会静默回落到 `[]`，于是「0 个异常插件」
  //    这个结论看起来像事实，实际是「压根没读到过」。
  const { data: skills } = useQuery({
    queryKey: queryKeys.plugins,
    queryFn: () => invoke<{ skills: { name: string }[], skipped: number }>('neobot_skill_list'),
  })
  const abnormalCount = skills?.skipped ?? 0

  const navs: { label: string, value: ConfigTab, icon: typeof Cpu }[] = [
    { label: t('config.application'), value: 'application', icon: LogoWindows },
    // 「档案」按上面的裁决不渲染；保留在类型里是为了不动上游的 ConfigTab 定义面。
    { label: t('config.harness'), value: 'harness', icon: Cpu },
    { label: '技能', value: 'skills', icon: Puzzle },
    // i18n key 故意用字面量而非 t()：接口清单是**开发者面板**，
    // 走 i18n 会让 6 份语言文件都要加一条，而它并不面向终端用户。
    { label: 'API', value: 'api', icon: Code },
  ]

  const [activeTab, setActiveTab] = useState<ConfigTab>(props.tab ?? 'application')

  // 服务重启/退出前由 store 触发，命令式收起本对话框（卸载时自动注销）
  useListener(hooks['config.dialog.hidden'].on, disclosure.cancel)

  return (
    <Modal isOpen={disclosure.visible} onOpenChange={disclosure.cancel}>
      <Modal.Backdrop>
        <Modal.Container size="lg">
          <Modal.Dialog data-testid="dsh-config-dialog" className="w-[800px] max-w-[calc(100vw-48px)] h-[min(720px,calc(100vh-96px))] pr-2.5">
            <Modal.CloseTrigger data-testid="dsh-config-dialog-close" />
            <Modal.Header className="mb-3">
              <Modal.Heading>
                {t('app.config')}
              </Modal.Heading>
            </Modal.Header>
            <Modal.Body className="flex gap-6 pr-0">
              <aside className="w-[164px]">
                <nav className="flex flex-col gap-2 w-full">
                  {navs.map((item) => {
                    const isActive = item.value === activeTab
                    return (
                      <button
                        key={item.value}
                        data-testid={`dsh-config-nav-${item.value}`}
                        aria-current={isActive ? 'true' : undefined}
                        onClick={() => setActiveTab(item.value)}
                        className={cn(
                          'text-foreground h-[40px] rounded-md flex items-center gap-2 py-[9px] px-[16px] hover:bg-background-secondary cursor-pointer',
                          isActive ? 'bg-background-secondary' : '',
                        )}
                      >
                        <item.icon className="w-5 h-5 mr-2" />
                        <span>{item.label}</span>
                        <If cond={item.value === 'skills' && abnormalCount > 0}>
                          <span data-testid="dsh-config-nav-skills-badge" className="ml-auto flex size-5 items-center justify-center rounded-full bg-danger text-[10px] font-semibold leading-none text-white">
                            {abnormalCount}
                          </span>
                        </If>
                      </button>
                    )
                  })}
                </nav>
              </aside>
              <div data-testid="dsh-config-panel-body" className="flex flex-col flex-1 overflow-auto min-h-0 pr-2.5">
                <Switch value={activeTab} as="div">
                  <Case cond="application">
                    <ConfigDebug />
                  </Case>
                  <Case cond="harness">
                    <ConfigCore />
                  </Case>
                  <Case cond="skills">
                    <ConfigSkills />
                  </Case>
                  {/* API 契约：后端的可视化。本仓加的页签。 */}
                  <Case cond="api">
                    <ConfigApi />
                  </Case>
                </Switch>
              </div>
            </Modal.Body>
          </Modal.Dialog>
        </Modal.Container>
      </Modal.Backdrop>
    </Modal>
  )
}
