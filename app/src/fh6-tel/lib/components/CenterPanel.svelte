<script lang="ts">
  import { displayPacket, speedMph, speedKph, rpmPercent } from '$lib/stores/telemetry';
  import AttitudeIndicator from './AttitudeIndicator.svelte';
  import SteeringIndicator from './SteeringIndicator.svelte';

  let { useMph = true }: { useMph: boolean } = $props();

  let pkt = $derived($displayPacket);
  let speed = $derived(useMph ? Math.round($speedMph) : Math.round($speedKph));
  let unit  = $derived(useMph ? 'MPH' : 'KPH');
  let rpm   = $derived($rpmPercent);
  let isRedline = $derived(rpm > 90);

  let gearLabel = $derived((() => {
    if (!pkt) return '—';
    if (pkt.gear === 0)  return 'R';
    if (pkt.gear === 11) return 'N';
    return String(pkt.gear);
  })());

  // SVG arc gauge — 270° sweep, gap at 6-o'clock
  const CX = 160, CY = 148, R = 112;
  const C = 2 * Math.PI * R;
  const bgArc = (270 / 360) * C;
  const ROT = 135;

  let rpmArc = $derived((rpm / 100) * bgArc);

  // Boost
  let boost = $derived(pkt?.boost ?? 0);
  let boostActive = $derived(boost > 0.5);

  // G-force
  const G_MAX = 2.0;
  let latG  = $derived(pkt ? -(pkt.accelX / 9.81) : 0);
  let longG = $derived(pkt ? -(pkt.accelZ / 9.81) : 0);
  let gDotX = $derived(Math.min(Math.max(latG  / G_MAX, -1), 1) * 50 + 50);
  let gDotY = $derived(Math.min(Math.max(-longG / G_MAX, -1), 1) * 50 + 50);
  let gMag  = $derived(Math.hypot(latG, longG));
  let gDotColor = $derived(gMag > 1.5 ? '#ef4444' : gMag > 0.8 ? '#f59e0b' : '#22c55e');

  // Inputs
  let throttleFrac = $derived((pkt?.throttle ?? 0) / 255);
  let brakeFrac    = $derived((pkt?.brake    ?? 0) / 255);
  let clutchFrac   = $derived((pkt?.clutch   ?? 0) / 255);
  let handbrakeOn  = $derived((pkt?.handbrake ?? 0) > 127);
</script>

