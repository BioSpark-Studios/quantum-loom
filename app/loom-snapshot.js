/* ╔══════════════════════════════════════════════════════════════════╗
   ║         QUANTUM LOOM — SNAPSHOT ASSEMBLER  v1.0                 ║
   ║  Reads all ql-* keys, assembles the unified LoomSnapshot,       ║
   ║  resolves cross-references, and reports missing data warnings.   ║
   ╚══════════════════════════════════════════════════════════════════╝ */

(function(global) {
'use strict';

/* ── Safe JSON parse ─────────────────────────────────────────────── */
function _j(key, fallback) {
  try {
    const v = localStorage.getItem(key);
    return v ? JSON.parse(v) : fallback;
  } catch(e) { return fallback; }
}

/* ══ buildSnapshot ════════════════════════════════════════════════════
   Assembles and returns a LoomSnapshot from all ql-* storage keys.
   Safe to call at any time — missing keys return empty arrays/objects.
═══════════════════════════════════════════════════════════════════════ */
function buildSnapshot() {
  const warnings = [];

  /* ── Raw reads ────────────────────────────────────────────────── */
  const meta          = _j('ql-workspace-meta', {}) || {};
  const characters    = Array.isArray(_j('ql-workspace', [])) ? _j('ql-workspace', []) : [];
  const capsules      = _j('ql-library', []) || [];
  const acts          = _j('ql-scenes', []) || [];
  const propagations  = _j('ql-propagations', []) || [];
  const seqData       = _j('ql-sequencer', {}) || {};
  const tracks        = seqData.tracks   || [];
  const patterns      = seqData.patterns || [];
  const bpm           = seqData.bpm      || 120;
  const chCount       = seqData.chCount  || 24;
  const zones         = _j('ql-zones', []) || [];
  const compiledMap   = _j('ql-compiled', {}) || {};
  const compiled      = Object.values(compiledMap);

  /* ── Project meta ─────────────────────────────────────────────── */
  // ql-workspace now stores characters[] directly; meta is in ql-workspace-meta
  // Fallback: if ql-workspace is an object (old format), treat as meta
  const rawWS = _j('ql-workspace', null);
  let projectMeta = { name: 'Untitled Project', desc: '', key_signature: '', bars_per_chapter: 4 };
  if (rawWS && !Array.isArray(rawWS) && typeof rawWS === 'object') {
    projectMeta = { ...projectMeta, ...rawWS };
  }

  /* ── Capsule index (id → capsule) ──────────────────────────────── */
  const capsuleIdx = {};
  capsules.forEach(c => { capsuleIdx[c.id] = c; });

  /* ── Character index ────────────────────────────────────────────── */
  const charIdx = {};
  characters.forEach(c => { charIdx[c.id] = c; });

  /* ── Resolve clips with capsule data ──────────────────────────── */
  const resolvedTracks = tracks.map(track => ({
    ...track,
    clips: (track.clips || []).map(clip => {
      const cap = capsuleIdx[clip.capId];
      if (!cap && clip.capId) warnings.push(`Clip "${clip.name}" references missing capsule "${clip.capId}"`);
      return { ...clip, capsule: cap || null };
    }),
  }));

  /* ── Resolve scene actors to character objects ─────────────────── */
  const resolvedActs = acts.map(act => ({
    ...act,
    scenes: (act.scenes || []).map(scene => {
      const resolvedActors = (scene.actors || []).map(actorId => {
        const char = charIdx[actorId];
        if (!char) warnings.push(`Scene "${scene.name}" references missing actor "${actorId}"`);
        return char || { id: actorId, name: '(unknown)' };
      });
      // Resolve heraldry references
      const heraldry = scene.heraldry || {};
      const resolvedHeraldry = {
        ...heraldry,
        standard_cap: capsuleIdx[heraldry.standard] || null,
        seal_cap:     capsuleIdx[heraldry.seal]     || null,
        glyph_caps:   (heraldry.glyphs || []).map(g => capsuleIdx[g]).filter(Boolean),
        rune_caps:    (heraldry.runes  || []).map(r => capsuleIdx[r]).filter(Boolean),
        emblem_cap:   capsuleIdx[heraldry.emblem]   || null,
      };
      return { ...scene, resolvedActors, heraldry: resolvedHeraldry };
    }),
  }));

  /* ── Compute PropArc values at each chapter ────────────────────── */
  // Returns value of a PropArc at a given chapter
  function propValueAt(prop, chapter) {
    if (chapter < prop.startChapter) return prop.startValue;
    const elapsed = chapter - prop.startChapter;
    let v;
    switch(prop.shape) {
      case 'linear':      v = prop.startValue + elapsed * prop.rate; break;
      case 'exponential': v = prop.startValue * Math.pow(prop.rate, elapsed); break;
      case 'sine':        v = prop.startValue + Math.sin(elapsed * prop.rate) * (prop.maxValue - prop.startValue) / 2; break;
      case 'step':        v = prop.startValue + Math.floor(elapsed * prop.rate); break;
      default:            v = prop.startValue + elapsed * prop.rate;
    }
    return Math.min(prop.maxValue, Math.max(prop.minValue, v));
  }

  /* ── Chapter timeline — for each chapter, what's active ──────── */
  function getChapterContext(chapter) {
    const activeTracks = resolvedTracks.map(track => {
      const activeClips = track.clips.filter(c => c.startCh <= chapter && c.endCh >= chapter);
      return { ...track, activeClips };
    }).filter(t => !t.muted && t.activeClips.length > 0);

    const propValues = {};
    propagations.forEach(p => { propValues[p.id] = propValueAt(p, chapter); });

    const activeScene = resolvedActs
      .flatMap(a => a.scenes)
      .filter(s => s.chapter === chapter)[0] || null;

    return { chapter, activeTracks, propValues, activeScene };
  }

  /* ── Summary statistics ─────────────────────────────────────────── */
  const stats = {
    characterCount:  characters.length,
    capsuleCount:    capsules.length,
    actCount:        acts.length,
    sceneCount:      acts.reduce((n, a) => n + (a.scenes?.length || 0), 0),
    trackCount:      tracks.length,
    clipCount:       tracks.reduce((n, t) => n + (t.clips?.length || 0), 0),
    propagationCount: propagations.length,
    zoneCount:       zones.length,
    compiledCount:   compiled.length,
    chapterSpan:     chCount,
    bpm,
  };

  /* ── Validate ───────────────────────────────────────────────────── */
  if (!characters.length) warnings.push('No characters — Workspace is empty');
  if (!capsules.length)   warnings.push('No capsules — Library is empty');
  if (!acts.length)       warnings.push('No acts — Scene Manager is empty');
  if (!tracks.length)     warnings.push('No tracks — Sequencer is empty');

  const hasMissingDialogue = characters.some(c => !c.dialogue_voice);
  if (hasMissingDialogue) warnings.push('Some characters have no dialogue_voice — screenplay/novel output will be generic');

  /* ── Assembled snapshot ─────────────────────────────────────────── */
  return {
    meta:         projectMeta,
    characters,
    capsules,
    capsuleIdx,
    charIdx,
    acts:         resolvedActs,
    propagations,
    tracks:       resolvedTracks,
    patterns,
    zones,
    compiled,
    bpm,
    chCount,
    stats,
    warnings,
    generatedAt:  new Date().toISOString(),

    /* Convenience helpers */
    getChapterContext,
    propValueAt,
    clipsAt(chapter, type) {
      return resolvedTracks
        .filter(t => !type || t.type === type)
        .flatMap(t => t.clips.filter(c => c.startCh <= chapter && c.endCh >= chapter));
    },
    scenesAt(chapter) {
      return resolvedActs.flatMap(a => a.scenes.filter(s => s.chapter === chapter));
    },
    actForScene(sceneId) {
      return resolvedActs.find(a => a.scenes.some(s => s.id === sceneId)) || null;
    },
    charById(id) { return charIdx[id] || null; },
    capById(id)  { return capsuleIdx[id] || null; },
  };
}

/* ══ OUTPUT RENDERER BASE ═════════════════════════════════════════════
   Every output format is a pure function: (snapshot, options) → output
═══════════════════════════════════════════════════════════════════════ */

const Renderers = {};

/* ── Register a renderer ─────────────────────────────────────────── */
function registerRenderer(formatId, fn) {
  Renderers[formatId] = fn;
}

/* ── Run a renderer ──────────────────────────────────────────────── */
function render(formatId, options = {}) {
  const snapshot = buildSnapshot();
  const renderer = Renderers[formatId];
  if (!renderer) {
    return {
      format: formatId,
      title: 'Error',
      sections: [{ type:'body', content:`No renderer registered for format "${formatId}"` }],
      metadata: {},
      warnings: [`Unknown format: ${formatId}`],
    };
  }
  return renderer(snapshot, options);
}

/* ── Section builder helpers ─────────────────────────────────────── */
function section(type, content, label, meta) {
  return { type, content: content || '', label: label || '', meta: meta || {} };
}
const h1      = (t) => section('heading',    t);
const h2      = (t) => section('subheading', t);
const body    = (t) => section('body',       t);
const divider = () => section('divider',     '');
const dialogue = (speaker, text) => section('dialogue', text, speaker);
const action   = (t) => section('action', t);

/* ══ BUILT-IN RENDERER: WORLD BIBLE ══════════════════════════════════ */
registerRenderer('manual', (snap, opts) => {
  const sections = [];
  const warnings = [...snap.warnings];

  sections.push(h1(snap.meta.name || 'World Bible'));
  sections.push(body(`Generated: ${snap.generatedAt} · ${snap.stats.characterCount} characters · ${snap.stats.sceneCount} scenes · ${snap.stats.capsuleCount} capsules`));
  sections.push(divider());

  /* World Laws */
  const seals = snap.capsules.filter(c => c.type === 'seal');
  if (seals.length) {
    sections.push(h2('World Laws'));
    seals.forEach(s => {
      sections.push(section('body', `${s.symbol}  ${s.name}${s.desc ? '\n' + s.desc : ''}${s.logic ? '\n\nLogic: ' + s.logic : ''}`, '', { tags: s.tags, authority: s.authority }));
    });
    sections.push(divider());
  }

  /* Factions & Crests */
  const crests = snap.capsules.filter(c => c.type === 'crest');
  if (crests.length) {
    sections.push(h2('Factions & Crests'));
    crests.forEach(c => {
      sections.push(body(`${c.symbol}  ${c.name}${c.desc ? ' — ' + c.desc : ''}`));
    });
    sections.push(divider());
  }

  /* Characters */
  if (snap.characters.length) {
    sections.push(h2('Characters'));
    snap.characters.forEach(c => {
      const vecStr = Object.entries(c.vectors || {}).map(([k,v]) => `${k}: ${v}`).join(' · ');
      const traitStr = (c.traits || []).join(', ');
      sections.push(body(
        `${c.name}  [${c.role}]  ${c.faction ? '· ' + c.faction : ''}\n` +
        `HP: ${c.hp}  |  ${vecStr}\n` +
        (traitStr ? `Traits: ${traitStr}\n` : '') +
        (c.dialogue_voice ? `Voice: ${c.dialogue_voice}` : '')
      ));
    });
    sections.push(divider());
  }

  /* World Forces */
  if (snap.propagations.length) {
    sections.push(h2('World Forces'));
    snap.propagations.forEach(p => {
      sections.push(body(`${p.symbol}  ${p.name}  [${p.shape} · rate ${p.rate}]\nRange: ${p.minValue}–${p.maxValue} · starts ch.${p.startChapter}${p.desc ? '\n' + p.desc : ''}`));
    });
    sections.push(divider());
  }

  /* Chronology */
  if (snap.acts.length) {
    sections.push(h2('Chronology'));
    snap.acts.forEach(act => {
      sections.push(section('subheading', act.name, '', { color: act.color, sealed: act.sealed }));
      (act.scenes || []).forEach(s => {
        const actorNames = s.resolvedActors?.map(a => a.name).join(', ') || '';
        sections.push(body(
          `Ch.${s.chapter}  ${s.name}  ${s.sealed ? '[SEALED]' : ''}\n` +
          (actorNames ? `Actors: ${actorNames}\n` : '') +
          (s.notes ? `Notes: ${s.notes}` : '')
        ));
      });
    });
    sections.push(divider());
  }

  /* Compiled Rule Systems */
  if (snap.compiled.length) {
    sections.push(h2('Compiled Rule Systems'));
    snap.compiled.forEach(g => {
      const ins  = g.inputs?.map(p => `${p.label}:${p.type}`).join(', ') || 'none';
      const outs = g.outputs?.map(p => `${p.label}:${p.type}`).join(', ') || 'none';
      sections.push(body(
        `${g.display?.icon || '⬡'}  ${g.label}  v${g.version}\n` +
        `Inputs: ${ins}\nOutputs: ${outs}\n` +
        (g.display?.description ? g.display.description : '')
      ));
    });
  }

  return { format:'manual', title: (snap.meta.name || 'Project') + ' — World Bible', sections, metadata: snap.stats, warnings };
});

/* ══ BUILT-IN RENDERER: MUSIC SESSION SHEET ══════════════════════════ */
registerRenderer('session', (snap, opts) => {
  const sections = [];

  sections.push(h1('Session Sheet — ' + (snap.meta.name || 'Project')));
  sections.push(body(`BPM: ${snap.bpm}  ·  Chapters: ${snap.chCount}  ·  ${snap.stats.trackCount} tracks  ·  ${snap.patterns.length} patterns${snap.meta.key_signature ? '  ·  Key: ' + snap.meta.key_signature : ''}`));
  sections.push(divider());

  /* Instrument roles from track types */
  const roleMap = {
    character: 'Melodic Lead', emotion: 'Harmonic Pads', environ: 'Ambient Texture',
    event:     'Hit Markers',  effect:  'FX / Automation', tension: 'Tension Riser',
  };
  if (snap.tracks.length) {
    sections.push(h2('Instrument Roles'));
    snap.tracks.filter(t => !t.muted).forEach(t => {
      sections.push(body(`${t.name}  →  ${roleMap[t.type] || t.type.toUpperCase()}`));
    });
    sections.push(divider());
  }

  /* Section map from acts/scenes */
  if (snap.acts.length) {
    sections.push(h2('Section Map'));
    const bpc = snap.meta.bars_per_chapter || 4;
    snap.acts.forEach(act => {
      sections.push(section('subheading', act.name));
      (act.scenes || []).sort((a,b) => a.chapter - b.chapter).forEach(s => {
        const bar = (s.chapter - 1) * bpc + 1;
        sections.push(body(`Bar ${bar}  (Ch.${s.chapter})  —  ${s.name}`));
      });
    });
    sections.push(divider());
  }

  /* Patterns */
  if (snap.patterns.length) {
    sections.push(h2('Step Patterns'));
    snap.patterns.forEach(p => {
      const grid = (p.active || []).map((on, i) => on ? (p.events?.[i] || '●') : '·').join(' ');
      sections.push(body(`${p.name}  [${p.steps} steps · ${p.interval}]\n${grid}`));
    });
    sections.push(divider());
  }

  /* Dynamics from PropArcs */
  if (snap.propagations.length) {
    sections.push(h2('Dynamic Forces'));
    snap.propagations.forEach(p => {
      const peakCh = (() => {
        let maxV = p.startValue, maxCh = p.startChapter;
        for(let ch = p.startChapter; ch <= snap.chCount; ch++) {
          const v = snap.propValueAt(p, ch);
          if(v > maxV) { maxV = v; maxCh = ch; }
        }
        return maxCh;
      })();
      sections.push(body(`${p.symbol}  ${p.name}  [${p.shape}]  peaks at ch.${peakCh} (${Math.round(snap.propValueAt(p, peakCh))})`));
    });
  }

  return { format:'session', title: 'Session Sheet — ' + (snap.meta.name || 'Project'), sections, metadata: snap.stats, warnings: snap.warnings };
});

/* ══ RENDER TO TEXT ═══════════════════════════════════════════════════
   Converts a RenderedOutput to a plain text string for download/copy.
═══════════════════════════════════════════════════════════════════════ */
function renderToText(output) {
  return output.sections.map(s => {
    switch(s.type) {
      case 'heading':    return '\n' + s.content.toUpperCase() + '\n' + '═'.repeat(Math.min(60, s.content.length)) + '\n';
      case 'subheading': return '\n' + s.content + '\n' + '─'.repeat(Math.min(40, s.content.length));
      case 'body':       return s.content;
      case 'dialogue':   return `\n${s.label ? s.label.toUpperCase() + '\n' : ''}${s.content}`;
      case 'action':     return s.content;
      case 'divider':    return '\n' + '·'.repeat(40) + '\n';
      default:           return s.content;
    }
  }).join('\n');
}

/* ══ RENDER TO HTML ═══════════════════════════════════════════════════
   Converts a RenderedOutput to an HTML string.
═══════════════════════════════════════════════════════════════════════ */
function renderToHTML(output, theme = {}) {
  const bg    = theme.bg    || '#04050a';
  const fg    = theme.fg    || '#e8e6e0';
  const accent= theme.accent|| '#b1d0f0';
  const fm    = theme.fm    || 'Orbitron, monospace';
  const fd    = theme.fd    || 'Space Grotesk, sans-serif';

  const sectionHTML = output.sections.map(s => {
    switch(s.type) {
      case 'heading':    return `<h1 style="font-family:${fm};letter-spacing:.2em;text-transform:uppercase;color:${accent};border-bottom:1px solid ${accent}30;padding-bottom:8px;margin:32px 0 16px">${esc(s.content)}</h1>`;
      case 'subheading': return `<h2 style="font-family:${fm};font-size:.8em;letter-spacing:.15em;text-transform:uppercase;color:${fg};opacity:.7;margin:24px 0 8px">${esc(s.content)}</h2>`;
      case 'body':
        if (s.meta?.type === 'beat_pause')
          return `<p style="font-family:${fc};font-size:.75em;letter-spacing:.2em;text-align:center;color:${fg};opacity:.35;margin:8px 0 14px">${esc(s.content)}</p>`;
        if (s.meta?.type === 'parenthetical')
          return `<p style="font-family:${fd};font-style:italic;color:${fg};opacity:.5;margin:0 0 8px;padding-left:1.5em">${esc(s.content)}</p>`;
        return `<p style="font-family:${fd};line-height:1.75;white-space:pre-wrap;margin:0 0 12px;color:${fg};opacity:.85">${esc(s.content)}</p>`;
      case 'dialogue':   return `<div style="margin:16px 0 16px 24px;border-left:2px solid ${accent}50;padding-left:12px"><span style="font-family:${fm};font-size:.6em;letter-spacing:.15em;color:${accent}">${esc(s.label||'')}</span><p style="font-family:${fd};font-style:italic;margin:4px 0;color:${fg}">${esc(s.content)}</p></div>`;
      case 'action':     return `<p style="font-family:${fd};font-weight:600;margin:0 0 12px;color:${fg}">${esc(s.content)}</p>`;
      case 'divider':    return `<hr style="border:none;border-top:1px solid ${fg}15;margin:24px 0">`;
      default:           return `<p style="font-family:${fd};color:${fg}">${esc(s.content)}</p>`;
    }
  }).join('\n');

  const warnHTML = output.warnings?.length
    ? `<div style="background:rgba(232,184,75,.08);border:1px solid rgba(232,184,75,.3);border-radius:6px;padding:12px 16px;margin:0 0 24px;font-family:${fd};font-size:.8em;color:#e8b84b">${output.warnings.map(w => `⚠ ${esc(w)}`).join('<br>')}</div>`
    : '';

  return `<!DOCTYPE html><html><head><meta charset="UTF-8"><title>${esc(output.title)}</title></head>
<body style="background:${bg};color:${fg};max-width:740px;margin:0 auto;padding:40px 24px;font-size:16px">
<h1 style="font-family:${fm};letter-spacing:.25em;font-size:1em;text-transform:uppercase;color:${accent}">${esc(output.title)}</h1>
${warnHTML}
${sectionHTML}
</body></html>`;
}

function esc(s) {
  return String(s || '').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}

/* ══ BUILT-IN RENDERER: NOVEL ════════════════════════════════════════ */
registerRenderer('novel', (snap, opts) => {
  const sections = [];
  const warnings = [...snap.warnings];
  const inclSealed = opts.includeSealed !== false;
  sections.push(h1(snap.meta.name || 'Novel'));
  sections.push(divider());
  if (!snap.acts.length) {
    warnings.push('No acts — add scenes in Scene Manager');
    return { format:'novel', title:snap.meta.name||'Novel', sections, metadata:snap.stats, warnings };
  }
  snap.acts.forEach(act => {
    if (!inclSealed && act.sealed) return;
    sections.push(h1(act.name));
    (act.scenes||[]).sort((a,b)=>a.chapter-b.chapter).forEach(scene => {
      if (!inclSealed && scene.sealed) return;
      sections.push(h2(`Chapter ${scene.chapter} — ${scene.name}`));
      const envClips = snap.clipsAt(scene.chapter,'environ');
      if (envClips.length) {
        const d = envClips.map(c=>c.capsule?.desc||c.name).filter(Boolean).join(' ');
        if (d) sections.push(body(d));
      } else if (scene.notes) { sections.push(body(scene.notes)); }
      const glyphs = scene.heraldry?.glyph_caps||[];
      if (glyphs.length) sections.push(body(glyphs.map(g=>`${g.symbol} ${g.name}`).join(' · ')));
      const actors = scene.resolvedActors||[];
      if (actors.length) sections.push(section('body',actors.map(a=>{const c=snap.charById(a.id)||a;return c.name+(c.traits?.length?` [${c.traits.slice(0,2).join(', ')}]`:'')+(c.dialogue_voice?` — ${c.dialogue_voice}`:'');}).join('\n'),'Cast'));
      snap.clipsAt(scene.chapter,'emotion').forEach(c=>{if(c.capsule?.desc)sections.push(body(c.capsule.desc));});
      snap.clipsAt(scene.chapter,'event').forEach(c=>{
        const pw=c.prose_weight??0.5;
        if(pw<0.25){sections.push(section('body',`(${c.capsule?.desc||c.name})`,'',{type:'parenthetical',prose_weight:pw}));}
        else if(pw>0.75){sections.push(action(c.capsule?.desc||c.name));if(c.capsule?.desc&&c.name!==c.capsule.desc)sections.push(body(`[${c.name} — extended beat]`));}
        else{sections.push(action(c.capsule?.desc||c.name));}
      });
      snap.propagations.forEach(p=>{const v=snap.propValueAt(p,scene.chapter);const pct=Math.round((v-p.minValue)/(p.maxValue-p.minValue)*100);if(pct>70)sections.push(body(`[${p.symbol} ${p.name} surges — ${pct}%]`));});
      snap.clipsAt(scene.chapter,'character').forEach(c=>{if(c.capsule?.desc)sections.push(body(c.capsule.desc));});
      // Real dialogue lines (quoted speech)
      const sceneDialogue=(scene.dialogue||[]).sort((a,b)=>a.beat-b.beat);
      sceneDialogue.forEach(d=>{
        const voice=d.direction?`(${d.direction}) `:'';
        if(d.line) sections.push(body(`${voice}"${d.line}"`));
      });
      if (scene.sealed) sections.push(body('[Scene concluded]'));
      sections.push(divider());
    });
  });
  return { format:'novel', title:snap.meta.name||'Novel', sections, metadata:snap.stats, warnings };
});

/* ══ BUILT-IN RENDERER: SCREENPLAY ══════════════════════════════════ */
registerRenderer('screenplay', (snap, opts) => {
  const sections = [];
  const warnings = [...snap.warnings];
  const noType = snap.acts.flatMap(a=>a.scenes||[]).filter(s=>!s.scene_type);
  if (noType.length) warnings.push(`${noType.length} scene(s) missing scene_type (defaulting to INT)`);
  sections.push(h1(snap.meta.name||'Screenplay'));
  sections.push(body('FADE IN:'));
  sections.push(divider());
  snap.acts.forEach(act => {
    sections.push(h2(act.name.toUpperCase()));
    (act.scenes||[]).sort((a,b)=>a.chapter-b.chapter).forEach(scene => {
      const locType = scene.scene_type||'INT';
      const zoneNames = (scene.zones||[]).map(zid=>{const z=snap.zones.find(z=>z.id===zid);return z?z.name.toUpperCase():zid.toUpperCase();});
      const location = zoneNames[0]||scene.name.toUpperCase();
      const tod = (scene.time_of_day||'DAY').toUpperCase();
      sections.push(h2(`${locType}. ${location} - ${tod}`));
      const envClips = snap.clipsAt(scene.chapter,'environ');
      if (envClips.length) {
        // Weight environ clips: low-weight collapse to a single line; high-weight get own paragraph each
        const heavy = envClips.filter(c=>(c.prose_weight??0.5)>=0.75);
        const normal = envClips.filter(c=>{const pw=c.prose_weight??0.5;return pw>=0.25&&pw<0.75;});
        const light  = envClips.filter(c=>(c.prose_weight??0.5)<0.25);
        if (normal.length||light.length) {
          const d=[...normal,...light].map(c=>c.capsule?.desc||c.name).filter(Boolean).join(' ');
          if(d) sections.push(action(d));
        }
        heavy.forEach(c=>{
          const txt=c.capsule?.desc||c.name;
          if(txt){sections.push(action(txt));sections.push(section('body','BEAT.','',{type:'beat_pause'}));}
        });
      }
      const actors = scene.resolvedActors||[];
      if (actors.length) {const names=actors.map(a=>(snap.charById(a.id)||a).name.toUpperCase()).join(', ');sections.push(action(`${names} ${actors.length===1?'enters':'are present'}.`));}
      snap.clipsAt(scene.chapter,'event').forEach(c=>{
        const pw = c.prose_weight ?? 0.5;
        if (pw < 0.25) return;                        // omit — background detail only
        const txt = c.capsule?.desc || c.name;
        sections.push(action(txt));
        if (pw >= 0.75) {                             // expanded — add beat pause
          sections.push(section('body','BEAT.','',{type:'beat_pause'}));
          if (c.capsule?.desc && c.name !== c.capsule.desc)
            sections.push(action(`[${c.name}]`));     // secondary label for extended beat
        }
      });
      snap.clipsAt(scene.chapter,'emotion').forEach(c=>sections.push(section('body',`(${c.name})`)));
      // Render real dialogue lines if present, else placeholder
      actors.forEach(a=>{
        const ch = snap.charById(a.id)||a;
        const lines = (scene.dialogue||[]).filter(d=>d.actorName===ch.name||d.actorId===ch.id).sort((x,y)=>x.beat-y.beat);
        if(lines.length){
          lines.forEach(d=>{
            sections.push(dialogue(ch.name.toUpperCase(), d.line||'…'));
            if(d.direction) sections.push(section('body',`(${d.direction})`,'',{type:'parenthetical'}));
          });
        } else if(ch.dialogue_voice){
          sections.push(dialogue(ch.name.toUpperCase(),`[${ch.dialogue_voice} — add lines in Scene Manager]`));
        } else {
          warnings.push(`No dialogue for ${ch.name} in ${scene.name}`);
        }
      });
      snap.propagations.forEach(p=>{const v=snap.propValueAt(p,scene.chapter);const pct=Math.round((v-p.minValue)/(p.maxValue-p.minValue)*100);if(pct>75)sections.push(action(`[${p.symbol} — TENSION AT ${pct}%]`));});
      if (scene.sealed) sections.push(body('CUT TO:'));
      sections.push(divider());
    });
  });
  sections.push(body('FADE OUT.\n\nTHE END'));
  return { format:'screenplay', title:snap.meta.name||'Screenplay', sections, metadata:snap.stats, warnings };
});

/* ══ BUILT-IN RENDERER: SONG ═════════════════════════════════════════ */
registerRenderer('song', (snap, opts) => {
  const sections = [];
  const warnings = [...snap.warnings];
  if (!snap.propagations.length) warnings.push('No PropArcs — song structure needs an arc force');
  if (!snap.patterns.length) warnings.push('No step patterns — add patterns in Sequencer');
  const arc = snap.propagations[0];
  const activeChar = snap.characters.find(c=>c.isActive)||snap.characters[0];
  sections.push(h1(snap.meta.name||'Song'));
  if (snap.meta.key_signature) sections.push(body(`Key: ${snap.meta.key_signature}  ·  BPM: ${snap.bpm}`));
  if (activeChar) sections.push(body(`POV: ${activeChar.name}${activeChar.dialogue_voice?' · '+activeChar.dialogue_voice:''}`));
  sections.push(divider());
  if (!arc) { sections.push(body('[No arc force — add PropArc in Scene Manager]')); return {format:'song',title:snap.meta.name||'Song',sections,metadata:snap.stats,warnings}; }
  const totalChs = snap.chCount;
  const bp = { v1:Math.max(1,Math.floor(totalChs*.1)), pre:Math.max(1,Math.floor(totalChs*.3)), ch:Math.max(1,Math.floor(totalChs*.45)), v2:Math.max(1,Math.floor(totalChs*.55)), br:Math.max(1,Math.floor(totalChs*.75)), fc:Math.max(1,Math.floor(totalChs*.88)), out:totalChs };
  let peakCh=arc.startChapter, peakV=arc.startValue;
  for(let c=arc.startChapter;c<=totalChs;c++){const v=snap.propValueAt(arc,c);if(v>peakV){peakV=v;peakCh=c;}}
  const seedNear=(ch,type)=>{for(const c of snap.clipsAt(ch,type)){if(c.capsule?.lyric_seed)return c.capsule.lyric_seed;if(c.capsule?.desc)return c.capsule.desc;}return null;};
  const mood=(ch)=>{const pct=(snap.propValueAt(arc,ch)-arc.minValue)/(arc.maxValue-arc.minValue);return pct<.25?'introspective, quiet':pct<.5?'building tension':pct<.75?'urgent, rising':'explosive, peak';};
  const pat=snap.patterns[0]; const meter=pat?.meter||'4/4'; const density=pat?((pat.active||[]).filter(Boolean).length+'/'+(pat.steps||16)+' beats'):'';
  sections.push(h2('VERSE 1'));
  sections.push(body(`[Ch.${bp.v1}–${bp.pre-1} · ${mood(bp.v1)} · ${meter} ${density}]`));
  const v1=seedNear(bp.v1,'emotion')||seedNear(bp.v1,'character'); if(v1)sections.push(body(v1));
  sections.push(body('[establish world and longing]')); sections.push(divider());
  sections.push(h2('PRE-CHORUS'));
  sections.push(body(`[Ch.${bp.pre} · ${mood(bp.pre)} · ${arc.symbol} building]`));
  const pre=seedNear(bp.pre,'event'); if(pre)sections.push(body(pre));
  sections.push(body('[the turn — something about to break]')); sections.push(divider());
  sections.push(h2('CHORUS'));
  sections.push(body(`[Ch.${bp.ch} · ${mood(bp.ch)} · hook — peak ch.${peakCh}]`));
  const chs=seedNear(peakCh,'event')||seedNear(peakCh,'emotion'); if(chs)sections.push(body(chs));
  sections.push(body(`[${arc.symbol} — ${arc.name} — the central declaration]`)); sections.push(divider());
  sections.push(h2('VERSE 2'));
  sections.push(body(`[Ch.${bp.v2} · deepened perspective]`));
  const v2=seedNear(bp.v2,'character')||seedNear(bp.v2,'emotion'); if(v2)sections.push(body(v2)); sections.push(divider());
  sections.push(h2('BRIDGE'));
  sections.push(body(`[Ch.${bp.br} · ${mood(bp.br)} · contrast + resolution]`));
  const br=seedNear(bp.br,'effect')||seedNear(bp.br,'emotion'); if(br)sections.push(body(br));
  sections.push(body('[the reckoning — what changes?]')); sections.push(divider());
  sections.push(h2('FINAL CHORUS'));
  sections.push(body(`[Ch.${bp.fc} · elevated reprise]`));
  sections.push(body(`[${arc.symbol} — ${arc.name} — resolved or transformed]`)); sections.push(divider());
  sections.push(h2('OUTRO'));
  const out=seedNear(totalChs,'character'); if(out)sections.push(body(out));
  sections.push(body('[what remains after the story]'));
  if(pat){sections.push(divider());sections.push(h2('Rhythmic Reference — '+pat.name));const grid=(pat.active||[]).map((on,i)=>on?(pat.events?.[i]||'●'):'·').join(' ');sections.push(body(`${pat.steps} steps · ${pat.interval}\n${grid}`));}
  return { format:'song', title:snap.meta.name||'Song', sections, metadata:snap.stats, warnings };
});

/* ══ BUILT-IN RENDERER: GAME SCRIPT ══════════════════════════════════ */
registerRenderer('game', (snap, opts) => {
  const sections = [];
  const warnings = [...snap.warnings];
  sections.push(h1(snap.meta.name||'Game Script'));
  sections.push(body(`${snap.stats.sceneCount} quests · ${snap.stats.characterCount} NPCs · ${snap.stats.compiledCount} rule systems · ${snap.stats.propagationCount} stat trackers`));
  sections.push(divider());
  if (snap.propagations.length) {
    sections.push(h2('STAT TRACKERS'));
    snap.propagations.forEach(p=>sections.push(body(`${p.symbol} ${p.name}\n  Range: ${p.minValue}–${p.maxValue} · Shape: ${p.shape} · Rate: ${p.rate}\n  Starts ch.${p.startChapter} at ${p.startValue}${p.desc?'\n  '+p.desc:''}`)));
    sections.push(divider());
  }
  if (snap.characters.length) {
    sections.push(h2('NPC SHEETS'));
    snap.characters.forEach(char=>{const disp=Math.round(((char.vectors?.trust||char.vectors?.hope||5)/10)*100);const traits=(char.traits||[]).join(', ');sections.push(section('body',`${char.name.toUpperCase()}  [${char.role}]\nFaction: ${char.faction||'none'} · HP: ${char.hp} · Disposition: ${disp}%\n${traits?'Traits: '+traits+'\n':''}Vectors: ${Object.entries(char.vectors||{}).map(([k,v])=>k+':'+v).join(' ')}\n${char.dialogue_voice?'Voice: '+char.dialogue_voice:'[no dialogue_voice set]'}`,'',{id:char.id}));});
    sections.push(divider());
  }
  snap.acts.forEach(act=>{
    sections.push(h2('ACT: '+act.name.toUpperCase()));
    (act.scenes||[]).sort((a,b)=>a.chapter-b.chapter).forEach(scene=>{
      sections.push(section('subheading',`QUEST: ${scene.name}  [Ch.${scene.chapter}]`));
      const zones=(scene.zones||[]).map(zid=>snap.zones.find(z=>z.id===zid)).filter(Boolean);
      if(zones.length)sections.push(body('Location: '+zones.map(z=>z.name).join(', ')));
      const graphs=snap.compiled.filter(g=>g.owner===scene.id||(g.display?.tags||[]).some(t=>scene.name.toLowerCase().includes(t.toLowerCase())));
      if(graphs.length){sections.push(body('Entry conditions: '+graphs.map(g=>g.label).join(', ')));graphs.forEach(g=>sections.push(body(`  ◈ ${g.label}: [${(g.inputs||[]).map(p=>p.label+':'+p.type).join(', ')||'none'}] → [${(g.outputs||[]).map(p=>p.label).join(', ')||'none'}]`)));}
      const actors=scene.resolvedActors||[];
      if(actors.length)sections.push(body('NPCs: '+actors.map(a=>(snap.charById(a.id)||a).name).join(', ')));
      actors.forEach(a=>{
        const ch=snap.charById(a.id)||a;
        if(ch.name==='(unknown)')return;
        const mood=snap.clipsAt(scene.chapter,'emotion')[0]?.name||'neutral';
        const lines=(scene.dialogue||[]).filter(d=>d.actorName===ch.name||d.actorId===ch.id).sort((x,y)=>x.beat-y.beat);
        if(lines.length){
          sections.push(section('subheading',`${ch.name} [${ch.dialogue_voice||'default'} · mood:${mood}]`));
          lines.forEach((d,i)=>{
            const branchCap = snap.capsules.find(c=>c.branch_label&&c.name===ch.name);
            const choiceLabel = branchCap?.branch_label||`Choice ${i+1}`;
            sections.push(dialogue(ch.name,d.line||'…'));
            sections.push(section('body',`  → [${choiceLabel}]`,'',{type:'branch'}));
          });
        } else {
          sections.push(dialogue(ch.name,`[${ch.dialogue_voice||'default'} · mood:${mood} · DIALOGUE NODE — add lines in Scene Manager]`));
        }
      });
      snap.clipsAt(scene.chapter,'event').forEach(c=>sections.push(section('body','TRIGGER: '+(c.capsule?.desc||c.name))));
      snap.propagations.forEach(p=>{const v=snap.propValueAt(p,scene.chapter);if(p.conditions?.length)sections.push(body(`Check ${p.symbol} ${p.name} = ${Math.round(v)}`));});
      if(scene.notes)sections.push(body('Notes: '+scene.notes));
      if(scene.sealed)sections.push(body('[Quest concluded]'));
      sections.push(divider());
    });
  });
  if(snap.compiled.length){sections.push(h2('RULE SYSTEMS'));snap.compiled.forEach(g=>sections.push(body(`${g.display?.icon||'◈'} ${g.label}  v${g.version}\nInputs: ${(g.inputs||[]).map(p=>p.label+':'+p.type).join(', ')||'none'}\nOutputs: ${(g.outputs||[]).map(p=>p.label).join(', ')||'none'}\n${g.display?.description||''}`)));}
  return { format:'game', title:snap.meta.name||'Game Script', sections, metadata:snap.stats, warnings };
});

/* ══ EXPORT ═══════════════════════════════════════════════════════════ */
const LoomSnapshot = {
  build:            buildSnapshot,
  render,
  registerRenderer,
  renderToText,
  renderToHTML,
  section, h1, h2, body, divider, dialogue, action,
};

// UMD export — works as <script> tag or module
if (typeof module !== 'undefined' && module.exports) {
  module.exports = LoomSnapshot;
} else {
  global.LoomSnapshot = LoomSnapshot;
}

})(typeof window !== 'undefined' ? window : global);
