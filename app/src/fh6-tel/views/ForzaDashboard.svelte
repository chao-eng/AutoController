<script lang="ts">
  import { onMount } from 'svelte';
  import { isDesktop } from '$lib/ipc';
  import { startTelemetryListener, replay } from '$lib/stores/telemetry';
  import { loadSettings, settings, saveSettings } from '$lib/stores/sessions';
  import TopBar from '$lib/components/TopBar.svelte';
  import CompassBar from '$lib/components/CompassBar.svelte';
  import CenterPanel from '$lib/components/CenterPanel.svelte';
  import TireWidget from '$lib/components/TireWidget.svelte';
  import FloatingPanel from '$lib/components/FloatingPanel.svelte';
  import LapBar from '$lib/components/LapBar.svelte';
  import SessionDrawer from '$lib/components/SessionDrawer.svelte';
  import SessionViewer from '$lib/components/SessionViewer.svelte';
  import ReplayBar from '$lib/components/ReplayBar.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import type { SessionRow } from '$lib/types';

  let showSessions = $state(false);
  let showSettings = $state(false);
  let viewerSession = $state<SessionRow | null>(null);
  let toasts = $state<{ id: number; message: string }[]>([]);
  let nextToastId = 0;
  let pendingUpdate = $state<{ version: string; install: () => Promise<void> } | null>(null);
  let updateInstalling = $state(false);

  function addToast(message: string) {
    const id = nextToastId++;
    toasts = [...toasts, { id, message }];
    setTimeout(() => { toasts = toasts.filter(t => t.id !== id); }, 4000);
  }

  onMount(async () => {
    await loadSettings();
    await startTelemetryListener({ onError: (m) => addToast(m), onBindFailed: (m) => addToast(m) });
    if (isDesktop) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const info = await invoke<{ version: string; is_deb: boolean } | null>('check_for_update');
        if (info) {
          pendingUpdate = {
            version: info.version,
            install: async () => {
              updateInstalling = true;
              await invoke('install_update', { isDeb: info.is_deb });
            },
          };
        }
      } catch {
        // Offline or update endpoint unreachable — ignore
      }
    }
  });

  let s = $derived($settings);

  // Replaying takes over the live dashboard — get the overlays out of the way.
  $effect(() => {
    if ($replay.active) {
      showSessions = false;
      viewerSession = null;
    }
  });
</script>

