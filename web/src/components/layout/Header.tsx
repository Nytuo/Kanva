import { useState, useRef, useEffect, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { Bell, Moon, Sun, Search, LogOut, Server } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { useAuthStore } from '@/store/auth';
import { useServerStore } from '@/store/server';
import { useThemeStore } from '@/store/theme';
import { getInitials } from '@/lib/utils';
import { useNotifications, useBoards, useSearchUsers } from '@/hooks/useApi';

export default function Header() {
  const navigate = useNavigate();
  const user = useAuthStore((s) => s.user);
  const logout = useAuthStore((s) => s.logout);
  const { theme, setTheme } = useThemeStore();
  const { data: notifications } = useNotifications();
  const activeServer = useServerStore((s) => s.getActiveServer());
  const isStandalone = useServerStore((s) => s.isStandalone());

  const [query, setQuery] = useState('');
  const [showResults, setShowResults] = useState(false);
  const searchRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const debouncedQuery = useDebounce(query, 250);
  const { data: boards } = useBoards();
  const { data: searchedUsers } = useSearchUsers(debouncedQuery);

  // Filter boards client-side
  const matchedBoards = useMemo(() => {
    if (!debouncedQuery || debouncedQuery.length < 2 || !boards) return [];
    const q = debouncedQuery.toLowerCase();
    return boards.filter((b) => b.title.toLowerCase().includes(q)).slice(0, 5);
  }, [boards, debouncedQuery]);

  const matchedUsers = (searchedUsers || []).slice(0, 5);
  const hasResults = matchedBoards.length > 0 || matchedUsers.length > 0;

  // Close dropdown on click outside
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (searchRef.current && !searchRef.current.contains(e.target as Node)) {
        setShowResults(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, []);

  const unreadCount = notifications?.filter((n: { is_read: boolean }) => !n.is_read).length || 0;

  const toggleTheme = () => {
    if (theme === 'light') setTheme('dark');
    else if (theme === 'dark') setTheme('system');
    else setTheme('light');
  };

  return (
    <header className="flex items-center justify-between border-b px-4 py-2 bg-card">
      {/* Search */}
      <div ref={searchRef} className="flex items-center gap-2 flex-1 max-w-md relative">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            ref={inputRef}
            placeholder="Search boards, people..."
            className="pl-9 h-9"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setShowResults(true);
            }}
            onFocus={() => query.length >= 2 && setShowResults(true)}
            onKeyDown={(e) => {
              if (e.key === 'Escape') {
                setShowResults(false);
                inputRef.current?.blur();
              }
            }}
          />
        </div>

        {/* Search results dropdown */}
        {showResults && debouncedQuery.length >= 2 && (
          <div className="absolute top-full left-0 right-0 mt-1 bg-popover border rounded-lg shadow-lg z-50 max-h-80 overflow-y-auto">
            {!hasResults ? (
              <p className="text-sm text-muted-foreground p-4 text-center">
                No results for "{debouncedQuery}"
              </p>
            ) : (
              <>
                {matchedBoards.length > 0 && (
                  <div>
                    <p className="text-[10px] font-semibold text-muted-foreground uppercase px-3 pt-2 pb-1">
                      Boards
                    </p>
                    {matchedBoards.map((board) => (
                      <button
                        key={board.id}
                        className="flex items-center gap-2 w-full px-3 py-2 text-sm hover:bg-accent transition-colors text-left"
                        onClick={() => {
                          navigate(`/board/${board.id}`);
                          setQuery('');
                          setShowResults(false);
                        }}
                      >
                        <div
                          className="h-4 w-4 rounded-sm flex-shrink-0"
                          style={{ backgroundColor: board.background_color || '#3b82f6' }}
                        />
                        <span className="truncate">{board.title}</span>
                      </button>
                    ))}
                  </div>
                )}

                {matchedUsers.length > 0 && (
                  <div>
                    <p className="text-[10px] font-semibold text-muted-foreground uppercase px-3 pt-2 pb-1">
                      People
                    </p>
                    {matchedUsers.map((u: { id: string; display_name: string; username: string; avatar_url?: string }) => (
                      <div
                        key={u.id}
                        className="flex items-center gap-2 w-full px-3 py-2 text-sm hover:bg-accent transition-colors"
                      >
                        <Avatar className="h-5 w-5">
                          {u.avatar_url && <AvatarImage src={u.avatar_url} />}
                          <AvatarFallback className="text-[8px]">
                            {getInitials(u.display_name)}
                          </AvatarFallback>
                        </Avatar>
                        <span className="truncate">{u.display_name}</span>
                        <span className="text-xs text-muted-foreground ml-auto">
                          @{u.username}
                        </span>
                      </div>
                    ))}
                  </div>
                )}
              </>
            )}
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2">
        {/* Server indicator */}
        {activeServer && (
          <button
            onClick={() => navigate('/servers')}
            className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors px-2 py-1 rounded-md hover:bg-accent"
            title={`Connected to ${activeServer.name}`}
          >
            <Server className="h-3 w-3" />
            <span className="hidden lg:inline max-w-[120px] truncate">{activeServer.name}</span>
            {isStandalone && (
              <span className="text-[10px] bg-primary/10 text-primary px-1 py-0.5 rounded">
                Local
              </span>
            )}
          </button>
        )}

        <Button variant="ghost" size="icon" className="relative" onClick={toggleTheme}>
          {theme === 'dark' ? <Moon className="h-4 w-4" /> : <Sun className="h-4 w-4" />}
        </Button>

        <Button variant="ghost" size="icon" className="relative">
          <Bell className="h-4 w-4" />
          {unreadCount > 0 && (
            <span className="absolute -top-1 -right-1 h-4 w-4 rounded-full bg-destructive text-[10px] font-bold text-destructive-foreground flex items-center justify-center">
              {unreadCount}
            </span>
          )}
        </Button>

        <div className="flex items-center gap-2 ml-2">
          <Avatar className="h-8 w-8">
            <AvatarImage src={user?.avatar_url} />
            <AvatarFallback className="text-xs">
              {user ? getInitials(user.display_name) : '?'}
            </AvatarFallback>
          </Avatar>
          <span className="text-sm font-medium hidden md:block">{user?.display_name}</span>
        </div>

        <Button variant="ghost" size="icon" onClick={logout}>
          <LogOut className="h-4 w-4" />
        </Button>
      </div>
    </header>
  );
}

/** Simple debounce hook */
function useDebounce(value: string, delay: number) {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const t = setTimeout(() => setDebounced(value), delay);
    return () => clearTimeout(t);
  }, [value, delay]);
  return debounced;
}
