/**
 * 技能面板 —— **本仓自有页签**，不是上游文件。
 *
 * # 为什么用它顶替上游「插件」页签
 *
 * 上游第 3 个页签是「插件」，那套 DSH 插件体系（`get_dsh_plugins` /
 * `enable_dsh_plugin` / `snapshot_plugin` …）在本仓**一条命令都没注册**。
 * 它的失效方式极其安静：react-query 拿到 reject 后回落到 `[]`，
 * 面板画出一个空列表 —— 看上去像「你没装插件」，而不是「这个功能不存在」。
 * 按钮点了才报 `command not found`，而没人会为空白面板点按钮。
 *
 * 而 NeoBot 真正有的「扩展件」是**技能**（`nt_skills`，数据目录 `skills/`，
 * 与 `skills/assets/icons/neobot/` 同根）。技能与插件在用户视角是同一件事：
 * 「让 agent 多会一件事」。所以这里不新造第三个入口，而是**把插件的位置
 * 换成技能**，页签数量与上游一致（应用/档案/核心/… 顺序不变）。
 *
 * ⛔ 数据只来自后端命令，不硬编码清单：硬编码清单必然腐化
 *    （本仓已吃过 CAPABILITY-MAP 声称 111 条、46 条不存在的亏）。
 */

import { useCallback, useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface SkillView {
  skills: { name: string, description: string, path: string }[]
  /** 坏包数（整个技能根目录扫出来跳过的）；>0 说明有技能装坏了。 */
  skipped: number
}

export function ConfigSkills() {
  const [data, setData] = useState<SkillView | null>(null)
  const [err, setErr] = useState('')
  const [src, setSrc] = useState('')
  const [busy, setBusy] = useState(false)

  const reload = useCallback(() => {
    void invoke<SkillView>('neobot_skill_list')
      .then((v) => {
        setData(v)
        setErr('')
      })
      .catch((e) => {
        setData(null)
        setErr(String(e).slice(0, 160))
      })
  }, [])
  useEffect(reload, [reload])

  const install = useCallback(async () => {
    const path = src.trim()
    if (!path || busy) return
    setBusy(true)
    try {
      await invoke('neobot_skill_install', { path })
      setSrc('')
      reload()
    } catch (e) {
      setErr(String(e).slice(0, 160))
    } finally {
      setBusy(false)
    }
  }, [src, busy, reload])

  return (
    <div className="flex flex-col gap-3 p-1">
      <div>
        <div className="text-sm font-semibold text-ink">技能</div>
        <p className="mt-1 text-xs text-muted">
          技能是数据目录下的 <code className="text-[11px]">skills/</code>
          子目录，一个技能一个文件夹。装上即可被 agent 使用。
        </p>
      </div>

      {err && (
        <div className="rounded-lg bg-btn-danger-hover p-2 text-xs break-all text-[#c33b38]">{err}</div>
      )}

      {data && data.skipped > 0 && (
        <div className="rounded-lg bg-btn-danger-hover p-2 text-xs text-[#c33b38]">
          有 {data.skipped} 个技能包读取失败已被跳过（坏包不炸列表）。
        </div>
      )}

      <div className="overflow-hidden rounded-lg border border-line">
        {data === null && !err && <div className="p-3 text-xs text-muted">正在读技能…</div>}
        {data && data.skills.length === 0 && (
          <div className="p-3 text-xs text-muted">
            还没有技能。装一个试试：下面填技能目录的绝对路径。
          </div>
        )}
        {data &&
          data.skills.map(s => (
            <div key={s.name} className="border-b border-line px-3 py-2 last:border-b-0">
              <div className="text-[13px] font-medium text-ink">{s.name}</div>
              {s.description && <div className="mt-0.5 text-[11px] text-muted">{s.description}</div>}
              <div className="mt-0.5 truncate text-[10px] text-muted" title={s.path}>
                {s.path}
              </div>
            </div>
          ))}
      </div>

      <div className="flex gap-1.5">
        <input
          value={src}
          onChange={e => setSrc(e.target.value)}
          onKeyDown={e => {
            if (e.key === 'Enter') {
              e.preventDefault()
              void install()
            }
          }}
          placeholder="技能目录绝对路径…"
          aria-label="技能路径"
          className="min-w-0 flex-1 rounded-lg border border-line bg-panel px-2 py-1.5 text-[12px] text-ink outline-none focus:border-info-hover"
        />
        <button
          type="button"
          onClick={() => void install()}
          disabled={busy || !src.trim()}
          className="shrink-0 rounded-lg bg-btn-fill px-3 py-1.5 text-[12px] text-btn-ink disabled:opacity-40"
        >
          {busy ? '装着…' : '安装'}
        </button>
      </div>
    </div>
  )
}
