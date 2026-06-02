<script lang="ts">
  import { displayPacket } from '$lib/stores/telemetry';

  let pkt = $derived($displayPacket);
  let headingDeg = $derived(pkt ? ((pkt.yaw * 180 / Math.PI) % 360 + 360) % 360 : 0);

  let compassDir = $derived((() => {
    const d = headingDeg;
    if (d < 22.5 || d >= 337.5) return '北';
    if (d < 67.5)  return '东北';
    if (d < 112.5) return '东';
    if (d < 157.5) return '东南';
    if (d < 202.5) return '南';
    if (d < 247.5) return '西南';
    if (d < 292.5) return '西';
    return '西北';
  })());

  const LABELS: Record<number, string> = {
    0: '北', 45: '东北', 90: '东', 135: '东南',
    180: '南', 225: '西南', 270: '西', 315: '西北',
  };
  const CX = 400;
  const SCALE = 3;

  let ticks = $derived((() => {
    const result = [];
    for (let t = 0; t < 360; t += 5) {
      let diff = ((t - headingDeg) % 360 + 360) % 360;
      if (diff > 180) diff -= 360;
      if (Math.abs(diff) > 133) continue;
      result.push({
        x: CX + diff * SCALE,
        deg: t,
        isCardinal: t % 45 === 0,
        isMajor: t % 10 === 0,
        label: LABELS[t] ?? null,
      });
    }
    return result;
  })());
</script>

<div class="compass-strip">
  <svg viewBox="0 0 800 30" class="compass-svg" preserveAspectRatio="xMidYMid meet">
    <!-- Tick marks and cardinal labels -->
    {#each ticks as tick}
      {@const h = tick.isCardinal ? 11 : tick.isMajor ? 7 : 4}
      <line x1={tick.x} y1="0" x2={tick.x} y2={h}
        class={tick.isCardinal ? 'tick-cardinal' : 'tick-minor'}
        stroke-linecap="round"
      />
      {#if tick.label}
        <text x={tick.x} y={h + 8}
          text-anchor="middle"
          font-size={tick.deg % 90 === 0 ? '9' : '7.5'}
          font-weight={tick.deg % 90 === 0 ? '800' : '600'}
          class={tick.deg % 90 === 0 ? 'label-cardinal' : 'label-inter'}
          font-family="system-ui, sans-serif">
          {tick.label}
        </text>
      {:else if tick.isMajor}
        <text x={tick.x} y="22"
          text-anchor="middle" font-size="5.5"
          class="label-deg"
          font-family="system-ui, sans-serif">
          {tick.deg}
        </text>
      {/if}
    {/each}

    <!-- Fixed centre pointer -->
    <polygon points="{CX},{17} {CX - 5},{26} {CX + 5},{26}" class="centre-ptr"/>

    <!-- Edge fade overlays -->
    <defs>
      <linearGradient id="cf-l" x1="0" y1="0" x2="1" y2="0">
        <stop offset="0%"  class="fade-stop-solid"/>
        <stop offset="18%" class="fade-stop-clear"/>
      </linearGradient>
      <linearGradient id="cf-r" x1="0" y1="0" x2="1" y2="0">
        <stop offset="82%" class="fade-stop-clear"/>
        <stop offset="100%" class="fade-stop-solid"/>
      </linearGradient>
    </defs>
    <rect x="0"   y="0" width="160" height="30" fill="url(#cf-l)"/>
    <rect x="640" y="0" width="160" height="30" fill="url(#cf-r)"/>
  </svg>

  <div class="heading-readout">
    <span class="heading-num">{Math.round(headingDeg).toString().padStart(3, '0')}°</span>
    <span class="heading-dir">{compassDir}</span>
  </div>
</div>

<style>
  .compass-strip {
    flex-shrink: 0;
    height: clamp(24px, 2.8vh, 32px);
    background: var(--bg-panel);
    border-bottom: 1px solid var(--bd-dim);
    position: relative;
    overflow: hidden;
  }
  .compass-svg { width: 100%; height: 100%; display: block; }

  /* SVG element classes using CSS variables */
  .tick-cardinal { stroke: var(--ac); stroke-width: 1.5; }
  .tick-minor    { stroke: var(--bd-subtle); stroke-width: 1; }
  .label-cardinal { fill: var(--tx-mid); }
  .label-inter    { fill: var(--ac); }
  .label-deg      { fill: var(--bd-muted); }
  .centre-ptr     { fill: var(--tx-mid); opacity: 0.85; }

  /* Gradient stop colours via CSS vars aren't supported in SVG stops directly,
     so we define the background colour on the overlay rects instead */
  .fade-stop-solid { stop-color: var(--bg-panel); stop-opacity: 1; }
  .fade-stop-clear { stop-color: var(--bg-panel); stop-opacity: 0; }

  .heading-readout {
    position: absolute;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    display: flex;
    align-items: baseline;
    gap: 0.2rem;
    pointer-events: none;
  }
  .heading-num {
    font-size: clamp(0.55rem, 1.1vw, 0.72rem);
    font-weight: 800;
    color: var(--tx-mid);
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.04em;
  }
  .heading-dir {
    font-size: clamp(0.45rem, 0.9vw, 0.6rem);
    font-weight: 700;
    color: var(--ac);
    letter-spacing: 0.08em;
  }
</style>
