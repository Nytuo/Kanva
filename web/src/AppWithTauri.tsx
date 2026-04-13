import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTauriEmbeddedServer } from '@/hooks/useTauri';
import App from './App';

/**
 * Wrapper that handles Tauri startup before rendering the main App.
 *
 * Desktop: waits for the embedded server to start, then lands on /servers.
 * Mobile:  ready immediately (no embedded server), also lands on /servers.
 * Web:     no-op — renders App directly.
 *
 * The user always sees the server selection screen first so they can choose
 * between the embedded server and any configured remote servers.
 */
export default function AppWithTauri() {
  const { isTauri, isReady, error } = useTauriEmbeddedServer();
  const navigate = useNavigate();

  // Once the Tauri layer is ready, drop the user on the server selection page.
  // This fires exactly once per app launch (isReady goes false→true once).
  useEffect(() => {
    if (isTauri && isReady) {
      navigate('/servers', { replace: true });
    }
  }, [isReady]); // eslint-disable-line react-hooks/exhaustive-deps

  // Show a loading screen while the embedded server is starting up.
  if (isTauri && !isReady && !error) {
    return (
      <div className="flex items-center justify-center h-screen bg-background">
        <div className="text-center">
          <div className="animate-spin h-10 w-10 border-4 border-primary border-t-transparent rounded-full mx-auto mb-4" />
          <h2 className="text-lg font-semibold mb-1">Starting Kanva</h2>
          <p className="text-sm text-muted-foreground">Initializing local server…</p>
        </div>
      </div>
    );
  }

  if (isTauri && error) {
    return (
      <div className="flex items-center justify-center h-screen bg-background">
        <div className="text-center max-w-md">
          <h2 className="text-lg font-semibold text-destructive mb-2">
            Failed to start local server
          </h2>
          <p className="text-sm text-muted-foreground mb-4">{error}</p>
          <p className="text-xs text-muted-foreground">
            You can still connect to a remote Kanva server.
          </p>
          <App />
        </div>
      </div>
    );
  }

  return <App />;
}