{#if pendingUpdate}
  <div class="update-bar">
    <span>Update v{pendingUpdate.version} available</span>
    <button class="update-install" disabled={updateInstalling} onclick={() => pendingUpdate?.install()}>
      {updateInstalling ? 'Installing…' : 'Install & restart'}
    </button>
    <button class="update-dismiss" onclick={() => (pendingUpdate = null)}>✕</button>
  </div>
{/if}

<div class="dashboard">
  <TopBar
    useMph={s?.useMph ?? true}
    onSettings={() => (showSettings = true)}
    onSessions={() => (showSessions = !showSessions)}
    tiresVisible={s?.tiresVisible ?? true}
    onToggleTires={async () => { if (s) await saveSettings({ ...s, tiresVisible: !(s.tiresVisible ?? true) }); }}
  />
  <CompassBar />

  <div class="main">
    <div class="center-area">
      <CenterPanel useMph={s?.useMph ?? true} />
    </div>
  </div>

  {#if s?.tiresVisible ?? true}
    <FloatingPanel
      id="fh6-tires"
      title="轮胎数据"
      defaultWidth={200}
      defaultTop={64}
      onClose={async () => { if (s) await saveSettings({ ...s, tiresVisible: false }); }}
    >
      <TireWidget
        tireTempCold={s?.tireTempCold ?? 60}
        tireTempOptimal={s?.tireTempOptimal ?? 85}
        tireTempHot={s?.tireTempHot ?? 110}
      />
    </FloatingPanel>
  {/if}

  <div class="lap-bar">
    <LapBar />
  </div>
</div>

{#if showSessions}
  <SessionDrawer
    onClose={() => (showSessions = false)}
    onOpen={(session) => (viewerSession = session)}
  />
{/if}

{#if viewerSession}
  <SessionViewer
    session={viewerSession}
    useMph={s?.useMph ?? true}
    onClose={() => (viewerSession = null)}
  />
{/if}

<ReplayBar />

{#if toasts.length > 0}
  <div class="toast-stack">
    {#each toasts as toast (toast.id)}
      <div class="toast">{toast.message}</div>
    {/each}
  </div>
{/if}

{#if showSettings}
  <SettingsModal onClose={() => (showSettings = false)} />
{/if}

<style>
  /* ── Theme: CSS custom properties (AutoController project style) ────────── */
  :global(:root) {
    --bg-body:    #F5F6F7;
    --bg-panel:   #FFFFFF;
    --bg-card:    #F2F3F5;
    --bg-elevated:#FFFFFF;
    --bg-track:   #DEE0E3;
    --bd-dim:     #DEE0E3;
    --bd-subtle:  #DEE0E3;
    --bd-muted:   #8F959E;
    --bd-strong:  #646A73;
    --tx-hi:      #1F2329;
    --tx-mid:     #1F2329;
    --tx-lo:      #646A73;
    --tx-dim:     #646A73;
    --tx-xdim:    #8F959E;
    --tx-ghost:   #B0B5BC;
    --ac:         #3370FF;
    --ac-dim:     #E8F0FE;
    --adi-sky:    #DAE5F5;
    --adi-ground: #E8E0D5;
  }

  :global(*, *::before, *::after) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    background: var(--bg-body);
    color: var(--tx-hi);
    font-family: 'Plus Jakarta Sans', 'Inter', system-ui, -apple-system, sans-serif;
    overflow: hidden;
    height: 100vh;
    width: 100vw;
  }

  /* App-wide slim themed scrollbars */
  :global(*) {
    scrollbar-width: thin;
    scrollbar-color: #DEE0E3 transparent;
  }
  :global(*::-webkit-scrollbar) { width: 6px; height: 6px; }
  :global(*::-webkit-scrollbar-track) { background: transparent; }
  :global(*::-webkit-scrollbar-thumb) {
    background: #DEE0E3;
    border-radius: 3px;
  }
  :global(*::-webkit-scrollbar-thumb:hover) {
    background: #8F959E;
  }

  .dashboard {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
  }

  .main {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .center-area { background: var(--bg-body); overflow: hidden; width: 100%; height: 100%; }
  .lap-bar { height: clamp(2.5rem, 5.5vh, 4rem); flex-shrink: 0; }

  .update-bar {
    position: fixed; top: 0; left: 0; right: 0; z-index: 300;
    display: flex; align-items: center; gap: 0.75rem;
    padding: 0.35rem 1rem;
    background: #3370FF; border-bottom: none;
    font-size: 0.78rem; color: #fff;
  }
  .update-bar span { flex: 1; }
  .update-install {
    background: rgba(255,255,255,0.2); color: #fff; border: 1px solid rgba(255,255,255,0.3); border-radius: 4px;
    padding: 0.2rem 0.65rem; font-size: 0.75rem; cursor: pointer;
  }
  .update-install:disabled { opacity: 0.6; cursor: default; }
  .update-dismiss {
    background: none; border: none; color: rgba(255,255,255,0.7);
    font-size: 0.85rem; cursor: pointer; padding: 0 0.25rem;
  }
  .update-dismiss:hover { color: #fff; }

  .toast-stack {
    position: fixed; bottom: 4rem; left: 50%; transform: translateX(-50%);
    display: flex; flex-direction: column; gap: 0.5rem; z-index: 200;
    pointer-events: none;
  }
  .toast {
    background: #FFFFFF; border: 1px solid #F54A45; border-radius: 6px;
    box-shadow: 0 4px 12px rgba(31,35,41,0.1);
    color: #F54A45; font-size: 0.8rem; padding: 0.5rem 1rem;
    max-width: 420px; text-align: center;
  }


</style>
