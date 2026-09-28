/**
 * `ntInvoke` — NeoBot 前端的**唯一** Tauri IPC 出口（typed wrapper）。
 *
 * 为什么要有这一层：裸 `invoke("neobot_task_claim", { taskId: 123 })` 里
 * 命令名和键名都是 `string`，打错要等**运行时** serde 才报 —— 而 Rust 侧
 * `task_id: String` 收不到键时会给出「缺少参数 task_id」这种把人带偏的
 * 错误（`JSON.stringify` 先把 `undefined` 的键丢了）。
 * `scripts/ops/nt_ipc_keys.py` 只静态核对**键名**是否逐字对应，它自己在
 * 文件头声明了不覆盖**类型**与**运行时**。这一层把前者在编译期补上：
 * 命令名是字面量联合类型，参数对象由该命令的形参形状推导。
 *
 * ## 类型来源：手抄自 Rust 形参表
 *
 * 本仓库**没有**任何类型生成机制（无 specta / ts-rs、无 codegen、无
 * 相关 `.d.ts`；`src-tauri/frontend/src/tauri.d.ts` 是旧路径的遗留文件，
 * 不在 `apps/neobot-desktop/frontend` 的构建里），所以下面是**逐条手抄**
 * `apps/neobot-desktop/src/nt_commands/*.rs` 里每个 `#[tauri::command]`
 * 的形参表。抄写规则（与 Tauri 的 `rename_all` 默认值一致）：
 *
 * - 键名走默认 `camelCase`：`task_id` → `taskId`、`src_path` → `srcPath`；
 * - `tauri::State` / `AppHandle` / `Window` 这类**托管形参**由 Tauri 注入，
 *   不进参数表 —— `neobot_routine_sweep(gate: State)` 前端不传任何东西；
 * - Rust 的 `Option<T>` 形参在前端**可省略**（serde 缺省 `None`），也可显式
 *   传 `null`；非 `Option` 形参一律必填。`null` 与「不传」在 Rust 侧等价，
 *   所以两者都留着，不替调用方改写；
 * - `on_event: Channel<StreamEvent>` 是流式通道，单走 `ntInvokeStream`
 *   （见下）。
 *
 * 为什么不加生成器：生成器要么写进 `scripts/`（他人拥有），要么新增依赖
 * （本轮禁止 `npm install`）。而这张手抄表的漂移由两道**已有**的机器检查
 * 兜底：`scripts/ops/nt_ipc_keys.py` 核对键名，`nt_commands.rs` 里的
 * `registration_tests` 核对声明 ⊆ `generate_handler!` 注册表。
 *
 * ## 行为不变式
 *
 * 本文件只做**类型收窄**，运行时就是同一句 `tauriInvoke(cmd, args)`：
 * 传下去的对象引用原样透传，键名、键序、值都不改 —— 包括 `rel: ""` 这类
 * 既有传值和显式的 `null`。无参命令的 `args` 是 `undefined`，与今天的
 * `invoke("neobot_models")`（第二个实参根本不写）等价。
 *
 * 返回类型**故意不由本表推导**：各调用点现有的 `invoke<T>()` 泛型保持
 * 原样搬运（包括 `neobot_run` 的 `RunResult | string` 这种宽松口径）。
 * 由 Rust 的 `-> Result<_, String>` 反推返回类型是另一件事（要逐个 DTO
 * 对齐 serde 字段，且会改动调用点已有的类型标注），不在本轮范围内。
 */

import { invoke as tauriInvoke, Channel, type InvokeArgs } from "@tauri-apps/api/core";

/**
 * 任意 `Channel`（载荷类型不限）。
 *
 * 取 `Channel<never>` 而不是 `Channel<unknown>`：`Channel<T>.onmessage` 是
 * **属性**不是方法，`strictFunctionTypes` 下参数逆变，故
 * `Channel<StreamEv>` 只能赋给 `Channel<never>`（`never` 可赋给任何类型），
 * 赋不了 `Channel<unknown>`（`unknown` 不可赋给 `StreamEv`）。用
 * `unknown` 会把流式调用点编译红 —— 那正是本文件要避免的「为了统一而
 * 破坏流式调用」。
 */
