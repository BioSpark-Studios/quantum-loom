/**
 * QUANTUM LOOM — Preload Script
 * preload.js
 *
 * Runs in the renderer's context but has access to Node/Electron APIs.
 * Exposes a safe, narrow API to the hub via window.qlNative.
 * contextIsolation: true means the renderer cannot access require() directly.
 */

const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('qlNative', {

  // ── Is this running inside Electron? ──────────────────────────
  isDesktop: true,

  // ── File operations ───────────────────────────────────────────

  /** Save all ql-* localStorage data as a .loom file via native dialog */
  saveProject: (name, data) =>
    ipcRenderer.invoke('project:save', { name, data }),

  /** Open a .loom file via native dialog — returns { ok, data, filePath } */
  openProject: () =>
    ipcRenderer.invoke('project:open'),

  /** Save rendered output (.txt or .html) via native dialog */
  exportFile: (content, ext, defaultName) =>
    ipcRenderer.invoke('export:save', { content, ext, defaultName }),

  // ── Native dialogs ─────────────────────────────────────────────

  /** Show a native confirm dialog — returns { confirmed: bool } */
  confirm: (title, message, detail) =>
    ipcRenderer.invoke('dialog:confirm', { title, message, detail }),

  /** Show a native error alert */
  error: (title, content) =>
    ipcRenderer.invoke('dialog:error', { title, content }),

  /** Open the ~/Documents/Quantum Loom Projects folder */
  openFolder: () =>
    ipcRenderer.invoke('folder:open'),

  // ── Menu event listeners ───────────────────────────────────────

  /** Called when user picks File → New Project from native menu */
  onNewProject: (cb) => {
    ipcRenderer.on('menu:new-project', cb);
    return () => ipcRenderer.removeListener('menu:new-project', cb);
  },

  /** Called when user picks File → Open Project from native menu */
  onOpenProject: (cb) => {
    ipcRenderer.on('menu:open-project', cb);
    return () => ipcRenderer.removeListener('menu:open-project', cb);
  },

  /** Called when user picks File → Save Project from native menu */
  onSaveProject: (cb) => {
    ipcRenderer.on('menu:save-project', cb);
    return () => ipcRenderer.removeListener('menu:save-project', cb);
  },

  /** Called when user picks Help → User Manual */
  onOpenManual: (cb) => {
    ipcRenderer.on('menu:open-manual', cb);
    return () => ipcRenderer.removeListener('menu:open-manual', cb);
  },
});
