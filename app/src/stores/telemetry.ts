import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { ipc } from '../fh6-tel/lib/ipc'
import type { TelemetryPacket } from '../fh6-tel/lib/types'

export const useTelemetryStore = defineStore('telemetry', () => {
  const packet = ref<TelemetryPacket | null>(null)
  const isConnected = ref(false)

  interface ReplayState {
    active: boolean
    packets: TelemetryPacket[]
    index: number
    playing: boolean
    speed: number
    sessionId: number | null
    label: string
  }

  const emptyReplay: ReplayState = {
    active: false,
    packets: [],
    index: 0,
    playing: false,
    speed: 1,
    sessionId: null,
    label: '',
  }

  const replay = ref<ReplayState>({ ...emptyReplay })

  function startReplay(sessionId: number, label: string, packets: TelemetryPacket[]) {
    replay.value = {
      active: true,
      packets,
      index: 0,
      playing: false,
      speed: 1,
      sessionId,
      label,
    }
  }

  function exitReplay() {
    replay.value = { ...emptyReplay }
  }

  let _frozen: TelemetryPacket | null = null

  const displayPacket = computed((): TelemetryPacket | null => {
    const $r = replay.value
    const $p = packet.value
    if ($r.active && $r.packets.length > 0) {
      const i = Math.min(Math.max($r.index, 0), $r.packets.length - 1)
      return $r.packets[i]
    }
    if ($p !== null && $p.isRaceOn) {
      _frozen = $p
      return $p
    }
    return _frozen ?? $p
  })

  const speedMph = computed(() => {
    const $p = displayPacket.value
    return $p ? $p.speedMs * 2.23694 : 0
  })

  const speedKph = computed(() => {
    const $p = displayPacket.value
    return $p ? $p.speedMs * 3.6 : 0
  })

  const rpmPercent = computed(() => {
    const $p = displayPacket.value
    if (!$p || $p.engineMaxRpm === 0) return 0
    return ($p.currentEngineRpm / $p.engineMaxRpm) * 100
  })

  let lastPacketTime = 0
  let connectionTimer: ReturnType<typeof setInterval> | null = null
  let unsubscribeTelemetry: (() => void) | null = null

  async function startTelemetryListener(handlers: { onError?: (msg: string) => void; onBindFailed?: (msg: string) => void } = {}) {
    stopTelemetryListener()

    unsubscribeTelemetry = await ipc.subscribeTelemetry({
      onTick: (payload: TelemetryPacket) => {
        packet.value = payload
        lastPacketTime = Date.now()
        isConnected.value = true
      },
      onBindFailed: handlers.onBindFailed,
      onError: handlers.onError,
    })

    if (connectionTimer) clearInterval(connectionTimer)
    connectionTimer = setInterval(() => {
      if (Date.now() - lastPacketTime > 2000) {
        isConnected.value = false
      }
    }, 1000)
  }

  function stopTelemetryListener() {
    unsubscribeTelemetry?.()
    unsubscribeTelemetry = null

    if (connectionTimer) {
      clearInterval(connectionTimer)
      connectionTimer = null
    }

    lastPacketTime = 0
    isConnected.value = false
  }

  return {
    packet,
    isConnected,
    replay,
    displayPacket,
    speedMph,
    speedKph,
    rpmPercent,
    startReplay,
    exitReplay,
    startTelemetryListener,
    stopTelemetryListener,
  }
})
