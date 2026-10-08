# neobot 桌面发布规格（C2，2026-10-08）

> 源对照：`Tiga001/Captain_Who`（Apache-2.0，Rust，本地优先桌面 agent）、
> `iamlukethedev/Herald-OS`（MIT，agent-native OS）、`nexu-io/open-design`（Apache-2.0，
> 官方首发 dsh runtime 的形态）。源只借形状，未取代码。

## 1. 版本通道

| 通道 | tag 前缀 | 更新频率 | 适用面 |
|---|---|---|---|
| stable | `vX.Y.Z` | 月度 | 默认（auto-update 默认接它） |
| preview | `vX.Y.Z-preview.N` | 每周 | 内测/新功能抢鲜 |

不变量：同一 channel 的二进制签名身份必须一致（macOS notarize + Windows Authenticode）；
preview 升级到 stable 必须显式确认（不回滚数据）。

## 2. 问后再装（Captain_Who 吸收的形状）

- 启动时**不静默**执行自更新：先看到「发现新版本 vX.Y.Z（N MB）」卡片，二选一
  「立即更新 / 稍后」；「稍后」最多推迟 7 天，超过则再次提示。
- 桌面安装器行为只负责两个动作：**安装 + 签名校验**。运行时抽象不在安装期展开。

## 3. 签名/公证步骤（待 owner 提供凭证后执行）

1. macOS：`codesign --sign <DeveloperID>` + `xcrun notarytool submit --wait`；
2. Windows：`signtool sign /tr <ts> /td sha256`；
3. Linux：AppImage + `gpg --detach-sign`；
4. 产物随 tag 写入 GitHub Releases，SHA256 附录在 notes 里；
5. 更新通道的 `latest.json`（或渠道内静态清单）带版本+URL+SHA256，客户端校验失败即拒装。

## 4. 与 code map 路线的关系

- 本规格不动源码 ⇒ **不触发** code map 增量刷新（与 C1 同型：文档吸收）。
- 未实现项（签名凭证/notarize）记入 ROADMAP「发布」桶，不在本规格执行。
