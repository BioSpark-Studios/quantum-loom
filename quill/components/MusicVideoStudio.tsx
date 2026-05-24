import React, { useState, useCallback } from 'react';
import { Music, Film, Zap, Copy, ChevronDown, ChevronUp, Play, Loader } from 'lucide-react';
import { readLoomWorld, hasLoomData, worldContextPrompt } from '../services/loomBridge';
import {
  orchestrateMusicVideo, MusicVideoSpec, ShotSpec,
  EmotionVector, LyricSection
} from '../services/modules';
import { generateSceneVideo, generateSceneImage } from '../services/gemini';

// ── Helpers ───────────────────────────────────────────────────────────────────

function EmotionBar({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div className="flex items-center gap-2 text-xs">
      <span className="w-14 text-right text-neutral-400 shrink-0">{label}</span>
      <div className="flex-1 h-1.5 bg-neutral-800 rounded-full overflow-hidden">
        <div className="h-full rounded-full transition-all" style={{ width: `${value * 10}%`, background: color }} />
      </div>
      <span className="w-4 text-neutral-500 shrink-0">{value.toFixed(0)}</span>
    </div>
  );
}

function ShotCard({ shot, index, onGenerate }: {
  shot: ShotSpec;
  index: number;
  onGenerate: (shot: ShotSpec, index: number) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const [imgUrl, setImgUrl]     = useState<string | null>(null);
  const [loading, setLoading]   = useState(false);

  const handlePreview = async () => {
    setLoading(true);
    try {
      const url = await generateSceneImage(shot.veoPrompt);
      setImgUrl(url);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="border border-neutral-800 rounded-xl overflow-hidden bg-neutral-900/50">
      {/* Shot header */}
      <div
        className="flex items-center gap-3 p-3 cursor-pointer hover:bg-neutral-800/40 transition-colors"
        onClick={() => setExpanded(e => !e)}
      >
        <div className="w-8 h-8 rounded-lg flex items-center justify-center text-xs font-bold shrink-0"
          style={{ background: 'rgba(220,60,120,0.15)', color: '#dc3c78' }}>
          {index + 1}
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-xs font-semibold text-neutral-300">{shot.section.label}</span>
            <span className="text-xs text-neutral-600">·</span>
            <span className="text-xs text-neutral-500">{shot.camera.distance} · {shot.camera.movement}</span>
            <span className="text-xs text-neutral-600">·</span>
            <span className="text-xs" style={{ color: '#e0d8ff' }}>{shot.lighting.moodLabel}</span>
          </div>
          <p className="text-xs text-neutral-600 truncate mt-0.5">
            {shot.section.lines[0] || '—'}
          </p>
        </div>
        <span className="text-xs text-neutral-600 shrink-0">{shot.camera.cutLength.toFixed(1)}s</span>
        {expanded ? <ChevronUp className="w-3 h-3 text-neutral-600" /> : <ChevronDown className="w-3 h-3 text-neutral-600" />}
      </div>

      {expanded && (
        <div className="border-t border-neutral-800 p-3 space-y-3">
          {/* Image preview */}
          {imgUrl ? (
            <img src={imgUrl} alt="Shot preview" className="w-full rounded-lg object-cover" style={{ maxHeight: 180 }} />
          ) : (
            <div className="w-full rounded-lg bg-neutral-800/50 flex items-center justify-center" style={{ height: 120 }}>
              {loading
                ? <Loader className="w-5 h-5 text-neutral-500 animate-spin" />
                : <button onClick={handlePreview} className="text-xs text-neutral-500 hover:text-neutral-300 transition-colors">
                    Preview still →
                  </button>
              }
            </div>
          )}

          {/* Veo prompt */}
          <div>
            <div className="flex items-center justify-between mb-1">
              <span className="text-xs text-neutral-500 font-mono">VEO PROMPT</span>
              <button
                onClick={() => navigator.clipboard.writeText(shot.veoPrompt)}
                className="text-xs text-neutral-600 hover:text-neutral-400 flex items-center gap-1 transition-colors"
              >
                <Copy className="w-3 h-3" /> copy
              </button>
            </div>
            <p className="text-xs text-neutral-300 bg-neutral-800/50 rounded-lg p-2 leading-relaxed font-mono">
              {shot.veoPrompt}
            </p>
          </div>

          {/* Module signals */}
          <div className="grid grid-cols-2 gap-3">
            <div>
              <p className="text-xs text-neutral-600 mb-1 font-mono">BEHAVIOR · 07</p>
              <p className="text-xs text-neutral-400">{shot.behavior.motion}</p>
              <p className="text-xs text-neutral-600">{shot.behavior.place}</p>
            </div>
            <div>
              <p className="text-xs text-neutral-600 mb-1 font-mono">LIGHTING · 04</p>
              <p className="text-xs text-neutral-400">{shot.lighting.colorTemp}</p>
              <p className="text-xs text-neutral-600">{shot.lighting.atmosphere}</p>
            </div>
          </div>

          {/* Emotion vector */}
          <div className="space-y-1">
            <p className="text-xs text-neutral-600 font-mono mb-1">EMOTION ARC · 05</p>
            <EmotionBar label="hope"    value={shot.behavior.emotion.hope}    color="#f4c025" />
            <EmotionBar label="grief"   value={shot.behavior.emotion.grief}   color="#8050e0" />
            <EmotionBar label="resolve" value={shot.behavior.emotion.resolve} color="#1e8cff" />
            <EmotionBar label="fury"    value={shot.behavior.emotion.fury}    color="#ff6400" />
            <EmotionBar label="wonder"  value={shot.behavior.emotion.wonder}  color="#00c060" />
            <EmotionBar label="fear"    value={shot.behavior.emotion.fear}    color="#dc3c78" />
          </div>

          <button
            onClick={() => onGenerate(shot, index)}
            className="w-full py-2 rounded-lg text-xs font-semibold flex items-center justify-center gap-2 transition-all"
            style={{ background: 'rgba(220,60,120,0.15)', color: '#dc3c78', border: '1px solid rgba(220,60,120,0.25)' }}
          >
            <Film className="w-3 h-3" /> Generate Video Clip (Veo)
          </button>
        </div>
      )}
    </div>
  );
}

// ── Main Component ────────────────────────────────────────────────────────────

export default function MusicVideoStudio() {
  const world = readLoomWorld();
  const loomActive = hasLoomData();

  // Artist selection
  const [artistIndex, setArtistIndex] = useState(0);
  const artist = world.characters[artistIndex] ?? null;

  // Song inputs
  const [songTitle,    setSongTitle]    = useState('');
  const [lyrics,       setLyrics]       = useState('');
  const [genre,        setGenre]        = useState('cinematic pop');
  const [bpm,          setBpm]          = useState(96);
  const [location,     setLocation]     = useState(world.zones[0]?.name ?? 'desolate alien landscape');

  // Override emotion per-axis (if no Loom character)
  const [manualEmotion, setManualEmotion] = useState<EmotionVector>({
    hope: 5, fear: 4, resolve: 6, grief: 7, wonder: 5, fury: 3
  });

  // Output
  const [spec,      setSpec]      = useState<MusicVideoSpec | null>(null);
  const [loading,   setLoading]   = useState(false);
  const [activeTab, setActiveTab] = useState<'shots' | 'suno'>('shots');
  const [genStatus, setGenStatus] = useState<Record<number, 'idle' | 'loading' | 'done'>>({});
  const [videoUrls, setVideoUrls] = useState<Record<number, string>>({});

  const effectiveEmotion: EmotionVector = artist?.emotionVector
    ? artist.emotionVector as unknown as EmotionVector
    : manualEmotion;

  const handleOrchestrate = useCallback(() => {
    if (!lyrics.trim()) return;
    setLoading(true);
    try {
      const result = orchestrateMusicVideo(
        {
          name: artist?.name ?? 'Unknown Artist',
          emotionVector: effectiveEmotion,
          wound: artist?.wound,
          goal: artist?.goal,
          race: artist?.race,
        },
        lyrics,
        {
          bpm,
          genre,
          primaryLocation: location,
          worldContext: worldContextPrompt(world),
          songTitle: songTitle || 'Untitled',
        }
      );
      setSpec(result);
      setGenStatus({});
      setVideoUrls({});
    } finally {
      setLoading(false);
    }
  }, [artist, effectiveEmotion, lyrics, bpm, genre, location, songTitle, world]);

  const handleGenerateClip = useCallback(async (shot: ShotSpec, index: number) => {
    setGenStatus(s => ({ ...s, [index]: 'loading' }));
    try {
      const url = await generateSceneVideo(shot.veoPrompt);
      setVideoUrls(v => ({ ...v, [index]: url }));
      setGenStatus(s => ({ ...s, [index]: 'done' }));
    } catch (e) {
      setGenStatus(s => ({ ...s, [index]: 'idle' }));
      alert('Video generation failed: ' + (e as Error).message);
    }
  }, []);

  return (
    <div className="flex flex-col h-full overflow-hidden" style={{ color: '#e5e5e5', fontFamily: 'Inter, sans-serif' }}>

      {/* Header */}
      <div className="flex items-center gap-3 px-6 py-4 border-b border-neutral-800 shrink-0">
        <div className="w-8 h-8 rounded-lg flex items-center justify-center" style={{ background: 'rgba(220,60,120,0.15)' }}>
          <Music className="w-4 h-4" style={{ color: '#dc3c78' }} />
        </div>
        <div>
          <h2 className="font-bold text-white text-sm">Music Video Studio</h2>
          <p className="text-xs text-neutral-500">Choreography · Lighting · Behavior · Sound · Story</p>
        </div>
        {loomActive && (
          <span className="ml-auto text-xs px-2 py-1 rounded-full" style={{ background: 'rgba(200,151,58,0.15)', color: '#c8973a', border: '1px solid rgba(200,151,58,0.25)' }}>
            ◈ Loom connected — {world.projectName}
          </span>
        )}
      </div>

      <div className="flex flex-1 overflow-hidden">

        {/* Left panel — inputs */}
        <div className="w-72 shrink-0 border-r border-neutral-800 overflow-y-auto p-4 space-y-5">

          {/* Artist */}
          <div>
            <label className="text-xs text-neutral-500 font-mono block mb-2">ARTIST · MOD-05 ANIMUS</label>
            {loomActive && world.characters.length > 0 ? (
              <select
                value={artistIndex}
                onChange={e => setArtistIndex(Number(e.target.value))}
                className="w-full bg-neutral-800 border border-neutral-700 rounded-lg px-3 py-2 text-sm text-white"
              >
                {world.characters.map((c, i) => (
                  <option key={c.id} value={i}>{c.name}{c.race ? ` (${c.race})` : ''}</option>
                ))}
              </select>
            ) : (
              <p className="text-xs text-neutral-600 italic">No Loom characters — using manual emotion below.</p>
            )}

            {artist && (
              <div className="mt-2 p-2 rounded-lg bg-neutral-800/40 text-xs space-y-1">
                {artist.role && <p className="text-neutral-400">{artist.role}</p>}
                {artist.wound && <p className="text-neutral-500">Wound: {artist.wound}</p>}
                {artist.goal  && <p className="text-neutral-500">Goal: {artist.goal}</p>}
              </div>
            )}
          </div>

          {/* Manual emotion (shown when no Loom character) */}
          {(!loomActive || world.characters.length === 0) && (
            <div>
              <label className="text-xs text-neutral-500 font-mono block mb-2">EMOTION VECTOR · MOD-07</label>
              {(Object.keys(manualEmotion) as (keyof EmotionVector)[]).map(key => (
                <div key={key} className="flex items-center gap-2 mb-1">
                  <span className="text-xs text-neutral-500 w-14 text-right">{key}</span>
                  <input type="range" min={0} max={10} step={1}
                    value={manualEmotion[key]}
                    onChange={e => setManualEmotion(v => ({ ...v, [key]: Number(e.target.value) }))}
                    className="flex-1 accent-pink-500"
                  />
                  <span className="text-xs text-neutral-500 w-3">{manualEmotion[key]}</span>
                </div>
              ))}
            </div>
          )}

          {/* Song */}
          <div>
            <label className="text-xs text-neutral-500 font-mono block mb-2">SONG · MOD-12 COMPOSER</label>
            <input
              type="text"
              placeholder="Song title"
              value={songTitle}
              onChange={e => setSongTitle(e.target.value)}
              className="w-full bg-neutral-800 border border-neutral-700 rounded-lg px-3 py-2 text-sm text-white mb-2 placeholder-neutral-600"
            />
            <input
              type="text"
              placeholder="Genre (e.g. cinematic pop)"
              value={genre}
              onChange={e => setGenre(e.target.value)}
              className="w-full bg-neutral-800 border border-neutral-700 rounded-lg px-3 py-2 text-sm text-white mb-2 placeholder-neutral-600"
            />
            <div className="flex items-center gap-2 mb-2">
              <span className="text-xs text-neutral-500">BPM</span>
              <input type="number" min={60} max={200} value={bpm}
                onChange={e => setBpm(Number(e.target.value))}
                className="w-20 bg-neutral-800 border border-neutral-700 rounded-lg px-2 py-1.5 text-sm text-white"
              />
            </div>
          </div>

          {/* Location */}
          <div>
            <label className="text-xs text-neutral-500 font-mono block mb-2">LOCATION · MOD-01 ATLAS</label>
            {loomActive && world.zones.length > 0 ? (
              <select
                value={location}
                onChange={e => setLocation(e.target.value)}
                className="w-full bg-neutral-800 border border-neutral-700 rounded-lg px-3 py-2 text-sm text-white"
              >
                {world.zones.map(z => <option key={z.id} value={z.name}>{z.name}</option>)}
                <option value="custom">Custom…</option>
              </select>
            ) : (
              <input
                type="text"
                placeholder="Primary location / setting"
                value={location}
                onChange={e => setLocation(e.target.value)}
                className="w-full bg-neutral-800 border border-neutral-700 rounded-lg px-3 py-2 text-sm text-white placeholder-neutral-600"
              />
            )}
          </div>

          {/* Lyrics */}
          <div>
            <label className="text-xs text-neutral-500 font-mono block mb-2">LYRICS · MOD-10 QUILL</label>
            <p className="text-xs text-neutral-600 mb-1">Separate sections with blank lines. Optional: [Verse 1], [Chorus], etc.</p>
            <textarea
              value={lyrics}
              onChange={e => setLyrics(e.target.value)}
              placeholder={"[Verse 1]\nFirst line\nSecond line\n\n[Chorus]\nChorus line"}
              rows={10}
              className="w-full bg-neutral-800 border border-neutral-700 rounded-lg px-3 py-2 text-sm text-white placeholder-neutral-600 resize-y font-mono"
            />
          </div>

          {/* Orchestrate */}
          <button
            onClick={handleOrchestrate}
            disabled={loading || !lyrics.trim()}
            className="w-full py-3 rounded-xl font-bold text-sm flex items-center justify-center gap-2 transition-all disabled:opacity-40"
            style={{ background: loading ? 'rgba(220,60,120,0.2)' : 'rgba(220,60,120,0.85)', color: 'white' }}
          >
            {loading
              ? <><Loader className="w-4 h-4 animate-spin" /> Orchestrating…</>
              : <><Zap className="w-4 h-4" /> Direct the Video</>
            }
          </button>
        </div>

        {/* Right panel — output */}
        <div className="flex-1 overflow-y-auto">
          {!spec ? (
            <div className="flex flex-col items-center justify-center h-full text-center p-12">
              <Film className="w-12 h-12 text-neutral-700 mb-4" />
              <p className="text-neutral-500 text-sm">Add lyrics and hit <strong className="text-neutral-400">Direct the Video</strong></p>
              <p className="text-neutral-700 text-xs mt-2">The modules will map your artist's emotional arc across every shot.</p>
            </div>
          ) : (
            <div className="p-5">
              {/* Song summary */}
              <div className="flex items-center justify-between mb-4">
                <div>
                  <h3 className="font-bold text-white">{spec.songTitle}</h3>
                  <p className="text-xs text-neutral-500">{spec.artistName} · {spec.shots.length} shots · {Math.round(spec.totalDuration)}s</p>
                </div>
                <div className="flex gap-2">
                  <button onClick={() => setActiveTab('shots')}
                    className="text-xs px-3 py-1.5 rounded-lg transition-all"
                    style={activeTab === 'shots' ? { background: 'rgba(220,60,120,0.2)', color: '#dc3c78' } : { color: '#666' }}>
                    Shot List
                  </button>
                  <button onClick={() => setActiveTab('suno')}
                    className="text-xs px-3 py-1.5 rounded-lg transition-all"
                    style={activeTab === 'suno' ? { background: 'rgba(220,140,30,0.2)', color: '#dc8c1e' } : { color: '#666' }}>
                    Suno Prompt
                  </button>
                </div>
              </div>

              {activeTab === 'suno' && (
                <div className="mb-5">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-xs text-neutral-500 font-mono">SUNO PROMPT · MOD-12 SPB</span>
                    <button
                      onClick={() => navigator.clipboard.writeText(spec.sunoPrompt)}
                      className="text-xs text-neutral-600 hover:text-neutral-400 flex items-center gap-1 transition-colors"
                    >
                      <Copy className="w-3 h-3" /> copy
                    </button>
                  </div>
                  <p className="text-sm text-neutral-200 bg-neutral-800/60 rounded-xl p-4 font-mono leading-relaxed">
                    {spec.sunoPrompt}
                  </p>
                  <div className="mt-3 space-y-1">
                    {spec.shots.map((s, i) => (
                      <p key={i} className="text-xs text-neutral-500 font-mono">{s.sunoNote}</p>
                    ))}
                  </div>
                </div>
              )}

              {activeTab === 'shots' && (
                <div className="space-y-2">
                  {spec.shots.map((shot, i) => (
                    <div key={i}>
                      {videoUrls[i] ? (
                        <div className="rounded-xl overflow-hidden border border-neutral-700 mb-1">
                          <video src={videoUrls[i]} controls className="w-full" />
                          <p className="text-xs text-neutral-600 px-3 py-1">{shot.section.label}</p>
                        </div>
                      ) : null}
                      <ShotCard
                        shot={shot}
                        index={i}
                        onGenerate={handleGenerateClip}
                      />
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
