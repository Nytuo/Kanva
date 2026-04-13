import { useEffect } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { useAuthStore } from '@/store/auth';
import { useServerStore } from '@/store/server';
import { ThemeProvider } from '@/components/layout/ThemeProvider';
import { ToastProvider, Toaster } from '@/components/ui/toaster';
import { ErrorBoundary } from '@/components/ErrorBoundary';
import Layout from '@/components/layout/Layout';
import ServerSelectPage from '@/pages/ServerSelectPage';
import LoginPage from '@/pages/LoginPage';
import RegisterPage from '@/pages/RegisterPage';
import DashboardPage from '@/pages/DashboardPage';
import BoardPage from '@/pages/BoardPage';
import CalendarPage from '@/pages/CalendarPage';
import TeamsPage from '@/pages/TeamsPage';
import TeamDetailPage from '@/pages/TeamDetailPage';
import SettingsPage from '@/pages/SettingsPage';
import OAuthCallbackPage from '@/pages/OAuthCallbackPage';

/**
 * Requires an active server connection.
 * Redirects to server select if none is active.
 */
function ServerRequired({ children }: { children: React.ReactNode }) {
  const activeServerId = useServerStore((s) => s.activeServerId);
  if (!activeServerId) return <Navigate to="/servers" replace />;
  return <>{children}</>;
}

/**
 * Requires authentication on the active server.
 */
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const activeServerId = useServerStore((s) => s.activeServerId);
  const isAuthenticated = useAuthStore((s) => {
    if (!s.activeServerId) return false;
    const creds = s.credentials[s.activeServerId];
    return !!creds?.accessToken;
  });

  if (!activeServerId) return <Navigate to="/servers" replace />;
  if (!isAuthenticated) return <Navigate to="/login" replace />;
  return <>{children}</>;
}

/**
 * Guest routes (login/register) - redirect to dashboard if already authenticated.
 */
function GuestRoute({ children }: { children: React.ReactNode }) {
  const activeServerId = useServerStore((s) => s.activeServerId);
  const isAuthenticated = useAuthStore((s) => {
    if (!s.activeServerId) return false;
    const creds = s.credentials[s.activeServerId];
    return !!creds?.accessToken;
  });

  if (!activeServerId) return <Navigate to="/servers" replace />;
  if (isAuthenticated) return <Navigate to="/" replace />;
  return <>{children}</>;
}

/**
 * Routes that require teams to be enabled.
 * Falls back to dashboard in standalone mode.
 */
function TeamsRoute({ children }: { children: React.ReactNode }) {
  const teamsEnabled = useServerStore((s) => s.isTeamsEnabled());
  if (!teamsEnabled) return <Navigate to="/" replace />;
  return <>{children}</>;
}

function ServerSelectOrLoading() {
  return <ServerSelectPage />;
}

export default function App() {
  const activeServerId = useServerStore((s) => s.activeServerId);
  const syncActiveServer = useAuthStore((s) => s.syncActiveServer);

  // Keep auth store in sync with server store
  useEffect(() => {
    syncActiveServer(activeServerId);
  }, [activeServerId, syncActiveServer]);

  return (
    <ThemeProvider>
      <ToastProvider>
      <ErrorBoundary>
      <Routes>
        {/* Server selection - always accessible */}
        <Route path="/servers" element={<ServerSelectOrLoading />} />

        {/* Auth routes - require server, guest only */}
        <Route
          path="/login"
          element={
            <ServerRequired>
              <GuestRoute>
                <LoginPage />
              </GuestRoute>
            </ServerRequired>
          }
        />
        <Route
          path="/register"
          element={
            <ServerRequired>
              <GuestRoute>
                <RegisterPage />
              </GuestRoute>
            </ServerRequired>
          }
        />
        <Route
          path="/auth/callback/:provider"
          element={
            <ServerRequired>
              <OAuthCallbackPage />
            </ServerRequired>
          }
        />

        {/* Main app - require server + auth */}
        <Route
          path="/"
          element={
            <ProtectedRoute>
              <ErrorBoundary>
                <Layout />
              </ErrorBoundary>
            </ProtectedRoute>
          }
        >
          <Route index element={<DashboardPage />} />
          <Route path="board/:boardId" element={<BoardPage />} />
          <Route path="calendar" element={<CalendarPage />} />
          <Route
            path="teams"
            element={
              <TeamsRoute>
                <TeamsPage />
              </TeamsRoute>
            }
          />
          <Route
            path="teams/:teamId"
            element={
              <TeamsRoute>
                <TeamDetailPage />
              </TeamsRoute>
            }
          />
          <Route path="integrations" element={<Navigate to="/" replace />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>

        {/* Fallback */}
        <Route path="*" element={<Navigate to="/servers" replace />} />
      </Routes>
      </ErrorBoundary>
      <Toaster />
      </ToastProvider>
    </ThemeProvider>
  );
}