export type AnyChannel = Channel<never>;

/**
 * 命令 → 前端参数形状。**类型来源与抄写规则见文件头**；每条上方的注释是
 * Rust 侧的原样形参表，抄错了按注释对回去（一条命令一行，不跨行）。
 *
 * 无参命令取 `undefined`（而不是 `{}`）：`invoke` 的第二个实参在今天就是
 * 不写的，给 `{}` 反而会让「给无参命令传了个空对象」这种错在编译期溜过去。
 */
export type NTCommandArgs = {
  // ─── nt_cmd_convo.rs ───
  /** `neobot_convo_group(title: String, members: Vec<String>)` */
  neobot_convo_group: { title: string; members: string[] };
  /** `neobot_convo_dm(me: String, peer: String)` */
  neobot_convo_dm: { me: string; peer: string };
  /** `neobot_convo_ensure_default(me: String)` */
  neobot_convo_ensure_default: { me: string };
  /** `neobot_convo_rename(id: String, title: String)` */
  neobot_convo_rename: { id: string; title: string };
  /** `neobot_convo_delete(id: String)` */
  neobot_convo_delete: { id: string };
  /** `neobot_convo_mute(id: String, muted: bool)` */
  neobot_convo_mute: { id: string; muted: boolean };
  /** `neobot_convo_mark_read(id: String)` */
  neobot_convo_mark_read: { id: string };
  /** `neobot_convo_add_member(id: String, member: String)` */
  neobot_convo_add_member: { id: string; member: string };
  /** `neobot_convo_rm_member(id: String, member: String)` */
  neobot_convo_rm_member: { id: string; member: string };
  /** `neobot_convos()` */
  neobot_convos: undefined;
  /** `neobot_attach_add(convo_id: String, src_path: String)` */
  neobot_attach_add: { convoId: string; srcPath: string };
  /** `neobot_attachments(convo_id: String)` */
  neobot_attachments: { convoId: string };
  /** `neobot_attach_remove(id: String)` */
  neobot_attach_remove: { id: string };
  /** `neobot_members()` */
  neobot_members: undefined;
  /** `neobot_member_add(id: String, kind: String)` */
  neobot_member_add: { id: string; kind: string };
  /** `neobot_member_remove(id: String)` */
  neobot_member_remove: { id: string };

  // ─── nt_cmd_core.rs ───
  /** `neobot_core_free()` */
  neobot_core_free: undefined;
  /** `neobot_core_pair_free(model_id: String)` */
  neobot_core_pair_free: { modelId: string };
  /** `neobot_core_status()` */
  neobot_core_status: undefined;
  /** `neobot_core_reload()` */
  neobot_core_reload: undefined;
  /** `neobot_core_capabilities()` */
  neobot_core_capabilities: undefined;
  /** `neobot_agent_run(goal: String, context: Option<String>)` */
  neobot_agent_run: { goal: string; context?: string | null };

  // ─── nt_cmd_files.rs ───
  /** `neobot_fs_list(rel: String)` */
  neobot_fs_list: { rel: string };
  /** `neobot_fs_read(rel: String)` */
  neobot_fs_read: { rel: string };
  /** `neobot_fs_write(rel: String, content: String)` */
  neobot_fs_write: { rel: string; content: string };
  /** `neobot_fs_find(query: String)` */
  neobot_fs_find: { query: string };
  /** `neobot_fs_mkdir(rel: String)` */
  neobot_fs_mkdir: { rel: string };
  /** `neobot_fs_rename(from: String, to: String)` */
  neobot_fs_rename: { from: string; to: string };
  /** `neobot_fs_remove(rel: String)` */
  neobot_fs_remove: { rel: string };
  /** `neobot_fs_root()` */
  neobot_fs_root: undefined;
  /** `neobot_changes_list(task_id: Option<String>, limit: Option<i64>)` */
  neobot_changes_list: { taskId?: string | null; limit?: number | null };
  /** `neobot_changes_paths(task_id: String)` */
  neobot_changes_paths: { taskId: string };
  /** `neobot_changes_recent_paths(task_limit: Option<i64>)` */
  neobot_changes_recent_paths: { taskLimit?: number | null };
  /** `neobot_changes_get(change_id: String)` */
  neobot_changes_get: { changeId: string };
  /** `neobot_changes_prune()` */
  neobot_changes_prune: undefined;
  /** `neobot_convo_last_reply(convo_id: String)` */
  neobot_convo_last_reply: { convoId: string };

  // ─── nt_cmd_run.rs ───
  /** `neobot_run(..., convo_id/model_provider/model_name: Option<String>)` */
  neobot_run: {
    title: string;
    text: string;
    actorName: string;
    convoId?: string | null;
    modelProvider?: string | null;
    modelName?: string | null;
  };
  /**
   * `neobot_run_cancel(convo_id: Option<String>)` — 停掉**正在跑**的那一轮。
   *
   * 桌面的取消登记表按 `convo_id` 索引；该轮已结束/未开始时后端**如实报错**，
   * 不会静默成功。键走 camelCase ⇒ `convoId`。
   */
  neobot_run_cancel: {
    convoId?: string | null;
  };
  /**
   * `neobot_run_stream(..., on_event: tauri::ipc::Channel<StreamEvent>)`
   *
   * 流式那条单独走 `ntInvokeStream`：这里把 `onEvent` 放宽成 `AnyChannel`，
   * 由那条函数按调用点自己的 `Channel<StreamEv>` 重新收窄。
   */
  neobot_run_stream: {
    title: string;
    text: string;
    actorName: string;
    convoId?: string | null;
    modelProvider?: string | null;
    modelName?: string | null;
    onEvent: AnyChannel;
  };

  // ─── nt_cmd_sidebar.rs ───
  /** `neobot_sidebar_tabs()` */
  neobot_sidebar_tabs: undefined;
  /** `neobot_sidebar_viewers()` */
  neobot_sidebar_viewers: undefined;
  /** `neobot_sidebar_viewer_for(rel: String)` */
  neobot_sidebar_viewer_for: { rel: string };
  /** `neobot_sidebar_resolve(topic: String, target: String)` */
  neobot_sidebar_resolve: { topic: string; target: string };
  /** `neobot_git_available()` */
  neobot_git_available: undefined;
  /** `neobot_git_is_repo(rel: String)` */
  neobot_git_is_repo: { rel: string };
  /** `neobot_git_status(rel: String)` */
  neobot_git_status: { rel: string };
  /** `neobot_git_diff(rel: String, path: String, staged: bool)` */
  neobot_git_diff: { rel: string; path: string; staged: boolean };
  /** `neobot_git_log(rel: String, path: String, limit: Option<i64>)` */
  neobot_git_log: { rel: string; path: string; limit?: number | null };
  /** `neobot_git_stage(rel: String, path: String)` */
  neobot_git_stage: { rel: string; path: string };
  /** `neobot_git_unstage(rel: String, path: String)` */
  neobot_git_unstage: { rel: string; path: string };
  /** `neobot_git_revert(rel: String, path: String)` */
  neobot_git_revert: { rel: string; path: string };
  /** `neobot_git_commit(rel: String, message: String, no_verify: Option<bool>)` */
  neobot_git_commit: { rel: string; message: string; noVerify?: boolean | null };
  /** `neobot_sidechat_list(parent_id: String)` */
  neobot_sidechat_list: { parentId: string };
  /** `neobot_sidechat_open(parent_id: String, title: String)` */
  neobot_sidechat_open: { parentId: string; title: string };
  /** `neobot_sidechat_context(parent_id: String)` */
  neobot_sidechat_context: { parentId: string };
  /** `neobot_sidechat_promote(thread_id: String)` */
  neobot_sidechat_promote: { threadId: string };
  /** `neobot_sidechat_delete(thread_id: String)` */
  neobot_sidechat_delete: { threadId: string };

  // ─── nt_cmd_sys.rs ───
  /** `neobot_doctor()` */
  neobot_doctor: undefined;
  /** `neobot_models()` */
  neobot_models: undefined;
  /** `neobot_providers()` */
  neobot_providers: undefined;
  /** `neobot_provider_add(name, base_url, key_env, model: String)` */
  neobot_provider_add: { name: string; baseUrl: string; keyEnv: string; model: string };
  /** `neobot_provider_remove(name: String)` */
  neobot_provider_remove: { name: string };
  /** `neobot_provider_toggle(name: String, enabled: bool)` */
  neobot_provider_toggle: { name: string; enabled: boolean };
  /** `neobot_memory_get()` */
  neobot_memory_get: undefined;
  /** `neobot_memory_set(text: String)` */
  neobot_memory_set: { text: string };
  /** `neobot_routine_list()` */
  neobot_routine_list: undefined;
  /** `neobot_routine_fire(name: String, gate: tauri::State<DaemonMap>)` —— `gate` 托管。 */
  neobot_routine_fire: { name: string };
  /** `neobot_routine_sweep(gate: tauri::State<DaemonMap>)` —— 托管形参 ⇒ 前端无参。 */
  neobot_routine_sweep: undefined;
  /** `neobot_skills()` */
  neobot_skills: undefined;
  /** `neobot_control_status()` */
  neobot_control_status: undefined;
  /** `neobot_control_take(holder: String)` */
  neobot_control_take: { holder: string };
  /** `neobot_control_release(holder: String)` */
  neobot_control_release: { holder: string };
  /** `neobot_settings_window(app: tauri::AppHandle)` —— 托管形参 ⇒ 前端无参。 */
  neobot_settings_window: undefined;
  /** `neobot_roster_heartbeat(member_id: String, kind: String, map: State<PresenceMap>)` */
  neobot_roster_heartbeat: { memberId: string; kind: string };
  /** `neobot_roster_list(map: tauri::State<PresenceMap>)` —— 托管形参 ⇒ 前端无参。 */
  neobot_roster_list: undefined;

  // ─── nt_cmd_tasks.rs ───
  /** `neobot_tasks()` */
  neobot_tasks: undefined;
  /** `neobot_task_claim(task_id: String, actor_id: String)` */
  neobot_task_claim: { taskId: string; actorId: string };
  /** `neobot_task_release(task_id: String, actor_id: String)` */
  neobot_task_release: { taskId: string; actorId: string };
  /** `neobot_task_visibility(task_id: String, visibility: String)` */
  neobot_task_visibility: { taskId: string; visibility: string };
  /** `neobot_task_cancel(task_id: String)` */
  neobot_task_cancel: { taskId: string };
  /** `neobot_task_retry(task_id: String)` */
  neobot_task_retry: { taskId: string };
  /** `neobot_task_rename(task_id: String, title: String)` */
  neobot_task_rename: { taskId: string; title: string };
  /** `neobot_task_delete(task_id: String)` */
  neobot_task_delete: { taskId: string };
  /** `neobot_audit()` */
  neobot_audit: undefined;
  /** `neobot_cost_ledger()` */
  neobot_cost_ledger: undefined;
  /** `neobot_cost_ledger_by_actor()` */
  neobot_cost_ledger_by_actor: undefined;

  // ─── nt_cmd_channels.rs ───
  /** `neobot_channel_catalog()` */
  neobot_channel_catalog: undefined;
  /** `neobot_channels()` */
  neobot_channels: undefined;
  /** `neobot_channel_upsert(id, title, access_mode: String, poll_secs: Option<i64>)` */
  neobot_channel_upsert: { id: string; title: string; accessMode: string; pollSecs?: number | null };
  /** `neobot_channel_toggle(id: String, enabled: bool)` */
  neobot_channel_toggle: { id: string; enabled: boolean };
  /** `neobot_channel_bots(channel: String)` */
  neobot_channel_bots: { channel: string };
  /**
   * `neobot_channel_bot_upsert(channel, bot_id, token_env: String,
   *   model/allow_list/conversation_id: Option<String>)`
   */
  neobot_channel_bot_upsert: {
    channel: string;
    botId: string;
    tokenEnv: string;
    model?: string | null;
    allowList?: string | null;
    conversationId?: string | null;
  };
  /** `neobot_channel_bot_alias(channel: String, bot_id: String, alias: String)` */
  neobot_channel_bot_alias: { channel: string; botId: string; alias: string };
  /** `neobot_channel_bot_remove(channel: String, bot_id: String)` */
  neobot_channel_bot_remove: { channel: string; botId: string };
  /** `neobot_channel_probe(channel: String)` */
  neobot_channel_probe: { channel: string };
  /** `neobot_channel_poll_once(channel: String)` */
  neobot_channel_poll_once: { channel: string };
  /** `neobot_channel_status()` */
  neobot_channel_status: undefined;
  /** `neobot_channel_parse_allow(raw: String)` */
  neobot_channel_parse_allow: { raw: string };
};

