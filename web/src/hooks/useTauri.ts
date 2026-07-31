import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useServerStore } from '@/store/server';
import { useAuthStore } from '@/store/auth';
import { probeServer } from '@/lib/api';

// Standalone local account constants
const LOCAL_EMAIL = 'local@kanva.local';
const LOCAL_USERNAME = 'localuser';
const LOCAL_DISPLAY_NAME = 'Local User';
const LOCAL_PASSWORD_KEY = 'kanva_local_password'; // NOSONAR — localStorage key, not a credential

function getOrCreateLocalPassword(): string {
  let pw = localStorage.getItem(LOCAL_PASSWORD_KEY);
  if (!pw) {
    // Generate a random 24-char password once and store it
    pw = Array.from(crypto.getRandomValues(new Uint8Array(18)))
      .map((b) => b.toString(16).padStart(2, '0'))
      .join('')
      .slice(0, 24);
    localStorage.setItem(LOCAL_PASSWORD_KEY, pw);
  }
  return pw;
}

/**
 * Detect if running inside Tauri v2 (desktop or mobile app).
 */
export function isTauri(): boolean {
  return globalThis.window !== undefined && '__TAURI_INTERNALS__' in globalThis;
}

/**
 * Persist the active remote server URL to the native store so it survives
 * a full reinstall (localStorage is wiped; the native store is not).
 * Only has an effect on mobile Tauri.
 */
export async function persistRemoteServerUrl(url: string): Promise<void> {
  if (!isTauri()) return;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const isDesktop = await invoke<boolean>('is_desktop');
    if (!isDesktop) {
      await invoke('set_remote_server_url', { url });
    }
  } catch {
    // Best-effort — localStorage still has the data
  }
}

/**
 * Hook that manages server initialisation for both Tauri targets.
 *
 * Desktop Tauri:
 *   Listens for `embedded-server-ready`, registers the local server, auto-selects it.
 *
 * Mobile Tauri:
 *   Listens for `remote-server-ready` (fired by the Rust layer when a previously
 *   saved URL is found in the native store), registers and auto-selects that server.
 *   Falls through to ready immediately so the user can add a server manually.
 *
 * Web (non-Tauri):
 *   No-op.
 */
