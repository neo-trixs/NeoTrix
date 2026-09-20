/**
 * useHarnessState — Harness 运行状态 + 审批
 */
import { createSignal, onMount } from 'solid-js'
import { harness } from '../../../../api'
import type { HarnessApproval, HarnessStep, HarnessRunResponse } from '../../../../api/harness'

export function useHarnessState() {
  const [harnessRoute, setHarnessRoute] = createSignal<{ tag: string; domain: string; specialist: string } | null>(null)
  const [harnessReport, setHarnessReport] = createSignal<HarnessRunResponse | null>(null)
  const [harnessRunning, setHarnessRunning] = createSignal(false)
  const [harnessSteps, setHarnessSteps] = createSignal<HarnessStep[]>([])

  const [approvals, setApprovals] = createSignal<HarnessApproval[]>([])
  const refreshApprovals = () => {
    harness.harnessApprovalList().then(setApprovals).catch(() => setApprovals([]))
  }
  onMount(() => { refreshApprovals() })

  return {
    harnessRoute, setHarnessRoute,
    harnessReport, setHarnessReport,
    harnessRunning, setHarnessRunning,
    harnessSteps, setHarnessSteps,
    approvals, setApprovals, refreshApprovals,
  }
}
