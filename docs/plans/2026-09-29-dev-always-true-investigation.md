# INVESTIGATION（调查，不是结论）：release 构建里 Tauri `dev` 恒为真 — 2026-09-29

> 状态：**未找到完整根因，链条已闭合到「feature 门」，差 CLI 侧最后一跳。**
> 本文档只记录实测证据与排除项，不下结论。修法（`devUrl` 置 null）已在别处验证生效，此处不复述为结论。

## 1. 已确认事实（贴证据行）

### F1. `DEP_TAURI_DEV` 是 `tauri` crate 自身 build 脚本发射的 `cargo:dev` 指令

- 发射点：`/Users/neo/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tauri-2.11.5/build.rs:261`
  ```rust
  println!("cargo:dev={dev}");
  ```
- 之所以下游读到的是 `DEP_TAURI_DEV`：同一 crate `Cargo.toml:19` 声明 `links = "Tauri"`，
  Cargo 把 `cargo:dev=...` 转成依赖方 build 脚本可见的 `DEP_TAURI_DEV`。这是 Cargo links/metadata 机制的标准行为（本轮未再另行验证，视为已知机制）。
- 消费点：`/Users/neo/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tauri-build-2.6.3/src/lib.rs:425-428`
  ```rust
  pub fn is_dev() -> bool {
    env::var_os("DEP_TAURI_DEV")
      .expect("missing `cargo:dev` instruction, please update tauri to latest")
      == "true"
  }
  ```

### F2. `dev` 的值只由 `custom-protocol` feature 决定，与 PROFILE / debug_assertions 无关

- 判定逻辑原文：`tauri-2.11.5/build.rs:256-261`
  ```rust
  let custom_protocol = has_feature("custom-protocol");
  let dev = !custom_protocol;
  alias("custom_protocol", custom_protocol);
  alias("dev", dev);

  println!("cargo:dev={dev}");
  ```
  其中 `has_feature` 读 `CARGO_FEATURE_<SHOUTY_SNAKE>`（同文件 230-243 行注释有写）。
- 同一判定在运行时侧是：`tauri-2.11.5/src/lib.rs:308-310`
  ```rust
  pub const fn is_dev() -> bool {
    !cfg!(feature = "custom-protocol")
  }
  ```
- 搜遍 `tauri-2.11.5/build.rs` 与 `tauri-build-2.6.3/src/lib.rs` 的 `dev` 相关行，
  没有任何 `debug_assertions` / `PROFILE` / `OPT_LEVEL` 参与。`--release` 不改变 `dev`。
- `custom-protocol` 的官方定义（`tauri-2.11.5/src/lib.rs:23` 文档行）：
  ```
  - **custom-protocol**: Feature managed by the Tauri CLI. When enabled, Tauri assumes a production environment instead of a development one.
  ```

### F3. `dev=true` 直达空资源分支的完整调用链（两条并行路径都确认）

- 路径 A（build 脚本 → codegen，桌面端实际走的）：
  `tauri-build-2.6.3/src/codegen/context.rs:132-133`
  ```rust
  let code = context_codegen(ContextData {
    dev: crate::is_dev(),
  ```
  → `tauri-codegen-2.7.0/src/context.rs:178`
  ```rust
  } else if dev && config.build.dev_url.is_some() {
    let assets = EmbeddedAssets::default();   // 空的
  ```
- 路径 B（宏展开，供 `generate_context!`）：
  `tauri-macros-2.6.3/src/context.rs:154`
  ```rust
  .map(|(config, config_parent)| ContextData {
    dev: cfg!(not(feature = "custom-protocol")),
  ```
- `tauri-build-2.6.3/src/lib.rs:519` 还把同一值注册成 cfg 别名：`cfg_alias("dev", is_dev());`
  `lib.rs:590` 的 `if !is_dev()`（macOS minimum_system_version 那段）同源。

### F4. 桌面端 crate 从未开启 `custom-protocol`，且默认 feature 也不含它

- 独立桌面仓（见 F6）`apps/neobot-desktop/Cargo.toml:21-24`：
  ```toml
  tauri-build = { version = "2", features = [] }
  tauri = { version = "2", features = [] }
  ```
- `tauri-2.11.5/Cargo.toml` 的 `[features] default` 列表为
  `wry, compression, common-controls-v6, dynamic-acl, x11, dbus` —— 无 `custom-protocol`。
  所以按 F2 的逻辑，只要不用 CLI 介入，`dev` 恒为 `true`，与 `--release` 无关。
- 桌面端 `build.rs` 只有两行（`tauri_build::build();`），无任何 feature 手动开启。

### F5. 排除项（`--cfg dev` / RUSTFLAGS / .cargo 配置不是原因）

- `env | grep -i -E "rust|cargo|tauri|profile|debug"`：无 `RUSTFLAGS`、`CARGO_ENCODED_RUSTFLAGS` 为空，
  无 `TAURI_*`、`PROFILE`、`DEBUG` 相关变量（PATH 里仅有常规条目）。
- 本仓 `.cargo/config.toml` 全文只有 `[alias]` 三行（xt/xc/xf），无 `[build] rustflags`，
  无 `RUSTFLAGS`，无 `--cfg` 注入。
