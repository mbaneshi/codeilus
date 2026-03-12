import type { CodeilusEvent } from '$lib/types';

let events = $state<CodeilusEvent[]>([]);
let connected = $state(false);
let ws: WebSocket | null = null;

export function connectWebSocket() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  ws = new WebSocket(`${protocol}//${window.location.host}/api/v1/ws`);

  ws.onopen = () => { connected = true; };
  ws.onclose = () => {
    connected = false;
    // Auto-reconnect after 2 seconds
    setTimeout(connectWebSocket, 2000);
  };
  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data) as CodeilusEvent;
      events = [...events.slice(-99), data];  // keep last 100
    } catch { /* ignore malformed */ }
  };
}

export function getEvents() { return events; }
export function isConnected() { return connected; }
