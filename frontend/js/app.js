
import { setStatus, log } from './logger.js';
import { createSession, fetchDiscovery } from './network/discovery.js';
import { startScanner } from './ui/render.js';
import { connectWs, ping } from './network/websocket.js';
import { initTransferManager, sendFileToAll } from './transfer/transfer_manager_singleton.js';

function bindButtons() {
  const createBtn = document.getElementById('createBtn');
  const scanBtn = document.getElementById('scanBtn');
  const searchBtn = document.getElementById('searchBtn');
  const connectBtn = document.getElementById('connectBtn');
  const pingBtn = document.getElementById('pingBtn');
  const sendBtn = document.getElementById('sendBtn');

  if (createBtn) createBtn.addEventListener('click', () => createSession().catch(() => log('Backend not ready.')));
  if (scanBtn) scanBtn.addEventListener('click', () => startScanner());
  if (searchBtn) searchBtn.addEventListener('click', () => fetchDiscovery());
  if (connectBtn) connectBtn.addEventListener('click', () => connectWs());
  if (pingBtn) pingBtn.addEventListener('click', () => ping());
  if (sendBtn) {
    sendBtn.addEventListener('click', async () => {
      const file = document.getElementById('fileInput')?.files?.[0];
      if (!file) {
        log('[transfer] ❌ No file selected');
        return;
      }

      const ready = initTransferManager();
      if (!ready) {
        log('[transfer] ❌ Transfer manager not ready');
        return;
      }

      log(`[transfer] sending ${file.name} (${file.size} bytes)`);
      setStatus('Transferring...');
      try {
        await sendFileToAll(file);
        log(`[transfer] ✅ send complete: ${file.name}`);
        setStatus('idle');
      } catch (e) {
        console.error('[transfer] send failed', e);
        log(`[transfer] ❌ send failed: ${e && e.message ? e.message : String(e)}`);
        setStatus('idle');
      }
    });
  }
}

window.addEventListener('load', () => {
  const hostname = window.location.hostname;
  const ipInput = document.getElementById('ip');
  if (ipInput && hostname) ipInput.value = hostname;

  // No auto-connect: only create session on explicit user action.
  // createSession() also enables/disables connect button.

  if (hostname) {
    // Keep status quiet; user can press Create Session.
  }

  bindButtons();
  initTransferManager();
  setInterval(fetchDiscovery, 5000);
});