export function useTauriEmbeddedServer() {
  const [isReady, setIsReady] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { servers, addServer, updateServer, setActiveServer } = useServerStore();
  const { syncActiveServer } = useAuthStore();

  useEffect(() => {
    if (!isTauri()) return;

    let cleanup: (() => void) | undefined;

    async function init() {
      try {
        // Dynamic import of Tauri APIs (only available in Tauri runtime)
        const { invoke } = await import('@tauri-apps/api/core');
        const { listen } = await import('@tauri-apps/api/event');

        const desktop = await invoke<boolean>('is_desktop');

        if (!desktop) {
          // ── Mobile Tauri ──────────────────────────────────────────────────
          // Always mark ready so the app renders (server list is in localStorage).
          setIsReady(true);

          // If the Rust layer found a saved URL in the native store it will fire
          // `remote-server-ready`; auto-register it in case localStorage was wiped.
          const unlisten = await listen<{ url: string }>(
            'remote-server-ready',
            (event) => {
              registerRemoteServer(event.payload.url);
            },
          );

          cleanup = () => unlisten();
          return;
        }

        // ── Desktop Tauri ───────────────────────────────────────────────────
        // If the user already has an active remote server, let the app render
        // immediately — the embedded server still starts and registers itself
        // in the background but won't override the user's server selection.
        const { activeServerId, servers } = useServerStore.getState();
        const activeIsRemote =
          activeServerId !== null &&
          servers.some((s) => s.id === activeServerId && !s.isEmbedded);
        if (activeIsRemote) {
          setIsReady(true);
        }

        // Check if embedded server already started (e.g., hot reload)
        const existingUrl = await invoke<string | null>('get_embedded_server_url');
        if (existingUrl) {
          registerEmbeddedServer(existingUrl);
          return;
        }

        // Listen for the embedded server to become ready
        const unlisten = await listen<{ url: string; port: number }>(
          'embedded-server-ready',
          (event) => {
            registerEmbeddedServer(event.payload.url);
          },
        );

        const unlistenError = await listen<string>(
          'embedded-server-error',
          (event) => {
            setError(event.payload);
          },
        );

        cleanup = () => {
          unlisten();
          unlistenError();
        };
      } catch (e) {
        console.error('Failed to initialize Tauri integration:', e);
      }
    }

    // Desktop: register/update the embedded local server entry.
    // Never auto-selects the embedded server — the user picks from the server
    // selection page instead.
    function registerEmbeddedServer(url: string) {
      const existing = servers.find((s) => s.isEmbedded);

      if (existing) {
        updateServer(existing.id, { url });
        probeServer(url).then((result) => {
          if (result.ok) updateServer(existing.id, { serverInfo: result.serverInfo });
        });
      } else {
        const id = addServer({ name: 'Local (Standalone)', url, isEmbedded: true });
        probeServer(url).then((result) => {
          if (result.ok) updateServer(id, { serverInfo: result.serverInfo });
        });
      }

      setIsReady(true);
    }

    // Mobile: register a remote server that was restored from the native store.
    function registerRemoteServer(url: string) {
      const existing = servers.find((s) => s.url === url);

      if (existing) {
        probeServer(url).then((result) => {
          if (result.ok) updateServer(existing.id, { serverInfo: result.serverInfo });
        });
        const { activeServerId } = useServerStore.getState();
        if (!activeServerId) {
          setActiveServer(existing.id);
          syncActiveServer(existing.id);
        }
      } else {
        const id = addServer({ name: 'Kanva Server', url, isEmbedded: false });
        probeServer(url).then((result) => {
          if (result.ok) updateServer(id, { serverInfo: result.serverInfo });
        });
        const { activeServerId } = useServerStore.getState();
        if (!activeServerId) {
          setActiveServer(id);
          syncActiveServer(id);
        }
      }
    }

    init();

    return () => {
      cleanup?.();
    };
  }, []); // Run once on mount

  return { isTauri: isTauri(), isReady, error };
}

/**
 * Hook that auto-registers and logs in a local user when running in standalone
 * desktop Tauri mode. Navigates to "/" on success.
 *
 * On first launch: creates the local account (register).
 * On subsequent launches: logs in with the stored password.
 */
export function useStandaloneAutoLogin(isEmbeddedReady: boolean) {
  const navigate = useNavigate();
  const activeServerId = useServerStore((s) => s.activeServerId);
  const embeddedServer = useServerStore((s) => s.servers.find((srv) => srv.isEmbedded));
  const isAuthenticated = useAuthStore((s) => !!s.credentials[s.activeServerId ?? '']?.accessToken);
  const { login, register } = useAuthStore.getState();

  useEffect(() => {
    if (!isTauri()) return;
    if (!isEmbeddedReady) return;
    if (!activeServerId || activeServerId !== embeddedServer?.id) return;
    if (isAuthenticated) {
      navigate('/', { replace: true });
      return;
    }

    const password = getOrCreateLocalPassword();

    async function autoLogin() {
      // Even if a token is already stored, verify it against the current
      // embedded server instance. The server persists its JWT secret across
      // restarts (see desktop/src-tauri/src/lib.rs), but on a first launch
      // or after a data-dir wipe the secret changes and the stored token
      // becomes invalid. Probing /auth/me is the cheapest way to check.
      if (isAuthenticated) {
        try {
          await useAuthStore.getState().fetchMe();
          navigate('/', { replace: true });
          return;
        } catch {
          // Token rejected — fall through to re-login below
        }
      }

      try {
        // Try to register first (first launch)
        await register(LOCAL_EMAIL, LOCAL_USERNAME, LOCAL_DISPLAY_NAME, password);
      } catch {
        // User already exists — just log in
        try {
          await login(LOCAL_EMAIL, password);
        } catch (loginErr) {
          console.error('Standalone auto-login failed:', loginErr);
          return;
        }
      }
      navigate('/', { replace: true });
    }

    autoLogin();
  }, [isEmbeddedReady, activeServerId, isAuthenticated]);
}
