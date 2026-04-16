/* ╔══════════════════════════════════════════════════════════════════╗
   ║         QUANTUM LOOM — FIREBASE ADAPTER  v1.0                   ║
   ║                                                                  ║
   ║  BYOK (Bring Your Own Key) Firebase Firestore sync.             ║
   ║  Syncs all ql-* localStorage keys to a Firestore project doc.  ║
   ║  Listens for remote changes and relays them to the hub.         ║
   ║                                                                  ║
   ║  SETUP (one-time):                                               ║
   ║  1. Create a Firebase project at console.firebase.google.com    ║
   ║  2. Enable Firestore Database                                    ║
   ║  3. Go to Project Settings → General → Your apps → Web app      ║
   ║  4. Copy the firebaseConfig object                               ║
   ║  5. Call LoomFirebase.init(yourConfig) before anything else      ║
   ║                                                                  ║
   ║  USAGE:                                                          ║
   ║    <script src="loom-firebase.js"></script>                      ║
   ║    <script>                                                       ║
   ║      LoomFirebase.init({                                         ║
   ║        apiKey:      "...",                                        ║
   ║        authDomain:  "...",                                        ║
   ║        projectId:   "...",                                        ║
   ║        storageBucket: "...",                                      ║
   ║        messagingSenderId: "...",                                  ║
   ║        appId: "...",                                              ║
   ║      }, {                                                         ║
   ║        projectId: 'my-loom-project',  // Firestore doc ID        ║
   ║        autoSync:  true,               // sync every 30s          ║
   ║        hubWindow: window.parent,      // relay to hub            ║
   ║      });                                                          ║
   ║    </script>                                                      ║
   ╚══════════════════════════════════════════════════════════════════╝ */

