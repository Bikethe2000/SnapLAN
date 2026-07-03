let ws = null;
let sessionId = null;
let peers = {};
let peerConnections = {};
let dataChannels = {};
let myPeerId = null;
let incomingFiles = {};
let sendQueue = {};

const ICE_SERVERS = [
    { urls: "stun:stun.relay.metered.ca:80" },
    {
        urls: "turn:global.relay.metered.ca:80",
        username: "e8dd65b92c62d3e36c99f931",     // FIXED: was "e8dd65b92c62d3e36c99f31" - missing a "9", mistyped during transcription
        credential: "uWdWNmkhvyqTEswO"             // FIXED: key was "credentials" (plural) - RTCIceServer requires "credential" (singular), so auth was silently ignored. Also fixed trailing char: capital letter "O", not digit "0".
    },
    {
        urls: "turn:global.relay.metered.ca:443",
        username: "e8dd65b92c62d3e36c99f931",
        credential: "uWdWNmkhvyqTEswO"
    },
    {
        urls: "turn:global.relay.metered.ca:443?transport=tcp",
        username: "e8dd65b92c62d3e36c99f931",
        credential: "uWdWNmkhvyqTEswO"
    },
];

function getIp() {
    const input = document.getElementById("ip");
    if (input.value.trim()) return input.value.trim();
    
    // Auto-detect: use the hostname the app was accessed from
    const hostname = window.location.hostname;
    if (hostname && hostname !== "localhost" && hostname !== "127.0.0.1") {
        return hostname;
    }
    
    // Fallback to default if running on localhost
    return "192.168.2.9";
}

function log(msg) {
    const el = document.getElementById("log");
    el.innerHTML += "<div>" + msg + "</div>";
    el.scrollTop = el.scrollHeight;
}

function setStatus(s) {
    document.getElementById("status").innerText = "Status: " + s;
}

/* ---------------- SESSION ---------------- */

async function createSession() {
    const ip = getIp();

    setStatus("Creating session...");

    const res = await fetch(`https://${ip}:3000/create_session`);
    const data = await res.json();

    sessionId = data.session_id;

    log("🟢 Session: " + sessionId);
    log("📱 QR: " + data.qr_data);

    QRCode.toCanvas(document.getElementById("qr"), data.qr_data);

    document.getElementById("connectBtn").disabled = false;
    
    log("Auto-connecting host to session...");
    connectWs();

    setStatus("Session ready");
}

async function fetchDiscovery() {
    const ip = getIp();
    setStatus("Searching network...");

    try {
        const res = await fetch(`https://${ip}:3000/discovery`);
        const data = await res.json();
        renderDiscovered(data);
        setStatus("Search complete");
        log("🔍 Found " + data.length + " devices");
    } catch (e) {
        log("❌ Discovery failed: " + e.message);
        setStatus("Discovery error");
    }
}

/* ---------------- SCANNER ---------------- */

function startScanner() {
    const scanner = new Html5Qrcode("reader");

    Html5Qrcode.getCameras().then(devices => {
        const cam = devices[0];

        scanner.start(
            cam.id,
            { fps: 10, qrbox: 250 },
            (text) => {
                log("📷 QR: " + text);

                sessionId = text.split("/").pop();

                log("Session: " + sessionId);

                scanner.stop().catch(() => {});
                connectWs();
            }
        );
    });
}

/* ---------------- WS CONNECT ---------------- */

