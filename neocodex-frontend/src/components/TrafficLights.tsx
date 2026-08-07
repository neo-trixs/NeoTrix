import { invoke } from '@tauri-apps/api/core'

/**
 * 自绘 macOS 交通灯（红/黄/绿窗口控制点）。
 * 生产窗口使用 decorations:false，原生交通灯被隐藏，由本组件自绘并接入窗口控制命令。
 * 尺寸 12px（macOS 标准），比原生 overlay 更大更清晰。
 */
export function TrafficLights() {
  const minimize = () => invoke('window_minimize')
  const maximize = () => invoke('window_maximize')
  const close = () => invoke('window_close')

  return (
    <div class="traffic" data-tauri-drag-region>
      <button class="t-dot t-c" onClick={close} aria-label="关闭窗口" title="关闭" />
      <button class="t-dot t-m" onClick={minimize} aria-label="最小化" title="最小化" />
      <button class="t-dot t-x" onClick={maximize} aria-label="最大化" title="最大化" />
    </div>
  )
}