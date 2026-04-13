import { useState } from 'react';
import {
  Plug,
  ExternalLink,
  RefreshCw,
  Trash2,
  Check,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog';
import { useIntegrations } from '@/hooks/useApi';
import { cn } from '@/lib/utils';

const providers = [
  {
    id: 'github' as const,
    name: 'GitHub',
    description: 'Sync issues and pull requests from GitHub repositories.',
    icon: (
      <svg viewBox="0 0 24 24" className="h-8 w-8" fill="currentColor">
        <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
      </svg>
    ),
    color: 'bg-gray-900 dark:bg-gray-100 dark:text-gray-900',
  },
  {
    id: 'gitlab' as const,
    name: 'GitLab',
    description: 'Sync issues and merge requests from GitLab projects.',
    icon: (
      <svg viewBox="0 0 24 24" className="h-8 w-8" fill="currentColor">
        <path d="M22.65 14.39L12 22.13 1.35 14.39a.84.84 0 01-.3-.94l1.22-3.78 2.44-7.51A.42.42 0 014.82 2a.43.43 0 01.58 0 .42.42 0 01.11.18l2.44 7.49h8.1l2.44-7.51A.42.42 0 0118.6 2a.43.43 0 01.58 0 .42.42 0 01.11.18l2.44 7.51L23 13.45a.84.84 0 01-.35.94z" />
      </svg>
    ),
    color: 'bg-orange-600',
  },
  {
    id: 'atlassian' as const,
    name: 'Atlassian (Jira)',
    description: 'Import and sync Jira issues to your Kanva boards.',
    icon: (
      <svg viewBox="0 0 24 24" className="h-8 w-8" fill="currentColor">
        <path d="M11.571 11.513H0a5.218 5.218 0 005.232 5.215h2.13v2.057A5.215 5.215 0 0012.577 24V12.518a1.005 1.005 0 00-1.006-1.005zM5.684 5.685H.452A5.215 5.215 0 005.667 10.9h2.13v2.06a5.218 5.218 0 005.215 5.214V6.69a1.005 1.005 0 00-1.006-1.005zm11.79-.122V17.05a1.005 1.005 0 001.006 1.005h.445A5.218 5.218 0 0024 12.84v-2.06h-2.13a5.218 5.218 0 00-4.396-5.217z" />
      </svg>
    ),
    color: 'bg-blue-600',
  },
];

export default function IntegrationsPage() {
  const { data: integrations = [] } = useIntegrations();
  const [showSetup, setShowSetup] = useState<string | null>(null);
  const [setupConfig, setSetupConfig] = useState({
    repository: '',
    project: '',
    api_token: '',
    base_url: '',
  });

  const selectedProvider = providers.find((p) => p.id === showSetup);

  const handleSetup = () => {
    // Would call create integration API
    setShowSetup(null);
    setSetupConfig({ repository: '', project: '', api_token: '', base_url: '' });
  };

  return (
    <div className="p-6 max-w-5xl mx-auto">
      <div className="mb-6">
        <h1 className="text-2xl font-bold flex items-center gap-2">
          <Plug className="h-6 w-6" />
          Integrations
        </h1>
        <p className="text-muted-foreground mt-1">
          Connect your boards with external services to sync issues, PRs, and more.
        </p>
      </div>

      {/* Available integrations */}
      <section className="mb-8">
        <h2 className="text-lg font-semibold mb-4">Available Integrations</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {providers.map((provider) => {
            const connected = (integrations as { provider: string }[]).some(
              (i) => i.provider === provider.id,
            );

            return (
              <Card key={provider.id} className="overflow-hidden">
                <CardContent className="p-6">
                  <div className="flex items-start gap-4">
                    <div
                      className={cn(
                        'h-12 w-12 rounded-lg flex items-center justify-center text-white flex-shrink-0',
                        provider.color,
                      )}
                    >
                      {provider.icon}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <h3 className="font-semibold">{provider.name}</h3>
                        {connected && (
                          <span className="flex items-center gap-1 text-xs text-green-600 bg-green-100 px-2 py-0.5 rounded-full">
                            <Check className="h-3 w-3" />
                            Connected
                          </span>
                        )}
                      </div>
                      <p className="text-sm text-muted-foreground mt-1">
                        {provider.description}
                      </p>
                      <Button
                        size="sm"
                        variant={connected ? 'outline' : 'default'}
                        className="mt-3"
                        onClick={() => setShowSetup(provider.id)}
                      >
                        {connected ? 'Configure' : 'Connect'}
                        <ExternalLink className="h-3 w-3 ml-1" />
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      </section>

      {/* Active integrations list */}
      {(integrations as { id: string; provider: string; enabled: boolean }[]).length > 0 && (
        <section>
          <h2 className="text-lg font-semibold mb-4">Active Integrations</h2>
          <div className="space-y-3">
            {(integrations as { id: string; provider: string; enabled: boolean; board_id: string }[]).map(
              (integration) => {
                const provider = providers.find(
                  (p) => p.id === integration.provider,
                );
                return (
                  <Card key={integration.id}>
                    <CardContent className="p-4 flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <div
                          className={cn(
                            'h-8 w-8 rounded flex items-center justify-center text-white',
                            provider?.color || 'bg-gray-500',
                          )}
                        >
                          {provider?.icon}
                        </div>
                        <div>
                          <p className="font-medium">
                            {provider?.name || integration.provider}
                          </p>
                          <p className="text-xs text-muted-foreground">
                            Board: {integration.board_id}
                          </p>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <span
                          className={cn(
                            'text-xs px-2 py-0.5 rounded-full',
                            integration.enabled
                              ? 'bg-green-100 text-green-700'
                              : 'bg-red-100 text-red-700',
                          )}
                        >
                          {integration.enabled ? 'Enabled' : 'Disabled'}
                        </span>
                        <Button variant="ghost" size="icon" className="h-8 w-8">
                          <RefreshCw className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-destructive"
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                );
              },
            )}
          </div>
        </section>
      )}

      {/* Setup dialog */}
      <Dialog open={!!showSetup} onOpenChange={() => setShowSetup(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              Connect {selectedProvider?.name}
            </DialogTitle>
            <DialogDescription>
              Enter the configuration to connect your {selectedProvider?.name} account.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            {showSetup === 'github' && (
              <div>
                <label className="text-sm font-medium">Repository (owner/repo)</label>
                <Input
                  value={setupConfig.repository}
                  onChange={(e) =>
                    setSetupConfig({ ...setupConfig, repository: e.target.value })
                  }
                  placeholder="e.g., octocat/hello-world"
                />
              </div>
            )}
            {showSetup === 'gitlab' && (
              <>
                <div>
                  <label className="text-sm font-medium">Project Path</label>
                  <Input
                    value={setupConfig.project}
                    onChange={(e) =>
                      setSetupConfig({ ...setupConfig, project: e.target.value })
                    }
                    placeholder="e.g., group/project"
                  />
                </div>
                <div>
                  <label className="text-sm font-medium">GitLab URL (optional)</label>
                  <Input
                    value={setupConfig.base_url}
                    onChange={(e) =>
                      setSetupConfig({ ...setupConfig, base_url: e.target.value })
                    }
                    placeholder="https://gitlab.com"
                  />
                </div>
              </>
            )}
            {showSetup === 'atlassian' && (
              <>
                <div>
                  <label className="text-sm font-medium">Jira Project Key</label>
                  <Input
                    value={setupConfig.project}
                    onChange={(e) =>
                      setSetupConfig({ ...setupConfig, project: e.target.value })
                    }
                    placeholder="e.g., PROJ"
                  />
                </div>
                <div>
                  <label className="text-sm font-medium">Atlassian Domain</label>
                  <Input
                    value={setupConfig.base_url}
                    onChange={(e) =>
                      setSetupConfig({ ...setupConfig, base_url: e.target.value })
                    }
                    placeholder="e.g., your-domain.atlassian.net"
                  />
                </div>
              </>
            )}
            <div>
              <label className="text-sm font-medium">API Token / Access Token</label>
              <Input
                type="password"
                value={setupConfig.api_token}
                onChange={(e) =>
                  setSetupConfig({ ...setupConfig, api_token: e.target.value })
                }
                placeholder="Enter your access token"
              />
            </div>
            <Button onClick={handleSetup} className="w-full">
              Connect {selectedProvider?.name}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