function connectWs() {
    if (!sessionId) {
        log("No sessionId yet.")
        return;
    }
    const ip = getIp();

    log("[connectWs] Connecting to " + ip + " with sessionId: " + sessionId);

    if (ws) {
        log("🔄 Reconnecting...");
        ws.close();
    }

    ws = new WebSocket(`wss://${ip}:3000/ws`);
    log("[connectWs] WebSocket object created");

    setStatus("Connecting...");

    ws.onopen = () => {
        log("[WS] WebSocket OPEN");
        log("✅ WS connected");

        const helloMsg = {
            type: "ClientHello",
            device_name: "Browser (" + navigator.platform + ")",
            session_id: sessionId
        };
        log("[WS] Sending ClientHello");
        ws.send(JSON.stringify(helloMsg));

        document.getElementById("pingBtn").disabled = false;
        setStatus("Connected as " + navigator.platform);
    };

    ws.onmessage = (event) => {
        log("[WS] Message received: " + event.data.substring(0, 100));

        const msg = JSON.parse(event.data);

        if (msg.type === "PeerJoined") {
            myPeerId = msg.peer_id;
            log("[WS] PeerJoined - myPeerId: " + myPeerId);
            log("👋 PeerJoined: " + msg.device_name + " (" + msg.peer_id + ")");
        }

        if (msg.type === "peer_list") {
            log("[WS] peer_list received with " + msg.data.length + " peers");
            renderPeers(msg.data);
            log("👥 Updated peer list (" + msg.data.length + ")");
        }

        if (msg.type === "discovery_list") {
            log("[WS] discovery_list received");
            renderDiscovered(msg.data);
        }

        if (msg.type === "peer_online") {
            log("[WS] peer_online: " + msg.data.name);
            log("✨ Peer Online: " + msg.data.name);
        }

        if (msg.type === "peer_offline") {
            log("[WS] peer_offline: " + msg.data.id);
            log("💀 Peer Offline: " + msg.data.id);
            cleanupPeer(msg.data.id);
        }

        if (msg.type === "Pong") {
            log("[WS] Pong received");
            log("🏓 Pong");
        }
        if (msg.type === "Offer"){
            log("[WS] Offer received from " + msg.from + " to " + msg.to);
            if (msg.to === myPeerId)
                handleOffer(msg);
        }
        if (msg.type === "Answer"){
            log("[WS] Answer received from " + msg.from + " to " + msg.to);
            if (msg.to === myPeerId)
                handleAnswer(msg);
        }
        if (msg.type === "IceCandidate"){
            log("[WS] IceCandidate received from " + msg.from);
            if (msg.to === myPeerId)
                handleICE(msg);
        }
    };

    ws.onerror = (e) => {
        console.error("[WS] WebSocket ERROR:", e);
        log("❌ WS error: Connection failed. Check if HTTPS is accepted?");
        setStatus("WS Error");
    };

    ws.onclose = () => {
        log("[WS] WebSocket CLOSED");
        log("🔌 WS closed");
        setStatus("Disconnected");
        document.getElementById("pingBtn").disabled = true;
    };
}

/* ---------------- PEERS ---------------- */
function cleanupPeer(id) {
    if (peerConnections[id]) {
        try { peerConnections[id].close(); }
        catch (e) {}
        delete peerConnections[id];
    }
    if (dataChannels[id]) {
        delete dataChannels[id];
    }
    delete sendQueue[id];
    delete peers[id];
    const div = document.getElementById("peer-" + id);
    if (div) div.remove();
}

function renderPeers(list) {
    console.log("============ PEERS ============");
    console.log(list);
    const el = document.getElementById("peers");
    if (!el) return;
    const seen = new Set();
    peers = {};
    list.forEach(p => {
        if(p.id === myPeerId) {
            log("[renderPeers] Skipping self peers: " + p.id) 
            return;
        }
        seen.add(p.id);
        peers[p.id] = p;
        let div = document.getElementById("peer-" + p.id);
        const isNewPeer = !div;
        if(!div) {
            div = document.createElement("div");
            div.id = "peer-" + p.id;
            el.appendChild(div);
        }
        div.onclick = () => {
            console.log("============ CLICKED" + p.id + " ============");
            connectToPeer(p.id);
        };

        div.innerHTML = `<b>${p.name}</b><br><small>${p.id}</small>`;
        // Auto-connect to new peers
        if (isNewPeer && !peerConnections[p.id]) {
            if (myPeerId < p.id) {
                log("[renderPeers] Auto-connecting to new peer (we win tiee-break): " + p.id);
                connectToPeer(p.id);
            } else {
                log("[renderPeers] Auto-connecting to new peer (we lose tie-break): " + p.id);
            }
            log("[renderPeers] Auto-connecting to new peer: " + p.id);
            connectToPeer(p.id);
        }
    });

    [...el.children].forEach(child => {
        const id = child.id.replace("peer-", "");
        if (!seen.has(id)) child.remove();
    });
}

