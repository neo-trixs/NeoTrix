# 裁定：`CookieEntry` **同名不同物**，**不收敛**（2026-10-03）

> ⛔ 差点重复本会话**自己记录过**的错误：`KnowledgeSource`（工具名 40 / 理论名 14）、
> `_JudgeConfig` 与 `JudgeConfig`。**同名 ≠ 同一符号。**

## 一、发现路径（这次是复核，不是碰巧）
上一笔给 `browser_engine/cookies.rs` 的 `CookieEntry` 加了 redact 的 `Debug`。
本轮按**判据 1**（按**类型名逐个文件**读）复核 ⇒ `rg -l 'CookieJar|CookieEntry'`
命中 **10 个文件** ⇒ 撞见 `l1_action/nt_media/auth.rs` 里的**第二个**同名类型。

⇒ **若只查自己改过的那个文件，这个泄露面会永久留存。**

## 二、形状实测

| | A `browser_engine::CookieEntry` | B `nt_media::auth::CookieEntry` |
|---|---|---|
| 字段 | `{name, value, secure_only}` | `{name, value, domain, path, expires, secure}` |
| 可见性 | `pub(crate)` | `pub` |
| 域名存法 | **存成 map 的 key**：`HashMap<String, Vec<CookieEntry>>` | **每条自带** `domain` 字段 |
| 序列化 | 无 `Serialize` | `Serialize + Deserialize`（**文件持久化**） |
| `secure` 语义 | `secure_only: bool`（取 Cookie 头用） | `secure: bool`（带 `#[serde(default)]`） |

## 三、⭐ 裁定：**不收敛**
两者**建模的是不同东西**：
· **A** = **按 host 分桶的取用态** —— 域名是**桶的 key**，故不进字段
· **B** = **可落盘的 cookie 记录** —— 必须**自包含**才能序列化

⇒ 强行合并要引入 `Option<domain>` 之类的空态，且让A背上B 的持久化义务
⇒ **代价大于收益** ⇒ 不做。

## 四、⭐ 真正的残留风险不是「重复」，而是「同名」
两个类型同名 ⇒ **搜类型名会一次命中两处** ⇒ 复核时必须**逐个文件读**，
否则会像上一笔那样**只修一个、漏掉另一个**。

⇒ 已采取的措施（**两处都做**）：
· A：`browser_engine/cookies.rs` 手工 `Debug`，`value` ⇒ `<redacted>`
  （`redacted_debug_tests` 2 条）
· B：`nt_media/auth.rs` 手工 `Debug`，`value` ⇒ `<redacted>`
  （`cookie_debug_redaction_tests` 2 条，含「`Serialize` **不**被误伤」的边界锁）

⇒ ⭐ 结论：**该收敛的是「两处都安全」，不是「两个类型合并」。**

## 五、⛔ 未做
· 未合并两个类型（理由见§三）
· 未把 redact 抽成共享实现 —— ⭐ 考虑过，但**两处形状不同**，
  共用同一 `fmt` 反而要泛型化字段 ⇒ **不值得**；且两份独立的反向锁
  能在任一处被单独破坏时立刻发现，**比共享实现更敏感**。
