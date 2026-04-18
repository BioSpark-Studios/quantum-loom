/**
 * QUANTUM LOOM — Electron Shell
 * main.js
 *
 * Splash sequence:
 *   1. BioSpark splash opens instantly (800×600, frameless)
 *   2. Hub window loads silently in background
 *   3. After 5s: BioSpark fades, Loom splash appears
 *   4. Loom loops until user clicks → hub shows
 */

const { app, BrowserWindow, Menu, dialog, ipcMain, shell, nativeTheme } = require('electron');
const path  = require('path');
const fs    = require('fs');
const os    = require('os');

// This pulls in your signaling logic so it runs inside the Electron process
const broadcast = require('./app/loom-broadcast.js');

app.whenReady().then(() => {
  // Initialize the broadcast/signal logic
  if (typeof broadcast.init === 'function') {
    broadcast.init(); 
  }
  createWindow();
});

let mainWindow   = null;
let splashWindow = null;
let hubReady     = false;
let splashPhase  = 'biospark'; // 'biospark' | 'loom' | 'done'

const DOCS_DIR = path.join(os.homedir(), 'Documents', 'Quantum Loom Projects');
function ensureDocsDir() {
  if (!fs.existsSync(DOCS_DIR)) fs.mkdirSync(DOCS_DIR, { recursive: true });
}

/* ════════════════════════════════════════════════════════════════
   SPLASH WINDOW
════════════════════════════════════════════════════════════════ */
function createSplash() {
  splashWindow = new BrowserWindow({
    width:           800,
    height:          600,
    frame:           false,
    transparent:     false,
    resizable:       false,
    center:          true,
    alwaysOnTop:     true,
    backgroundColor: '#010a0d',
    webPreferences: {
      preload:          path.join(__dirname, 'splash-preload.js'),
      contextIsolation: true,
      nodeIntegration:  false,
    },
  });

  splashWindow.loadFile(path.join(__dirname, 'app', 'biospark-splash.html'));

  // After 5 seconds switch to Loom splash
  setTimeout(() => {
    if (splashWindow && !splashWindow.isDestroyed()) {
      splashPhase = 'loom';
      splashWindow.loadFile(path.join(__dirname, 'app', 'loom-splash.html'));
    }
  }, 5000);

  splashWindow.on('closed', () => { splashWindow = null; });
}

/* ════════════════════════════════════════════════════════════════
   MAIN WINDOW
════════════════════════════════════════════════════════════════ */
function createMainWindow() {
  nativeTheme.themeSource = 'dark';

  mainWindow = new BrowserWindow({
    width:          1440,
    height:         900,
    minWidth:       1024,
    minHeight:      680,
    show:           false,   // hidden until splash dismissed
    titleBarStyle:  process.platform === 'darwin' ? 'hiddenInset' : 'default',
    backgroundColor: '#04050a',
    icon:           path.join(__dirname, 'assets', 'icon.png'),
    webPreferences: {
      preload:                    path.join(__dirname, 'preload.js'),
      contextIsolation:           true,
      nodeIntegration:            false,
      webSecurity:                false,   // allows local file iframes
      allowRunningInsecureContent: false,
    },
  });

  mainWindow.loadFile(path.join(__dirname, 'app', 'quantum-loom-hub.html'));

  mainWindow.webContents.once('did-finish-load', () => {
    hubReady = true;
    // If user already clicked through splash, show hub now
    if (splashPhase === 'done') showHub();
  });

  mainWindow.on('closed', () => { mainWindow = null; });
}

/* ── Show hub and close splash ────────────────────────────────── */
function showHub() {
  splashPhase = 'done';
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.show();
    mainWindow.focus();
    if (process.env.QL_DEV) mainWindow.webContents.openDevTools();
  }
  if (splashWindow && !splashWindow.isDestroyed()) {
    // Give the fade-out animation time to finish (800ms) before destroying
    setTimeout(() => {
      if (splashWindow && !splashWindow.isDestroyed()) splashWindow.close();
    }, 900);
  }
}

/* ════════════════════════════════════════════════════════════════
   SPLASH IPC HANDLERS
════════════════════════════════════════════════════════════════ */
ipcMain.on('splash:ready', () => {
  // Splash finished its entrance animation — nothing to do
});

ipcMain.on('splash:dismiss', () => {
  if (hubReady) {
    showHub();
  } else {
    // Hub not ready yet — wait for it then show
    splashPhase = 'done';
    if (mainWindow) {
      mainWindow.webContents.once('did-finish-load', () => showHub());
    }
  }
});

/* ════════════════════════════════════════════════════════════════
   FILE OPERATIONS
════════════════════════════════════════════════════════════════ */
ipcMain.handle('project:save', async (_event, payload) => {
  ensureDocsDir();
  const projectName = payload.name || 'My Story';
  const safeName    = projectName.replace(/[^a-z0-9 _-]/gi, '_');

  const { filePath, canceled } = await dialog.showSaveDialog(mainWindow, {
    title:       'Save Quantum Loom Project',
    defaultPath: path.join(DOCS_DIR, safeName + '.loom'),
    buttonLabel: 'Save Project',
    filters:     [{ name: 'Quantum Loom Project', extensions: ['loom'] }, { name: 'JSON', extensions: ['json'] }],
  });

  if (canceled || !filePath) return { ok: false, reason: 'canceled' };
  try {
    fs.writeFileSync(filePath, JSON.stringify(payload.data, null, 2), 'utf8');
    return { ok: true, filePath };
  } catch (err) {
    return { ok: false, reason: err.message };
  }
});

