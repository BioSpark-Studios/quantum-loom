/**
 * Quantum Module orchestration for Music Video playback.
 * Modules 05 (Modeling/Animus), 06 (Choreography/Loom),
 * 07 (Behavior/Instinct), 10 (Story/Quill), 12 (Sound/Composer).
 *
 * Each function represents a module component translating
 * upstream signal into downstream wire output.
 */

// ── Types ─────────────────────────────────────────────────────────────────────

export interface EmotionVector {
  hope:    number; // 0-10
  fear:    number;
  resolve: number;
  grief:   number;
  wonder:  number;
  fury:    number;
}

export interface LyricSection {
  index:   number;
  label:   string; // "Verse 1", "Chorus", "Bridge", etc.
  lines:   string[];
}

export interface BehaviorSignal {
  motion:    string; // e.g. "hunched walk", "arms raised"
  place:     string; // location description
  emotion:   EmotionVector;
  intensity: number; // 0-1 overall emotional intensity
}

export interface LightingSignal {
  colorTemp:   string; // "warm amber", "cold blue", "harsh white"
  saturation:  string; // "desaturated", "vivid", "muted"
  shadows:     string; // "hard", "soft", "none"
  atmosphere:  string; // "foggy", "clear", "smoky", "rain"
  moodLabel:   string;
}

export interface CameraSignal {
  angle:     string; // "low angle", "eye level", "bird's eye", "dutch"
  distance:  string; // "extreme wide", "wide", "medium", "close", "extreme close"
  movement:  string; // "static", "slow pan", "tracking", "handheld", "crane"
  cutLength: number; // seconds
}

export interface ShotSpec {
  section:   LyricSection;
  behavior:  BehaviorSignal;
  lighting:  LightingSignal;
  camera:    CameraSignal;
  veoPrompt: string;
  sunoNote:  string; // hint for this section's sonic feel
}

export interface MusicVideoSpec {
  artistName:  string;
  songTitle:   string;
  bpm:         number;
  totalDuration: number; // seconds
  shots:       ShotSpec[];
  sunoPrompt:  string;   // full Suno prompt for the song
}

// ── Module 07: Behavior — Emotion-to-Motion (BHV wire) ───────────────────────

export function behaviorSignal(
  emotion: EmotionVector,
  location: string,
  lyricLines: string[]
): BehaviorSignal {
  const intensity = (emotion.grief + emotion.fury + emotion.fear) / 30
    - (emotion.hope + emotion.wonder) / 20;
  const clampedIntensity = Math.max(0, Math.min(1, 0.5 + intensity));

  // Dominant emotion drives motion tag
  const entries = Object.entries(emotion) as [keyof EmotionVector, number][];
  const dominant = entries.reduce((a, b) => b[1] > a[1] ? b : a)[0];

  const motionMap: Record<keyof EmotionVector, string[]> = {
    hope:    ['hands reaching upward', 'slow spin', 'walking toward light'],
    fear:    ['flinching', 'backing away slowly', 'huddled low'],
    resolve: ['standing firm', 'chin raised', 'steady forward march'],
    grief:   ['hunched walk', 'head down', 'knees to chest on ground'],
    wonder:  ['looking up in awe', 'arms slowly opening wide', 'turning slowly'],
    fury:    ['striking air', 'aggressive stride', 'screaming skyward'],
  };

  const motions = motionMap[dominant];
  const motion  = motions[Math.floor(clampedIntensity * (motions.length - 1))];

  return { motion, place: location, emotion, intensity: clampedIntensity };
}

// ── Module 04: Lighting — Mood Atmosphere Injector (VIS wire) ────────────────

