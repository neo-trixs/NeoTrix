# B-2 Noise IK —— 官方向量已获取，根因已定位（**可执行方案**）

> 2026-09-28。前一结论「B-2 无解，需官方测试向量」**已解除**：
> 官方向量文件已取到（见 §1），根因已用向量**逐字节证伪/证实**（见 §3）。

## 1. 官方向量来源（已落地到本地）

- `https://raw.githubusercontent.com/flynn/noise/master/vectors.txt`
  （Noise Protocol Framework 官方参考实现的 `vectors.txt`；`flynn/noise`
  的 `vector_test.go` 直接消费它，是规范的事实标准向量）
- 本轮匹配到的 suite：**`Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s`**
  （正好等于本仓实现的原语组合：X25519 + ChaCha20-Poly1305 + BLAKE2s）

关键输入（hex）：

```
init_static        = 000102...1e1f
resp_static        = 010203...1f20
gen_init_ephemeral = 202122...3e3f
gen_resp_ephemeral = 414243...5f60
preshared_key      = 2176657279736563726574766572797365637265747665727973656372657421
msg_0 = 358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254d06f15f78ad0914d9715147bb5a5004b27345a838bab4aa8bc5f144afc2cf4cca972105ba526e8c92b759e028200e766f827aa12a04ecbc0bdcd9e574e007945
msg_1 = 64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484668c46d966ca4fe339f9e47fd25f68de8a
msg_2 = 6013ea114b4c4884afb82bf029f72f924bd8a32c487a15a1cef4855ba234be   (payload "wesubmarine")
msg_3 = 8a2e7119635e41a35b7e64e0adac5483b66b1a9827895124ea07d58440b654   (payload "submarineyellow")
```

## 2. 规范事实（引自 `noise_spec/noise.md`，已核实）

```
IKpsk2:  <- s   ...   -> e, es, s, ss   <- e, ee, se, psk
```

即 msg1 由 **initiator** 写，token 序 `e, es, s, ss`；
msg2 由 **responder** 写，token 序 `e, ee, se, psk`。

## 3. 根因：**不是**「es 角色/时序接反」，而是 msg1 缺 3 个 token

台账 `OPEN-TASKS §5 D-2` 记的诊断是「es 角色/时序接反，握手恒 InvalidState」。
**这个诊断是错的**（R-SCAN-1 同族：先读现场再下结论）。实测：

| | 规范 IKpsk2 msg1 | 本仓实现 `_create_message1` |
|---|---|---|
| token 序 | `e, es, s, ss` | **只有 `e`** |
| 产物长度 | 96 B（32+32+32 加密的 s 段 + 3×32 + 32 tag） | **32 B** |
| 关键操作 | 每次 DH 后 `MixKey` | **一次 `MixKey` 都没有** |

代码事实（`noise_handshake.rs:133-146`）：`_create_message1` 只做
`hash_concat(&msg)`，**没有 `mix_key`**、没有读 `remote_static_public`、没有写静态公钥。
所以 `symmetric_key` 恒为全零，状态机推不到能加密的阶段。

**旁证**：`encrypt()`（`:367`）用 `Nonce::from_bytes(&[0u8; 12])` —— nonce 恒零且
**不自增**。Noise 规范要求 nonce = `4 字节零 || LE64(n)`，`n` 每条消息 +1。
这也是「为什么实现必然偏离向量」的第二处独立缺陷。

**结论**：这是**协议实现缺失**，不是参数/时序错误。修好需要按规范补齐
msg1 的 `es/s/ss`、msg2 的 `ee/se/psk`、msg3 的 `s/se` 与 nonce 自增。

## 4. 可执行方案（三选一，推荐 A）

> ⚠️ **本节已由 §7 取代**。方案 A 已执行完毕（2-message 形态，3-message 的
> `create_message3` 等 API 已删除）。原文保留作决策留档，**不要再照它施工**。
> 其中「补 3 个 token」只是必要条件 —— 实际还挖出 5 个更深缺陷，见 §7 表。

### A. 按官方向量重写并做**向量回归**（推荐，唯一能证明正确的路）

原语已齐备，**不需引入新依赖**：

- `crypto/keys.rs:60` 已有 `diffie_hellman()`（x25519-dalek StaticSecret）
- `crypto/kdf.rs` 已有 `hkdf_blake2s` / `hkdf_blake2s_3`（= 规范 `MixKey`/`MixKeyAndHash`）
- `crypto/aead.rs` 已是 ChaCha20-Poly1305（与 `ChaChaPoly` 同算法）

