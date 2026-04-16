/**
 * QUANTUM LOOM — LLM ROUTER
 * loom-llm.js
 *
 * Unified streaming interface for:
 *   Local  — Ollama, AnythingLLM
 *   Cloud  — Claude (Anthropic), OpenAI/ChatGPT, Google Gemini, Azure/Copilot
 *
 * Browser notes:
 *   Ollama / AnythingLLM / Gemini — work from browser directly
 *   Claude / OpenAI / Azure       — CORS blocked in browser; use Electron desktop
 *
 * Storage key: ql-llm-config
 */

(function(global) {
'use strict';

/* ════════════════════════════════════════════════════════════════
   PROVIDER REGISTRY
════════════════════════════════════════════════════════════════ */
const PROVIDERS = {
  ollama: {
    id:              'ollama',
    name:            'Ollama',
    emoji:           '🦙',
    type:            'local',
    defaultBase:     'http://localhost:11434',
    requiresKey:     false,
    requiresBase:    true,
    supportsStream:  true,
    hint:            'Run Ollama locally. No API key needed.',
  },
  anythingllm: {
    id:              'anythingllm',
    name:            'AnythingLLM',
    emoji:           '🗂',
    type:            'local',
    defaultBase:     'http://localhost:3001',
    requiresKey:     true,
    requiresBase:    true,
    requiresWorkspace: true,
    supportsStream:  true,
    hint:            'Your local AnythingLLM instance. Get API key from its settings.',
  },
  claude: {
    id:              'claude',
    name:            'Claude (Anthropic)',
    emoji:           '🔮',
    type:            'cloud',
    base:            'https://api.anthropic.com',
    requiresKey:     true,
    requiresBase:    false,
    supportsStream:  true,
    defaultModel:    'claude-sonnet-4-5',
    models: [
      { id: 'claude-opus-4-5',             label: 'Claude Opus 4.5 — most capable' },
      { id: 'claude-sonnet-4-5',           label: 'Claude Sonnet 4.5 — fast + smart' },
      { id: 'claude-haiku-4-5-20251001',   label: 'Claude Haiku 4.5 — fastest' },
    ],
    hint:            'Needs Electron app for browser CORS. Great for long-form prose.',
  },
  openai: {
    id:              'openai',
    name:            'OpenAI / ChatGPT',
    emoji:           '🟢',
    type:            'cloud',
    base:            'https://api.openai.com',
    requiresKey:     true,
    requiresBase:    false,
    supportsStream:  true,
    defaultModel:    'gpt-4o',
    models: [
      { id: 'gpt-4o',        label: 'GPT-4o — flagship' },
      { id: 'gpt-4o-mini',   label: 'GPT-4o Mini — fast' },
      { id: 'gpt-4-turbo',   label: 'GPT-4 Turbo' },
      { id: 'o1',            label: 'o1 — reasoning' },
      { id: 'o1-mini',       label: 'o1 Mini' },
    ],
    hint:            'Needs Electron app for browser CORS.',
  },
  gemini: {
    id:              'gemini',
    name:            'Google Gemini',
    emoji:           '♊',
    type:            'cloud',
    base:            'https://generativelanguage.googleapis.com',
    requiresKey:     true,
    requiresBase:    false,
    supportsStream:  true,
    defaultModel:    'gemini-2.0-flash',
    models: [
      { id: 'gemini-2.0-flash',         label: 'Gemini 2.0 Flash — fastest' },
      { id: 'gemini-1.5-pro',           label: 'Gemini 1.5 Pro — capable' },
      { id: 'gemini-1.5-flash',         label: 'Gemini 1.5 Flash' },
    ],
    hint:            'Works from browser. Get key from Google AI Studio.',
  },
  azure: {
    id:              'azure',
    name:            'Azure OpenAI / Copilot',
    emoji:           '☁️',
    type:            'cloud',
    requiresKey:     true,
    requiresBase:    true,
    requiresDeployment: true,
    supportsStream:  true,
    defaultModel:    'gpt-4o',
    models: [
      { id: 'gpt-4o',          label: 'GPT-4o' },
      { id: 'gpt-4',           label: 'GPT-4' },
      { id: 'gpt-35-turbo',    label: 'GPT-3.5 Turbo' },
    ],
    hint:            'Your Azure OpenAI endpoint. Needs Electron app for browser CORS.',
  },
};

/* ════════════════════════════════════════════════════════════════
   STATE
════════════════════════════════════════════════════════════════ */
let _cfg = {
  provider:    'ollama',
  base:        'http://localhost:11434',
  apiKey:      '',
  model:       '',
  workspace:   '',
  deployment:  '',
  temperature: 0.8,
  maxTokens:   4096,
};
let _listeners = {};

/* ════════════════════════════════════════════════════════════════
   CORE API
════════════════════════════════════════════════════════════════ */
const LoomLLM = {

  PROVIDERS,

  /* ── Init from config object or localStorage ─────────────────── */
  init(cfg) {
    const saved = LoomLLM.loadConfig();
    _cfg = { ..._cfg, ...saved, ...(cfg || {}) };
    LoomLLM.emit('ready', { provider: _cfg.provider });
    return LoomLLM;
  },

  loadConfig() {
    try { return JSON.parse(localStorage.getItem('ql-llm-config') || '{}'); }
    catch(e) { return {}; }
  },

  saveConfig(cfg) {
    _cfg = { ..._cfg, ...(cfg || {}) };
    localStorage.setItem('ql-llm-config', JSON.stringify(_cfg));
    LoomLLM.emit('config', _cfg);
  },

  status() {
    return {
      provider: _cfg.provider,
      model:    _cfg.model,
      ready:    !!(_cfg.provider),
      config:   { ..._cfg, apiKey: _cfg.apiKey ? '***' : '' },
    };
  },

  /* ── List available models for current provider ───────────────── */
  async listModels() {
    const p = PROVIDERS[_cfg.provider];
    if (!p) return [];

    // Static model list for cloud providers
    if (p.models) return p.models;

    // Dynamic fetch for local providers
    try {
      if (_cfg.provider === 'ollama') {
        const res = await fetch(`${_cfg.base || p.defaultBase}/api/tags`);
        const data = await res.json();
        return (data.models || []).map(m => ({ id: m.name, label: m.name }));
      }
      if (_cfg.provider === 'anythingllm') {
        const res = await fetch(`${_cfg.base || p.defaultBase}/api/v1/workspaces`, {
          headers: { 'Authorization': `Bearer ${_cfg.apiKey}` },
        });
        const data = await res.json();
        return (data.workspaces || []).map(w => ({ id: w.slug, label: w.name }));
      }
    } catch(e) {
      LoomLLM.emit('error', { msg: 'Could not list models: ' + e.message });
      return [];
    }
    return [];
  },

  /* ── Full (non-streaming) chat ────────────────────────────────── */
  async chat(messages, opts = {}) {
    const cfg = { ..._cfg, ...opts };
    switch(cfg.provider) {
      case 'ollama':      return LoomLLM._chatOllama(messages, cfg);
      case 'anythingllm': return LoomLLM._chatAnythingLLM(messages, cfg);
      case 'claude':      return LoomLLM._chatClaude(messages, cfg);
      case 'openai':      return LoomLLM._chatOpenAI(messages, cfg);
      case 'gemini':      return LoomLLM._chatGemini(messages, cfg);
      case 'azure':       return LoomLLM._chatAzure(messages, cfg);
      default: throw new Error('Unknown provider: ' + cfg.provider);
    }
  },

  /* ── Streaming chat — onChunk(text) called with each token ───── */
  async chatStream(messages, opts = {}, onChunk, onDone) {
    const cfg = { ..._cfg, ...opts };
    switch(cfg.provider) {
      case 'ollama':      return LoomLLM._streamOllama(messages, cfg, onChunk, onDone);
      case 'anythingllm': return LoomLLM._streamAnythingLLM(messages, cfg, onChunk, onDone);
      case 'claude':      return LoomLLM._streamClaude(messages, cfg, onChunk, onDone);
      case 'openai':      return LoomLLM._streamOpenAI(messages, cfg, onChunk, onDone);
      case 'gemini':      return LoomLLM._streamGemini(messages, cfg, onChunk, onDone);
      case 'azure':       return LoomLLM._streamAzure(messages, cfg, onChunk, onDone);
      default: throw new Error('Unknown provider: ' + cfg.provider);
    }
  },

  /* ── JSON structured output ───────────────────────────────────── */
  async structured(systemPrompt, userPrompt, schema, opts = {}) {
    const schemaHint = JSON.stringify(schema, null, 2);
    const messages = [
      { role: 'system', content: systemPrompt + '\n\nRespond ONLY with valid JSON matching this schema:\n' + schemaHint + '\nNo markdown, no preamble.' },
      { role: 'user',   content: userPrompt },
    ];
    const raw = await LoomLLM.chat(messages, opts);
    try {
      const clean = raw.replace(/```json\n?/g,'').replace(/```\n?/g,'').trim();
      return JSON.parse(clean);
    } catch(e) {
      LoomLLM.emit('error', { msg: 'Structured parse failed: ' + e.message });
      return null;
    }
  },

  /* ── Simple event emitter ────────────────────────────────────── */
  on(event, cb) {
    if (!_listeners[event]) _listeners[event] = [];
    _listeners[event].push(cb);
    return () => { _listeners[event] = _listeners[event].filter(l => l !== cb); };
  },
  emit(event, data) {
    (_listeners[event] || []).forEach(cb => { try { cb(data); } catch(e) {} });
  },

  /* ════════════════════════════════════════════════════════════════
     PROVIDER IMPLEMENTATIONS
  ════════════════════════════════════════════════════════════════ */

  /* ── OLLAMA ──────────────────────────────────────────────────── */
  async _chatOllama(messages, cfg) {
    const res = await fetch(`${cfg.base || 'http://localhost:11434'}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model:    cfg.model || 'llama3.2',
        messages: messages,
        stream:   false,
        options:  { temperature: cfg.temperature || 0.8, num_predict: cfg.maxTokens || 4096 },
      }),
    });
    const data = await res.json();
    return data.message?.content || '';
  },

  async _streamOllama(messages, cfg, onChunk, onDone) {
    const res = await fetch(`${cfg.base || 'http://localhost:11434'}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model:    cfg.model || 'llama3.2',
        messages: messages,
        stream:   true,
        options:  { temperature: cfg.temperature || 0.8, num_predict: cfg.maxTokens || 4096 },
      }),
    });
    const reader = res.body.getReader();
    const dec    = new TextDecoder();
    let full     = '';
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const lines = dec.decode(value).split('\n').filter(Boolean);
      for (const line of lines) {
        try {
          const obj = JSON.parse(line);
          if (obj.message?.content) { full += obj.message.content; onChunk(obj.message.content); }
          if (obj.done) { onDone?.(full); return; }
        } catch(e) {}
      }
    }
    onDone?.(full);
  },

  /* ── ANYTHINGLLM ─────────────────────────────────────────────── */
  async _chatAnythingLLM(messages, cfg) {
    const ws  = cfg.workspace || cfg.model || 'default';
    const msg = messages[messages.length - 1]?.content || '';
    const res = await fetch(`${cfg.base || 'http://localhost:3001'}/api/v1/workspace/${ws}/chat`, {
      method:  'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${cfg.apiKey}` },
      body:    JSON.stringify({ message: msg, mode: 'chat' }),
    });
    const data = await res.json();
    return data.textResponse || data.response || '';
  },

  async _streamAnythingLLM(messages, cfg, onChunk, onDone) {
    // AnythingLLM streaming via SSE
    const ws  = cfg.workspace || cfg.model || 'default';
    const msg = messages[messages.length - 1]?.content || '';
    const res = await fetch(`${cfg.base || 'http://localhost:3001'}/api/v1/workspace/${ws}/stream-chat`, {
      method:  'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${cfg.apiKey}` },
      body:    JSON.stringify({ message: msg, mode: 'chat' }),
    });
    const reader = res.body.getReader();
    const dec    = new TextDecoder();
    let full     = '';
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const chunks = dec.decode(value).split('data: ').filter(Boolean);
      for (const chunk of chunks) {
        try {
          const obj = JSON.parse(chunk.trim());
          if (obj.textResponse) { full += obj.textResponse; onChunk(obj.textResponse); }
          if (obj.close) { onDone?.(full); return; }
        } catch(e) {}
      }
    }
    onDone?.(full);
  },

  /* ── CLAUDE (ANTHROPIC) ──────────────────────────────────────── */
  async _chatClaude(messages, cfg) {
    const sys = messages.find(m => m.role === 'system');
    const msgs = messages.filter(m => m.role !== 'system');
    const res = await fetch('https://api.anthropic.com/v1/messages', {
      method: 'POST',
      headers: {
        'Content-Type':      'application/json',
        'x-api-key':         cfg.apiKey,
        'anthropic-version': '2023-06-01',
      },
      body: JSON.stringify({
        model:      cfg.model || 'claude-sonnet-4-5',
        max_tokens: cfg.maxTokens || 4096,
        ...(sys ? { system: sys.content } : {}),
        messages:   msgs,
      }),
    });
    const data = await res.json();
    if (data.error) throw new Error(data.error.message);
    return data.content?.[0]?.text || '';
  },

  async _streamClaude(messages, cfg, onChunk, onDone) {
    const sys  = messages.find(m => m.role === 'system');
    const msgs = messages.filter(m => m.role !== 'system');
    const res = await fetch('https://api.anthropic.com/v1/messages', {
      method: 'POST',
      headers: {
        'Content-Type':      'application/json',
        'x-api-key':         cfg.apiKey,
        'anthropic-version': '2023-06-01',
      },
      body: JSON.stringify({
        model:      cfg.model || 'claude-sonnet-4-5',
        max_tokens: cfg.maxTokens || 4096,
        stream:     true,
        ...(sys ? { system: sys.content } : {}),
        messages:   msgs,
      }),
    });
    const reader = res.body.getReader();
    const dec    = new TextDecoder();
    let full     = '';
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const lines = dec.decode(value).split('\n').filter(l => l.startsWith('data: '));
      for (const line of lines) {
        try {
          const obj = JSON.parse(line.slice(6));
          if (obj.type === 'content_block_delta' && obj.delta?.text) {
            full += obj.delta.text;
            onChunk(obj.delta.text);
          }
          if (obj.type === 'message_stop') { onDone?.(full); return; }
        } catch(e) {}
      }
    }
    onDone?.(full);
  },

  /* ── OPENAI / CHATGPT ────────────────────────────────────────── */
  async _chatOpenAI(messages, cfg) {
    const res = await fetch('https://api.openai.com/v1/chat/completions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${cfg.apiKey}` },
      body: JSON.stringify({
        model:       cfg.model || 'gpt-4o',
        messages:    messages,
        max_tokens:  cfg.maxTokens || 4096,
        temperature: cfg.temperature || 0.8,
      }),
    });
    const data = await res.json();
    if (data.error) throw new Error(data.error.message);
    return data.choices?.[0]?.message?.content || '';
  },

  async _streamOpenAI(messages, cfg, onChunk, onDone) {
    const res = await fetch('https://api.openai.com/v1/chat/completions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${cfg.apiKey}` },
      body: JSON.stringify({
        model:       cfg.model || 'gpt-4o',
        messages:    messages,
        max_tokens:  cfg.maxTokens || 4096,
        temperature: cfg.temperature || 0.8,
        stream:      true,
      }),
    });
    const reader = res.body.getReader();
    const dec    = new TextDecoder();
    let full     = '';
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const lines = dec.decode(value).split('\n').filter(l => l.startsWith('data: ') && !l.includes('[DONE]'));
      for (const line of lines) {
        try {
          const obj = JSON.parse(line.slice(6));
          const txt = obj.choices?.[0]?.delta?.content;
          if (txt) { full += txt; onChunk(txt); }
        } catch(e) {}
      }
    }
    onDone?.(full);
  },

  /* ── GOOGLE GEMINI ───────────────────────────────────────────── */
  async _chatGemini(messages, cfg) {
    const model = cfg.model || 'gemini-2.0-flash';
    const sys   = messages.find(m => m.role === 'system');
    const turns = messages.filter(m => m.role !== 'system').map(m => ({
      role:  m.role === 'assistant' ? 'model' : 'user',
      parts: [{ text: m.content }],
    }));
    const body = {
      contents: turns,
      generationConfig: { temperature: cfg.temperature || 0.8, maxOutputTokens: cfg.maxTokens || 4096 },
      ...(sys ? { systemInstruction: { parts: [{ text: sys.content }] } } : {}),
    };
    const res = await fetch(
      `https://generativelanguage.googleapis.com/v1beta/models/${model}:generateContent?key=${cfg.apiKey}`,
      { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) }
    );
    const data = await res.json();
    if (data.error) throw new Error(data.error.message);
    return data.candidates?.[0]?.content?.parts?.[0]?.text || '';
  },

  async _streamGemini(messages, cfg, onChunk, onDone) {
    const model = cfg.model || 'gemini-2.0-flash';
    const sys   = messages.find(m => m.role === 'system');
    const turns = messages.filter(m => m.role !== 'system').map(m => ({
      role:  m.role === 'assistant' ? 'model' : 'user',
      parts: [{ text: m.content }],
    }));
    const body = {
      contents: turns,
      generationConfig: { temperature: cfg.temperature || 0.8, maxOutputTokens: cfg.maxTokens || 4096 },
      ...(sys ? { systemInstruction: { parts: [{ text: sys.content }] } } : {}),
    };
    const res = await fetch(
      `https://generativelanguage.googleapis.com/v1beta/models/${model}:streamGenerateContent?key=${cfg.apiKey}&alt=sse`,
      { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) }
    );
    const reader = res.body.getReader();
    const dec    = new TextDecoder();
    let full     = '';
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const lines = dec.decode(value).split('\n').filter(l => l.startsWith('data: '));
      for (const line of lines) {
        try {
          const obj = JSON.parse(line.slice(6));
          const txt = obj.candidates?.[0]?.content?.parts?.[0]?.text;
          if (txt) { full += txt; onChunk(txt); }
        } catch(e) {}
      }
    }
    onDone?.(full);
  },

  /* ── AZURE OPENAI / COPILOT ──────────────────────────────────── */
  async _chatAzure(messages, cfg) {
    const deployment = cfg.deployment || cfg.model || 'gpt-4o';
    const endpoint   = cfg.base.replace(/\/$/, '');
    const res = await fetch(`${endpoint}/openai/deployments/${deployment}/chat/completions?api-version=2024-02-01`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'api-key': cfg.apiKey },
      body: JSON.stringify({
        messages:    messages,
        max_tokens:  cfg.maxTokens || 4096,
        temperature: cfg.temperature || 0.8,
      }),
    });
    const data = await res.json();
    if (data.error) throw new Error(data.error.message);
    return data.choices?.[0]?.message?.content || '';
  },

  async _streamAzure(messages, cfg, onChunk, onDone) {
    const deployment = cfg.deployment || cfg.model || 'gpt-4o';
    const endpoint   = cfg.base.replace(/\/$/, '');
    const res = await fetch(`${endpoint}/openai/deployments/${deployment}/chat/completions?api-version=2024-02-01`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'api-key': cfg.apiKey },
      body: JSON.stringify({
        messages:    messages,
        max_tokens:  cfg.maxTokens || 4096,
        temperature: cfg.temperature || 0.8,
        stream:      true,
      }),
    });
    const reader = res.body.getReader();
    const dec    = new TextDecoder();
    let full     = '';
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const lines = dec.decode(value).split('\n').filter(l => l.startsWith('data: ') && !l.includes('[DONE]'));
      for (const line of lines) {
        try {
          const obj = JSON.parse(line.slice(6));
          const txt = obj.choices?.[0]?.delta?.content;
          if (txt) { full += txt; onChunk(txt); }
        } catch(e) {}
      }
    }
    onDone?.(full);
  },

  /* ════════════════════════════════════════════════════════════════
     QUANTUM LOOM — DOMAIN-SPECIFIC HELPERS
  ════════════════════════════════════════════════════════════════ */

  /**
   * Build a rich narrative prompt from a LoomSnapshot.
   * Used by the Output Renderer AI Draft mode.
   */
  buildNarrativePrompt(snap, format, opts = {}) {
    const chStart = opts.chStart || 1;
    const chEnd   = opts.chEnd   || snap.chCount || 24;
    const meta    = snap.meta    || {};

    // ── System prompt ────────────────────────────────────────────
    const FORMAT_INSTRUCTIONS = {
      novel: `You are a professional novelist. Write vivid, immersive prose fiction. Use past tense, third-person limited perspective. Show don't tell. Honor each character's voice description in their dialogue. Expand high-weight events into full paragraphs. Use chapter headings.`,
      screenplay: `You are a professional screenwriter. Write in standard screenplay format: SLUGLINES IN CAPS, action lines in present tense, CHARACTER NAME centered above dialogue. Use BEAT. for pauses. Follow Hollywood format strictly.`,
      song: `You are a professional lyricist. Write structured song lyrics (verse/pre-chorus/chorus/bridge). Map emotional arc values to lyrical intensity. Use lyric seeds as starting phrases. Include [VERSE 1], [CHORUS], etc. labels.`,
      manual: `You are a world-building expert. Write a comprehensive World Bible document. Include character profiles, faction relationships, geographic descriptions, laws and rules, and a timeline of key events.`,
      game: `You are a game narrative designer. Write in game script format with QUEST headers, NPC dialogue nodes, player choice branches (→ [Choice]), trigger conditions, and quest objectives.`,
      session: `You are a music producer creating session notes. Write a detailed creative brief including: project overview, BPM/key, track-by-track notes, mood references, and chapter-by-chapter emotional arc summary.`,
    };

    const system = FORMAT_INSTRUCTIONS[format] || FORMAT_INSTRUCTIONS.novel;

    // ── Character block ──────────────────────────────────────────
    const charBlock = (snap.characters || []).map(ch => {
      const v = ch.emotionVector || ch.vectors || {};
      const vectors = Object.entries(v).map(([k,val]) => `${k}:${val}`).join(', ');
      return `• ${ch.name} [${ch.role || 'character'}${ch.faction ? ' — ' + ch.faction : ''}]
  Voice: ${ch.dialogue_voice || ch.dialogueVoice || 'neutral'}
  Emotion: ${vectors || 'balanced'}
  Traits: ${(ch.traits || []).join(', ') || 'none listed'}`;
    }).join('\n\n');

    // ── Scene block ──────────────────────────────────────────────
    const actBlock = (snap.acts || []).map(act => {
      const scenesInRange = (act.scenes || []).filter(s =>
        s.chapter >= chStart && s.chapter <= chEnd
      );
      if (!scenesInRange.length) return null;

      const sceneDetails = scenesInRange.map(scene => {
        const locType  = scene.scene_type  || 'INT';
        const tod      = scene.time_of_day || 'DAY';
        const zoneName = (scene.zones || [])[0] || scene.name;
        const slug     = `${locType}. ${zoneName.toUpperCase()} — ${tod}`;

        const actors   = (scene.resolvedActors || scene.actors || [])
          .map(a => (typeof a === 'string' ? a : a.name)).join(', ');

        // Propagation state at this chapter
        const propState = snap.propValueAt
          ? Object.entries(snap.propValueAt(scene.chapter))
              .map(([k,v]) => `${k}=${Math.round(v)}`)
              .join(', ')
          : '';

        // Dialogue lines
        const lines = (scene.dialogue || [])
          .sort((a,b) => (a.beat||0) - (b.beat||0))
          .map(d => `  ${d.actorName}: "${d.line}"${d.direction ? ` (${d.direction})` : ''}`)
          .join('\n');

        // Clips in this scene
        const clips = snap.clipsAt
          ? snap.clipsAt(scene.chapter)
              .filter(c => c.prose_weight > 0.2)
              .map(c => `[${c.name || c.id} weight:${Math.round((c.prose_weight||0.5)*100)}%]`)
              .join(', ')
          : '';

        return `  Chapter ${scene.chapter}: ${scene.name}
  Location: ${slug}
  Actors: ${actors || 'none specified'}
  ${propState ? 'Arc state: ' + propState : ''}
  ${clips ? 'Key events: ' + clips : ''}
  ${scene.notes ? 'Director notes: ' + scene.notes : ''}
  Dialogue:
${lines || '  (no scripted dialogue — write from character voices)'}`;
      }).filter(Boolean).join('\n\n');

      return `ACT: ${act.name || act.id}
${sceneDetails}`;
    }).filter(Boolean).join('\n\n---\n\n');

    // ── Arc block ─────────────────────────────────────────────────
    const arcBlock = (snap.propagations || []).map(arc =>
      `• ${arc.name}: ${arc.curve || 'linear'} curve, range ${arc.min||0}–${arc.max||100}, chapters ${arc.startCh||1}–${arc.endCh||chEnd}`
    ).join('\n');

    // ── Assemble user prompt ──────────────────────────────────────
    const user = `## PROJECT: ${meta.name || 'Untitled Story'}
${meta.desc ? '## LOGLINE: ' + meta.desc : ''}
## FORMAT: ${format.toUpperCase()}
## CHAPTERS: ${chStart} through ${chEnd}
## BPM: ${snap.bpm || 120} | KEY: ${meta.key_signature || 'none'}

---

## CHARACTERS
${charBlock || 'No characters defined.'}

---

## NARRATIVE ARCS
${arcBlock || 'No arcs defined.'}

---

## ACTS & SCENES
${actBlock || 'No scenes in chapter range.'}

---

Write the complete ${format} now. Start immediately with the first scene — no preamble, no "here is your" introduction. Write like a professional.`;

    return { system, user };
  },

  /**
   * Parse a lore document (plain text) and extract structured data
   * for import into the Quantum Loom library.
   */
  async parseLoreDocument(text, opts = {}) {
    const system = `You are a narrative data extractor for the Quantum Loom storytelling system.
Extract structured data from the provided lore text and return ONLY valid JSON.
Extract: characters (name, role, description, traits), locations (name, description, atmosphere),
events (name, description, chapter/sequence), factions (name, description).`;

    const schema = {
      characters: [{ name:'', role:'', description:'', traits:[], dialogue_voice:'' }],
      locations:  [{ name:'', description:'', atmosphere:'' }],
      events:     [{ name:'', description:'', type:'event' }],
      factions:   [{ name:'', description:'' }],
    };

    return LoomLLM.structured(system, `Extract all narrative data from this text:\n\n${text}`, schema, opts);
  },

  /**
   * Generate a batch of library capsules from a concept description.
   */
  async generateLibraryBatch(concept, count = 10, types = ['glyph','rune','event'], opts = {}) {
    const system = `You are a world-building assistant for the Quantum Loom system. Generate creative, thematically consistent narrative capsules. Return ONLY valid JSON.`;
    const schema = {
      capsules: Array(3).fill({ name:'', type:'glyph', description:'', world_logic:'', lyric_seed:'', tags:[] }),
    };
    const prompt = `Generate ${count} Quantum Loom capsules for the concept: "${concept}".
Use these types: ${types.join(', ')}.
Each capsule needs: name (evocative, 2-4 words), type (one of: ${types.join(', ')}), description (1-2 sentences), world_logic (a short rule or effect), lyric_seed (a poetic line), tags (3-5 keywords).`;

    const result = await LoomLLM.structured(system, prompt, schema, opts);
    return result?.capsules || [];
  },
};

/* ════════════════════════════════════════════════════════════════
   EXPORT
════════════════════════════════════════════════════════════════ */
if (typeof module !== 'undefined' && module.exports) {
  module.exports = LoomLLM;
} else {
  global.LoomLLM = LoomLLM;
}

})(typeof window !== 'undefined' ? window : global);
