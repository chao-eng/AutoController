<script lang="ts">
  import { packet } from '$lib/stores/telemetry';

  let pkt = $derived($packet);

  // steer is i8: -128 (full left) to +127 (full right)
  let steerNorm = $derived(pkt ? pkt.steer / 128 : 0); // -1..1
  let steerFillLeft = $derived(steerNorm < 0 ? Math.abs(steerNorm) * 50 : 0);
  let steerFillRight = $derived(steerNorm > 0 ? steerNorm * 50 : 0);

  let bars = $derived([
    { label: 'THR', value: pkt ? pkt.throttle / 255 : 0, color: '#22c55e' },
    { label: 'BRK', value: pkt ? pkt.brake / 255 : 0, color: '#ef4444' },
    { label: 'CLT', value: pkt ? pkt.clutch / 255 : 0, color: '#94a3b8' },
    { label: 'HBK', value: pkt ? pkt.handbrake / 255 : 0, color: '#f97316' },
  ]);
</script>

<div class="strip">
  <!-- Steering indicator -->
  <div class="steer-group">
    <span class="steer-label">STR</span>
    <div class="steer-track">
      <div class="steer-center"></div>
      <!-- Left fill -->
      <div class="steer-fill steer-left"
        style="width: {steerFillLeft}%; right: 50%;">
      </div>
      <!-- Right fill -->
      <div class="steer-fill steer-right"
        style="width: {steerFillRight}%; left: 50%;">
      </div>
    </div>
  </div>

  <!-- Vertical input bars -->
  <div class="bars">
    {#each bars as bar}
      <div class="bar-col">
        <div class="bar-track">
          <div class="bar-fill" style="height: {bar.value * 100}%; background: {bar.color};"></div>
        </div>
        <span class="bar-label">{bar.label}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .strip {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 0.5rem 0.35rem 0.35rem;
    gap: 0.4rem;
    box-sizing: border-box;
    overflow: hidden;
  }

  /* Steering */
  .steer-group {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.2rem;
    flex-shrink: 0;
  }
  .steer-label {
    font-size: clamp(0.48rem, 1.1vw, 0.6rem);
    font-weight: 700;
    letter-spacing: 0.08em;
    color: #4b5563;
  }
  .steer-track {
    width: 100%;
    height: 8px;
    background: #151e2e;
    border-radius: 4px;
    position: relative;
    overflow: hidden;
  }
  .steer-center {
    position: absolute;
    left: 50%;
    top: 0;
    width: 1.5px;
    height: 100%;
    background: #252f42;
    transform: translateX(-50%);
  }
  .steer-fill {
    position: absolute;
    top: 0;
    height: 100%;
    border-radius: 4px;
    transition: width 33ms linear;
  }
  .steer-left { background: #6366f1; border-radius: 4px 0 0 4px; }
  .steer-right { background: #6366f1; border-radius: 0 4px 4px 0; }

  /* Vertical bars */
  .bars {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: row;
    gap: clamp(0.2rem, 0.8vw, 0.4rem);
    align-items: flex-end;
  }
  .bar-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    flex: 1;
    min-width: 0;
    height: 100%;
  }
  .bar-track {
    flex: 1;
    width: 100%;
    background: #151e2e;
    border-radius: 3px;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    overflow: hidden;
    min-height: 0;
  }
  .bar-fill {
    width: 100%;
    transition: height 33ms linear;
    border-radius: 3px;
  }
  .bar-label {
    font-size: clamp(0.45rem, 1vw, 0.6rem);
    font-weight: 700;
    letter-spacing: 0.05em;
    color: #4b5563;
    white-space: nowrap;
  }
</style>
