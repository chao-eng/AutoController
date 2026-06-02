<script lang="ts">
  import { onDestroy } from 'svelte';
  import { replay, exitReplay } from '$lib/stores/telemetry';

  const SPEEDS = [0.5, 1, 2, 4];
  let timer: ReturnType<typeof setInterval> | null = null;

  function clearTimer() {
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
  }

  onDestroy(clearTimer);

  // Drives the playback loop. Recording is ~60 Hz, so we advance the index by
  // `speed` every frame (~60 fps); fractional progress is carried so 0.5x works.
  let carry = 0;
  function ensureLoop(playing: boolean, speed: number) {
    clearTimer();
    carry = 0;
    if (!playing) return;
    timer = setInterval(() => {
      replay.update((r) => {
        if (!r.active) return r;
        carry += r.speed;
        const step = Math.floor(carry);
        carry -= step;
        const next = r.index + step;
        if (next >= r.packets.length - 1) {
          clearTimer();
          return { ...r, index: r.packets.length - 1, playing: false };
        }
        return { ...r, index: next };
      });
    }, 1000 / 60);
  }

  $effect(() => {
    ensureLoop($replay.playing, $replay.speed);
  });

  function togglePlay() {
    replay.update((r) => {
      // Restart from beginning if we're parked at the end.
      const atEnd = r.index >= r.packets.length - 1;
      return { ...r, playing: !r.playing, index: atEnd ? 0 : r.index };
    });
  }

  function scrub(e: Event) {
    const v = Number((e.target as HTMLInputElement).value);
    replay.update((r) => ({ ...r, index: v, playing: false }));
  }

  function setSpeed(s: number) {
    replay.update((r) => ({ ...r, speed: s }));
  }

  function fmt(idx: number) {
    const sec = idx / 60;
    const m = Math.floor(sec / 60);
    const s = (sec % 60).toFixed(1).padStart(4, '0');
    return `${m}:${s}`;
  }

  let total = $derived($replay.packets.length);
</script>

{#if $replay.active}
  <div class="replay-bar">
    <div class="left">
      <span class="badge">数据回放</span>
      <span class="label" title={$replay.label}>{$replay.label}</span>
    </div>

    <div class="controls">
      <button class="play" onclick={togglePlay}>
        {$replay.playing ? '⏸' : '▶'}
      </button>
      <span class="time">{fmt($replay.index)}</span>
      <input
        class="scrub"
        type="range"
        min="0"
        max={Math.max(total - 1, 0)}
        value={$replay.index}
        oninput={scrub}
      />
      <span class="time">{fmt(Math.max(total - 1, 0))}</span>
      <div class="speeds">
        {#each SPEEDS as s}
          <button class:active={$replay.speed === s} onclick={() => setSpeed(s)}>
            {s}×
          </button>
        {/each}
      </div>
    </div>

    <button class="exit" onclick={exitReplay}>退出回放</button>
  </div>
{/if}

<style>
  .replay-bar {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 110;
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 1rem;
    background: var(--bg-panel);
    border-top: 1px solid var(--ac);
    box-shadow: 0 -4px 20px rgba(31,35,41,0.08);
  }
  .left {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    min-width: 0;
    flex: 0 1 240px;
  }
  .badge {
    background: var(--ac);
    color: #fff;
    font-size: 0.65rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    padding: 0.15rem 0.4rem;
    border-radius: 3px;
  }
  .label {
    color: var(--tx-lo);
    font-size: 0.78rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .controls {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }
  .play {
    background: var(--ac);
    color: #fff;
    border: none;
    border-radius: 50%;
    width: 2rem;
    height: 2rem;
    font-size: 0.85rem;
    cursor: pointer;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    padding: 0;
  }
  .time {
    color: var(--tx-dim);
    font-size: 0.72rem;
    font-variant-numeric: tabular-nums;
    min-width: 3rem;
    text-align: center;
  }
  .scrub {
    flex: 1;
    accent-color: var(--ac);
    cursor: pointer;
  }
  .speeds {
    display: flex;
    gap: 0.2rem;
  }
  .speeds button {
    background: var(--bg-elevated);
    border: 1px solid var(--bd-dim);
    color: var(--tx-dim);
    font-size: 0.7rem;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    cursor: pointer;
  }
  .speeds button.active {
    border-color: var(--ac);
    color: var(--tx-hi);
  }
  .exit {
    background: none;
    border: 1px solid var(--bd-subtle);
    color: var(--tx-lo);
    font-size: 0.75rem;
    padding: 0.35rem 0.7rem;
    border-radius: 4px;
    cursor: pointer;
    flex-shrink: 0;
  }
  .exit:hover {
    border-color: #ef4444;
    color: #ef4444;
  }
</style>