function renderDiscovered(list) {
    const el = document.getElementById("discovered");
    if (!el) return;
    const currentIp = getIp();
    const seen = new Set();

    list.forEach(d => {
        // Skip current device

        if (d.ip === currentIp) return;

        const key = d.ip + ":" + d.port;
        seen.add(key);
        let div = document.getElementById("discovered-" + key);
        if(!div) {
            div = document.createElement("div");
            div.id = "discovered-" + key;
            div.style.background = "#2a2a2a";
            div.style.padding = "8px";
            div.style.margin = "4px";
            div.style.borderRadius = "6px";
            div.style.borderLeft = "4px solid #4CAF50";
            el.appendChild(div);
        }
        div.innerHTML = `<b>${d.name}</b><br><small>${d.ip}:${d.port}</small>`;
        div.onclick = () => {
            log("🔗 Connecting to " + d.name + " (" + d.ip + ")...");
            document.getElementById("ip").value = d.ip;
            createSession();
        };
        div.onmouseover = () => {
            div.style.background = "#3a3a";
        }
    });

    [...el.children].forEach(child => {
        const key = child.id.replace("discovered-", "");
        if (!seen.has(key)) child.remove();
    });
}

// Automatically try to connect to our own backend on load
window.addEventListener('load', () => {
    const hostname = window.location.hostname;
    if (hostname) {
        document.getElementById("ip").value = hostname;
        log("🏠 Auto-connecting to local backend: " + hostname);
        createSession().catch(e => {
            log("ℹ️ Local backend not ready. Click 'Create Session' to start manually.");
        });
    }
});

function createPeer(remoteId, isOfferer) {
    log("[createPeer] Creating peer connection for " + remoteId + ", isOfferer: " + isOfferer);
    
    // Don't create duplicate peer connections
    if (peerConnections[remoteId]) {
        log("[createPeer] Peer connection already exists for " + remoteId + ", reusing");
        return peerConnections[remoteId];
    }
    
    // Initialize queue for this peer if not already done
    if (!sendQueue[remoteId]) {
        sendQueue[remoteId] = [];
        log("[createPeer] Initialized send queue for " + remoteId);
    }

    try {
        const pc = new RTCPeerConnection({
            iceServers: ICE_SERVERS
        });
        log("[createPeer] RTCPeerConnection object created");
        
        if (isOfferer) {
            log("[createPeer] Creating data channel (offerer side)");
            const dc = pc.createDataChannel("snaplan");
            log("[createPeer] Data channel created, state: " + dc.readyState);
            setupChannel(remoteId, dc);
        } else {
            log("[createPeer] Setting up ondatachannel handler (receiver side)");
            pc.ondatachannel = (e) => {
                log("[createPeer] ondatachannel event received from " + remoteId + ", channel state: " + e.channel.readyState);
                setupChannel(remoteId, e.channel);
                log("🟡 DataChannel received from " + remoteId);
            };
        }

        pc.onicecandidate = (e) => {
            if (e.candidate) {
                log("[createPeer] ICE candidate for " + remoteId);
                ws.send(JSON.stringify({
                    type: "IceCandidate",
                    from: myPeerId,
                    to: remoteId,
                    candidate: JSON.stringify(e.candidate)
                }));
            }
        };
        
        pc.onconnectionstatechange = () => {
            log("[createPeer: Connection state for " + remoteId + ": " + pc.connectionState);
            if(pc.connectionState === "failed" || pc.connectionState === "disconnected" ) {
                log("Connection to " + remoteId + " " + pc.connectionState + "- chec TURN server config if this persists")
            }
        };
        
        pc.oniceconnectionstatechange = () => {
            log("[createPeer] ICE connection state for " + remoteId + ": " + pc.iceConnectionState);
            if(pc.iceConnectionState === "failed") {
                log("ICE failed for " + remoteId + " - likely needs a TURN server (STUN alone isn't enough behind this NAT");
            }
        };
        
        peerConnections[remoteId] = pc;
        log("[createPeer] Peer connection stored for " + remoteId);
        return pc;
    } catch (e) {
        console.error("[createPeer] Failed to create peer connection:", e);
        log("❌ Failed to create peer connection: " + e.message);
        throw e;
    }
}

