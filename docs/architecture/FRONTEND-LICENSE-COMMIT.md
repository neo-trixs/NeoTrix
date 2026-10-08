# Frontend License Evaluation P2_COMMITT

##条款摘要
在 `apps/neobot-desktop/frontend/LICENSE.details` 中，MIT 条款被覆写：
- 不允许商业二次开发（No Commercial Secondary Development）
- 禁止任何商业化再分发
- 在所有衍生产品中保留原许可条款

## 判断
当前该目录下至少有以下子模块：
- packages/dsh-tauri
- packages/dsh-tauri-archive
- packages/dsh-tauri-experimental
- packages/dsh-tauri-extension
- packages/dsh-tauri-model
- packages/dsh-tauri-pet
- packages/dsh-tauri-rightclick
- packages/dsh-tauri-scheduler
- packages/dsh-tauri-ui
- packages/dsh-tauri-ui-playground
- packages/dsh-tauri-worktree

所有上述模块均被包含在 `apps/neobot-desktop/frontend` 的统一许可条款中。
`frontend` 自身没有独立的 LICENSE 文件，所有条款通过该目录根部的 `LICENSE.details
` 及 `LICENSE.upstream-dsh` 组合声明。

## 原文链接
- 主文件：`apps/neobot-desktop/frontend/LICENSE.details`
- 备份：`apps/neobot-desktop/frontend/LICENSE.upstream-dsh`
- 参考：`apps/neobot-desktop/neobot-ui/src/vendor/openghost`（已被记录体现）

## 处理原则
1. 该目录下所有子模块必须同时遵守 `LICENSE.details` 中的覆盖条款。
2. 不允许仅通过删掉覆盖文档来放行商业二次开发。
3. 任何对 `frontend` 树的修改都必须在 `.neotrix/LICENSE-EXCEPTIONS.md` 中重新签署。

## 责任签字
（由项目所有者或指定法务/安全负责人手写后提交。）

| 签署人 | 日期 | 复审日期 | 关闭条件 |
|--------|------|----------|----------|
|        |      |          |          |

## 对照表

| 校验项 | 必须达到 |
|--------|----------|
| 所有子模块均满足 `LICENSE.details` | ✅ |
| 商业二次开发被明确禁止 | ✅ |
| `frontend` 目录不存在 `LICENSE` 裸签署 | ✅ |
| 与 `MIT` 冲突时优先执行覆盖条款 | ✅ |