需改（`noise_handshake.rs`，479 行，改动集中在 4 处）：

1. 协议名 `Noise_IKpsk2_25519_ChaCha`（25B）→ `Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s`
   （**必须**，否则与向量 suite 不符；规范要求 4 段 `pattern_DH_cipher_hash`）
2. `_create_message1`：加 `mix_key(DH(e_i, s_r))` + `hash_concat` 后写
   `EncryptAndHash(local_static_public)`
3. `_create_message2` / `_consume_message2` / `create_message3`：补
   `ee / se / MixKeyAndHash(psk)` 的正确次序
4. `encrypt`/`decrypt`：加 `nonce: u64` 字段，`Nonce::from_bytes(&[0,0,0,0, n.to_le_bytes()])`，
   每次调用后 `n += 1`

**验收（硬标准）**：新增一个 `#[test]`（**不加 `#[ignore]`**），
把 §1 的向量逐字节喂进去，断言 `msg_0..msg_3` 与官方向量**完全相等**。
外加**握手对称性**测试：两侧 chaining key 必须相等。

风险与兜底：该模块**零生产调用方**（已核实：全仓除 `crypto/mod.rs:14` 的
`pub mod` 外无任何引用），故重写不影响任何在用能力；最坏情况可整模块删。

### B. 保守降级 + 显式标注（不推荐，但成本最低）

把协议名改成能自洽的私有名（如 `NeoTrix_ZTNet_IKpsk2_v1_25519_ChaChaPoly_BLAKE2s`），
在模块头写明「**非 Noise 规范实现，仅本仓私有协议，未经第三方审计**」，
并把测试断言改为**往返自洽**（不声称符合 Noise）。
**代价**：失去互操作性，且掩盖了「实现不完整」这一事实 —— 与本仓
「假证据/ theater 代码」的历史教训相反，**不推荐**。

### C. 删除（零调用方 + 实现残缺）

既然零生产调用方，删掉 `noise_handshake.rs` + `mod.rs:14` 的 `pub mod`
可消除一个「看起来是加密通道、实际不工作」的安全假象。
**代价**：若后续要接 WireGuard 兼容，得从零重做。

## 5. 为什么此前判定「无解」

前一位的结论是「需 Noise spec 的正式测试向量交叉验证」——**前提正确**，
但当时**没有网络**，把「拿不到向量」当成了「无解」。
真相是：向量是公开的静态文件，且本仓原语组合
（25519 + ChaChaPoly + BLAKE2s）**恰好**在向量文件的 8 个 IKpsk2 suite 里有对应条目。
⇒ **「无外部依赖」这个约束是可以满足的**（只差一个网络读取，不需要引入 snow/noise-rust crate）。

## 6. 建议执行顺序

1. 先只做 **§3 的两处旁证**（msg1 长度、nonce 恒零）写成一个**失败**的
   向量测试，落盘为可复现证据（此时它必须红 —— 红是正确结果）。
2. 再按 **§4-A** 改 4 处，让它转绿。
3. 跑全量 `cargo test -p neotrix --lib -- --test-threads=4` 确认无回归。
4. 同步 `DECISIONS-2026-09-28.md`：B-2 从「⛔ 需外部输入」改为
   「✅ 已解（原语齐备 + 官方向量已获取）」，并**订正 D-2 的错误诊断**
   （es 时序接反 → msg1 缺 token）。

---

## 7. 落地结果（2026-09-29）：6 处修正 + 可执行证据

§3 只覆盖了 msg1 缺 token，**实际落地时又挖出 5 个更深的缺陷**，全部由
官方向量逐字节比对抓出。凡「已实测」均指已用独立实现复现，非手推。

