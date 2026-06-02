import fh6Ordinals from '$lib/fh6-car-ordinals.json';
import legacyOrdinals from '$lib/car-ordinals.json';

// FH6 game-confirmed entries take priority; legacy FM8/FH5 data fills the rest
const map: Record<string, string> = {
  ...(legacyOrdinals as Record<string, string>),
  ...(fh6Ordinals as Record<string, string>),
};

export function carName(ordinal: number): string {
  return map[String(ordinal)] ?? `Car #${ordinal}`;
}
