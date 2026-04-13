import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface ServerConnection {
  /** Unique id for this saved server */
  id: string;
  /** Display label chosen by user */
  name: string;
  /** Base URL, e.g. "https://kanva.example.com" or "http://127.0.0.1:9321" */
  url: string;
  /** Whether this is the embedded local server (auto-managed) */
  isEmbedded: boolean;
  /** Server capabilities, fetched from /api/server-info */
  serverInfo?: ServerInfo;
}

export interface ServerInfo {
  version: string;
  standalone: boolean;
  teams_enabled: boolean;
}

interface ServerState {
  /** All saved server connections */
  servers: ServerConnection[];
  /** Currently active server id */
  activeServerId: string | null;

  /** Add a new server connection */
  addServer: (server: Omit<ServerConnection, 'id'>) => string;
  /** Remove a saved server */
  removeServer: (id: string) => void;
  /** Update an existing server (e.g. after fetching server-info) */
  updateServer: (id: string, updates: Partial<ServerConnection>) => void;
  /** Switch the active server */
  setActiveServer: (id: string | null) => void;

  /** Get the currently active server */
  getActiveServer: () => ServerConnection | null;

  /** Whether the active server has teams enabled */
  isTeamsEnabled: () => boolean;
  /** Whether the active server is the embedded standalone */
  isStandalone: () => boolean;
}

let counter = 0;
function generateId(): string {
  return `srv_${Date.now()}_${++counter}`;
}

export const useServerStore = create<ServerState>()(
  persist(
    (set, get) => ({
      servers: [],
      activeServerId: null,

      addServer: (server) => {
        const id = generateId();
        set((state) => ({
          servers: [...state.servers, { ...server, id }],
        }));
        return id;
      },

      removeServer: (id) => {
        set((state) => ({
          servers: state.servers.filter((s) => s.id !== id),
          activeServerId: state.activeServerId === id ? null : state.activeServerId,
        }));
      },

      updateServer: (id, updates) => {
        set((state) => ({
          servers: state.servers.map((s) =>
            s.id === id ? { ...s, ...updates } : s,
          ),
        }));
      },

      setActiveServer: (id) => {
        set({ activeServerId: id });
      },

      getActiveServer: () => {
        const { servers, activeServerId } = get();
        return servers.find((s) => s.id === activeServerId) ?? null;
      },

      isTeamsEnabled: () => {
        const server = get().getActiveServer();
        if (!server) return false;
        // If we haven't fetched server info yet, assume teams enabled for remote servers
        if (!server.serverInfo) return !server.isEmbedded;
        return server.serverInfo.teams_enabled;
      },

      isStandalone: () => {
        const server = get().getActiveServer();
        if (!server) return false;
        return server.isEmbedded || (server.serverInfo?.standalone ?? false);
      },
    }),
    {
      name: 'kanva-servers',
      partialize: (state) => ({
        servers: state.servers,
        activeServerId: state.activeServerId,
      }),
    },
  ),
);