ipcMain.handle('project:open', async () => {
  ensureDocsDir();
  const { filePaths, canceled } = await dialog.showOpenDialog(mainWindow, {
    title:       'Open Quantum Loom Project',
    defaultPath: DOCS_DIR,
    buttonLabel: 'Open Project',
    filters:     [{ name: 'Quantum Loom Project', extensions: ['loom'] }, { name: 'JSON', extensions: ['json'] }],
    properties:  ['openFile'],
  });

  if (canceled || !filePaths.length) return { ok: false, reason: 'canceled' };
  try {
    const raw  = fs.readFileSync(filePaths[0], 'utf8');
    const data = JSON.parse(raw);
    return { ok: true, data, filePath: filePaths[0] };
  } catch (err) {
    return { ok: false, reason: 'Could not read file: ' + err.message };
  }
});

ipcMain.handle('export:save', async (_event, { content, ext, defaultName }) => {
  ensureDocsDir();
  const filter = ext === 'html'
    ? { name: 'HTML Document', extensions: ['html'] }
    : { name: 'Text File',     extensions: ['txt']  };

  const { filePath, canceled } = await dialog.showSaveDialog(mainWindow, {
    title:       'Export — ' + (defaultName || 'output'),
    defaultPath: path.join(DOCS_DIR, (defaultName || 'export') + '.' + ext),
    buttonLabel: 'Export',
    filters:     [filter],
  });

  if (canceled || !filePath) return { ok: false, reason: 'canceled' };
  try {
    fs.writeFileSync(filePath, content, 'utf8');
    shell.showItemInFolder(filePath);
    return { ok: true, filePath };
  } catch (err) {
    return { ok: false, reason: err.message };
  }
});

ipcMain.handle('dialog:confirm', async (_event, { title, message, detail }) => {
  const result = await dialog.showMessageBox(mainWindow, {
    type: 'warning', buttons: ['Cancel', 'Confirm'],
    defaultId: 0, cancelId: 0, title, message, detail,
  });
  return { confirmed: result.response === 1 };
});

ipcMain.handle('dialog:error', async (_event, { title, content }) => {
  await dialog.showErrorBox(title || 'Error', content);
  return {};
});

ipcMain.handle('folder:open', async () => {
  ensureDocsDir();
  shell.openPath(DOCS_DIR);
  return {};
});

/* ════════════════════════════════════════════════════════════════
   NATIVE MENU
════════════════════════════════════════════════════════════════ */
function buildMenu() {
  const isMac = process.platform === 'darwin';
  const template = [
    ...(isMac ? [{ label: app.name, submenu: [
      { role: 'about' }, { type: 'separator' }, { role: 'services' },
      { type: 'separator' }, { role: 'hide' }, { role: 'hideOthers' },
      { role: 'unhide' }, { type: 'separator' }, { role: 'quit' },
    ]}] : []),
    { label: 'File', submenu: [
      { label: 'New Project',   accelerator: 'CmdOrCtrl+N', click: () => mainWindow?.webContents.send('menu:new-project') },
      { type: 'separator' },
      { label: 'Open Project…', accelerator: 'CmdOrCtrl+O', click: () => mainWindow?.webContents.send('menu:open-project') },
      { label: 'Save Project…', accelerator: 'CmdOrCtrl+S', click: () => mainWindow?.webContents.send('menu:save-project') },
      { type: 'separator' },
      { label: 'Open Projects Folder', click: () => shell.openPath(DOCS_DIR) },
      { type: 'separator' },
      isMac ? { role: 'close' } : { role: 'quit' },
    ]},
    { label: 'Edit', submenu: [
      { role: 'undo' }, { role: 'redo' }, { type: 'separator' },
      { role: 'cut' }, { role: 'copy' }, { role: 'paste' }, { role: 'selectAll' },
    ]},
    { label: 'View', submenu: [
      { role: 'reload' }, { type: 'separator' },
      { role: 'resetZoom' }, { role: 'zoomIn' }, { role: 'zoomOut' },
      { type: 'separator' }, { role: 'togglefullscreen' },
      ...(process.env.QL_DEV ? [{ type: 'separator' }, { role: 'toggleDevTools' }] : []),
    ]},
    { label: 'Help', submenu: [
      { label: 'User Manual', click: () => mainWindow?.webContents.send('menu:open-manual') },
      { type: 'separator' },
      { label: 'About Quantum Loom', click: async () => {
        await dialog.showMessageBox(mainWindow, {
          type: 'info', title: 'Quantum Loom', message: 'Quantum Loom',
          detail: `Version ${app.getVersion()}\n\nA narrative DAW for story, music & world-building.\n\nBuilt for writers, teachers & students who think in systems.`,
          buttons: ['OK'],
        });
      }},
    ]},
  ];
  Menu.setApplicationMenu(Menu.buildFromTemplate(template));
}

/* ════════════════════════════════════════════════════════════════
   APP LIFECYCLE
════════════════════════════════════════════════════════════════ */
app.whenReady().then(() => {
  createSplash();
  createMainWindow();
  buildMenu();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createMainWindow();
      mainWindow.once('ready-to-show', () => mainWindow.show());
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
