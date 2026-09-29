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
| 产物长度 | 97 B（32+32+32 加密的 s 段 + 3×32 + 32 tag） | **32 B** |
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
