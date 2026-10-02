# 知会：`nt_pet.rs` 2 处 `.unwrap()` 待修（属另一窗口在途特性，我未改）

## 为什么这份知会存在

`scripts/check-unwrap.sh --strict` 当前 rc=1（`.github/workflows/ci.yml:49` 是**阻断 step**）。
修掉 `apps/` 那 2 处之后，只剩这 2 条：
```
crates/neotrix-neobot/src/nt_pet.rs:225
crates/neotrix-neobot/src/nt_pet.rs:226
```
⓰ 该文件最后修改是提交 `2d8159b5`「memory 修订/撤销 + 原子写 + 用量记账 + **桌宠**」
⇒ 判定属**另一窗口的在途特性** ⇒ 我**刻意不抢改**：
抢改会打断对方后续编辑，且若把它们「基线化」就等于**洗白真实待修项**。
⇒ 让门继续红着，**指向真实待修，比伪装成合规更有价值**。

## 改法（逐位等价，失败行为也等价）

```rust
// 现状（225-226）
u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
// 改成
u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
```

**等价性论证**：
1. 该函数在 `bytes.len() >= 24` 守卫下 ⇒ `bytes[16..20]` 恒为**长度 4** 的切片。
2. `try_into::<[u8;4]>()` 在长度 `4 == N` 时**恒 `Ok`** ⇒ 那个 `.unwrap()` 是**不可达分支**，
   今天不构成运行时风险，纯粹是给棘轮看的噪音。
3. 两个表达式产出**同一个** `[u8;4]`（同序同字节），`u32::from_be_bytes` 是纯函数
   ⇒ **对任意输入逐位相同**；**失败行为也等价**（两边都不可能 panic）。
4. ⓘ **等价性唯一依赖那个 `len >= 24` 守卫**。若日后有人把这两行提出 `if` 分支，
   版本稳健写法是 `bytes.get(16..20)`。

**风格证据**（不是引入新风格，是补齐既有风格）：
- 同函数 WEBP 三臂（VP8X/VP8L/VP8）**全是裸索引** `bytes[24]`、`u32::from(bytes[26])`，
  没有一处 `try_into`。
- 更直接：**同函数里已有同一 idiom 的现成先例** ——
  `bytes[23..26] == [0x9d, 0x01, 0x2a]`（拿切片和**数组字面量**比）。

## 范围与测试

- ⭐ **全文件生产代码只有这两处** `unwrap`。其余 20 处 `.expect(` **全在
  `#[cfg(test)] mod tests`**（模块起点在文件后段），棘轮不计 ⇒ **改这两行就干净了**，
  没有「一次改完」的第三处。
- `spritesheet_dimensions` **无直接测试**，但两处测试会**走通 PNG 分支** ⇒ 改错必红。
  **WEBP 三臂（VP8X/VP8L/VP8）零覆盖** —— 可顺手补一个头解析测试。
- ⏰ 测试命令（⛔ 不要 `--all-targets`）：
  ```sh
  sh scripts/ops/nt_mem_gate.sh; echo $?        # 先过内存闸，非 0 禁起构建
  cargo test -p neotrix-neobot --lib nt_pet
  ```
  ⓘ 包名是 `neotrix-neobot`；⚠️ **`cargo xl` 在本仓没有定义**
  （`AGENTS.md` §0/§1 把它当日常档，但 `.cargo/config.toml` 只有 `xt`/`xc`/`xf`）
  ⇒ 等价物是 `cargo xt -p neotrix-neobot nt_pet`。
- ⚠️ 复核时**别信行号**：以上行号是快照，对方若已改同一函数会再漂。
  定位命令：`rg -n 'try_into|from_be_bytes' crates/neotrix-neobot/src/nt_pet.rs`
