import { useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useAuthStore } from '@/store/auth';
import { api } from '@/lib/api';

export default function OAuthCallbackPage() {
  const { provider } = useParams();
  const navigate = useNavigate();
  const setTokens = useAuthStore((s) => s.setTokens);
  const setUser = useAuthStore((s) => s.setUser);

  useEffect(() => {
    const searchParams = new URLSearchParams(window.location.search);
    const code = searchParams.get('code');

    if (!code || !provider) {
      navigate('/login');
      return;
    }

    api.get(`/auth/oauth/${provider}/callback`, { params: { code } })
      .then((response) => {
        const { access_token, refresh_token, user } = response.data;
        setTokens(access_token, refresh_token);
        setUser(user);
        navigate('/');
      })
      .catch(() => {
        navigate('/login');
      });
  }, [provider, navigate, setTokens, setUser]);

  return (
    <div className="flex min-h-screen items-center justify-center">
      <div className="text-center">
        <div className="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full mx-auto mb-4" />
        <p className="text-muted-foreground">Completing sign in...</p>
      </div>
    </div>
  );
}