function setupChannel(remoteId, dc) {
    log("[setupChannel] Setting up channel for peer " + remoteId + ", current state: " + dc.readyState);
    dataChannels[remoteId] = dc;
    
    dc.onmessage = (e) => {
        log("[DC] Message from " + remoteId);
        handleData(e.data);
    };
    
    dc.onopen = () => {
        log("[setupChannel] DataChannel OPEN for " + remoteId);
        log("🟢 DataChannel open: " + remoteId);
        const queue = sendQueue[remoteId];
        if (queue && queue.length > 0) {
            log("[setupChannel] Flushing " + queue.length + " queued messages for " + remoteId);
            queue.forEach(m => {
                try {
                    dc.send(JSON.stringify(m));
                    log("[setupChannel] Sent queued message, type: " + m.type);
                } catch (e) {
                    console.error("[setupChannel] Error sending queued message: ", e);
                }
            });
            delete sendQueue[remoteId];
        }
    };
    
    dc.onerror = (e) => {
        console.error("[setupChannel] DataChannel ERROR for " + remoteId + ": " + e.message);
        log("❌ DC error: " + e.message);
    };
    
    dc.onclose = () => {
        log("[setupChannel] DataChannel CLOSED for " + remoteId);
        log("🔴 DC closed: " + remoteId);
        delete dataChannels[remoteId];
    };
}

async function connectToPeer(remoteId) {
    if (remoteId === myPeerId) {
        log("[connectToPeer] Refusing to connect to self: " + remoteId);
        return;
    }
    log("[connectToPeer] Starting connection to " + remoteId);
    const pc = createPeer(remoteId, true);
    try {
        const offer = await pc.createOffer();
        log("[connectToPeer] Offer created");
        await pc.setLocalDescription(offer);
        log("[connectToPeer] Local description set");
        ws.send(JSON.stringify({
            type: "Offer",
            from: myPeerId,
            to: remoteId,
            sdp: JSON.stringify(offer)
        }));
        log("[connectToPeer] Offer sent to " + remoteId);
        log("📨 Offer sent to " + remoteId);
    } catch (e) {
        console.error("[connectToPeer] Error: ", e);
        log("❌ Connection error: " + e.message);
    }
}

async function handleOffer(msg) {
    log("[handleOffer] Received offer from " + msg.from);
    const existing = peerConnections[msg.from];
    if (existing && existing.signalingState !== "stable") {
        if (myPeerId < msg.from) {
            log("[handleOffer] Glae detected, we win tie-break - ignoring incoming offer from " + msg.from)
            return;
        } else {
            log("[handleOffer] Glare detected, they win tie-brak - discarding our connection attempt to " + msg.from);
            cleanupPeer(msg.from);
        }
    }
    try {
        const pc = createPeer(msg.from, false);
        log("[handleOffer] Peer connection created");
        log("[handleOffer] Signaling state: " + pc.signalingState);
        await pc.setRemoteDescription(JSON.parse(msg.sdp));
        log("[handleOffer] Remote description set");
        const answer = await pc.createAnswer();
        log("[handleOffer] Answer created");
        await pc.setLocalDescription(answer);
        log("[handleOffer] Local description set");
        ws.send(JSON.stringify({
            type: "Answer",
            from: myPeerId,
            to: msg.from,
            sdp: JSON.stringify(answer)
        }));
        log("[handleOffer] Answer sent to " + msg.from);
        log("📨 Answer sent to " + msg.from);
    } catch (e) {
        console.error("[handleOffer] Error: ", e);
        log("❌ Offer handling error: " + e.message);
    }
}

async function handleAnswer(msg) {
    log("[handleAnswer] Received answer from " + msg.from);
    const pc = peerConnections[msg.from];
    if (!pc) {
        console.warn("[handleAnswer] No peer connection for " + msg.from);
        return;
    }
    if (pc.signalingState != "have-local-offer") {
        log("[handleAnswer] Ignoring duplicate answer, state: " + pc.signalingState);
        return;
    }
    try {
        await pc.setRemoteDescription(
            new RTCSessionDescription(JSON.parse(msg.sdp))
        );
        log("[handleAnswer] Remote description set for " + msg.from);
    } catch (e) {
        console.error("[handleAnswer] Error: ", e);
    }
}

