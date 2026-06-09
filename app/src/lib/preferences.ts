export const preferenceKeys = {
  scriptOrder: 'script.order',
  scriptBreakpoints: 'script.breakpoints',
  scriptApiPanelCollapsed: 'script.apiPanelCollapsed',
  scriptListCollapsed: 'script.listCollapsed',
  updatesDismissedVersion: 'updates.dismissedVersion',
  updatesSnoozeUntil: 'updates.snoozeUntil',
  floatingPanel: (id: string) => `floatingPanel:${id}`,
} as const

const legacyPreferenceKeys: Record<string, string[]> = {
  [preferenceKeys.scriptOrder]: ['script_order'],
  [preferenceKeys.scriptBreakpoints]: ['script_breakpoints'],
  [preferenceKeys.scriptApiPanelCollapsed]: ['api_panel_collapsed'],
  [preferenceKeys.scriptListCollapsed]: ['script_list_collapsed'],
}

function storage(): Storage | null {
  try {
    if (typeof window === 'undefined') return null
    return window.localStorage
  } catch (e) {
    console.warn('Unable to access preferences storage:', e)
    return null
  }
}

function parsePreference<T>(raw: string, fallback: T): T {
  try {
    return JSON.parse(raw) as T
  } catch {
    if (typeof fallback === 'boolean') {
      return (raw === 'true') as T
    }
    if (typeof fallback === 'number') {
      const parsed = Number(raw)
      return (Number.isFinite(parsed) ? parsed : fallback) as T
    }
    if (typeof fallback === 'string') {
      return raw as T
    }
    return fallback
  }
}

export function readPreference<T>(key: string, fallback: T, legacyKeys: string[] = legacyPreferenceKeys[key] ?? []): T {
  const store = storage()
  if (!store) return fallback

  try {
    const raw = store.getItem(key)
    if (raw !== null) return parsePreference(raw, fallback)

    for (const legacyKey of legacyKeys) {
      const legacyRaw = store.getItem(legacyKey)
      if (legacyRaw === null) continue

      const migrated = parsePreference(legacyRaw, fallback)
      writePreference(key, migrated)
      return migrated
    }
  } catch (e) {
    console.warn(`Unable to read preference "${key}":`, e)
  }

  return fallback
}

export function writePreference<T>(key: string, value: T) {
  const store = storage()
  if (!store) return

  try {
    store.setItem(key, JSON.stringify(value))
  } catch (e) {
    console.warn(`Unable to write preference "${key}":`, e)
  }
}

export function removePreference(key: string) {
  const store = storage()
  if (!store) return

  try {
    store.removeItem(key)
  } catch (e) {
    console.warn(`Unable to remove preference "${key}":`, e)
  }
}
