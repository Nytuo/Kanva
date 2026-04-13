import axios, { type AxiosInstance } from 'axios';
import { useAuthStore, getAccessToken, getRefreshToken } from '@/store/auth';
import { useServerStore } from '@/store/server';

/**
 * Dynamic API client that routes to the currently active server.
 *
 * The base URL is resolved on every request from the server store,
 * so switching servers takes effect immediately.
 */
function createApiClient(): AxiosInstance {
  const instance = axios.create({
    headers: {
      'Content-Type': 'application/json',
    },
  });

  // Resolve baseURL dynamically before each request
  instance.interceptors.request.use((config) => {
    const server = useServerStore.getState().getActiveServer();
    if (server) {
      // Strip trailing slash if present
      const base = server.url.replace(/\/+$/, '');
      config.baseURL = `${base}/api`;
    } else {
      // Fallback: env variable or same-origin
      const fallback = import.meta.env.VITE_API_URL || '';
      config.baseURL = `${fallback}/api`;
    }

    // Attach auth token — use helper that reads through credentials map directly,
    // bypassing the getter which is lost after Zustand persist rehydration.
    const token = getAccessToken();
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }

    return config;
  });

  // Response interceptor for token refresh
  instance.interceptors.response.use(
    (response) => response,
    async (error) => {
      const originalRequest = error.config;

      // Only attempt refresh on 401s from authenticated requests (i.e. requests
      // that already had an Authorization header). Auth endpoints like /auth/login
      // return 401 for bad credentials — we must NOT intercept those.
      const hadAuthHeader = !!originalRequest?.headers?.Authorization;

      if (
        error.response?.status === 401 &&
        hadAuthHeader &&
        !originalRequest._retry
      ) {
        originalRequest._retry = true;

        try {
          const refreshToken = getRefreshToken();
          if (!refreshToken) throw new Error('No refresh token');

          // Use the same base URL that was resolved for the original request
          const baseURL = originalRequest.baseURL || '';
          const response = await axios.post(`${baseURL}/auth/refresh`, {
            refresh_token: refreshToken,
          });

          const { access_token, refresh_token } = response.data;
          useAuthStore.getState().setTokens(access_token, refresh_token);

          originalRequest.headers.Authorization = `Bearer ${access_token}`;
          return instance(originalRequest);
        } catch {
          useAuthStore.getState().logout();
          window.location.href = '/login';
        }
      }

      return Promise.reject(error);
    },
  );

  return instance;
}

export const api = createApiClient();

/**
 * Probe a server URL to check connectivity and fetch capabilities.
 * Used by the server selector UI before saving a connection.
 */
export async function probeServer(url: string): Promise<{
  ok: boolean;
  serverInfo?: { version: string; standalone: boolean; teams_enabled: boolean };
  error?: string;
}> {
  try {
    const base = url.replace(/\/+$/, '');

    // First check health
    await axios.get(`${base}/health`, { timeout: 5000 });

    // Then fetch server info
    const infoRes = await axios.get(`${base}/api/server-info`, { timeout: 5000 });

    return { ok: true, serverInfo: infoRes.data };
  } catch (err: unknown) {
    const message =
      err instanceof Error ? err.message : 'Could not connect to server';
    return { ok: false, error: message };
  }
}
