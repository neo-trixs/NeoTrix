# 外部吸收记录：ChunUI / growth-engineer / pi（2026-10-01）

> **吸收判据**（沿用本仓惯例）：先读原文 → 再判断「可吸收的是什么」→ 最后写**落地证明**。
> ⛔ 不复制代码到生产（AGENTS.md R-P79 要求同会话接到生产可用，而设计模式不属于「代码接入」，
>   故本轮以「可执行不变量 / 新门 / 新配置」作为落地形态，见各节「落地」）。
> ⚠️ 本文只写**实测到的**；未能核实的字段一律标注「未核实」，**不编造**。

## 一览

| 源 | 许可 | 规模 | 代码可吸收 | 实际落地 |
|---|---|---|---|---|
| `liseami/ChunUI` | MIT | ~495★ SwiftUI | ⛔（iOS/Swift） | 两条设计判据 + 字号漂移实测 |
| `GetBrew/growth-engineer` | MIT | ~122★ TypeScript | ⛔（GTM 域） | `nt_build_lock.sh`（8/8 自证） |
| `earendil-works/pi` | MIT | 111.6k★ / 6,688 commits TypeScript | ⛔（agent harness） | 供应链 posture 补缺（本节四） |

---

## 四、`earendil-works/pi`：供应链 posture 的缺口（唯一可移植的洞）

### 它的硬化清单（摘自 README「Supply-chain hardening」）

> We treat npm dependency changes as reviewed code changes.

1. 直接依赖**精确锁版本**；内部包才用 range
2. `.npmrc` 设 `save-exact=true` **和 `min-release-age=2`** —— 同日发布的版本不参与解析
3. `package-lock.json` 是依赖唯一真相；**pre-commit 拦截误提交 lockfile**
4. shrinkwrap 生成对「依赖生命周期脚本」有**显式白名单**，新依赖需人工审
5. release smoke test 在仓库外建隔离安装再打 tag

### 本仓现状（实测，非推测）

| 维度 | pi | 本仓 | 结论 |
|---|---|---|---|
| 供应链检查 | 预防 + 检测 | `scripts/check-supply-iocs.sh` | ⚠️ 仅**检测** |
| 扫描面 | JS lockfile | **仅 `Cargo.lock`** | ❌ 前端完全无覆盖 |
| 冷却期 | `min-release-age=2` | 无（无 `.npmrc`） | ❌ 缺 |
| 精确锁定 | `save-exact=true` | `frontend/` 恰好合规 / `neobot-ui/` 6 处宽松 | ⚠️ 不一致 |
| lockfile | 提交并当唯一真相 | **两个前端都没有 lockfile** | ❌ 安装不可复现 |

### 关键判读：现有门是**反应式**的

`check-supply-iocs.sh` 的头部注释自述为
「the zero-cost **tripwire**」+「Scans `Cargo.lock` for packages named in public advisories」。
⇒ 它在恶意版本**已发布**后才报警，无法阻止解析。
⇒ **pi 贡献的是「预防」，而本仓只有「检测」。这是真正的缺口，不是重复建设。**

### 落地：仓库根 `.npmrc`（只含一条）

```ini
save-exact=true
```

* ✅ **已实测**：`npm config get save-exact` → `true`。
* 与既有纪律一致：`frontend/package.json` 的 22+66 依赖本就**全部精确锁定**，
  `save-exact=true` 只是把该既有事实**固定下来**。
* ⛔ **刻意不加** pi 同款的 `min-release-age`（同日新版本不参与解析）：
  实测 `npm config get min-release-age` 返回 **`null`**，而同文件里
  `save-exact` 返回 `true` ⇒ npm **读了本文件但不认识这个键**，
  在本机 npm 11.12.1 上是**空转的**。
  ⇒ 留 no-op 键 = **假防御**（读 .npmrc 的人会以为冷却期已生效）。
  ⇒ 等本机 npm 真正支持且能回读验证时再加。

### ⛔ 顺带实测到的前置问题（非本轮引入）

`npm install --dry-run` 在 `neobot-ui` 下**本就崩溃**：

```
npm error Cannot read properties of null (reading 'matches')
```

对照实验：把本轮新增的 `.npmrc` **移走**后**同样报错** ⇒ 与本轮改动无关，
是该应用**无 `package-lock.json`** 条件下 npm 11.12.1 的既有行为。
⚠️ 记录在此以免**下一个 agent 把它误归因给最近的改动** ——
「最近改过」不等于「是我改坏的」，R-SCAN-2 的反例。

### ⛔ 明确不做（留给下一批，需先定 canonical）

* **不生成 `package-lock.json`** —— 生成它是重操作（联网 + 全量解析），且 `apps/neobot-desktop/`
  属他窗 WIP 区，产出会与那边必然冲突。
* **不扫 JS IOC** —— 需要一份可信的 JS 侧 IOC 源；凭空造名单 = 假安全，
  比没有更危险（AGENTS.md §4.2「本地红 ≠ CI 红」同源教训）。
  ⇒ 正确做法是**先拿 Socket/公开 advisory 的 JS 名单**，再扩 `check-supply-iocs.sh`
  的扫描面到 `package-lock.json`，而不是现在硬塞一个空壳门。

---

## 五、`liseami/ChunUI`：两条设计判据（+ 一次实测发现的扁平缺陷）

代码 ⛔ 不可移植（SwiftUI / iOS 设计系统，无一段能落到 TypeScript + Tailwind）。
可移植的是两条**硬约束写法**：

