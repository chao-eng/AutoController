<script lang="ts">
  import { displayPacket, packet } from '$lib/stores/telemetry';

  let pkt = $derived($displayPacket);
  let rawPkt = $derived($packet);
  let inEvent = $derived(
    (rawPkt?.isRaceOn === true) && ((rawPkt?.racePosition ?? 0) > 0)
  );

  function formatTime(seconds: number): string {
    if (seconds <= 0) return '—:——.———';
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toFixed(3).padStart(6, '0')}`;
  }
</script>

<div class="lapbar">
  <div class="lap-item">
    <span class="lap-key">圈数</span>
    <span class="lap-val">{pkt ? pkt.lapNumber : '—'}</span>
  </div>
  <div class="sep"></div>
  <div class="lap-item">
    <span class="lap-key">当前圈</span>
    <span class="lap-val current">{formatTime(pkt?.currentLap ?? 0)}</span>
  </div>
  <div class="sep"></div>
  <div class="lap-item">
    <span class="lap-key">上一圈</span>
    <span class="lap-val">{formatTime(pkt?.lastLap ?? 0)}</span>
  </div>
  <div class="sep"></div>
  <div class="lap-item">
    <span class="lap-key">最快圈</span>
    <span class="lap-val best">{formatTime(pkt?.bestLap ?? 0)}</span>
  </div>
  <div class="sep"></div>
  <div class="lap-item">
    <span class="lap-key">本次会话 {#if inEvent}<span class="live-pip"></span>{/if}</span>
    <span class="lap-val session-time" class:session-live={inEvent}>
      {formatTime(pkt?.currentRaceTime ?? 0)}
    </span>
  </div>
</div>

<style>
  .lapbar {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    background: var(--bg-panel);
    border-top: 1px solid var(--bd-dim);
    padding: 0 clamp(0.5rem, 2vw, 1.5rem);
    overflow: hidden;
    gap: 0;
  }
  .lap-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0 clamp(0.5rem, 2vw, 1.5rem);
    min-width: 0;
    flex-shrink: 1;
  }
  .lap-key {
    font-size: clamp(0.42rem, 1vw, 0.55rem);
    font-weight: 700;
    letter-spacing: 0.15em;
    color: var(--tx-xdim);
    white-space: nowrap;
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }
  .live-pip {
    display: inline-block;
    width: 5px; height: 5px;
    border-radius: 50%;
    background: #22c55e;
    box-shadow: 0 0 4px #22c55e;
    flex-shrink: 0;
  }
  .lap-val {
    font-size: clamp(0.7rem, 1.8vw, 1.1rem);
    font-weight: 800;
    font-variant-numeric: tabular-nums;
    color: var(--tx-mid);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .lap-val.current { color: var(--ac); }
  .lap-val.best    { color: #a855f7; }

  .session-time {
    width: clamp(5rem, 10vw, 7rem);
    text-align: center;
    display: inline-block;
    color: var(--tx-xdim);
    transition: color 0.3s;
  }
  .session-time.session-live { color: var(--tx-mid); }

  .sep {
    width: 1px;
    height: clamp(1rem, 3vh, 2rem);
    background: var(--bd-subtle);
    flex-shrink: 0;
  }
</style>