| # | 缺陷 | 位置 | 症状 |
|---|---|---|---|
| 1 | **`h` / `ck` 未分离** —— 结构体只有一个 `hash` 字段兼两职 | `_NoiseHandshake.hash` | 每次 `MixHash` 污染 ck、每次 `MixKey` 污染 h。msg_0 自第 32B 起分叉 |
| 2 | **AEAD 未传 associated data** —— `encrypt_and_hash` 调 `seal()`，内部是 `Aad::empty()` | `aead.rs` / `encrypt_and_hash` | 噪声的 AD 就是握手哈希 `h`；漏传则与任何 Noise 实现无法互操作 |
| 3 | **`MixKeyAndHash` 实现错** —— 写成 `MixKey(ikm)` + `MixHash(ikm)`（两路，且混的是 psk 本身） | `mix_key_and_hash` | 规范是**三路** HKDF，`MixHash(temp_h)`。msg_1 tag 变 `311c6ddf…`，正确值 `8c46d966…` |
| 4 | **`Split` 的 `zerolen` 用了 32 个零字节** —— 应为**空切片** | `split` | k1/k2 全错，传输轮两方向都不匹配 |
| 5 | **responder 的 `se` 角色接反** —— 混 `DH(s_r,e_i)`，与 msg1 的 `es` 同值 | `_create_message2` | 规范 §10.3 responder 侧是 `DH(e_r,s_i)`。两侧 ck 立刻分叉，`Split` 永不可能一致 |
| 6 | **验收测试自身写错** —— 期望明文 `b"hellosubmarine"`(14B) | 向量测试 | 向量 `msg_2_payload` 是 `"yellowsubmarine"`(15B)，密文 31B。原断言长度就不可能成立 |

附带修正：空 prologue 的 `MixHash(&[])`、PSK 握手的 `e` token 绑定
`MixKey(e.pub)`（`noise/state.py` 有同款实现，穷举 8 变体后它是唯一解）。

### 验证证据

- `noiseprotocol`（Python 第三方实现）固定 ephemeral 后复现官方向量 **msg_0 逐字节一致**，
  可信度基准成立。
- 按修正后的逻辑做**逐行等价**模拟 → 官方向量 **4/4 全部逐字节匹配**：
  `msg_0` 96B、`msg_1` 48B、`msg_2` 31B、`msg_3` 31B，且
  `h`/`ck` 两侧收敛一致、`init.send == resp.recv`。
- 交叉验证：`se` / `psk` 组合穷举 8 变体，唯一命中为
  `se = DH(e_r,s_i)` + `psk = HKDF3 + MixHash(temp_h)`；其中
  `HKDF2+MixHash(psk)` 变体产出 `311c6ddf4e488057`，与独立审计代理
  单独测得的值一致 ⇒ 两个独立证据互证。

### 落地终态（2026-09-29 · 已收口）

**已合入主干 `feat/capability-absorb-20260828`。** 本节原为「未决：cargo 未验证」，
该记录已于同日作废 —— 保留它会让下一个 agent 以为 B-2 仍未验证（R-SCAN-3：
过期记录比没有记录更危险）。最终实测：

| 项 | 值 |
|---|---|
| 提交 | `a9d00624`（代码）+ `54e48f2e`（文档） |
| 验收测试 | `full_handshake_matches_official_vectors ... ok`（官方向量 4/4 逐字节） |
| `cargo check --tests -p neotrix` | 0 error |
| `cargo test -p neotrix --lib` | 12194 passed / 0 failed / 41 ignored |
| `cargo check -p neotrix --features ios-bridge` | Finished，0 error |
| `check-layer-deps.sh --strict` | PASS 0 new / 8 known |
| 交接 | `sessions/handoff-20260929-b2-noise-final.md` |

### 仍未决（唯一一项，需另开票）

`noise_handshake` 仍是**零生产消费者** —— 全仓仅 `crypto/mod.rs` 的 `pub mod` 声明
与文件内测试。**B-2 的范围是「实现对齐 spec 并可验证」，已达成**；但按
R-P79「外部技术必须同会话接到生产可用」，把它接进 `nt_shield_ztnet` 的实际传输
路径是**另一件事**，本会话未做，也不应悄悄算作已完成。已记入 `TODO.md`。

## 8. 方法论教训

- **L23 手推的「必要条件」远不等于「充分条件」**。本轮先手推出「空 prologue」
  与「PSK e 绑定」两条，实测二者都真、但都不够，仍剩 5 个缺陷。协议类改动
  不要停在「手推自洽」，必须逐字节对外部向量。
- **L24 权威 oracle 要先自证**。先让第三方实现复现官方向量拿到可信基准，
  再用它做穷举定位 —— 否则会出现「我的模型和我的实现一起错」，
  永远收敛不到真值。
- **L25 穷举优于辩论**。`se`/`psk` 的正确组合不是推理出来的，是 8 变体
  穷举唯一命中出来的。争议点直接枚举。