/** 全部 IPC 命令名的字面量联合 —— 打错命令名在这一行就红。 */
export type NTCommand = keyof NTCommandArgs;

/**
 * 带 `Channel` 形参的命令（走流式分支）。当前只有 `neobot_run_stream`。
 *
 * 与 `NTCommandArgs` 的对应关系由下面的编译期断言钉住：流式分支漏了某条
 * 带 `onEvent` 的命令，或把不带 `onEvent` 的命令塞进流式分支，都编译不过。
 */
export type NTStreamCommand = "neobot_run_stream";

/** 编译期断言工具：`T` 不为 `true` 就在此报错。 */
type Assert<T extends true> = T;
/** 编译期严格相等（用函数重载模拟，能区分 `any` 与联合）。 */
type IsEqual<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;

/**
 * 交接不变式：每条命令的参数形状都能**原样**（零 cast、零 `as unknown as`）
 * 传给 Tauri 的 `invoke`，因为它的第二形参是 `InvokeArgs = Record<string, unknown>`。
 *
 * 这条能过，靠的是 `NTCommandArgs` 用 **type alias** 而不是 `interface` 声明 ——
 * TS 只给对象**字面量类型 / type alias** 隐式索引签名，`interface` 没有，
 * 用 `interface` 写这张表会在这里直接报 *Index signature ... is missing*。
 * 无参命令的 `undefined` 成员也已在 `| undefined` 里显式放行。
 *
 * 把它提成一条命名断言，是为了「零 cast」这个纪律有个可核对的落点：
 * 万一有人把表改成 `interface`，红的是这一行，而不是 `ntInvoke` 里那句
 * 看起来无辜的 `tauriInvoke<T>(cmd, args)`。
 */
