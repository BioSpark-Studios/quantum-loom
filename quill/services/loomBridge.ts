/**
 * Loom Bridge — reads Quantum Loom localStorage data into Quill structures.
 * When Quill runs standalone (browser) it reads directly from localStorage.
 * When Quill is embedded as a Loom layer it receives data via postMessage.
 */

import { Scene, GenesisContainer, MythosContainer, BaseCapsule, ContainerType } from '../types';

// ── Read helpers ──────────────────────────────────────────────────────────────

function lsGet<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? JSON.parse(raw) : fallback;
  } catch {
    return fallback;
  }
}

// ── Public API ────────────────────────────────────────────────────────────────

export interface LoomWorldSummary {
  projectName: string;
  characters: { id: string; name: string; race?: string; role?: string; wound?: string; goal?: string }[];
  zones: { id: string; name: string }[];
  scenes: { id: string; title: string; desc?: string; chapter?: number }[];
  factions: string[];
  worldName?: string;
  biome?: string;
}

/** Pull the current Loom world state into a summary object. */
export function readLoomWorld(): LoomWorldSummary {
  const meta   = lsGet<any>('ql-workspace-meta', {});
  const chars  = lsGet<any[]>('ql-workspace', []);
  const zones  = lsGet<any[]>('ql-zones', []);
  const scenes = lsGet<any[]>('ql-scenes', []);
  const terrain  = lsGet<any>('qa-terrain-snapshot', null);
  const society  = lsGet<any>('qa-society-snapshot', null);

  return {
    projectName: meta.name || 'Untitled Loom Project',
    characters: chars.map(c => ({ id: c.id, name: c.name, race: c.race, role: c.role, wound: c.wound, goal: c.goal })),
    zones: zones.map(z => ({ id: z.id, name: z.name || z.label || z.id })),
    scenes: scenes.map(s => ({ id: s.id, title: s.title || s.name || '', desc: s.desc, chapter: s.chapter })),
    factions: society?.factions?.map((f: any) => f.name || f) ?? [],
    worldName: terrain?.worldName,
    biome: terrain?.biome,
  };
}

/** Convert Loom scenes into Quill Scene objects for the timeline. */
export function importLoomScenes(): Scene[] {
  const raw = lsGet<any[]>('ql-scenes', []);
  return raw.map((s, i) => ({
    id: s.id || `loom-scene-${i}`,
    number: s.chapter ?? i + 1,
    heading: s.title || s.name || `Scene ${i + 1}`,
    description: s.desc || '',
    characters: [],
    dialogue: [],
    setting: '',
    timeOfDay: '',
    mood: '',
    visualPrompt: s.desc || s.title || '',
    audioNotes: '',
    duration: 5,
    status: 'pending' as const,
  }));
}

/** Check whether any Loom world data exists in this browser session. */
export function hasLoomData(): boolean {
  return !!(localStorage.getItem('ql-workspace') || localStorage.getItem('ql-scenes'));
}

/** Format world context as a prompt string for Gemini/LLM calls. */
export function worldContextPrompt(world: LoomWorldSummary): string {
  const lines: string[] = [`World: "${world.projectName}"`];
  if (world.worldName) lines.push(`Setting: ${world.worldName} (${world.biome ?? 'unknown biome'})`);
  if (world.characters.length) lines.push(`Characters: ${world.characters.map(c => `${c.name}${c.race ? ' (' + c.race + ')' : ''}${c.role ? ' — ' + c.role : ''}`).join(', ')}`);
  if (world.factions.length) lines.push(`Factions: ${world.factions.join(', ')}`);
  if (world.zones.length) lines.push(`Locations: ${world.zones.map(z => z.name).join(', ')}`);
  return lines.join('\n');
}
