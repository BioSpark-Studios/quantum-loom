/**
 * QUANTUM LOOM — VAULT BROADCASTER
 * loom-broadcast.js
 *
 * Drop this file in your Quantum Loom folder alongside loom-snapshot.js.
 * It reads the live snapshot on each chapter tick and broadcasts
 * structured events to the parent Vault frame and socket.io server.
 *
 * Usage in quantum-loom-hub.html:
 *   <script src="loom-broadcast.js"></script>
 *   LoomBroadcast.init({ socket: yourSocketIOInstance });
 */

(function(global) {
'use strict';

let _socket     = null;
let _lastChapter = -1;
let _tickInterval = null;
let _active     = false;
let _emotionDebounce = null;

const LoomBroadcast = {

  init(opts = {}) {
    _socket = opts.socket || null;
    if (opts.autoTick !== false) {
      const tickMs = opts.tickInterval ?? 500;
      _tickInterval = setInterval(() => LoomBroadcast.tick(), tickMs);
    }
    _active = true;
    console.log('[LoomBroadcast] initialized', _socket ? '+ socket.io' : '(postMessage only)');
  },

  destroy() {
    _active = false;
    if (_tickInterval) { clearInterval(_tickInterval); _tickInterval = null; }
  },

  /* ── Manual tick (called by sequencer play or chapter advance) ── */
  tick(chapter) {
    if (!_active || typeof LoomSnapshot === 'undefined') return;
    const snap = LoomSnapshot.build();
    const ch   = chapter ?? (snap.chCount ? _lastChapter + 1 : 1);
    if (ch === _lastChapter) return;
    _lastChapter = ch;
    LoomBroadcast._fireChapterEvents(snap, ch);
  },

  /* ── Broadcast a one-off event (e.g. dialogue, seal) ─────────── */
  emit(type, data) {
    LoomBroadcast._send(type, data);
  },

  /* ── Core chapter event suite ────────────────────────────────── */
  _fireChapterEvents(snap, chapter) {

    const ctx = snap.getChapterContext(chapter);

    /* 1. ARC_VALUES — simplest, most useful for 3D shaders */
    const arcValues = {};
    const arcDeltas = {};
    snap.propagations.forEach(p => {
      arcValues[p.id || p.name] = snap.propValueAt(p, chapter);
      arcDeltas[p.id || p.name] = snap.propValueAt(p, chapter) - snap.propValueAt(p, chapter - 1);
    });
    LoomBroadcast._send('ARC_VALUES', { chapter, values: arcValues, deltas: arcDeltas });

    /* 2. ENV_STATE — weather from environ tracks + zone traits */
    const environClips = ctx.activeTracks
      .filter(t => t.type === 'environ')
      .flatMap(t => t.activeClips);

    const weather          = environClips[0]?.name?.toLowerCase().replace(/\s+/g,'-') || 'clear';
    const weatherIntensity = environClips[0]?.prose_weight ?? 0.5;
    const activeScene      = ctx.activeScene;

    LoomBroadcast._send('ENV_STATE', {
      chapter,
      weather,
      weatherIntensity,
      timeOfDay: activeScene?.time_of_day || 'DAY',
      season:    snap.meta?.season || 'Spring',
      zones: snap.zones.map(z => ({
        id:       z.id,
        name:     z.name,
        active:   activeScene ? (activeScene.zones || []).includes(z.id) : false,
        traits:   z.traits || {},
        heraldry: z.heraldry || {},
        actors:   z.actors  || [],
      })),
      activeGlyphEffects: LoomBroadcast._resolveGlyphEffects(snap, activeScene),
    });

    /* 3. CHAPTER_TICK — the master heartbeat */
    LoomBroadcast._send('CHAPTER_TICK', {
      chapter,
      bpm:       snap.bpm || 120,
      beat:      0,
      arcs:      arcValues,
      activeTracks: ctx.activeTracks.map(t => ({
        id:    t.id,
        type:  t.type,
        label: t.label || t.name,
        clips: t.activeClips.map(c => ({
          id:           c.id,
          name:         c.name,
          prose_weight: c.prose_weight ?? 0.5,
          capsule:      c.capsule ? { name: c.capsule.name, type: c.capsule.type, world_logic: c.capsule.logic } : null,
        })),
      })),
      activeScene: activeScene ? {
        id:          activeScene.id,
        name:        activeScene.name,
        chapter:     activeScene.chapter,
        scene_type:  activeScene.scene_type  || 'INT',
        time_of_day: activeScene.time_of_day || 'DAY',
        zone:        (activeScene.zones || [])[0] || '',
        actors:      (activeScene.resolvedActors || activeScene.actors || []).map(a =>
          typeof a === 'string' ? a : a.name
        ),
      } : null,
    });

    /* 4. CHARACTER_SNAPSHOT — one per active character */
    const activeActors = new Set(
      (activeScene?.resolvedActors || activeScene?.actors || [])
        .map(a => typeof a === 'string' ? a : a.id)
    );

    snap.characters.forEach(ch => {
      if (activeActors.size > 0 && !activeActors.has(ch.id) && !activeActors.has(ch.name)) return;
      LoomBroadcast._send('CHARACTER_SNAPSHOT', {
        id:             ch.id,
        name:           ch.name,
        role:           ch.role,
        faction:        ch.faction,
        hp:             ch.hp ?? 80,
        dialogue_voice: ch.dialogue_voice || '',
        vectors:        ch.vectors || {},
        traits:         ch.traits  || [],
        crest:          ch.crest   || null,
        totem:          ch.totem   || null,
        cipher:         ch.cipher  || null,
        sigils:         ch.sigils  || [],
      });
    });

    /* 5. CHARACTER_DIALOGUE — scripted lines for this scene */
    if (activeScene?.dialogue?.length) {
      activeScene.dialogue
        .sort((a,b) => (a.beat||0) - (b.beat||0))
        .forEach(line => {
          LoomBroadcast._send('CHARACTER_DIALOGUE', {
            characterId:   line.actorId   || line.actorName,
            characterName: line.actorName,
            line:          line.line || '',
            direction:     line.direction || '',
            beat:          line.beat || 0,
            sceneId:       activeScene.id,
          });
        });
    }
  },

  /* ── Resolve glyph world_logic effects for active scene ─────── */
  _resolveGlyphEffects(snap, scene) {
    if (!scene?.heraldry?.glyphs) return [];
    return scene.heraldry.glyphs
      .map(gId => snap.capsuleIdx?.[gId])
      .filter(Boolean)
      .map(cap => ({
        name:     cap.name,
        modifier: cap.logic ? { raw: cap.logic } : {},
      }));
  },

  /* ── MIDI CC broadcast (call from rack controller) ───────────── */
  emitMIDICC(unitId, unitLabel, cc, channel, value, label) {
    LoomBroadcast._send('MIDI_CC', {
      channel, cc, value,
      normalized: value / 127,
      label, unitId, unitLabel,
    });
  },

  /* ── MIDI Note broadcast (call from pad hit) ─────────────────── */
  emitMIDINote(unitId, note, noteName, velocity, state, padName, padIcon) {
    LoomBroadcast._send('MIDI_NOTE', {
      channel: 1, note, noteName, velocity, state,
      padName: padName || '', padIcon: padIcon || '', unitId,
    });
  },

  /* ── MIDI Clock (call from sequencer BPM loop) ───────────────── */
  emitMIDIClock(bpm, beat, bar, chapter, pulse) {
    LoomBroadcast._send('MIDI_CLOCK', { bpm, beat, bar, chapter, pulse });
  },

  /* ── Scene transition helpers ────────────────────────────────── */
  emitSceneEnter(scene, act, snap) {
    LoomBroadcast._send('SCENE_ENTER', {
      sceneId:     scene.id,
      name:        scene.name,
      chapter:     scene.chapter,
      actName:     act?.name || '',
      scene_type:  scene.scene_type  || 'INT',
      time_of_day: scene.time_of_day || 'DAY',
      zone:        (scene.zones || [])[0] || '',
      zoneTraits:  LoomBroadcast._getZoneTraits(scene, snap),
      actors:      (scene.resolvedActors || scene.actors || []).map(a =>
        typeof a === 'string' ? a : a.name
      ),
      heraldry:    scene.heraldry || {},
      notes:       scene.notes   || '',
    });
  },

  emitSceneExit(scene, nextSceneId) {
    LoomBroadcast._send('SCENE_EXIT', {
      sceneId:   scene.id,
      chapter:   scene.chapter,
      nextScene: nextSceneId || null,
    });
  },

  _getZoneTraits(scene, snap) {
    const zoneId   = (scene.zones || [])[0];
    const zone     = snap?.zones?.find(z => z.id === zoneId);
    return zone?.traits || {};
  },

  /* ── Transport ───────────────────────────────────────────────── */
  emitPlaybackStart(chapter, bpm, chCount, projectName) {
    LoomBroadcast._send('PLAYBACK_START', { chapter, bpm, chCount, projectName });
  },

  emitPlaybackStop(chapter) {
    LoomBroadcast._send('PLAYBACK_STOP', { chapter });
  },

  emitPlaybackSeek(fromChapter, toChapter) {
    LoomBroadcast._send('PLAYBACK_SEEK', { fromChapter, toChapter });
  },

  /* ── Core sender ─────────────────────────────────────────────── */
  _send(type, data) {
    const event = { source: 'quantum-loom', type, ts: Date.now(), data };

    // 1. postMessage to parent Vault frame
    try { window.parent?.postMessage(event, '*'); } catch(e) {}

    // 2. socket.io for multiplayer sync
    try { _socket?.emit('loom:event', event); } catch(e) {}

    // 3. Local event for same-page listeners (Electron, debug)
    try { window.dispatchEvent(new CustomEvent('loom:event', { detail: event })); } catch(e) {}
  },
};

/* ── Listen for agent reverse-channel events from Vault ─────────── */
window.addEventListener('message', e => {
  if (!e.data || e.data.source !== 'quantum-vault') return;
  const { type, data } = e.data;

  switch(type) {
    case 'AGENT_EMOTION': {
      /* Accumulate rapid emotion deltas and debounce the localStorage write */
      try {
        const ws = JSON.parse(localStorage.getItem('ql-workspace') || '[]');
        const ch = ws.find(c => c.id === data.agentId || c.name === data.agentName);
        if (ch && ch.vectors && data.emotion in ch.vectors) {
          ch.vectors[data.emotion] = Math.min(10, Math.max(0,
            (ch.vectors[data.emotion] || 5) + (data.delta || 0)
          ));
          clearTimeout(_emotionDebounce);
          _emotionDebounce = setTimeout(() => {
            try {
              localStorage.setItem('ql-workspace', JSON.stringify(ws));
            } catch(storageErr) {
              if (storageErr.name === 'QuotaExceededError') {
                console.warn('[LoomBroadcast] Storage quota exceeded — emotion update not persisted');
              } else {
                console.error('[LoomBroadcast] Emotion storage error:', storageErr.message);
              }
            }
          }, 200);
          console.log(`[LoomBroadcast] Agent emotion update: ${data.agentName}.${data.emotion} → ${ch.vectors[data.emotion]}`);
        }
      } catch(err) {
        console.error('[LoomBroadcast] AGENT_EMOTION handler error:', err.message);
      }
      break;
    }
    case 'AGENT_PORTAL_REQUEST': {
      console.log(`[LoomBroadcast] Portal request: ${data.agentId} → ${data.toVaultId}`);
      /* Emit SCENE_EXIT for the departing agent — host handles portal logic */
      LoomBroadcast._send('AGENT_PORTAL_REQUEST', data);
      break;
    }
    case 'AGENT_INTERACTION': {
      LoomBroadcast._send('AGENT_INTERACTION', data);
      break;
    }
  }
});

/* ── Export ──────────────────────────────────────────────────────── */
if (typeof module !== 'undefined' && module.exports) {
  module.exports = LoomBroadcast;
} else {
  global.LoomBroadcast = LoomBroadcast;
}

})(typeof window !== 'undefined' ? window : global);