- 仓库内搜 `devUrl|frontendDist` 只命中文档/脚本/训练数据，无残留的第二份 `tauri.conf.json`
  （本仓 `**/tauri.conf.json` glob 零命中）。`--cfg dev` 假设无证据支持，予以排除。

### F6. 本仓根本没有 Tauri 依赖图 —— 出事的构建发生在独立仓

- `rg -n "tauri" Cargo.toml / neotrix-core/Cargo.toml`：零命中。
- `rg -n -A3 'name = "tauri"' Cargo.lock`：零命中（`tauri-build/codegen` 同样零命中）。
- `skills/dev-tools/build-desktop/build-desktop.sh:26-27` 明写：桌面端已于 2026-09-28 从本仓迁出，
  位置在 `~/Downloads/Neo/neobot`（`src-tauri` 归档于 5c02e738，`apps/neobot-desktop` 移除于 d5413335）。
- 独立仓现状：`apps/neobot-desktop/tauri.conf.json:9-10` 当前为
  `"frontendDist": "frontend/dist", "devUrl": null` —— 即修法已落地。
  出事时的值（`http://localhost:1422`）仅见于本仓脚本注释（`build-desktop.sh:59-60`）与
  `docs/architecture/LESSONS-20260928-blank-window-and-embedding.md:18-26` 的记录，
  独立仓当前文件已看不到它。
- `build-desktop.sh:131-134` 的 `cmd_build` 走的是**裸 `cargo build`**：
  `--release` 分支 runs `(cd "$REPO" && cargo build --release -p neobot-desktop)`，
  只有 `package`/`package:dir` 阶梯才走 `npx tauri build`（同文件 146-160 行）。

## 2. 未解问题（不许猜，只列缺口）

1. **Tauri CLI 在哪一步打开 `custom-protocol`？** 本地 registry 里没有 `tauri-cli` 源码，
   `rg custom.protocol tauri-build-2.6.3` 零命中。只知道文档说 "managed by the Tauri CLI"（F2），
   但 CLI 是拼 `--features custom-protocol`、改 Cargo.toml，还是走别的注入，没有亲眼看到。
   在看到之前，不能断言「`cargo build` 必 dev 真而 `tauri build` 必 dev 假」，只能说现有证据指向这个方向。
2. **出事的那次 release 构建到底用的是哪条命令？** `cmd_build`（裸 cargo，最可疑）还是
   `tauri build`（CLI，理论上应开 custom-protocol）？LESSONS 文档只记录了现象与二进制对照
   （空 dist vs 满 dist 都是 22921120 字节；删 devUrl 后 +49,536B），没记录构建命令。
   没有这条信息，F4 的「恒真」只能解释裸 cargo 路径，解释不了 CLI 路径。
3. `DEP_TAURI_DEV` 的 links 前缀推导（`links="Tauri"` → `DEP_TAURI_*`）是按 Cargo 文档认定的，
   本轮没有跑构建打印 env 实证（任务禁令禁止 `cargo build`）。严格说这是一条未实证的环节，
   但它是标准机制，可信度高，列在这里以示诚实。

## 3. 下一步（具体文件/命令，谁来接往下查）

1. 找 CLI 侧证据（二选一，不用都做）：
   - 在独立仓看 `npx tauri build` 实际拼出的 cargo 命令：`cd ~/Downloads/Neo/neobot && npx tauri build --help`，
     或开 `--verbose` 跑一次 `--no-bundle` 构建，看它是否带 `--features ...custom-protocol`；
     省时间的办法是搜 CLI 源码：`rg -n "custom-protocol" ~/.npm/_npx/*/node_modules/@tauri-apps/cli*/ 2>/dev/null | head`。
   - 或在独立仓 `target/` 里找已产出的 fingerprint/dir：`rg -l "custom-protocol" ~/Downloads/Neo/neobot/target/release/build/tauri-*/output 2>/dev/null`，
     看 `cargo:dev=` 那行到底是什么（注意：这会读 build 产物目录，不触发构建，不违反禁令；但别删别碰）。
2. 确认出事命令：问当事人或翻该次构建的 shell 历史，目标是回答「那次 22921120 字节的 release 构建是 `cargo build --release -p neobot-desktop` 还是 `npx tauri build`」。
   若是前者，本调查的 feature 门解释即充分；若是后者，回到第 1 条继续查 CLI。
3. 只读验证 dev 值（不动构建）：`cargo metadata --format-version 1 --manifest-path ~/Downloads/Neo/neobot/Cargo.toml`（任务允许的只读命令），
   确认 `neobot-desktop` 对 `tauri` 的 feature 集合里无 `custom-protocol`。
   不要跑 `cargo build`/`cargo check --all-targets`（任务禁令 + 本机 16G swap 风险）。

## 4. 调查过程备忘（时间到即停）

- 按任务路径 1→2→3 走完：`cargo::metadata=TAURI_DEV` 的说法修正为 `cargo:dev`（`build.rs:261`），
  经 `links="Tauri"` 变成 `DEP_TAURI_DEV`；判定逻辑是 feature 门不是 `debug_assertions`，故第 3 步的
  RUSTFLAGS 排查转为排除项（已排除）。
- 未动文件（除本报告新建）、未跑 `cargo build`、未碰 git。`cargo metadata` 最终没跑（Cargo.lock 的零命中已足够证明本仓无 Tauri 图；独立仓的 metadata 留给下一步）。
