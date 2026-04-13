import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { User } from '@/types';
import { api } from '@/lib/api';

/**
 * Auth credentials stored per server.
 * Key is the server connection id.
 */
interface PerServerAuth {
  user: User | null;
  accessToken: string | null;
  refreshToken: string | null;
}

interface AuthState {
  /** Credentials keyed by server id */
  credentials: Record<string, PerServerAuth>;

  /** Currently active server id (mirrors server store) */
  activeServerId: string | null;

  isLoading: boolean;

  // Derived getters
  user: User | null;
  accessToken: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;

  /** Set which server is active (called when server store changes) */
  syncActiveServer: (serverId: string | null) => void;

  login: (email: string, password: string) => Promise<void>;
  register: (email: string, username: string, displayName: string, password: string) => Promise<void>;
  logout: () => void;
  setTokens: (accessToken: string, refreshToken: string) => void;
  setUser: (user: User) => void;
  fetchMe: () => Promise<void>;

  /** Clear credentials for a specific server */
  clearServerAuth: (serverId: string) => void;
}

function getActive(state: { credentials: Record<string, PerServerAuth>; activeServerId: string | null }): PerServerAuth {
  if (!state.activeServerId) return { user: null, accessToken: null, refreshToken: null };
  return state.credentials[state.activeServerId] ?? { user: null, accessToken: null, refreshToken: null };
}

/** Read the access token from raw Zustand state (safe after rehydration). */
export function getAccessToken(): string | null {
  return getActive(useAuthStore.getState()).accessToken;
}

/** Read the refresh token from raw Zustand state (safe after rehydration). */
export function getRefreshToken(): string | null {
  return getActive(useAuthStore.getState()).refreshToken;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => {
      function currentCreds() {
        return getActive(get());
      }

      function updateCreds(updates: Partial<PerServerAuth>) {
        const { activeServerId, credentials } = get();
        if (!activeServerId) return;
        const existing = credentials[activeServerId] ?? { user: null, accessToken: null, refreshToken: null };
        set({
          credentials: {
            ...credentials,
            [activeServerId]: { ...existing, ...updates },
          },
        });
      }

      return {
        credentials: {},
        activeServerId: null,
        isLoading: false,

        get user() { return currentCreds().user; },
        get accessToken() { return currentCreds().accessToken; },
        get refreshToken() { return currentCreds().refreshToken; },
        get isAuthenticated() { return !!currentCreds().accessToken; },

        syncActiveServer: (serverId) => {
          set({ activeServerId: serverId });
        },

        login: async (email: string, password: string) => {
          set({ isLoading: true });
          try {
            const response = await api.post('/auth/login', { email, password });
            const { access_token, refresh_token, user } = response.data;
            updateCreds({
              user,
              accessToken: access_token,
              refreshToken: refresh_token,
            });
            set({ isLoading: false });
          } catch (error) {
            set({ isLoading: false });
            throw error;
          }
        },

        register: async (email, username, displayName, password) => {
          set({ isLoading: true });
          try {
            const response = await api.post('/auth/register', {
              email,
              username,
              display_name: displayName,
              password,
            });
            const { access_token, refresh_token, user } = response.data;
            updateCreds({
              user,
              accessToken: access_token,
              refreshToken: refresh_token,
            });
            set({ isLoading: false });
          } catch (error) {
            set({ isLoading: false });
            throw error;
          }
        },

        logout: () => {
          api.post('/auth/logout').catch(() => {});
          updateCreds({
            user: null,
            accessToken: null,
            refreshToken: null,
          });
        },

        setTokens: (accessToken, refreshToken) => {
          updateCreds({ accessToken, refreshToken });
        },

        setUser: (user) => {
          updateCreds({ user });
        },

        fetchMe: async () => {
          try {
            const response = await api.get('/auth/me');
            updateCreds({ user: response.data });
          } catch {
            get().logout();
          }
        },

        clearServerAuth: (serverId: string) => {
          const { credentials } = get();
          const next = { ...credentials };
          delete next[serverId];
          set({ credentials: next });
        },
      };
    },
    {
      name: 'kanva-auth',
      partialize: (state) => ({
        credentials: state.credentials,
        activeServerId: state.activeServerId,
      }),
    },
  ),
);
