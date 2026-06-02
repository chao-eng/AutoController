<script lang="ts">
  import { settings, saveSettings } from '$lib/stores/sessions';
  import type { AppSettings } from '$lib/types';

  let { onClose }: { onClose: () => void } = $props();

  let draft = $state<AppSettings | null>(null);

  $effect(() => {
    if ($settings && !draft) {
      draft = { ...$settings };
    }
  });

  async function save() {
    if (!draft) return;
    await saveSettings(draft);
    onClose();
  }
</script>

{#if draft}
  <div class="overlay" role="dialog" aria-modal="true">
    <div class="modal">
      <h2>设置</h2>

      <label>
        UDP 接收端口
        <input type="number" bind:value={draft.port} min="1024" max="65535" />
        <span class="hint">端口更改将在重新启动应用后生效。</span>
      </label>

      <label>
        速度单位
        <select bind:value={draft.useMph}>
          <option value={true}>mph</option>
          <option value={false}>km/h</option>
        </select>
      </label>

      <label>
        主题界面
        <select bind:value={draft.theme}>
          <option value="dark">深色</option>
          <option value="cobalt2">钴蓝2</option>
          <option value="purple">紫色</option>
        </select>
      </label>

      <label class="checkbox-label">
        <input type="checkbox" bind:checked={draft.autoRecord} />
        自动记录游戏会话
      </label>

      <fieldset>
        <legend>轮胎温度区间 (°C)</legend>
        <label>低温区间低于 <input type="number" bind:value={draft.tireTempCold} /></label>
        <label>合适区间最高 <input type="number" bind:value={draft.tireTempOptimal} /></label>
        <label>高温区间高于 <input type="number" bind:value={draft.tireTempHot} /></label>
      </fieldset>

      <div class="actions">
        <button onclick={onClose}>取消</button>
        <button class="primary" onclick={save}>保存</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,0.7);
    display: flex; align-items: center; justify-content: center; z-index: 100;
  }
  .modal {
    background: var(--bg-elevated); border: 1px solid var(--bd-muted); border-radius: 10px;
    padding: 1.5rem; width: 420px; max-height: 88vh; overflow-y: auto;
    display: flex; flex-direction: column; gap: 1rem;
  }
  h2 { margin: 0; color: var(--tx-hi); font-size: 1.1rem; }
  label { display: flex; flex-direction: column; gap: 0.3rem; color: var(--tx-mid); font-size: 0.85rem; }
  .checkbox-label { flex-direction: row; align-items: center; gap: 0.5rem; }
  input[type="number"], input[type="text"], select {
    background: var(--bg-body); border: 1px solid var(--bd-muted); border-radius: 4px;
    color: var(--tx-hi); padding: 0.4rem; font-size: 0.9rem; width: 100%;
  }

  .actions { display: flex; justify-content: flex-end; gap: 0.5rem; }
  button {
    padding: 0.4rem 1rem; border-radius: 5px; border: 1px solid var(--bd-muted);
    background: var(--bg-elevated); color: var(--tx-mid); cursor: pointer; font-size: 0.85rem;
  }
  button.primary { background: var(--ac); border-color: var(--ac); color: var(--bg-body); }
  button:hover { filter: brightness(1.2); }
  .hint { font-size: 0.7rem; color: var(--tx-dim); margin-top: 0.15rem; }
</style>
