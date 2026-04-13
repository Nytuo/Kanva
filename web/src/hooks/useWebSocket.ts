import { useEffect, useRef, useCallback } from 'react';
import { useAuthStore } from '@/store/auth';
import { useServerStore } from '@/store/server';

interface WsMessage {
  event: string;
  board_id: string;
  data: unknown;
  user_id?: string;
}

export function useWebSocket(boardId: string | undefined, onMessage: (msg: WsMessage) => void) {
  const ws = useRef<WebSocket | null>(null);
  const token = useAuthStore((s) => s.accessToken);
  const activeServer = useServerStore((s) => s.getActiveServer());

  const send = useCallback((message: Partial<WsMessage>) => {
    if (ws.current?.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify(message));
    }
  }, []);

  useEffect(() => {
    if (!boardId || !token || !activeServer) return;

    // Derive WebSocket URL from the server's HTTP URL
    let wsUrl: string;
    if (import.meta.env.VITE_WS_URL) {
      wsUrl = import.meta.env.VITE_WS_URL;
    } else {
      // Convert http(s) to ws(s)
      const serverUrl = activeServer.url.replace(/\/+$/, '');
      wsUrl = serverUrl.replace(/^http/, 'ws');
    }

    const socket = new WebSocket(`${wsUrl}/ws?token=${token}&board_id=${boardId}`);

    socket.onopen = () => {
      console.log('WebSocket connected to', activeServer.name);
    };

    socket.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data) as WsMessage;
        onMessage(msg);
      } catch (e) {
        console.error('WS parse error:', e);
      }
    };

    socket.onclose = () => {
      console.log('WebSocket disconnected');
    };

    ws.current = socket;

    return () => {
      socket.close();
    };
  }, [boardId, token, activeServer?.url, onMessage]);

  return { send };
}