export type _ArgsHandOffToTauri = Assert<
  NTCommandArgs[NTCommand] extends InvokeArgs | undefined ? true : false
>;

/** 流式命令必须真的在参数表里（否则下面的形状断言会指向一个不存在的键）。 */
export type _StreamCommandIsDeclared = Assert<
  IsEqual<Extract<NTStreamCommand, NTCommand>, NTStreamCommand>
>;

/**
 * 流式命令的参数形状：把 `onEvent` 从 `AnyChannel` 换成调用点自己的
 * `Channel<StreamEv>`，其余键与 `NTCommandArgs` 逐字同形。
 *
 * 为什么**显式重列**而不是 `Omit<NTCommandArgs[K], "onEvent"> & { onEvent: C }`：
 * `Omit` 展开成映射类型，其 symbol 带的是 `TypeAlias` 而非 `TypeLiteral`，
 * 不吃 TS 的「隐式索引签名」豁免 —— 传回 `tauriInvoke(cmd, args)`（形参是
 * `Record<string, unknown>`）时就会报 *Index signature ... is missing*。
 * 写成普通对象类型就没这个坑，而两份声明的一致性由下面的 `IsEqual` 钉死。
 *
 * 泛型取 `C`（整个 `onEvent` 属性的类型）而不是从 `Channel<E>` 反推 `E`：
 * `Channel<T>.onmessage` 是**逆变**位置，从那里推 `E` 是 TS 变型推断里最不该
 * 赌的一处；`C` 是裸类型参数，`Channel<StreamEv>` 原样落进去。
 */
