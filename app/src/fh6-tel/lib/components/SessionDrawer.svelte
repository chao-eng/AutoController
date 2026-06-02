<script lang="ts">
  import { onMount } from 'svelte';
  import { sessions, loadSessions, deleteSession, clearAllSessions, setSessionBookmark } from '$lib/stores/sessions';
  import { carName } from '$lib/car-name';
  import type { SessionRow } from '$lib/types';

  let { onClose, onOpen }: { onClose: () => void; onOpen: (s: SessionRow) => void } =
    $props();

  onMount(loadSessions);

  function formatTime(seconds: number) {
    if (!seconds || seconds <= 0) return '—';
    const m = Math.floor(seconds / 60);
    const s = (seconds % 60).toFixed(3).padStart(6, '0');
    return `${m}:${s}`;
  }

  function formatDate(ms: number) {
    return new Date(ms).toLocaleString();
  }

  async function handleDelete(session: SessionRow, e: MouseEvent) {
    e.stopPropagation();
    const label = session.name ?? formatDate(session.startedAt);
    if (!confirm(`确定要删除会话 "${label}" 吗？`)) return;
    await deleteSession(session.id);
  }

  async function toggleBookmark(session: SessionRow, e: MouseEvent) {
    e.stopPropagation();
    await setSessionBookmark(session.id, !session.bookmarked);
  }

  async function handleClearAll() {
    const n = $sessions.length;
    if (n === 0) return;
    if (!confirm(`确定要删除所有 ${n} 个会话吗？此操作无法撤销。`))
      return;
    await clearAllSessions();
  }
</script>

<div class="drawer">
  <div class="drawer-header">
    <h3>历史会话</h3>
    <div class="header-actions">
      <button
        class="clear-all"
        disabled={$sessions.length === 0}
        onclick={handleClearAll}
      >
        清除全部
      </button>
      <button class="close" onclick={onClose}>✕</button>
    </div>
  </div>

  <div class="drawer-body">
    <div class="session-list">
      {#each $sessions as session}
        <div
          class="session-row"
          role="button"
          tabindex="0"
          onclick={() => onOpen(session)}
          onkeydown={(e) => e.key === 'Enter' && onOpen(session)}
        >
          <button
            class="star"
            class:on={session.bookmarked}
            title={session.bookmarked ? '取消收藏' : '收藏'}
            onclick={(e) => toggleBookmark(session, e)}
          >
            {session.bookmarked ? '★' : '☆'}
          </button>
          <div class="session-info">
            <span class="session-name">
              {session.name ?? carName(session.carOrdinal)}
            </span>
            <span class="session-date">{formatDate(session.startedAt)}</span>
            <span class="session-best">最快: {formatTime(session.bestLap ?? 0)}</span>
          </div>
          <button class="delete-btn" onclick={(e) => handleDelete(session, e)}>🗑</button>
        </div>
      {:else}
        <p class="empty">暂无记录的会话数据。</p>
      {/each}
    </div>
  </div>
</div>

<style>
  .drawer {
    position: fixed; right: 0; top: 0; bottom: 0; width: 420px;
    background: var(--bg-panel); border-left: 1px solid var(--bd-dim);
    display: flex; flex-direction: column; z-index: 50;
    box-shadow: -4px 0 24px rgba(0,0,0,0.5);
  }
  .drawer-header {
    display: flex; justify-content: space-between; align-items: center;
    padding: 1rem; border-bottom: 1px solid var(--bd-dim);
  }
  h3 { margin: 0; color: var(--tx-hi); }
  .header-actions { display: flex; align-items: center; gap: 0.6rem; }
  .drawer-header .close { background: none; border: none; color: var(--tx-dim); font-size: 1.1rem; cursor: pointer; }
  .drawer-header .close:hover { color: var(--tx-hi); }
  .clear-all {
    background: none; border: 1px solid var(--bd-muted); color: var(--tx-dim);
    font-size: 0.72rem; padding: 0.25rem 0.55rem; border-radius: 4px; cursor: pointer;
  }
  .clear-all:hover:not(:disabled) { border-color: #ef4444; color: #ef4444; }
  .clear-all:disabled { opacity: 0.4; cursor: default; }
  .drawer-body { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 1rem; padding: 0.5rem; }
  .session-list { display: flex; flex-direction: column; gap: 0.3rem; }
  .session-row {
    display: flex; align-items: center; gap: 0.5rem;
    padding: 0.6rem 0.75rem; border-radius: 6px; cursor: pointer;
    border: 1px solid transparent; background: var(--bg-elevated);
  }
  .session-row:hover { border-color: var(--ac); }
  .star {
    background: none; border: none; cursor: pointer;
    font-size: 1.05rem; color: var(--tx-dim); line-height: 1; flex-shrink: 0;
  }
  .star.on { color: #fbbf24; }
  .session-info { display: flex; flex-direction: column; gap: 0.1rem; flex: 1; min-width: 0; }
  .session-name {
    font-size: 0.85rem; font-weight: 600; color: var(--tx-mid);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .session-date { font-size: 0.7rem; color: var(--tx-dim); }
  .session-best { font-size: 0.75rem; color: #a855f7; font-weight: 700; }
  .delete-btn { background: none; border: none; cursor: pointer; font-size: 0.9rem; color: var(--tx-dim); flex-shrink: 0; }
  .delete-btn:hover { color: #ef4444; }
  .empty { color: var(--tx-xdim); font-size: 0.85rem; text-align: center; padding: 2rem; }
</style>