async function handleICE(msg) {
    log("[handleICE] Received ICE candidate from " + msg.from);
    const pc = peerConnections[msg.from];
    if (!pc) {
        console.warn("[handleICE] No peer connection for " + msg.from);
        return;
    }
    try {
        await pc.addIceCandidate(JSON.parse(msg.candidate));
        log("[handleICE] ICE candidate added for " + msg.from);
    } catch (e) {
        console.error("[handleICE] Error: ", e);
    }
}

/* ---------------- FILE SHARING ---------- */
async function sendFile() {
    log("[sendFile] Starting file send");
    const file = document.getElementById("fileInput").files[0];
    if (!file) {
        console.warn("[sendFile] No file selected");
        log("❌ No file selected");
        return;
    }

    log("[sendFile] File selected: " + file.name + " (" + file.size + " bytes)");

    // Check if we have any connected peers
    const dcEntries = Object.entries(dataChannels);
    log("[sendFile] Total dataChannels: " + dcEntries.length);
    dcEntries.forEach(([id, dc]) => {
        log("[sendFile] DC " + id + " state: " + dc.readyState);
    });
    
    const peerCount = Object.values(dataChannels).filter(dc => dc.readyState === "open").length;
    log("[sendFile] Connected peers (open): " + peerCount);
    log("[sendFile] Peer IDs: " + Object.keys(dataChannels).join(", "));
    
    if (peerCount === 0) {
        console.warn("[sendFile] No peers connected");
        log("❌ No peers connected. Data channels exist: " + dcEntries.length + ", but none are 'open'");
        log("ℹ️ Waiting for data channels to open...");
        return;
    }

    const chunkSize = 16 * 1024; // 16KB
    const id = crypto.randomUUID();

    log("[sendFile] File transfer ID: " + id);
    log("[sendFile] Chunk size: " + chunkSize + " bytes");

    log("📤 Sending file: " + file.name + " (" + (file.size / 1024).toFixed(2) + " KB) to " + peerCount + " peer(s)");
    
    broadcastAllPeers({
        type: "file-meta",
        id,
        name: file.name,
        size: file.size,
    });
    
    let offset = 0;
    let index = 0;
    let sentChunks = 0;

    log("[sendFile] Starting chunk transmission");
    while (offset < file.size) {
        const slice = file.slice(offset, offset + chunkSize);
        const buffer = await slice.arrayBuffer();
        
        // Use a more robust way to convert buffer to base64
        let binary = "";
        const bytes = new Uint8Array(buffer);
        const len = bytes.byteLength;
        for (let i = 0; i < len; i++) {
            binary += String.fromCharCode(bytes[i]);
        }

        const msgData = btoa(binary);
        log("[sendFile] Chunk " + index + ": " + len + " bytes, base64: " + msgData.length + " chars");
        
        broadcastAllPeers({
            type: "file-chunk",
            id,
            index,
            data: msgData,
        });
        offset += chunkSize;
        index++;
        sentChunks++;
    }
    log("[sendFile] All chunks sent, sending file-end");
    broadcastAllPeers({
        type: "file-end",
        id
    });
    log("✅ File sent: " + sentChunks + " chunks");
    log("[sendFile] File transfer completed");
}

function broadcastAllPeers(msg) {
    log("[broadcastAllPeers] Broadcasting message type: " + msg.type);
    let sent = 0;
    let queued = 0;
    let failed = 0;
    
    Object.entries(dataChannels).forEach(([id, dc]) => {
        log("[broadcastAllPeers] Peer " + id + " - DC state: " + dc.readyState);
        if (dc.readyState !== "open") {
            if (!sendQueue[id]) {
                sendQueue[id] = [];
            }
            sendQueue[id].push(msg);
            queued++;
            log("[broadcastAllPeers] ⏳ QUEUED for " + id + " (state: " + dc.readyState + "), queue length: " + sendQueue[id].length);
            return;
        }
        try {
            const jsonStr = JSON.stringify(msg);
            log("[broadcastAllPeers] Sending to " + id + ": " + msg.type + " (" + jsonStr.length + " bytes)");
            dc.send(jsonStr);
            sent++;
            log("[broadcastAllPeers] ✅ SENT to " + id);
        } catch (e) {
            failed++;
            console.error("[broadcastAllPeers] ❌ ERROR sending to " + id + ": " + e.message);
        }
    });
    
    log("[broadcastAllPeers] Summary - " + msg.type + ": sent=" + sent + ", queued=" + queued + ", failed=" + failed);
}