1. **字号是三梯度铁律**，不是「建议」—— 枚举式定档 + 全局校验，违反即构建失败
2. **宿主注入的接线座** —— 组件不自取依赖，由宿主显式注入（可测 + 可换肤）

### 本仓实测：字号确有两套体系并存

`apps/neobot-desktop/frontend` 与 `neobot-ui` 的 CSS 里有 **11 种 `font-size` 取值**，
同时又在用 Tailwind 的 `text-xs/sm/base` **另一套**；另有 **13 处非标准 `10px`/`11px`**
分布在 4 个文件。

⇒ 这正是 ChunUI 铁律要防的形态：**「定档」存在但不被强制**，于是各处自由发挥。
`skills/design/design-core/SKILL.md` 目前**只要求生成 typography scale，缺硬约束**。

⚠️ **未落地的原因**：收敛它必须先定哪套是真身（见下节），且该目录属他窗 WIP 区，
按 AGENTS.md §2「无定点不改」，本轮只测不改。

---

## 六、`GetBrew/growth-engineer`：把散文纪律变成可执行不变量

### 它命中的痛点（原文）

> Heavy commands wait their turn behind a lock, so several worktrees can run
> checks without running out of memory.

**本仓同一天内两次因他窗 cargo 而构建超时。** 而 AGENTS.md §2 的
「⛔ 禁并行全量构建 / 多窗口同跑必爆 swap」是**散文**，无任何机制保证 ——
这就是「已写规则 ≠ 已被执行」。

### 动手前先查已有设施（避免重复造轮子）

| 设施 | 已有能力 | 为什么不闭合 |
|---|---|---|
| `nt_mem_gate.sh` | 报内存是否够 | 只报告，**不串行**。「内存够」≠「没人在编译」 |
| `scripts/wait-for-cargo.sh` | `pgrep` 等 cargo 空闲 | 「**检查后再执行**」⇒ 竞态窗口：两窗口可同时通过检查再同时开跑 |

⇒ `nt_build_lock.sh` 用**原子 `mkdir`** 把「检查 + 占位」合成一步，**闭合该窗口**。
是**叠加，不是取代**。

### 落地：`scripts/ops/nt_build_lock.sh` + `nt_build_lock_selftest.sh`

* 不用 `flock`：macOS 默认无，BSD/GNU 语义不同 ⇒ 改用 POSIX 原子的 `mkdir`
* 带 PID + 时间戳：持有者已死 / 超龄（默认 30min）自动回收
* 带超时：超时**默认仍穿透执行**；`--strict` 才改为放弃并返回 `75`（EX_TEMPFAIL）
  ⇒ 锁**永不**让工作流永久停住
* `release()` **只删自己持有的锁**（比对 pid）
* 自证 **8/8 全绿**

### 自证抓到的三个真 bug（含我自己写的两个）

1. **`release()` 原为无条件 `rm -rf`** ⇒ 会在「A 超时穿透、B 已拿到锁」时删掉 **B 的锁**
   ⇒ 第三个进程可同时进入 ⇒ **互斥失效**。已改为比对 pid。
2. **用例⑦失败是我用例设计的缺陷** —— 我给「并发互斥」用例也设了 `--timeout 0`，
   于是等待方走「超时穿透」分支，表现为重叠。那是**设计意图**，不是缺陷。
   ⇒ 教训：**测试用例必须与工具的语义契约对齐**，否则会把设计意图误判为 bug。
3. ⓘ **全角括号又踩 R-SCAN-4** —— 提示文案里的 `（$LOCK_DIR）` 在 `set -u` 下报
   `unbound variable`。**全角标点在 shell 里不是排版。**

---

## 七、本轮两次自查抓到的方法论失误（同一坑，第 2 次）

用 Python `csv.writer` 回写 `docs/architecture/absorption-sources/repos.csv`，
造成 **992 行假重排**，把 3 条真实增量淹没在噪声里：

1. `csv.writer` 默认 `lineterminator='\r\n'`，而原文件是**纯 LF**
2. **表头原本不加引号**，我给所有字段加了引号

`8d079756` 已记过「改 JSON 要先看原文件用什么格式写的」，本轮把同一教训
**套用到 CSV 时又踩了一次**。

⇒ **升级后的纪律**：凡用序列化库回写**既有数据文件**，必先确认
**换行符 + 引号策略**，否则必产生整文件噪声 diff；且**回写后必须逐行比对**
（本次验证：与前一版本仅 1 处差异 = 新增 `liseami/ChunUI`，其余 494 行逐字相同）。

---

## 八、待下一批处理（本轮 ⛔ 未做）

1. **定 `apps/neobot-desktop` 的 canonical 前端**：
   `frontend/` 与 `neobot-ui/` 是**两个独立 vite 应用**，
   14 个同名相对文件、**5 个逐字相同**（含 127 行 `nb-markdown.css`、3 个
   `vendor/openghost/*.js`）。须先查明构建/发布入口归属，才能定谁是真身。
2. **字号收敛**：依 ChunUI 铁律定档 + 加构建期校验（须先有 1）。
3. **扩 `check-supply-iocs.sh` 扫描面到 `package-lock.json`**（须先有可信 JS IOC 源）。
4. `KKKKhazix/AIHOT` —— 许可证**未核实**，按规矩**未吸收**。
5. `trendshift.io` 榜单 —— 可免登陆访问，但榜单条目本身**不等于**已核实的许可/可移植性；
   已登记为源，尚未逐仓核验。