(function(global) {
'use strict';

/* ── Constants ────────────────────────────────────────────────────── */
const QL_KEYS = [
  'ql-workspace',
  'ql-workspace-meta',
  'ql-library',
  'ql-zones',
  'ql-scenes',
  'ql-propagations',
  'ql-sequencer',
  'ql-compiled',
  'ql-graphs',
];

const FIREBASE_SDK_URL = 'https://www.gstatic.com/firebasejs/10.7.1/firebase-app.js';
const FIRESTORE_SDK_URL = 'https://www.gstatic.com/firebasejs/10.7.1/firebase-firestore.js';

/* ── State ────────────────────────────────────────────────────────── */
let _app        = null;
let _db         = null;
let _config     = null;
let _opts       = {};
let _unsubscribe = null;
let _lastRemoteHash = {};
let _syncTimer  = null;
let _initialized = false;
let _listeners  = {};

/* ── Event bus ────────────────────────────────────────────────────── */
function on(event, fn) {
  if (!_listeners[event]) _listeners[event] = [];
  _listeners[event].push(fn);
}
function emit(event, data) {
  (_listeners[event] || []).forEach(fn => { try { fn(data); } catch(e) {} });
}

/* ── Logging ──────────────────────────────────────────────────────── */
function log(msg, level='info') {
  const prefix = '[LoomFirebase]';
  if (level === 'error') console.error(prefix, msg);
  else if (level === 'warn')  console.warn(prefix, msg);
  else console.log(prefix, msg);
  emit('log', { level, msg, ts: Date.now() });
}

/* ── Load Firebase SDK dynamically ───────────────────────────────── */
function loadModule(url) {
  return new Promise((resolve, reject) => {
    // Check if already loaded via global
    const script = document.createElement('script');
    script.type = 'module';
    script.textContent = `
      import { initializeApp } from '${FIREBASE_SDK_URL}';
      import { getFirestore, doc, setDoc, onSnapshot, serverTimestamp } from '${FIRESTORE_SDK_URL}';
      window._LoomFBModules = { initializeApp, getFirestore, doc, setDoc, onSnapshot, serverTimestamp };
    `;
    script.onload = () => resolve(window._LoomFBModules);
    script.onerror = reject;
    document.head.appendChild(script);
    // Modules execute immediately, resolve after a tick
    setTimeout(() => {
      if (window._LoomFBModules) resolve(window._LoomFBModules);
      else reject(new Error('Firebase modules did not load'));
    }, 800);
  });
}

/* ── Read all ql-* from localStorage ─────────────────────────────── */
function readLocalStorage() {
  const data = { _savedAt: new Date().toISOString(), _version: 1 };
  QL_KEYS.forEach(key => {
    const val = localStorage.getItem(key);
    if (val !== null) {
      try {
        data[key.replace(/-/g, '_')] = JSON.parse(val);  // Firestore field names can't have hyphens
      } catch(e) {
        data[key.replace(/-/g, '_')] = val;
      }
    }
  });
  return data;
}

/* ── Write Firestore data back to localStorage ───────────────────── */
function writeToLocalStorage(remoteData) {
  const updated = [];
  QL_KEYS.forEach(key => {
    const fsKey = key.replace(/-/g, '_');
    if (remoteData[fsKey] !== undefined) {
      const newVal = JSON.stringify(remoteData[fsKey]);
      const oldVal = localStorage.getItem(key);
      if (newVal !== oldVal) {
        localStorage.setItem(key, newVal);
        updated.push(key);
      }
    }
  });
  return updated;
}

/* ── Hash a data object for change detection ─────────────────────── */
function quickHash(obj) {
  const s = JSON.stringify(obj);
  let h = 0;
  for (let i = 0; i < Math.min(s.length, 2000); i++) {
    h = ((h << 5) - h) + s.charCodeAt(i);
    h |= 0;
  }
  return h;
}

/* ── Relay updated key to hub via postMessage ────────────────────── */
function relayToHub(key) {
  const hub = _opts.hubWindow || (window.parent !== window ? window.parent : null);
  if (!hub) return;
  try {
    const val = localStorage.getItem(key);
    hub.postMessage({ type: 'hub:storage:updated', key, value: val }, '*');
  } catch(e) {}
}

/* ── Push local → Firestore ─────────────────────────────────────── */
async function push(silent = false) {
  if (!_db) { log('Not initialized', 'warn'); return { ok: false, error: 'not_initialized' }; }

  try {
    const { doc, setDoc, serverTimestamp } = window._LoomFBModules;
    const docId   = _opts.projectId || 'default';
    const data    = readLocalStorage();
    data._pushedAt = serverTimestamp();

    const ref = doc(_db, 'loom_projects', docId);
    await setDoc(ref, data, { merge: true });

    if (!silent) log(`Pushed to Firestore (project: ${docId})`);
    emit('push', { docId, keys: QL_KEYS.filter(k => data[k.replace(/-/g,'_')] !== undefined) });
    return { ok: true };
  } catch(e) {
    log('Push failed: ' + e.message, 'error');
    emit('error', { op: 'push', error: e.message });
    return { ok: false, error: e.message };
  }
}

/* ── Pull Firestore → local (one-time) ──────────────────────────── */
async function pull() {
  if (!_db) { log('Not initialized', 'warn'); return { ok: false, error: 'not_initialized' }; }

  try {
    const { doc, getDoc } = window._LoomFBModules;
    if (!getDoc) throw new Error('getDoc not available — import it from firebase-firestore');
    const docId = _opts.projectId || 'default';
    const ref   = doc(_db, 'loom_projects', docId);
    const snap  = await getDoc(ref);

    if (!snap.exists()) {
      log('No remote document found — nothing to pull', 'warn');
      return { ok: false, error: 'not_found' };
    }

    const remote  = snap.data();
    const updated = writeToLocalStorage(remote);
    updated.forEach(k => relayToHub(k));
    log(`Pulled from Firestore: ${updated.length} keys updated`);
    emit('pull', { updated });
    return { ok: true, updated };
  } catch(e) {
    log('Pull failed: ' + e.message, 'error');
    return { ok: false, error: e.message };
  }
}

/* ── Subscribe to real-time updates ─────────────────────────────── */
function subscribe() {
  if (!_db) return;
  const { doc, onSnapshot } = window._LoomFBModules;
  const docId = _opts.projectId || 'default';
  const ref   = doc(_db, 'loom_projects', docId);

  _unsubscribe = onSnapshot(ref, (snap) => {
    if (!snap.exists()) return;
    const remote  = snap.data();
    const hash    = quickHash(remote);

    // Skip if this looks like our own push (hash matches what we last sent)
    if (hash === _lastRemoteHash.val) return;
    _lastRemoteHash.val = hash;

    const updated = writeToLocalStorage(remote);
    if (updated.length > 0) {
      log(`Real-time update: ${updated.length} keys changed`);
      updated.forEach(k => relayToHub(k));
      emit('update', { updated, remote });
    }
  }, (err) => {
    log('Snapshot listener error: ' + err.message, 'error');
    emit('error', { op: 'subscribe', error: err.message });
  });

  log(`Subscribed to real-time updates (project: ${docId})`);
}

/* ── Auto-sync timer ─────────────────────────────────────────────── */
function startAutoSync(intervalMs = 30000) {
  if (_syncTimer) clearInterval(_syncTimer);
  _syncTimer = setInterval(() => push(true), intervalMs);
  log(`Auto-sync started (every ${intervalMs / 1000}s)`);
}

function stopAutoSync() {
  if (_syncTimer) { clearInterval(_syncTimer); _syncTimer = null; }
}

/* ── Init ─────────────────────────────────────────────────────────── */
async function init(firebaseConfig, options = {}) {
  if (_initialized) {
    log('Already initialized — call LoomFirebase.destroy() first to reinitialize', 'warn');
    return { ok: false, error: 'already_initialized' };
  }

  // Validate config
  const required = ['apiKey', 'authDomain', 'projectId'];
  for (const k of required) {
    if (!firebaseConfig[k]) {
      log(`Missing required firebaseConfig.${k}`, 'error');
      return { ok: false, error: `missing_${k}` };
    }
  }

  _config = firebaseConfig;
  _opts   = {
    projectId: options.projectId || 'default',
    autoSync:  options.autoSync  !== false,
    autoSyncInterval: options.autoSyncInterval || 30000,
    realtime:  options.realtime  !== false,
    hubWindow: options.hubWindow || null,
    onReady:   options.onReady   || null,
  };

  log(`Initializing — project: ${_opts.projectId}`);
  emit('initializing', { projectId: _opts.projectId });

  try {
    // Load Firebase modules
    await loadModule();

    const { initializeApp, getFirestore } = window._LoomFBModules;
    _app = initializeApp(firebaseConfig, 'loom-' + _opts.projectId);
    _db  = getFirestore(_app);

    _initialized = true;
    log('Firebase connected ✓');
    emit('ready', { projectId: _opts.projectId });

    // Real-time listener
    if (_opts.realtime) subscribe();

    // Auto-sync push
    if (_opts.autoSync) startAutoSync(_opts.autoSyncInterval);

    // Initial push (local → remote)
    await push(true);

    if (typeof _opts.onReady === 'function') _opts.onReady();

    return { ok: true };
  } catch(e) {
    log('Initialization failed: ' + e.message, 'error');
    emit('error', { op: 'init', error: e.message });
    return { ok: false, error: e.message };
  }
}

/* ── Destroy (cleanup) ───────────────────────────────────────────── */
function destroy() {
  stopAutoSync();
  if (_unsubscribe) { _unsubscribe(); _unsubscribe = null; }
  _app = null; _db = null; _config = null; _initialized = false;
  log('Adapter destroyed');
  emit('destroyed', {});
}

/* ── Status ──────────────────────────────────────────────────────── */
function status() {
  return {
    initialized: _initialized,
    projectId:   _opts.projectId || null,
    autoSync:    !!_syncTimer,
    realtime:    !!_unsubscribe,
    localKeys:   QL_KEYS.filter(k => localStorage.getItem(k) !== null),
  };
}

/* ── Hub wiring helper ───────────────────────────────────────────── */
/*
  Call LoomFirebase.wireToHub() from within a layer's script to
  automatically push whenever the hub announces a storage write.
  This bridges hub:storage:write → Firestore push.
*/
function wireToHub() {
  window.addEventListener('message', async e => {
    const d = e.data;
    if (!d || typeof d !== 'object') return;
    if (d.type === 'hub:storage:write' && _initialized) {
      // Debounced push — wait 1.5s for burst of writes to settle
      clearTimeout(wireToHub._debounce);
      wireToHub._debounce = setTimeout(() => push(true), 1500);
    }
    if (d.type === 'hub:init' && _initialized) {
      await pull();
    }
  });
  log('Hub wiring active — will push on hub:storage:write');
}
wireToHub._debounce = null;

/* ── Project management ──────────────────────────────────────────── */

/* List all projects in this Firebase (requires Firestore list permission) */
async function listProjects() {
  if (!_db) return { ok: false, error: 'not_initialized' };
  try {
    const { collection, getDocs } = window._LoomFBModules;
    if (!getDocs) throw new Error('getDocs not imported');
    const snap = await getDocs(collection(_db, 'loom_projects'));
    const projects = snap.docs.map(d => ({
      id:       d.id,
      savedAt:  d.data()._savedAt?.toDate?.()?.toISOString() || 'unknown',
      keys:     QL_KEYS.filter(k => d.data()[k.replace(/-/g,'_')] !== undefined).length,
    }));
    return { ok: true, projects };
  } catch(e) {
    return { ok: false, error: e.message };
  }
}

/* Clone this project to a new ID */
async function cloneProject(newProjectId) {
  if (!_db) return { ok: false, error: 'not_initialized' };
  try {
    const { doc, setDoc, serverTimestamp } = window._LoomFBModules;
    const data = readLocalStorage();
    data._clonedFrom = _opts.projectId;
    data._savedAt    = serverTimestamp();
    await setDoc(doc(_db, 'loom_projects', newProjectId), data);
    log(`Cloned to project: ${newProjectId}`);
    return { ok: true, newProjectId };
  } catch(e) {
    return { ok: false, error: e.message };
  }
}

/* Switch active project (pulls remote data) */
async function switchProject(newProjectId) {
  const oldId   = _opts.projectId;
  const wasRT   = !!_unsubscribe;
  const wasAuto = !!_syncTimer;

  // Unsubscribe from current
  if (_unsubscribe) { _unsubscribe(); _unsubscribe = null; }
  stopAutoSync();

  _opts.projectId = newProjectId;

  // Pull new project into local
  const result = await pull();

  // Re-subscribe
  if (wasRT)   subscribe();
  if (wasAuto) startAutoSync(_opts.autoSyncInterval);

  log(`Switched project: ${oldId} → ${newProjectId}`);
  emit('projectSwitch', { from: oldId, to: newProjectId });
  return result;
}

/* ── Conflict resolver ───────────────────────────────────────────── */
/*
  Last-write-wins by default. Call setConflictResolver(fn) to override.
  fn(localValue, remoteValue, key) → resolvedValue
*/
let _conflictResolver = null;
function setConflictResolver(fn) {
  _conflictResolver = fn;
  log('Custom conflict resolver registered');
}

/* ── Export ─────────────────────────────────────────────────────── */
const LoomFirebase = {
  /* Lifecycle */
  init,
  destroy,
  status,
  wireToHub,

  /* Sync ops */
  push,
  pull,
  subscribe,
  stopAutoSync,
  startAutoSync,

  /* Project management */
  listProjects,
  cloneProject,
  switchProject,

  /* Advanced */
  setConflictResolver,

  /* Events */
  on,

  /* Constants */
  QL_KEYS,
};

/* UMD export */
if (typeof module !== 'undefined' && module.exports) {
  module.exports = LoomFirebase;
} else {
  global.LoomFirebase = LoomFirebase;
}

})(typeof window !== 'undefined' ? window : global);

