<script lang="ts">
  import { displayPacket } from '$lib/stores/telemetry';

  let pkt = $derived($displayPacket);
  let steerNorm = $derived((pkt?.steer ?? 0) / 127);
  let steerDeg  = $derived(steerNorm * 120);
</script>

<div class="steer-wrap">
  <svg viewBox="-50 -50 100 100" class="steer-svg">
    <!-- Fixed notch at top of bezel -->
    <rect x="-2" y="-48" width="4" height="7" rx="1.5" class="notch"/>

    <!-- Rotating wheel group -->
    <g transform="rotate({steerDeg})" style="transition: transform 40ms linear;">
      <circle cx="0" cy="0" r="40" fill="none" class="rim" stroke-width="8" stroke-linecap="round"/>
      <line x1="0"    y1="-32" x2="0"    y2="0" class="spoke" stroke-width="4" stroke-linecap="round"/>
      <line x1="27.7" y1="16"  x2="0"    y2="0" class="spoke" stroke-width="4" stroke-linecap="round"/>
      <line x1="-27.7"y1="16"  x2="0"    y2="0" class="spoke" stroke-width="4" stroke-linecap="round"/>
      <circle cx="0" cy="0"    r="8"  class="hub" stroke-width="1.5"/>
      <!-- Rotation marker -->
      <circle cx="0" cy="-38" r="4" class="marker"/>
    </g>
  </svg>

  <div class="steer-label">
    <span class="steer-dir">
      {#if Math.abs(steerNorm) < 0.05}中
      {:else if steerNorm < 0}左
      {:else}右
      {/if}
    </span>
    <span class="steer-val">{Math.round(Math.abs(steerNorm) * 100)}%</span>
  </div>
</div>

<style>
  .steer-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.15rem;
    flex-shrink: 0;
  }
  .steer-svg {
    width: clamp(48px, 7.5vw, 76px);
    height: clamp(48px, 7.5vw, 76px);
    flex-shrink: 0;
  }
  .steer-label { display: flex; gap: 0.25rem; align-items: baseline; }
  .steer-dir {
    font-size: clamp(0.4rem, 0.8vw, 0.52rem);
    font-weight: 700;
    color: var(--ac);
    letter-spacing: 0.06em;
    min-width: 1.5rem;
    text-align: center;
  }
  .steer-val {
    font-size: clamp(0.38rem, 0.75vw, 0.48rem);
    font-weight: 700;
    color: var(--tx-xdim);
    font-variant-numeric: tabular-nums;
  }

  /* SVG colours via CSS variables */
  .notch  { fill: var(--bd-muted); }
  .rim    { stroke: var(--bd-strong); }
  .spoke  { stroke: var(--bd-strong); }
  .hub    { fill: var(--bg-elevated); stroke: var(--bd-strong); }
  .marker { fill: var(--ac); }
</style>
