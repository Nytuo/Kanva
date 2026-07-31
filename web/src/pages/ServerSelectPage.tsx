import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Plus,
  Trash2,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Wifi,
  HardDrive,
  ArrowRight,
  Pencil,
  Globe,
  Shield,
  Users,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card, CardContent } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog';
import { useServerStore, type ServerConnection, type ServerInfo } from '@/store/server';
import { useAuthStore } from '@/store/auth';
import { probeServer } from '@/lib/api';
import { cn } from '@/lib/utils';

interface ProbeResult {
  status: 'idle' | 'loading' | 'success' | 'error';
  serverInfo?: ServerInfo;
  error?: string;
}

export default function ServerSelectPage() {
  const { servers, activeServerId, addServer, removeServer, updateServer, setActiveServer } =
    useServerStore();
  const { syncActiveServer } = useAuthStore();
  const navigate = useNavigate();

  const [showAdd, setShowAdd] = useState(false);
  const [newName, setNewName] = useState('');
  const [newUrl, setNewUrl] = useState('');
  const [probe, setProbe] = useState<ProbeResult>({ status: 'idle' });
  const [editingId, setEditingId] = useState<string | null>(null);

  const handleProbe = async (url: string) => {
    setProbe({ status: 'loading' });
    const result = await probeServer(url);
    if (result.ok) {
      setProbe({ status: 'success', serverInfo: result.serverInfo });
    } else {
      setProbe({ status: 'error', error: result.error });
    }
  };

  const handleAddServer = () => {
    if (!newName.trim() || !newUrl.trim()) return;

    addServer({
      name: newName.trim(),
      url: newUrl.trim().replace(/\/+$/, ''),
      isEmbedded: false,
      serverInfo: probe.serverInfo,
    });

    setShowAdd(false);
    resetForm();
  };

  const handleUpdateServer = (id: string) => {
    if (!newName.trim() || !newUrl.trim()) return;
    updateServer(id, {
      name: newName.trim(),
      url: newUrl.trim().replace(/\/+$/, ''),
      serverInfo: probe.serverInfo,
    });
    setEditingId(null);
    resetForm();
  };

  const resetForm = () => {
    setNewName('');
    setNewUrl('');
    setProbe({ status: 'idle' });
  };

  const handleConnect = (server: ServerConnection) => {
    setActiveServer(server.id);
    syncActiveServer(server.id);
    navigate('/login');
  };

  const handleRemove = (id: string) => {
    if (window.confirm('Remove this server connection?')) {
      removeServer(id);
    }
  };

  const handleEditClick = (server: ServerConnection) => {
    setNewName(server.name);
    setNewUrl(server.url);
    setProbe(
      server.serverInfo
        ? { status: 'success', serverInfo: server.serverInfo }
        : { status: 'idle' },
    );
    setEditingId(server.id);
    setShowAdd(true);
  };

  const isEditing = editingId !== null;

  const embeddedServer = servers.find((s) => s.isEmbedded);

  return (
    <div className="min-h-screen flex items-center justify-center bg-background p-4">
      <div className="w-full max-w-lg">
        {/* Header */}
        <div className="text-center mb-8">
          <div className="flex items-center justify-center gap-2 mb-3">
            <img src="/logo.png" alt="Kanva" className="h-10 w-10 rounded-lg" />
            <h1 className="text-3xl font-bold">Kanva</h1>
          </div>
          <p className="text-muted-foreground">
            Connect to a Kanva server to get started
          </p>
        </div>

        {/* Embedded server card (if running in Tauri) */}
        {embeddedServer && (
          <Card
            className={cn(
              'mb-4 cursor-pointer transition-all hover:shadow-md',
              activeServerId === embeddedServer.id && 'ring-2 ring-primary',
            )}
            onClick={() => handleConnect(embeddedServer)}
          >
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center">
                  <HardDrive className="h-5 w-5 text-primary" />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <h3 className="font-semibold">Local (Standalone)</h3>
                    <span className="text-xs bg-primary/10 text-primary px-2 py-0.5 rounded-full">
                      Embedded
                    </span>
                  </div>
                  <p className="text-sm text-muted-foreground truncate">
                    {embeddedServer.url}
                  </p>
                </div>
                <ArrowRight className="h-5 w-5 text-muted-foreground" />
              </div>
              {embeddedServer.serverInfo && (
                <div className="flex items-center gap-3 mt-3 text-xs text-muted-foreground">
                  <span className="flex items-center gap-1">
                    <Shield className="h-3 w-3" /> v{embeddedServer.serverInfo.version}
                  </span>
                  <span className="flex items-center gap-1">
                    <HardDrive className="h-3 w-3" /> Standalone
                  </span>
                  {!embeddedServer.serverInfo.teams_enabled && (
                    <span className="flex items-center gap-1 text-yellow-600">
                      <Users className="h-3 w-3" /> Teams disabled
                    </span>
                  )}
                </div>
              )}
            </CardContent>
          </Card>
        )}

        {/* Separator if embedded exists */}
        {embeddedServer && servers.length > 1 && (
          <div className="flex items-center gap-3 my-4">
            <hr className="flex-1" />
            <span className="text-xs text-muted-foreground uppercase">Remote Servers</span>
            <hr className="flex-1" />
          </div>
        )}

        {/* Remote server list */}
        <div className="space-y-2 mb-4">
          {servers
            .filter((s) => !s.isEmbedded)
            .map((server) => (
              <Card
                key={server.id}
                className={cn(
                  'cursor-pointer transition-all hover:shadow-md',
                  activeServerId === server.id && 'ring-2 ring-primary',
                )}
                onClick={() => handleConnect(server)}
              >
                <CardContent className="p-4">
                  <div className="flex items-center gap-3">
                    <div className="h-10 w-10 rounded-lg bg-blue-500/10 flex items-center justify-center">
                      <Globe className="h-5 w-5 text-blue-500" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className="font-semibold">{server.name}</h3>
                      <p className="text-sm text-muted-foreground truncate">
                        {server.url}
                      </p>
                    </div>
                    <div className="flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8"
                        onClick={() => handleEditClick(server)}
                      >
                        <Pencil className="h-3 w-3" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 text-destructive"
                        onClick={() => handleRemove(server.id)}
                      >
                        <Trash2 className="h-3 w-3" />
                      </Button>
                    </div>
                  </div>
                  {server.serverInfo && (
                    <div className="flex items-center gap-3 mt-2 text-xs text-muted-foreground">
                      <span className="flex items-center gap-1">
                        <Shield className="h-3 w-3" /> v{server.serverInfo.version}
                      </span>
                      {server.serverInfo.teams_enabled && (
                        <span className="flex items-center gap-1 text-green-600">
                          <Users className="h-3 w-3" /> Teams
                        </span>
                      )}
                    </div>
                  )}
                </CardContent>
              </Card>
            ))}
        </div>

        {/* Add server button */}
        <Button
          variant="outline"
          className="w-full border-dashed"
          onClick={() => {
            resetForm();
            setEditingId(null);
            setShowAdd(true);
          }}
        >
          <Plus className="h-4 w-4 mr-2" />
          Add Server
        </Button>

        {servers.length === 0 && !embeddedServer && (
          <p className="text-center text-sm text-muted-foreground mt-6">
            No servers configured. Add a remote server or launch the desktop app for standalone mode.
          </p>
        )}

        {/* Add/Edit server dialog */}
        <Dialog
          open={showAdd}
          onOpenChange={(open) => {
            if (!open) {
              setShowAdd(false);
              setEditingId(null);
              resetForm();
            }
          }}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>{isEditing ? 'Edit Server' : 'Add Server'}</DialogTitle>
              <DialogDescription>
                {isEditing
                  ? 'Update the server connection details'
                  : 'Enter the URL of a self-hosted Kanva server'}
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4">
              <div>
                <label className="text-sm font-medium">Server Name</label>
                <Input
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  placeholder="e.g., Work Server"
                  autoFocus
                />
              </div>
              <div>
                <label className="text-sm font-medium">Server URL</label>
                <div className="flex gap-2">
                  <Input
                    value={newUrl}
                    onChange={(e) => {
                      setNewUrl(e.target.value);
                      setProbe({ status: 'idle' });
                    }}
                    placeholder="https://kanva.example.com"
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && newUrl.trim()) {
                        handleProbe(newUrl.trim());
                      }
                    }}
                  />
                  <Button
                    variant="outline"
                    onClick={() => handleProbe(newUrl.trim())}
                    disabled={!newUrl.trim() || probe.status === 'loading'}
                  >
                    {probe.status === 'loading' ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <Wifi className="h-4 w-4" />
                    )}
                  </Button>
                </div>

                {/* Probe result */}
                {probe.status === 'success' && (
                  <div className="flex items-center gap-2 mt-2 text-sm text-green-600">
                    <CheckCircle2 className="h-4 w-4" />
                    <span>
                      Connected - Kanva v{probe.serverInfo?.version}
                      {probe.serverInfo?.standalone && ' (Standalone)'}
                      {probe.serverInfo?.teams_enabled && ' - Teams enabled'}
                    </span>
                  </div>
                )}
                {probe.status === 'error' && (
                  <div className="flex items-center gap-2 mt-2 text-sm text-destructive">
                    <AlertCircle className="h-4 w-4" />
                    <span>{probe.error}</span>
                  </div>
                )}
              </div>

              <Button
                onClick={() =>
                  isEditing
                    ? handleUpdateServer(editingId!)
                    : handleAddServer()
                }
                className="w-full"
                disabled={
                  !newName.trim() ||
                  !newUrl.trim() ||
                  probe.status === 'loading'
                }
              >
                {isEditing ? 'Update Server' : 'Add Server'}
              </Button>
            </div>
          </DialogContent>
        </Dialog>
      </div>
    </div>
  );
}
