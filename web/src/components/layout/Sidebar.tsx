import { Link, useLocation, useNavigate } from 'react-router-dom';
import {
  LayoutDashboard,
  Calendar,
  Users,
  Settings,
  Star,
  Plus,
  ChevronLeft,
  ChevronRight,
  Kanban,
  Server,
  ChevronDown,
} from 'lucide-react';
import { useState } from 'react';
import type React from 'react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { useBoards, useTeams, useCreateBoard } from '@/hooks/useApi';
import { useServerStore } from '@/store/server';

/** Small swatch style for sidebar board icons (color or gradient, no images) */
function boardSwatchStyle(bgColor?: string | null): React.CSSProperties {
  if (bgColor?.startsWith('linear-gradient')) return { backgroundImage: bgColor };
  return { backgroundColor: bgColor || '#3b82f6' };
}

export default function Sidebar() {
  const [collapsed, setCollapsed] = useState(false);
  const location = useLocation();
  const navigate = useNavigate();
  const { data: boards } = useBoards();
  const teamsEnabled = useServerStore((s) => s.isTeamsEnabled());
  const { data: teams } = useTeams();
  const createBoard = useCreateBoard();
  const activeServer = useServerStore((s) => s.getActiveServer());

  const navItems = [
    { icon: LayoutDashboard, label: 'Dashboard', path: '/' },
    { icon: Calendar, label: 'Calendar', path: '/calendar' },
    ...(teamsEnabled
      ? [{ icon: Users, label: 'Teams', path: '/teams' }]
      : []),
    { icon: Settings, label: 'Settings', path: '/settings' },
  ];

  const starredBoards = boards?.filter((b) => b.is_starred) || [];
  const recentBoards = boards?.filter((b) => !b.is_archived).slice(0, 5) || [];

  const handleCreateBoard = () => {
    createBoard.mutate({ title: 'Untitled Board' });
  };

  return (
    <aside
      className={cn(
        'flex flex-col border-r bg-card transition-all duration-200',
        collapsed ? 'w-16' : 'w-64',
      )}
    >
      {/* Logo + Server indicator */}
      <div className="flex flex-col border-b">
        <div className="flex items-center justify-between p-4">
          {!collapsed && (
            <Link to="/" className="flex items-center gap-2">
              <Kanban className="h-6 w-6 text-primary" />
              <span className="font-bold text-lg">Kanva</span>
            </Link>
          )}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setCollapsed(!collapsed)}
            className="h-8 w-8"
          >
            {collapsed ? <ChevronRight className="h-4 w-4" /> : <ChevronLeft className="h-4 w-4" />}
          </Button>
        </div>

        {/* Server switcher */}
        {!collapsed && activeServer && (
          <button
            onClick={() => navigate('/servers')}
            className="mx-2 mb-2 flex items-center gap-2 rounded-md px-3 py-1.5 text-xs text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
          >
            <Server className="h-3 w-3 flex-shrink-0" />
            <span className="truncate flex-1 text-left">{activeServer.name}</span>
            {activeServer.isEmbedded && (
              <span className="text-[10px] bg-primary/10 text-primary px-1.5 py-0.5 rounded">
                Local
              </span>
            )}
            <ChevronDown className="h-3 w-3 flex-shrink-0" />
          </button>
        )}
        {collapsed && (
          <Button
            variant="ghost"
            size="icon"
            className="mx-auto mb-2 h-8 w-8"
            onClick={() => navigate('/servers')}
            title={activeServer?.name || 'Switch server'}
          >
            <Server className="h-4 w-4" />
          </Button>
        )}
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto p-2">
        <div className="space-y-1">
          {navItems.map((item) => {
            const isActive = location.pathname === item.path;
            return (
              <Link
                key={item.path}
                to={item.path}
                className={cn(
                  'flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors',
                  isActive
                    ? 'bg-primary/10 text-primary font-medium'
                    : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
                )}
              >
                <item.icon className="h-4 w-4 flex-shrink-0" />
                {!collapsed && <span>{item.label}</span>}
              </Link>
            );
          })}
        </div>

        {!collapsed && (
          <>
            {/* Starred Boards */}
            {starredBoards.length > 0 && (
              <div className="mt-6">
                <div className="flex items-center gap-2 px-3 py-1 text-xs font-semibold text-muted-foreground uppercase">
                  <Star className="h-3 w-3" />
                  Starred
                </div>
                {starredBoards.map((board) => (
                  <Link
                    key={board.id}
                    to={`/board/${board.id}`}
                    className={cn(
                      'flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors',
                      location.pathname === `/board/${board.id}`
                        ? 'bg-primary/10 text-primary'
                        : 'text-muted-foreground hover:bg-accent',
                    )}
                  >
                    <div
                      className="h-4 w-4 rounded-sm flex-shrink-0"
                      style={boardSwatchStyle(board.background_color)}
                    />
                    <span className="truncate">{board.title}</span>
                  </Link>
                ))}
              </div>
            )}

            {/* Recent Boards */}
            <div className="mt-6">
              <div className="flex items-center justify-between px-3 py-1">
                <span className="text-xs font-semibold text-muted-foreground uppercase">
                  Boards
                </span>
                <Button variant="ghost" size="icon" className="h-5 w-5" onClick={handleCreateBoard}>
                  <Plus className="h-3 w-3" />
                </Button>
              </div>
              {recentBoards.map((board) => (
                <Link
                  key={board.id}
                  to={`/board/${board.id}`}
                  className={cn(
                    'flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors',
                    location.pathname === `/board/${board.id}`
                      ? 'bg-primary/10 text-primary'
                      : 'text-muted-foreground hover:bg-accent',
                  )}
                >
                  <div
                    className="h-4 w-4 rounded-sm flex-shrink-0"
                    style={boardSwatchStyle(board.background_color)}
                  />
                  <span className="truncate">{board.title}</span>
                </Link>
              ))}
            </div>

            {/* Teams - only shown when enabled */}
            {teamsEnabled && teams && teams.length > 0 && (
              <div className="mt-6">
                <div className="text-xs font-semibold text-muted-foreground uppercase px-3 py-1">
                  Teams
                </div>
                {teams.map((team) => (
                  <Link
                    key={team.id}
                    to={`/teams/${team.id}`}
                    className="flex items-center gap-3 rounded-md px-3 py-2 text-sm text-muted-foreground hover:bg-accent"
                  >
                    <Users className="h-4 w-4" />
                    <span className="truncate">{team.name}</span>
                  </Link>
                ))}
              </div>
            )}
          </>
        )}
      </nav>
    </aside>
  );
}
