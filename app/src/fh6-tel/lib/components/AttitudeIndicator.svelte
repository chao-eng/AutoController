<script lang="ts">
  import { displayPacket } from '$lib/stores/telemetry';

  let pkt = $derived($displayPacket);
  let rollDeg  = $derived(pkt ? pkt.roll  * 180 / Math.PI : 0);
  let pitchDeg = $derived(pkt ? pkt.pitch * 180 / Math.PI : 0);

  const PITCH_SCALE = 1.6;
  let pitchOffset = $derived(Math.min(Math.max(pitchDeg * PITCH_SCALE, -50), 50));
  let sphereRot   = $derived(-rollDeg);

  const ROLL_TICKS = [-60, -45, -30, -20, -10, 0, 10, 20, 30, 45, 60];
</script>

<div class="adi-wrap">
  <svg viewBox="-50 -50 100 100" class="adi-svg">
    <defs>
      <clipPath id="adi-clip">
        <circle cx="0" cy="0" r="46"/>
      </clipPath>
    </defs>

    <!-- Outer bezel -->
    <circle cx="0" cy="0" r="48" class="bezel-bg" stroke-width="1.5"/>

    <!-- Fixed bezel roll scale -->
    {#each ROLL_TICKS as tick}
      {@const angle = (tick - 90) * Math.PI / 180}
      {@const tickLen = tick === 0 ? 6 : Math.abs(tick) % 30 === 0 ? 5 : 3}
      {@const inner = 42 - tickLen}
      <line
        x1={42 * Math.cos(angle)} y1={42 * Math.sin(angle)}
        x2={inner * Math.cos(angle)} y2={inner * Math.sin(angle)}
        class={tick === 0 ? 'bezel-tick-zero' : 'bezel-tick'}
        stroke-width={tick === 0 ? 1.5 : 1}
        stroke-linecap="round"
      />
    {/each}

    <!-- Rotating sphere — clip applied here so it tracks the SVG origin -->
    <g transform="rotate({sphereRot})" clip-path="url(#adi-clip)"
       style="transition: transform 40ms linear;">
      <g transform="translate(0, {pitchOffset})" style="transition: transform 40ms linear;">
        <rect x="-200" y="-200" width="400" height="200" style="fill: var(--adi-sky);"/>
        <rect x="-200" y="0"    width="400" height="200" style="fill: var(--adi-ground);"/>
        <line x1="-200" y1="0" x2="200" y2="0" class="horizon-line"/>
        {#each [-25, -20, -15, -10, -5, 5, 10, 15, 20, 25] as deg}
          {@const y = -deg * PITCH_SCALE}
          {@const w = Math.abs(deg) % 10 === 0 ? 14 : 8}
          <line x1={-w} y1={y} x2={w} y2={y} class="pitch-rung" stroke-linecap="round"/>
          {#if Math.abs(deg) % 10 === 0}
            <text x={w + 2} y={y + 1.5} font-size="3.5" class="pitch-label"
              font-family="system-ui">{Math.abs(deg)}</text>
          {/if}
        {/each}
      </g>
    </g>

    <!-- Fixed aircraft wings -->
    <line x1="-38" y1="0" x2="-14" y2="0" class="wing" stroke-width="2.5" stroke-linecap="round"/>
    <line x1="14"  y1="0" x2="38"  y2="0" class="wing" stroke-width="2.5" stroke-linecap="round"/>
    <line x1="-14" y1="0" x2="-14" y2="5" class="wing" stroke-width="2.5" stroke-linecap="round"/>
    <line x1="14"  y1="0" x2="14"  y2="5" class="wing" stroke-width="2.5" stroke-linecap="round"/>
    <circle cx="0" cy="0" r="2.5" class="wing-dot"/>

    <!-- Roll pointer -->
    <g transform="rotate({rollDeg})" style="transition: transform 40ms linear;">
      <polygon points="0,-43 -3,-37 3,-37" class="roll-ptr"/>
    </g>

    <circle cx="0" cy="0" r="46" fill="none" class="bezel-ring" stroke-width="1"/>
  </svg>

  <div class="adi-readouts">
    <span class="adi-val">横滚 {rollDeg.toFixed(1)}°</span>
    <span class="adi-val">俯仰 {pitchDeg.toFixed(1)}°</span>
  </div>
</div>

<style>
  .adi-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.2rem;
    flex-shrink: 0;
  }
  .adi-svg {
    width: clamp(66px, 10.5vw, 100px);
    height: clamp(66px, 10.5vw, 100px);
    overflow: hidden;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .adi-readouts { display: flex; gap: 0.5rem; }
  .adi-val {
    font-size: clamp(0.42rem, 0.85vw, 0.54rem);
    font-weight: 700;
    color: var(--tx-xdim);
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.04em;
  }

  /* SVG element colours via CSS variables */
  .bezel-bg       { fill: var(--bg-panel); stroke: var(--bd-subtle); }
  .bezel-tick-zero{ stroke: var(--tx-dim); }
  .bezel-tick     { stroke: var(--bd-subtle); }
  .bezel-ring     { stroke: var(--bd-muted); }
  .horizon-line   { stroke: var(--tx-dim); stroke-width: 1.2; }
  .pitch-rung     { stroke: var(--bd-strong); stroke-width: 0.8; }
  .pitch-label    { fill: var(--bd-strong); }
  .wing           { stroke: var(--tx-hi); }
  .wing-dot       { fill: var(--tx-hi); }
  .roll-ptr       { fill: var(--tx-hi); }
</style>
