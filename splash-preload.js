/**
 * QUANTUM LOOM — Splash Preload
 * splash-preload.js
 *
 * Exposes splashBridge to the splash screen HTML so it can
 * signal ready and dismiss events back to main.js
 */
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('splashBridge', {
  ready:   () => ipcRenderer.send('splash:ready'),
  dismiss: () => ipcRenderer.send('splash:dismiss'),
});