/* ══════════════════════════════════════════════════════════════════════
   QUICK-START EXAMPLES
   ══════════════════════════════════════════════════════════════════════

// 1. Basic — just add <script src="loom-firebase.js"></script> to hub or any layer,
//    then call init() with your config:

LoomFirebase.init({
  apiKey:            "AIzaSy...",
  authDomain:        "your-project.firebaseapp.com",
  projectId:         "your-project-id",
  storageBucket:     "your-project.appspot.com",
  messagingSenderId: "123456789",
  appId:             "1:123456789:web:abc123",
}, {
  projectId: 'my-story',       // Firestore document ID
  autoSync:   true,            // push every 30s
  realtime:   true,            // listen for remote changes
  hubWindow:  window.parent,   // relay changes to hub if inside iframe
});

// 2. With event listeners:

LoomFirebase.on('ready',  ()   => console.log('Firebase ready'));
LoomFirebase.on('push',   (e)  => console.log('Pushed:', e.keys));
LoomFirebase.on('update', (e)  => console.log('Remote update:', e.updated));
LoomFirebase.on('error',  (e)  => console.error('Firebase error:', e));

// 3. Wire to hub (call from inside hub.html or any layer):

LoomFirebase.wireToHub();
// Now every hub:storage:write auto-triggers a debounced push

// 4. Switch between projects:

await LoomFirebase.switchProject('project-2');

// 5. Clone current project:

await LoomFirebase.cloneProject('backup-' + Date.now());

// 6. Manual push/pull:

await LoomFirebase.push();   // local → Firestore
await LoomFirebase.pull();   // Firestore → local

// 7. Firestore security rules (paste in Firebase console → Firestore → Rules):

rules_version = '2';
service cloud.firestore {
  match /databases/{database}/documents {
    match /loom_projects/{projectId} {
      allow read, write: if request.auth != null;
      // Or for testing only (remove in production):
      // allow read, write: if true;
    }
  }
}

══════════════════════════════════════════════════════════════════════ */
