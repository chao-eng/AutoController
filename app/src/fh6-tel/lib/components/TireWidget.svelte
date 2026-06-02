<script lang="ts">
  import { displayPacket } from '$lib/stores/telemetry';

  let {
    tireTempCold = 60,
    tireTempOptimal = 85,
    tireTempHot = 110,
  }: {
    tireTempCold: number;
    tireTempOptimal: number;
    tireTempHot: number;
  } = $props();

  let pkt = $derived($displayPacket);

  function tempColor(t: number): string {
    if (t < tireTempCold) return '#3b82f6';
    if (t < tireTempOptimal) return '#22c55e';
    if (t < tireTempHot) return '#f59e0b';
    return '#ef4444';
  }

  function slipColor(s: number): string {
    const a = Math.abs(s);
    if (a < 0.05) return '#22c55e';
    if (a < 0.15) return '#f59e0b';
    return '#ef4444';
  }

  function suspColor(s: number): string {
    if (s > 0.88) return '#ef4444';
    if (s > 0.72) return '#f59e0b';
    if (s < 0.12) return '#3b82f6';
    return '#6366f1';
  }

  function wearColor(w: number): string {
    if (w > 0.7) return '#22c55e';
    if (w > 0.4) return '#f59e0b';
    return '#ef4444';
  }

  let tires = $derived([
    { label: '前左', temp: pkt?.tireTempFl ?? 0, slip: pkt?.tireSlipRatioFl ?? 0, susp: pkt?.suspensionFl ?? 0.5, wear: pkt?.tireWearFl ?? null },
    { label: '前右', temp: pkt?.tireTempFr ?? 0, slip: pkt?.tireSlipRatioFr ?? 0, susp: pkt?.suspensionFr ?? 0.5, wear: pkt?.tireWearFr ?? null },
    { label: '后左', temp: pkt?.tireTempRl ?? 0, slip: pkt?.tireSlipRatioRl ?? 0, susp: pkt?.suspensionRl ?? 0.5, wear: pkt?.tireWearRl ?? null },
    { label: '后右', temp: pkt?.tireTempRr ?? 0, slip: pkt?.tireSlipRatioRr ?? 0, susp: pkt?.suspensionRr ?? 0.5, wear: pkt?.tireWearRr ?? null },
  ]);
</script>

<div class="widget">
  <div class="tire-grid">
    {#each tires as tire}
      {@const tc = tempColor(tire.temp)}
      {@const sc = suspColor(tire.susp)}
      {@const clampedSusp = Math.min(Math.max(tire.susp, 0), 1)}
      {@const rodLen = (1 - clampedSusp) * 22}
      {@const rodTop = 34 - rodLen}

      <div class="tile" style="--tc: {tc};">
        <!-- Tire info column -->
        <div class="tire-info">
          <span class="tlabel">{tire.label}</span>
          <span class="temp" style="color: {tc};">
            {pkt ? Math.round(tire.temp) + '°' : '—'}
          </span>
          <div class="bottom-row">
            <span class="slip-dot" style="background: {slipColor(tire.slip)};"></span>
            {#if tire.wear !== null}
              <span class="wear" style="color: {wearColor(tire.wear)};">
                {Math.round(tire.wear * 100)}%
              </span>
            {/if}
          </div>
        </div>

        <!-- Shock absorber icon column -->
        <div class="shock-col">
          <svg viewBox="0 0 10 58" preserveAspectRatio="xMidYMid meet" class="shock-svg">
            <!-- Top mount -->
            <circle cx="5" cy="3" r="2" class="shock-mount"/>

            <!-- Upper connector to rod or cylinder mouth -->
            <line x1="5" y1="5"
                  x2="5" y2={rodLen > 0.5 ? rodTop : 34}
                  class="shock-strut" stroke-width="1.5" stroke-linecap="round"/>

            <!-- Coloured rod (exposed portion) -->
            {#if rodLen > 0.5}
              <rect x="3.8" y={rodTop} width="2.4" height={rodLen} rx="1.2" fill={sc}
                    style="transition: height 60ms linear, y 60ms linear;"/>
            {/if}

            <!-- Cylinder body -->
            <rect x="1.5" y="34" width="7" height="16" rx="2.5"
                  class="shock-cylinder" stroke-width="1.2"/>

            <!-- Rod entry highlight at cylinder mouth -->
            <rect x="3.2" y="33.2" width="3.6" height="2.5" rx="1"
                  fill={sc} opacity="0.5"/>

            <!-- Bottom connector -->
            <line x1="5" y1="50" x2="5" y2="55"
                  class="shock-strut" stroke-width="1.5" stroke-linecap="round"/>

            <!-- Bottom mount -->
            <circle cx="5" cy="55.5" r="2" class="shock-mount"/>
          </svg>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .widget {
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 0.4rem;
    box-sizing: border-box;
    overflow: hidden;
  }

  .tire-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr 1fr;
    gap: 0.35rem;
    flex: 1;
    min-height: 0;
  }

  .tile {
    display: flex;
    flex-direction: row;
    gap: 0.25rem;
    background: var(--bg-card);
    border: 1.5px solid color-mix(in srgb, var(--tc) 28%, var(--bg-elevated));
    border-radius: 8px;
    padding: 0.3rem 0.2rem 0.3rem 0.4rem;
    transition: border-color 0.3s;
    overflow: hidden;
    min-height: 0;
    position: relative;
  }
  .tile::before {
    content: '';
    position: absolute;
    inset: 0;
    background: radial-gradient(ellipse at 30% 30%,
      color-mix(in srgb, var(--tc) 7%, transparent) 0%, transparent 65%);
    pointer-events: none;
  }

  /* Tire info */
  .tire-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: space-between;
    padding: 0.1rem 0;
  }
  .tlabel {
    font-size: clamp(0.48rem, 1.2vw, 0.62rem);
    font-weight: 800;
    color: var(--tx-xdim);
    letter-spacing: 0.1em;
    align-self: flex-start;
  }
  .temp {
    font-size: clamp(0.85rem, 2.8vw, 1.2rem);
    font-weight: 900;
    font-variant-numeric: tabular-nums;
    line-height: 1;
  }
  .bottom-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
  }
  .slip-dot {
    width: clamp(6px, 1.2vw, 9px);
    height: clamp(6px, 1.2vw, 9px);
    border-radius: 50%;
    flex-shrink: 0;
    transition: background 0.12s;
  }
  .wear {
    font-size: clamp(0.45rem, 1.1vw, 0.6rem);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  /* Shock absorber icon */
  .shock-col {
    width: clamp(10px, 2.2vw, 18px);
    flex-shrink: 0;
    display: flex;
    align-items: stretch;
    padding: 0.15rem 0;
  }
  .shock-svg {
    width: 100%;
    height: 100%;
    overflow: visible;
  }

  /* Shock SVG element colours */
  .shock-mount    { fill: var(--bd-strong); }
  .shock-strut    { stroke: var(--bd-strong); }
  .shock-cylinder { fill: var(--bg-panel); stroke: var(--bd-strong); }
</style>