export function lightingSignal(emotion: EmotionVector, timeOfDay?: string): LightingSignal {
  const { hope, fear, resolve, grief, wonder, fury } = emotion;

  if (grief > 7)   return { colorTemp: 'cold desaturated blue', saturation: 'desaturated', shadows: 'hard', atmosphere: 'heavy rain', moodLabel: 'grief' };
  if (fury  > 7)   return { colorTemp: 'harsh red-orange', saturation: 'vivid', shadows: 'hard', atmosphere: 'smoky', moodLabel: 'fury' };
  if (fear  > 7)   return { colorTemp: 'sickly green-grey', saturation: 'muted', shadows: 'deep', atmosphere: 'foggy', moodLabel: 'dread' };
  if (wonder > 7)  return { colorTemp: 'ethereal violet-gold', saturation: 'vivid', shadows: 'soft', atmosphere: 'clear star-filled', moodLabel: 'wonder' };
  if (hope  > 7)   return { colorTemp: 'warm amber sunrise', saturation: 'vivid', shadows: 'soft', atmosphere: 'clear', moodLabel: 'hope' };
  if (resolve > 7) return { colorTemp: 'cool steel blue', saturation: 'muted', shadows: 'hard', atmosphere: 'clear', moodLabel: 'resolve' };

  // Blended state
  const warmth = (hope + wonder) - (grief + fear);
  return warmth > 0
    ? { colorTemp: 'warm diffused golden', saturation: 'natural', shadows: 'soft', atmosphere: 'hazy', moodLabel: 'bittersweet' }
    : { colorTemp: 'cool grey-blue', saturation: 'muted', shadows: 'medium', atmosphere: 'overcast', moodLabel: 'melancholy' };
}

// ── Module 06: Choreography — Encounter Stage Director (BHV wire) ─────────────

export function cameraSignal(
  behavior: BehaviorSignal,
  sectionIndex: number,
  totalSections: number,
  bpm: number
): CameraSignal {
  const beatDuration  = 60 / bpm;
  const isChorus      = sectionIndex % 3 === 1; // rough chorus detection
  const isClimax      = sectionIndex > totalSections * 0.7;
  const { intensity } = behavior;

  // Cut length: high intensity = faster cuts
  const baseCut  = isChorus ? beatDuration * 4 : beatDuration * 8;
  const cutLength = Math.max(2, baseCut * (1 - intensity * 0.5));

  // Camera angle driven by emotional power
  const angle = intensity > 0.7 ? 'low angle'
    : behavior.emotion.wonder > 6 ? 'bird\'s eye'
    : behavior.emotion.grief  > 6 ? 'slightly low'
    : 'eye level';

  // Distance driven by emotional intimacy
  const distance = isClimax ? 'extreme close'
    : isChorus ? 'medium'
    : intensity > 0.6 ? 'close'
    : 'wide';

  // Movement driven by intensity
  const movement = intensity > 0.8 ? 'handheld'
    : isChorus ? 'slow tracking'
    : behavior.emotion.wonder > 6 ? 'slow crane up'
    : behavior.emotion.grief  > 6 ? 'static'
    : 'slow pan';

  return { angle, distance, movement, cutLength };
}

// ── Module 10: Story — Narrative Beat Publisher (NAR wire) ────────────────────

export function parseLyricSections(lyrics: string): LyricSection[] {
  const sections: LyricSection[] = [];
  const blocks = lyrics.split(/\n{2,}/);
  let index = 0;

  for (const block of blocks) {
    const trimmed = block.trim();
    if (!trimmed) continue;

    const lines = trimmed.split('\n').map(l => l.trim()).filter(Boolean);

    // Detect label from first line if bracketed
    const labelMatch = lines[0].match(/^\[(.+)\]$/);
    const label = labelMatch
      ? labelMatch[1]
      : index === 0 ? 'Intro'
      : index % 3 === 0 ? `Verse ${Math.ceil(index / 3)}`
      : index % 3 === 1 ? 'Chorus'
      : 'Bridge';

    sections.push({
      index,
      label,
      lines: labelMatch ? lines.slice(1) : lines,
    });
    index++;
  }

  return sections;
}

// ── Module 12: Sound — Suno Prompt Builder (NAR→AUD wire) ────────────────────

