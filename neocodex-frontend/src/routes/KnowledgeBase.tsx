/* ════════════════════════════════════════════
   routes/KnowledgeBase.tsx — 知识库页 (Phase 1 mock-first)

   吸收 Cherry Studio / LobeChat KB 页模式：
   - 顶部：返回 + 标题 + 搜索 + 新建
   - 库卡片网格：文档数/切片数/更新时间，卡片操作（打开/重命名/删除）
   - 详情抽屉：库内文档列表（Phase 2 深化）

   数据经 stores/kb.ts 的 dataSource seam — 后端接线时组件零改动。
   ════════════════════════════════════════════ */
import { createSignal, For, Show, onMount } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { ArrowLeft, Search, Plus, Loader2, Database, FileText, Pencil, Trash2, RefreshCw, Layers } from 'lucide-solid'
import { clsx } from 'clsx'
import { createKbStore, type KbLibrary } from '../stores/kb'
import { ConfirmModal, type ModalReq } from '../components/ConfirmModal'
export function KnowledgeBase() {
  const navigate = useNavigate()
  const kb = createKbStore()

  const [showCreate, setShowCreate] = createSignal(false)
  const [newName, setNewName] = createSignal('')
  const [newDesc, setNewDesc] = createSignal('')
  const [renameTarget, setRenameTarget] = createSignal<KbLibrary | null>(null)
  const [renameVal, setRenameVal] = createSignal('')
  const [deleteTarget, setDeleteTarget] = createSignal<KbLibrary | null>(null)
  const [showAddDoc, setShowAddDoc] = createSignal(false)
  const [newDocTitle, setNewDocTitle] = createSignal('')
  const [newDocText, setNewDocText] = createSignal('')
  const [docDeleting, setDocDeleting] = createSignal<string | null>(null)

  onMount(() => void kb.refresh())

  function relTime(ts: number): string {
    const diff = Date.now() - ts
    if (diff < 60_000) return '刚刚'
    if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} 分钟前`
    if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)} 小时前`
    return `${Math.floor(diff / 86_400_000)} 天前`
  }

  async function submitCreate() {
    const name = newName().trim()
    if (!name) return
    try {
      await kb.create(name, newDesc().trim())
      setNewName('')
      setNewDesc('')
      setShowCreate(false)
    } catch {
      /* store 已置 error */
    }
  }

  async function confirmRename() {
    const t = renameTarget()
    const name = renameVal().trim()
    if (!t || !name || name === t.name) {
      setRenameTarget(null)
      return
    }
    await kb.rename(t.id, name)
    setRenameTarget(null)
  }

  function askDelete(lib: KbLibrary) {
    setDeleteTarget(lib)
  }

  function deleteReq(): ModalReq | null {
    const t = deleteTarget()
    if (!t) return null
    return {
      title: '删除知识库',
      message: `确定删除「${t.name}」？${t.docCount} 个文档与索引将被移除。`,
      confirmLabel: '删除',
      danger: true,
    }
  }

  return (
    <div class="min-h-screen mac-safe bg-bg-primary text-text-primary flex flex-col">
      {/* 顶栏 */}
      <header class="flex items-center gap-3 px-5 h-12 border-b border-border-primary/40 shrink-0">
        <button
          class="flex items-center gap-1.5 text-13px text-text-muted hover:text-text-primary transition-colors"
          onClick={() => navigate('/chat')}
          aria-label="返回对话"
        >
          <ArrowLeft class="w-4 h-4" />
          对话
        </button>
        <h1 class="text-14px font-semibold flex items-center gap-1.5">
          <Database class="w-4 h-4 text-nt-io-600" />
          知识库
        </h1>
        <div class="flex-1" />
        <div class="relative">
          <Search class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
          <input
            type="search"
            value={kb.keyword()}
            onInput={(e) => kb.setKeyword(e.currentTarget.value)}
            placeholder="搜索知识库"
            aria-label="搜索知识库"
            class="w-56 h-8 pl-8 pr-3 rounded-lg text-13px bg-bg-secondary border border-border-primary/50 focus:border-nt-io-500 focus:outline-none placeholder:text-text-muted"
          />
        </div>
        <button
          class="flex items-center gap-1.5 h-8 px-3 rounded-lg text-13px font-medium bg-nt-io-500 text-white hover:bg-nt-io-600 transition-colors"
          onClick={() => setShowCreate(true)}
          aria-label="新建知识库"
        >
          <Plus class="w-4 h-4" />
          新建
        </button>
      </header>

      {/* 内容区 */}
      <main class="flex-1 overflow-y-auto p-5">
        <Show when={!kb.loading()} fallback={
          <div class="flex items-center justify-center py-24 text-text-muted" role="status">
            <Loader2 class="w-5 h-5 animate-spin mr-2" /> 加载中…
          </div>
        }>
          <Show when={!kb.error()} fallback={
            <div class="text-center py-24">
              <p class="text-13px text-text-muted mb-3">{kb.error()}</p>
              <button class="text-13px text-nt-io-600 hover:underline" onClick={() => void kb.refresh()}>
                重试
              </button>
            </div>
          }>
            <Show when={kb.filtered().length > 0} fallback={
              <div class="text-center py-24 text-text-muted text-13px">
                {kb.keyword() ? '没有匹配的知识库' : '还没有知识库 — 点右上角「新建」创建第一个'}
              </div>
            }>
              <div class="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-4 max-w-5xl mx-auto">
                <For each={kb.filtered()}>
                  {(lib) => (
                    <article
                      class={clsx(
                        'group relative rounded-xl border bg-bg-secondary p-4 hover:shadow-sm transition-all cursor-pointer',
                        kb.activeLibraryId() === lib.id ? 'border-nt-io-500/60 ring-1 ring-nt-io-500/30' : 'border-border-primary/50 hover:border-nt-io-500/50',
                      )}
                      onClick={() => kb.setActiveLibraryId(kb.activeLibraryId() === lib.id ? null : lib.id)}
                      aria-label={`知识库 ${lib.name}`}
                    >
                      <div class="flex items-start gap-2.5">
                        <div class="w-9 h-9 rounded-lg bg-nt-io-500/10 flex items-center justify-center shrink-0">
                          <Layers class="w-4.5 h-4.5 text-nt-io-600" />
                        </div>
                        <div class="min-w-0 flex-1">
                          <h2 class="text-13px font-semibold truncate" title={lib.name}>{lib.name}</h2>
                          <p class="text-12px text-text-muted line-clamp-2 mt-0.5 min-h-[32px]" title={lib.description}>
                            {lib.description || '暂无描述'}
                          </p>
                        </div>
                      </div>
                      <div class="flex items-center gap-3 mt-3 text-11px text-text-muted">
                        <span class="inline-flex items-center gap-1"><FileText class="w-3 h-3" />{lib.docCount} 文档</span>
                        <span class="inline-flex items-center gap-1"><Database class="w-3 h-3" />{lib.chunkCount} 切片</span>
                        <span class="ml-auto">{relTime(lib.updatedAt)}</span>
                      </div>
                      {/* 卡片操作 */}
                      <div class="absolute top-2 right-2 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                        <button
                          class="p-1.5 rounded-md hover:bg-white/60 text-text-muted hover:text-text-primary"
                          aria-label={`重命名 ${lib.name}`}
                          title="重命名"
                          onClick={(e) => { e.stopPropagation(); setRenameTarget(lib); setRenameVal(lib.name) }}
                        >
                          <Pencil class="w-3.5 h-3.5" />
                        </button>
                        <button
                          class="p-1.5 rounded-md hover:bg-red-50 text-text-muted hover:text-red-500"
                          aria-label={`删除 ${lib.name}`}
                          title="删除"
                          onClick={(e) => { e.stopPropagation(); askDelete(lib) }}
                        >
                          <Trash2 class="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </article>
                  )}
                </For>
              </div>

              {/* ── B2: 选中库的文档列表 (kb_doc_list/ingest/delete 直连) ── */}
              <Show when={kb.activeLibraryId()}>
                <section class="max-w-5xl mx-auto mt-6 rounded-xl border border-border-primary/50 bg-bg-secondary p-4" aria-label="文档列表">
                  <div class="flex items-center gap-2 mb-3">
                    <h2 class="text-13px font-semibold">文档</h2>
                    <span class="text-11px text-text-muted">{kb.docs().length} 个</span>
                    <div class="flex-1" />
                    <button
                      class="flex items-center gap-1.5 h-7 px-2.5 rounded-lg text-12px font-medium bg-nt-io-500 text-white hover:bg-nt-io-600 transition-colors"
                      onClick={() => setShowAddDoc(true)}
                      aria-label="添加文档"
                    >
                      <Plus class="w-3.5 h-3.5" />
                      入库
                    </button>
                  </div>
                  <Show when={!kb.docsLoading()} fallback={
                    <p class="text-12px text-text-muted py-4 text-center" role="status">加载文档…</p>
                  }>
                    <Show when={kb.docs().length > 0} fallback={
                      <p class="text-12px text-text-muted py-4 text-center border border-dashed border-border-primary/50 rounded-lg">
                        库内暂无文档 — 点「入库」添加第一篇
                      </p>
                    }>
                      <ul class="space-y-2">
                        <For each={kb.docs()}>
                          {(doc) => (
                            <li class="flex items-center gap-3 rounded-lg border border-border-primary/40 px-3 py-2">
                              <FileText class="w-4 h-4 text-text-muted shrink-0" />
                              <span class="text-13px font-medium truncate flex-1">{doc.title}</span>
                              <span class={clsx(
                                'text-10px px-1.5 py-0.5 rounded-full font-medium',
                                doc.status === 'ready' ? 'bg-emerald-50 text-emerald-700' : 'bg-amber-50 text-amber-700',
                              )}>
                                {doc.status === 'ready' ? '已索引' : '索引中'}
                              </span>
                              <span class="text-11px text-text-muted shrink-0">{doc.sizeKb} KB</span>
                              <button
                                class="p-1 rounded-md hover:bg-red-50 text-text-muted hover:text-red-500 disabled:opacity-40"
                                aria-label={`删除文档 ${doc.title}`}
                                title="删除"
                                disabled={docDeleting() === doc.id}
                                onClick={() => {
                                  setDocDeleting(doc.id)
                                  void kb.removeDoc(doc.id).finally(() => setDocDeleting(null))
                                }}
                              >
                                <Trash2 class="w-3.5 h-3.5" />
                              </button>
                            </li>
                          )}
                        </For>
                      </ul>
                    </Show>
                  </Show>
                </section>
              </Show>
            </Show>
          </Show>
        </Show>
      </main>

      {/* 新建弹层 */}
      <Show when={showCreate()}>
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/30" onClick={() => setShowCreate(false)}>
          <div
            class="w-[380px] rounded-xl bg-bg-primary border border-border-primary shadow-lg p-5"
            onClick={(e) => e.stopPropagation()}
            role="dialog"
            aria-label="新建知识库"
          >
            <h2 class="text-14px font-semibold mb-4">新建知识库</h2>
            <label class="block text-12px text-text-muted mb-1">名称</label>
            <input
              class="w-full h-9 px-3 rounded-lg text-13px bg-bg-secondary border border-border-primary/50 focus:border-nt-io-500 focus:outline-none mb-3"
              value={newName()}
              onInput={(e) => setNewName(e.currentTarget.value)}
              onKeyDown={(e) => e.key === 'Enter' && void submitCreate()}
              autofocus
              aria-label="知识库名称"
            />
            <label class="block text-12px text-text-muted mb-1">描述</label>
            <textarea
              class="w-full h-20 px-3 py-2 rounded-lg text-13px bg-bg-secondary border border-border-primary/50 focus:border-nt-io-500 focus:outline-none resize-none"
              value={newDesc()}
              onInput={(e) => setNewDesc(e.currentTarget.value)}
              aria-label="知识库描述"
            />
            <div class="flex justify-end gap-2 mt-4">
              <button class="h-8 px-3 rounded-lg text-13px text-text-muted hover:text-text-primary" onClick={() => setShowCreate(false)}>
                取消
              </button>
              <button
                class={clsx('h-8 px-4 rounded-lg text-13px font-medium text-white transition-colors',
                  newName().trim() ? 'bg-nt-io-500 hover:bg-nt-io-600' : 'bg-nt-io-500/40 cursor-not-allowed')}
                disabled={!newName().trim()}
                onClick={() => void submitCreate()}
              >
                创建
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* 重命名弹层 */}
      <Show when={renameTarget()}>
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/30" onClick={() => setRenameTarget(null)}>
          <div
            class="w-[360px] rounded-xl bg-bg-primary border border-border-primary shadow-lg p-5"
            onClick={(e) => e.stopPropagation()}
            role="dialog"
            aria-label="重命名知识库"
          >
            <h2 class="text-14px font-semibold mb-4">重命名知识库</h2>
            <input
              class="w-full h-9 px-3 rounded-lg text-13px bg-bg-secondary border border-border-primary/50 focus:border-nt-io-500 focus:outline-none"
              value={renameVal()}
              onInput={(e) => setRenameVal(e.currentTarget.value)}
              onKeyDown={(e) => e.key === 'Enter' && void confirmRename()}
              autofocus
              aria-label="新名称"
            />
            <div class="flex justify-end gap-2 mt-4">
              <button class="h-8 px-3 rounded-lg text-13px text-text-muted hover:text-text-primary" onClick={() => setRenameTarget(null)}>
                取消
              </button>
              <button class="h-8 px-4 rounded-lg text-13px font-medium bg-nt-io-500 text-white hover:bg-nt-io-600" onClick={() => void confirmRename()}>
                保存
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* 入库文档弹层 */}
      <Show when={showAddDoc()}>
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/30" onClick={() => setShowAddDoc(false)}>
          <div
            class="w-[440px] rounded-xl bg-bg-primary border border-border-primary shadow-lg p-5"
            onClick={(e) => e.stopPropagation()}
            role="dialog"
            aria-label="入库文档"
          >
            <h2 class="text-14px font-semibold mb-4">入库文档</h2>
            <label class="block text-12px text-text-muted mb-1">标题</label>
            <input
              class="w-full h-9 px-3 rounded-lg text-13px bg-bg-secondary border border-border-primary/50 focus:border-nt-io-500 focus:outline-none mb-3"
              value={newDocTitle()}
              onInput={(e) => setNewDocTitle(e.currentTarget.value)}
              aria-label="文档标题"
              autofocus
            />
            <label class="block text-12px text-text-muted mb-1">正文 (自动切片入 FTS 索引)</label>
            <textarea
              class="w-full h-32 px-3 py-2 rounded-lg text-13px bg-bg-secondary border border-border-primary/50 focus:border-nt-io-500 focus:outline-none resize-none font-mono"
              value={newDocText()}
              onInput={(e) => setNewDocText(e.currentTarget.value)}
              aria-label="文档正文"
            />
            <div class="flex justify-end gap-2 mt-4">
              <button class="h-8 px-3 rounded-lg text-13px text-text-muted hover:text-text-primary" onClick={() => setShowAddDoc(false)}>
                取消
              </button>
              <button
                class={clsx(
                  'h-8 px-4 rounded-lg text-13px font-medium text-white transition-colors',
                  newDocTitle().trim() && newDocText().trim() ? 'bg-nt-io-500 hover:bg-nt-io-600' : 'bg-nt-io-500/40 cursor-not-allowed',
                )}
                disabled={!newDocTitle().trim() || !newDocText().trim()}
                onClick={() => {
                  void kb.ingestDoc(newDocTitle().trim(), newDocText()).then(() => {
                    setNewDocTitle('')
                    setNewDocText('')
                    setShowAddDoc(false)
                  })
                }}
              >
                入库
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* 删除确认 */}
      <ConfirmModal
        req={deleteReq()}
        onConfirm={() => { const t = deleteTarget(); setDeleteTarget(null); if (t) void kb.remove(t.id) }}
        onClose={() => setDeleteTarget(null)}
      />
    </div>
  )
}
