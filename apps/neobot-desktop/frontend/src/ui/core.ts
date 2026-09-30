/**
 * 极小核心工具集。
 *
 * ⛔ **只有确实被复用的东西才放这里**（当前仅 `esc`）。
 * 本仓有一条教训：core 里堆「将来也许用得上」的工具，会让
 * 死代码审计与命名门持续报警，最后大家习惯性忽略它们。
 */

/** HTML 转义。用于任何把**用户或模型产出**拼进 innerHTML 的地方。 */
export function esc(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