function handleData(msg) {
    try {
        log("[handleData] Received data, parsing JSON");
        const data = JSON.parse(msg);
        log("[handleData] Message type: " + data.type);
        
        if (data.type === "file-meta") {
            log("[handleData] File meta - ID: " + data.id + ", Name: " + data.name + ", Size: " + data.size);
            incomingFiles[data.id] = {
                name: data.name,
                chunks: [],
                size: data.size,
            };
            log("📥 Receiving file: " + data.name + " (" + (data.size / 1024).toFixed(2) + " KB)");
            log("[handleData] File initialized for reception");
        }
        
        if (data.type === "file-chunk") {
            log("[handleData] Chunk received - ID: " + data.id + ", Index: " + data.index + ", Data length: " + (data.data ? data.data.length : "null"));
            if (!incomingFiles[data.id]) {
                console.warn("[handleData] Received chunk for unknown file: " + data.id);
                return;
            }
            incomingFiles[data.id].chunks[data.index] = data.data;
            log("[handleData] Chunk " + data.index + " stored for file " + data.id);
            if (data.index % 10 === 0) {
                log("[handleData] Progress: chunk " + data.index + " for file " + data.id + ", total chunks so far: " + Object.keys(incomingFiles[data.id].chunks).length);
            }
        }
        
        if (data.type === "file-end") {
            log("[handleData] File end signal - ID: " + data.id);
            const file = incomingFiles[data.id];
            if (!file) {
                console.warn("[handleData] Received file-end for unknown file: " + data.id);
                return;
            }
            
            const expectedChunks = Math.ceil(file.size / (16 * 1024));
            log("[handleData] Total chunks received: " + file.chunks.length + ", expected: " + expectedChunks);
            
            // Check for missing chunks
            const missingChunks = [];
            for (let i = 0; i < expectedChunks; i++) {
                if (!file.chunks[i]) {
                    missingChunks.push(i);
                }
            }
            
            if (missingChunks.length > 0) {
                console.warn("[handleData] Missing chunks: " + missingChunks.join(","));
                log("⚠️ File incomplete - missing " + missingChunks.length + " chunks");
                return;
            }
            
            log("[handleData] All chunks complete, reconstructing file");
            
            // Reconstruct binary from base64 chunks
            const byteArrays = file.chunks.map((chunk, idx) => {
                log("[handleData] Processing chunk " + idx + ": " + (chunk ? chunk.length : "null") + " bytes");
                const binary = atob(chunk);
                const bytes = new Uint8Array(binary.length);
                for (let i = 0; i < binary.length; i++) {
                    bytes[i] = binary.charCodeAt(i);
                }
                return bytes;
            });

            log("[handleData] Creating blob from " + byteArrays.length + " byte arrays");
            const blob = new Blob(byteArrays);
            log("[handleData] Blob created: " + blob.size + " bytes");
            
            const url = URL.createObjectURL(blob);
            log("[handleData] Object URL created");
            
            const a = document.createElement("a");
            a.href = url;
            a.download = file.name;
            log("[handleData] Triggering download for " + file.name);
            a.click();
            
            log("✅ File received: " + file.name);
            log("[handleData] File saved and download triggered");
            delete incomingFiles[data.id];
        }
    } catch (e) {
        console.error("[handleData] Error in handleData:", e);
        log("❌ Error receiving file: " + e.message);
    }
}
/* ---------------- PING ---------------- */

function ping() {
    if (!ws) return;

    ws.send(JSON.stringify({ type: "Ping" }));
}

/* ---------------- HEARTBEAT ---------------- */

setInterval(() => {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: "Heartbeat" }));
    }
}, 5000);

window.addEventListener('load', () => {
    setInterval(fetchDiscovery, 5000);
});

// Explicitly export functions for HTML button handlers
window.createSession = createSession;
window.fetchDiscovery = fetchDiscovery;
window.connectWs = connectWs;
window.ping = ping;
window.sendFile = sendFile;
window.startScanner = startScanner;
window.connectToPeer = connectToPeer;