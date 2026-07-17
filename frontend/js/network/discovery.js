import { getBackendBaseUrl } from '../config.js';
import { log, setStatus } from '../logger.js';
import { state } from '../state.js';
import { connectWs } from './websocket.js';
import { renderDiscovered } from '../ui/render.js';

export async function createSession() {
    const baseUrl = getBackendBaseUrl();

    setStatus('Creating session...');

    const res = await fetch(`${baseUrl}/create_session`);
    const data = await res.json();

    state.sessionId = data.session_id;

    log(`🟢 Session: ${state.sessionId}`);
    log(`📱 QR: ${data.qr_data}`);

    QRCode.toCanvas(document.getElementById('qr'), data.qr_data);

    const connectBtn = document.getElementById('connectBtn');
    if (connectBtn) connectBtn.disabled = false;

    // Do not auto-connect WebSocket; user will connect via button.
    setStatus('Session ready');
}

export async function fetchDiscovery() {
    const baseUrl = getBackendBaseUrl();

    try {
        setStatus('Searching network...');
        const res = await fetch(`${baseUrl}/discovery`);
        const data = await res.json();
        renderDiscovered(data);
        setStatus('Search complete');
        log(`🔍 Found ${data.length} devices`);
    } catch (e) {
        setStatus('Discovery error');
        log(`❌ Discovery failed: ${e.message}`);
    }
}