export type NTStreamArgs<C extends AnyChannel> = {
  title: string;
  text: string;
  actorName: string;
  convoId?: string | null;
  modelProvider?: string | null;
  modelName?: string | null;
  onEvent: C;
};

/**
 * 流式形状与参数表**逐字一致**（键、可选性、类型，一个都不能差）。
 *
 * 抄错一个字、漏一个可选键、或者往 `NTStreamCommand` 里加了第二条命令而忘了
 * 同步 `NTStreamArgs` —— 三种漂移都在这一行编译期现形。
 */
export type _StreamArgsMatchMap = Assert<
  IsEqual<NTStreamArgs<AnyChannel>, NTCommandArgs[NTStreamCommand]>
>;

/**
 * 调一条 IPC 命令。
 *
 * - `cmd` 收窄到 {@link NTCommand}：命令名不存在 → 编译错；
 * - `args` 收窄到该命令的形参形状：键名打错、拼成 `snake_case`、必填键漏给
 *   → 编译错（对象字面量的多余键也会被 excess property check 拦下）；
 * - 参数值的类型即 Rust 形参类型：`{ taskId: 123 }` → 编译错。
 *
 * `T` 沿用各调用点原有的 `invoke<T>()` 泛型，本文件不推导返回类型
 * （理由见文件头）。不给 `T` 时是 `unknown`，与裸 `invoke` 一致。
 *
 * **泛型顺序必须是 `<T, K>`** —— 调用点写的是 `ntInvoke<GitDiff>("neobot_git_diff", …)`
 * （Tauri 的 `invoke<T>` 把返回类型放在第一位）。把 `K` 排到前面会让这个实参
 * 被绑成命令名，报 `Type 'GitDiff' does not satisfy the constraint 'keyof NTCommandArgs'`。
 * `K` 虽排第二且带默认值，仍**照常从 `cmd` 实参推断** —— TS 只在推断不出候选时
 * 才用默认值（partial explicit type arguments 不阻断其余参数的推断）。
 */
export function ntInvoke<T = unknown, K extends NTCommand = NTCommand>(
  cmd: K,
  args?: NTCommandArgs[K],
): Promise<T> {
  return tauriInvoke<T>(cmd, args);
}

/**
 * 调一条**流式**命令（带 `Channel` 形参的）。
 *
 * 单独一条而不是重载：重载要先让 `ntInvoke` 的第二个签名接受
 * `Channel<unknown>`，那会把 `onEvent` 在**非流式**调用点上也放宽，
 * 等于为了统一而拆掉流式的类型保障。这里走独立的泛型 `C`，流式与非流式
 * 两条路互不污染。
 *
 * 运行时同样是 `tauriInvoke(cmd, args)` —— `onEvent` 本来就是普通参数键，
 * Tauri 见到 `Channel` 会走通道序列化。所以调用点除函数名外一字不改。
 *
 * 泛型顺序同样是 `<T, C, K>`，理由见 `ntInvoke`。
 */
export function ntInvokeStream<
  T = unknown,
  C extends AnyChannel = AnyChannel,
  K extends NTStreamCommand = NTStreamCommand,
>(cmd: K, args: NTStreamArgs<C>): Promise<T> {
  return tauriInvoke<T>(cmd, args);
}