<div class="center">
  <svg viewBox="0 0 320 265" class="gauge-svg">
    <!-- Redline zone (last 10%) subtle red band -->
    <circle cx={CX} cy={CY} r={R}
      fill="none" stroke="rgba(239,68,68,0.12)" stroke-width="20" stroke-linecap="round"
      stroke-dasharray="{0.1 * bgArc} {C - 0.1 * bgArc}"
      transform="rotate({ROT + 0.9 * 270}, {CX}, {CY})"
    />
    <!-- Background track -->
    <circle cx={CX} cy={CY} r={R}
      fill="none" stroke-width="20" stroke-linecap="round"
      class="rpm-track"
      stroke-dasharray="{bgArc} {C - bgArc}"
      transform="rotate({ROT}, {CX}, {CY})"
    />
    <!-- RPM fill — accent colour, red at redline -->
    <circle cx={CX} cy={CY} r={R}
      fill="none" stroke-width="20" stroke-linecap="round"
      style="stroke: {isRedline ? '#ef4444' : 'var(--ac)'}; transition: stroke-dasharray 40ms linear, stroke 80ms ease;"
      stroke-dasharray="{rpmArc} {C - rpmArc}"
      transform="rotate({ROT}, {CX}, {CY})"
    />

    <!-- Speed -->
    <text x={CX} y={CY - 14}
      text-anchor="middle" font-size="66" font-weight="900"
      class="speed-text"
      font-family="'Segoe UI', system-ui, sans-serif"
      style="font-variant-numeric: tabular-nums;">
      {speed}
    </text>
    <text x={CX} y={CY + 10}
      text-anchor="middle" font-size="11" font-weight="700"
      class="unit-text"
      font-family="'Segoe UI', system-ui, sans-serif"
      letter-spacing="4">
      {unit}
    </text>

    <!-- Gear box -->
    <rect x={CX - 27} y={CY + 22} width="54" height="46" rx="8"
      class="gear-box"
      style="stroke: {isRedline ? '#ef4444' : 'var(--bd-muted)'};"
      stroke-width="2"
    />
    <text x={CX} y={CY + 58}
      text-anchor="middle" font-size="32" font-weight="900"
      style="fill: {isRedline ? '#ef4444' : 'var(--tx-mid)'};"
      font-family="'Segoe UI', system-ui, sans-serif">
      {gearLabel}
    </text>

    <!-- RPM tick marks every 10% -->
    {#each Array.from({length: 11}, (_, i) => i) as tick}
      {@const angle = (ROT + tick * 27) * Math.PI / 180}
      {@const inner = R - 14}
      {@const outer = R + 14}
      <line
        x1={CX + inner * Math.cos(angle)} y1={CY + inner * Math.sin(angle)}
        x2={CX + outer * Math.cos(angle)} y2={CY + outer * Math.sin(angle)}
        style="stroke: {tick >= 9 ? '#ef4444' : 'var(--bd-muted)'};"
        stroke-width={tick % 5 === 0 ? 2.5 : 1.5}
        stroke-linecap="round"
      />
    {/each}
  </svg>

  <div class="bottom-row">
    <!-- G-meter -->
    <div class="g-section">
      <div class="g-circle">
        <div class="g-ring g-ring-1"></div>
        <div class="g-ring g-ring-2"></div>
        <div class="g-ch-h"></div>
        <div class="g-ch-v"></div>
        <div class="g-dot"
          style="left:{gDotX}%; top:{gDotY}%; background:{gDotColor}; box-shadow:0 0 7px {gDotColor}80;">
        </div>
      </div>
      <div class="g-readout">
        <span class="g-axis">横向 {Math.abs(latG).toFixed(1)}G</span>
        <span class="g-axis">纵向 {Math.abs(longG).toFixed(1)}G</span>
      </div>
    </div>

    <!-- ADI -->
    <AttitudeIndicator />

    <!-- Steering wheel -->
    <SteeringIndicator />

    <!-- Input bars + boost LED + RPM -->
    <div class="gauges-col">
      <div class="input-row">
        <span class="input-label">油门</span>
        <div class="input-track">
          <div class="input-fill thr" style="width:{throttleFrac*100}%;"></div>
        </div>
        <span class="input-val">{pkt ? Math.round(throttleFrac * 100) : '—'}</span>
      </div>
      <div class="input-row">
        <span class="input-label">刹车</span>
        <div class="input-track">
          <div class="input-fill brk" style="width:{brakeFrac*100}%;"></div>
        </div>
        <span class="input-val">{pkt ? Math.round(brakeFrac * 100) : '—'}</span>
      </div>
      <div class="input-row">
        <span class="input-label">离合</span>
        <div class="input-track">
          <div class="input-fill clt" style="width:{clutchFrac*100}%;"></div>
        </div>
        <span class="input-val">{pkt ? Math.round(clutchFrac * 100) : '—'}</span>
      </div>

      <div class="input-row">
        <span class="input-label">手刹</span>
        <div class="hb-led" class:hb-on={handbrakeOn}></div>
        <span class="input-val hb-text" class:hb-active={handbrakeOn}>
          {handbrakeOn ? '开' : '关'}
        </span>
      </div>

      <div class="input-row">
        <span class="input-label">增压</span>
        <div class="bst-led" class:bst-on={boostActive}></div>
        <span class="input-val bst-val" class:bst-active={boostActive}>
          {pkt ? boost.toFixed(1) : '—'}
        </span>
        <span class="bst-unit">PSI</span>
      </div>

      {#if pkt}
        <div class="rpm-readout">
          <span class="rpm-num">{Math.round(pkt.currentEngineRpm).toLocaleString()}</span>
          <span class="rpm-unit">RPM</span>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .center {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }
  .gauge-svg {
    flex: 1;
    min-height: 0;
    width: 100%;
    height: 100%;
  }

  /* SVG colours */
  .rpm-track  { stroke: var(--bg-track); }
  .speed-text { fill: var(--tx-hi); }
  .unit-text  { fill: var(--tx-xdim); }
  .gear-box   { fill: var(--bg-elevated); }

  .bottom-row {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: clamp(0.4rem, 1.2vw, 0.8rem);
    padding: 0 clamp(0.4rem, 1.2vw, 0.8rem) clamp(0.3rem, 0.8vh, 0.6rem);
    min-height: 0;
  }

  /* G-meter */
  .g-section {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-shrink: 0;
  }
  .g-circle {
    width: clamp(50px, 7.5vw, 76px);
    height: clamp(50px, 7.5vw, 76px);
    border-radius: 50%;
    border: 1px solid var(--bd-muted);
    background: radial-gradient(circle, var(--bg-elevated) 0%, var(--bg-panel) 100%);
    position: relative;
    overflow: hidden;
    flex-shrink: 0;
  }
  .g-ring {
    position: absolute;
    border-radius: 50%;
    border: 1px solid var(--bd-subtle);
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
  }
  .g-ring-1 { width: 50%; height: 50%; }
  .g-ring-2 { width: 78%; height: 78%; }
  .g-ch-h { position: absolute; top: 50%; left: 5%; width: 90%; height: 1px; background: var(--bd-subtle); transform: translateY(-50%); }
  .g-ch-v { position: absolute; left: 50%; top: 5%; height: 90%; width: 1px; background: var(--bd-subtle); transform: translateX(-50%); }
  .g-dot {
    position: absolute;
    width: clamp(7px, 1.3vw, 11px);
    height: clamp(7px, 1.3vw, 11px);
    border-radius: 50%;
    transform: translate(-50%, -50%);
    transition: left 40ms linear, top 40ms linear, background 200ms, box-shadow 200ms;
    z-index: 1;
  }
  .g-readout { display: flex; flex-direction: column; gap: 0.1rem; }
  .g-axis {
    font-size: clamp(0.44rem, 0.9vw, 0.56rem);
    font-weight: 700;
    color: var(--tx-xdim);
    letter-spacing: 0.05em;
    font-variant-numeric: tabular-nums;
  }

  /* Input columns */
  .gauges-col {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.28rem;
    min-width: 0;
  }
  .input-row { display: flex; align-items: center; gap: 0.35rem; }
  .input-label {
    font-size: clamp(0.42rem, 0.85vw, 0.54rem);
    font-weight: 700;
    color: var(--tx-xdim);
    letter-spacing: 0.08em;
    width: 1.8rem;
    flex-shrink: 0;
  }
  .input-track {
    flex: 1;
    height: 9px;
    background: var(--bg-elevated);
    border-radius: 4px;
    overflow: hidden;
    min-width: 0;
    border: 1px solid var(--bd-subtle);
  }
  .input-fill { height: 100%; border-radius: 4px; transition: width 40ms linear; }
  .input-fill.thr { background: #22c55e; }
  .input-fill.brk { background: #ef4444; }
  .input-fill.clt { background: #a855f7; }
  .input-val {
    font-size: clamp(0.4rem, 0.82vw, 0.52rem);
    font-weight: 700;
    color: var(--tx-xdim);
    width: 1.8rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  /* Handbrake */
  .hb-led {
    width: clamp(7px, 1.1vw, 10px);
    height: clamp(7px, 1.1vw, 10px);
    border-radius: 50%;
    background: var(--bg-elevated);
    border: 1px solid var(--bd-muted);
    flex-shrink: 0;
    transition: background 80ms, box-shadow 80ms;
  }
  .hb-led.hb-on { background: #f97316; box-shadow: 0 0 7px #f97316aa; border-color: #c2410c; }
  .hb-text { color: var(--tx-ghost); }
  .hb-text.hb-active { color: #f97316; }

  /* Boost LED */
  .bst-led {
    width: clamp(7px, 1.1vw, 10px);
    height: clamp(7px, 1.1vw, 10px);
    border-radius: 50%;
    background: var(--bg-elevated);
    border: 1px solid var(--bd-muted);
    flex-shrink: 0;
    transition: background 80ms, box-shadow 80ms;
  }
  .bst-led.bst-on { background: #a855f7; box-shadow: 0 0 7px #a855f7aa; border-color: #7e22ce; }
  .bst-val { color: var(--tx-ghost); transition: color 0.15s; }
  .bst-val.bst-active { color: #d8b4fe; }
  .bst-unit {
    font-size: clamp(0.36rem, 0.72vw, 0.46rem);
    font-weight: 700;
    color: var(--tx-ghost);
    letter-spacing: 0.05em;
    flex-shrink: 0;
  }

  /* RPM */
  .rpm-readout { display: flex; align-items: baseline; gap: 0.3rem; margin-top: 0.1rem; }
  .rpm-num {
    font-size: clamp(0.68rem, 1.6vw, 1rem);
    font-weight: 800;
    color: var(--tx-lo);
    font-variant-numeric: tabular-nums;
  }
  .rpm-unit {
    font-size: clamp(0.4rem, 0.82vw, 0.56rem);
    font-weight: 700;
    color: var(--tx-xdim);
    letter-spacing: 0.1em;
  }
</style>