export function buildSunoPrompt(
  artistName: string,
  emotionVectors: EmotionVector[],
  genre: string,
  worldContext: string
): string {
  // Aggregate dominant emotions across the song
  const avg = (key: keyof EmotionVector) =>
    emotionVectors.reduce((s, v) => s + v[key], 0) / emotionVectors.length;

  const hopeMean    = avg('hope');
  const griefMean   = avg('grief');
  const furyMean    = avg('fury');
  const wonderMean  = avg('wonder');
  const resolveMean = avg('resolve');

  const moodTags: string[] = [];
  if (griefMean   > 5) moodTags.push('melancholic', 'raw');
  if (furyMean    > 5) moodTags.push('intense', 'driving');
  if (hopeMean    > 6) moodTags.push('uplifting', 'anthemic');
  if (wonderMean  > 6) moodTags.push('ethereal', 'cinematic');
  if (resolveMean > 6) moodTags.push('determined', 'powerful');
  if (moodTags.length === 0) moodTags.push('atmospheric', 'emotive');

  // Emotion-to-timbre mapping (ETM component)
  const timbreHints: string[] = [];
  if (griefMean > 6)   timbreHints.push('sparse piano', 'strings');
  if (furyMean  > 6)   timbreHints.push('distorted guitar', 'heavy drums');
  if (wonderMean > 6)  timbreHints.push('ambient pads', 'ethereal choir');
  if (hopeMean  > 6)   timbreHints.push('soaring vocals', 'bright synths');
  if (resolveMean > 6) timbreHints.push('driving bass', 'percussion');

  return [
    `${genre}, ${moodTags.join(', ')}`,
    timbreHints.length ? timbreHints.join(', ') : 'organic instrumentation',
    `artist: ${artistName}`,
    worldContext ? `world: ${worldContext}` : '',
    'high quality, professional production',
  ].filter(Boolean).join(', ');
}

// ── Module Orchestrator — full playback pipeline ──────────────────────────────

export function orchestrateMusicVideo(
  artist: {
    name: string;
    emotionVector: EmotionVector;
    wound?: string;
    goal?: string;
    race?: string;
  },
  lyrics: string,
  options: {
    bpm: number;
    genre: string;
    primaryLocation: string;
    worldContext: string;
    songTitle: string;
  }
): MusicVideoSpec {
  const sections     = parseLyricSections(lyrics);
  const totalSeconds = sections.length * (60 / options.bpm) * 8;

  // Arc: emotion evolves from wound toward goal across the song
  const shots: ShotSpec[] = sections.map((section, i) => {
    const progress = i / Math.max(1, sections.length - 1);

    // Behavior module: evolve emotion across song arc
    const evolvedEmotion: EmotionVector = {
      hope:    artist.emotionVector.hope    + progress * 2,
      fear:    artist.emotionVector.fear    * (1 - progress * 0.5),
      resolve: artist.emotionVector.resolve + progress * 3,
      grief:   artist.emotionVector.grief   * (1 - progress * 0.3),
      wonder:  artist.emotionVector.wonder,
      fury:    artist.emotionVector.fury    * (1 - progress * 0.4),
    };
    // Clamp 0-10
    for (const k of Object.keys(evolvedEmotion) as (keyof EmotionVector)[]) {
      evolvedEmotion[k] = Math.max(0, Math.min(10, evolvedEmotion[k]));
    }

    const behavior = behaviorSignal(evolvedEmotion, options.primaryLocation, section.lines);
    const lighting = lightingSignal(evolvedEmotion);
    const camera   = cameraSignal(behavior, i, sections.length, options.bpm);

    // Forge: LLM Prompt Architect assembles Veo prompt
    const veoPrompt = [
      `${camera.angle} ${camera.distance} shot`,
      `${camera.movement} camera`,
      `${artist.name}${artist.race ? ', ' + artist.race : ''}`,
      behavior.motion,
      `in ${behavior.place}`,
      `${lighting.colorTemp} lighting`,
      lighting.atmosphere,
      `${lighting.shadows} shadows`,
      lighting.saturation,
      'cinematic 4K music video',
      `mood: ${lighting.moodLabel}`,
    ].join(', ');

    const sunoNote = `[${section.label}] ${lighting.moodLabel} — ${
      Object.entries(evolvedEmotion)
        .filter(([, v]) => v > 6)
        .map(([k]) => k)
        .join('+') || 'balanced'
    }`;

    return { section, behavior, lighting, camera, veoPrompt, sunoNote };
  });

  const sunoPrompt = buildSunoPrompt(
    artist.name,
    shots.map(s => s.behavior.emotion),
    options.genre,
    options.worldContext
  );

  return {
    artistName:    artist.name,
    songTitle:     options.songTitle,
    bpm:           options.bpm,
    totalDuration: totalSeconds,
    shots,
    sunoPrompt,
  };
}
