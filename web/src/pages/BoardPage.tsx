import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  ArrowLeft,
  Star,
  StarOff,
  Users,
  MoreHorizontal,
  Globe,
  Lock,
  Filter,
  X,
  Archive,
  Trash2,
  Settings2,
  Edit2,
  Check,
  Paintbrush,
  LayoutGrid,
  NotebookText,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog';
import KanbanBoard from '@/components/board/KanbanBoard';
import NotesPanel from '@/components/notes/NotesPanel';
import BackgroundPicker from '@/components/board/BackgroundPicker';
import { boardBgStyle } from '@/components/board/board-backgrounds';
import { useBoard, useToggleStar, useUpdateBoard, useDeleteBoard } from '@/hooks/useApi';
import { getInitials } from '@/lib/utils';
import type { CardPriority } from '@/types';

export interface BoardFilters {
  search: string;
  priority: CardPriority | '';
  assigneeId: string;
  labelId: string;
  dueSoon: boolean; // due within 3 days
  overdue: boolean;
}

export const DEFAULT_FILTERS: BoardFilters = {
  search: '',
  priority: '',
  assigneeId: '',
  labelId: '',
  dueSoon: false,
  overdue: false,
};

function hasActiveFilters(f: BoardFilters) {
  return f.search || f.priority || f.assigneeId || f.labelId || f.dueSoon || f.overdue;
}

export default function BoardPage() {
  const { boardId } = useParams<{ boardId: string }>();
  const navigate = useNavigate();
  const { data: board, isLoading, error, refetch } = useBoard(boardId);
  const toggleStar = useToggleStar(boardId ?? '');
  const updateBoard = useUpdateBoard(boardId ?? '');
  const deleteBoard = useDeleteBoard();

  const [showFilters, setShowFilters] = useState(false);
  const [filters, setFilters] = useState<BoardFilters>(DEFAULT_FILTERS);
  const [editingTitle, setEditingTitle] = useState(false);
  const [titleDraft, setTitleDraft] = useState('');
  const [showBgPicker, setShowBgPicker] = useState(false);
  const [bgDraft, setBgDraft] = useState({ bgColor: '', bgImage: '' });
  const [view, setView] = useState<'board' | 'notes'>('board');

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full" />
      </div>
    );
  }

  if (error || !board) {
    const isNotFound = (error as { response?: { status?: number } })?.response?.status === 404;
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <p className="text-muted-foreground">
          {isNotFound ? 'Board not found' : error ? 'Failed to load board' : 'Board not found'}
        </p>
        {error && !isNotFound && (
          <p className="text-xs text-destructive">
            {(error as { response?: { data?: { error?: { message?: string } } } })?.response?.data?.error?.message ?? String(error)}
          </p>
        )}
        <Button variant="outline" onClick={() => navigate('/')}>
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back to Dashboard
        </Button>
      </div>
    );
  }

  const VisibilityIcon = board.visibility === 'public' ? Globe : board.visibility === 'team' ? Users : Lock;
  const activeFilterCount = Object.values(filters).filter(Boolean).length;

  const handleSaveTitle = () => {
    if (titleDraft.trim() && titleDraft !== board.title) {
      updateBoard.mutate({ title: titleDraft.trim() });
    }
    setEditingTitle(false);
  };

  const handleArchive = () => {
    updateBoard.mutate({ is_archived: !board.is_archived } as never);
  };

  const handleDelete = () => {
    if (window.confirm(`Delete board "${board.title}" and all its lists and cards? This cannot be undone.`)) {
      deleteBoard.mutate(board.id, {
        onSuccess: () => navigate('/'),
      });
    }
  };

  const handleSetVisibility = (v: 'private' | 'team' | 'public') => {
    updateBoard.mutate({ visibility: v });
  };

  const openBgPicker = () => {
    setBgDraft({
      bgColor: board.background_color || '#3b82f6',
      bgImage: board.background_image_url || '',
    });
    setShowBgPicker(true);
  };

  const handleSaveBg = () => {
    const update: { background_color?: string; background_image_url?: string } = {};
    if (bgDraft.bgImage) {
      update.background_image_url = bgDraft.bgImage;
      update.background_color = '';
    } else {
      update.background_color = bgDraft.bgColor;
      update.background_image_url = '';
    }
    updateBoard.mutate(update as never);
    setShowBgPicker(false);
  };

  return (
    <div
      className="flex flex-col h-full"
      style={boardBgStyle(board.background_color, board.background_image_url)}
    >
      {/* Board header */}
      <div className="flex items-center justify-between px-4 py-2 bg-black/20 backdrop-blur-sm">
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-white hover:bg-white/20"
            onClick={() => navigate('/')}
          >
            <ArrowLeft className="h-4 w-4" />
          </Button>

          {/* Editable title */}
          {editingTitle ? (
            <div className="flex items-center gap-1">
              <Input
                value={titleDraft}
                onChange={(e) => setTitleDraft(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleSaveTitle();
                  if (e.key === 'Escape') setEditingTitle(false);
                }}
                className="h-8 bg-white/20 border-white/40 text-white placeholder:text-white/60 w-48"
                autoFocus
              />
              <Button size="icon" variant="ghost" className="h-7 w-7 text-white hover:bg-white/20" onClick={handleSaveTitle}>
                <Check className="h-3 w-3" />
              </Button>
              <Button size="icon" variant="ghost" className="h-7 w-7 text-white hover:bg-white/20" onClick={() => setEditingTitle(false)}>
                <X className="h-3 w-3" />
              </Button>
            </div>
          ) : (
            <button
              className="text-lg font-bold text-white hover:underline decoration-white/50 cursor-pointer"
              onClick={() => { setTitleDraft(board.title); setEditingTitle(true); }}
            >
              {board.title}
            </button>
          )}

          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-white hover:bg-white/20"
            onClick={() => toggleStar.mutate()}
          >
            {board.is_starred ? (
              <Star className="h-4 w-4 fill-yellow-400 text-yellow-400" />
            ) : (
              <StarOff className="h-4 w-4" />
            )}
          </Button>

          <div className="flex items-center gap-1 text-white/70 text-xs">
            <VisibilityIcon className="h-3 w-3" />
            <span className="capitalize">{board.visibility}</span>
          </div>

          {board.is_archived && (
            <span className="text-xs bg-yellow-500/20 text-yellow-300 border border-yellow-500/30 px-2 py-0.5 rounded">
              Archived
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          {/* Board members */}
          <div className="flex -space-x-2 mr-1">
            {board.members.slice(0, 5).map((member) => (
              <Avatar key={member.user_id} className="h-7 w-7 border-2 border-background">
                {member.avatar_url && <AvatarImage src={member.avatar_url} />}
                <AvatarFallback className="text-[10px]">
                  {getInitials(member.display_name)}
                </AvatarFallback>
              </Avatar>
            ))}
            {board.members.length > 5 && (
              <div className="h-7 w-7 rounded-full bg-muted border-2 border-background flex items-center justify-center text-[10px] font-medium">
                +{board.members.length - 5}
              </div>
            )}
          </div>

          {/* Board / Notes view toggle */}
          <Button
            variant="ghost"
            size="sm"
            className="text-white hover:bg-white/20 gap-1"
            onClick={() => setView((v) => (v === 'board' ? 'notes' : 'board'))}
          >
            {view === 'board' ? <NotebookText className="h-4 w-4" /> : <LayoutGrid className="h-4 w-4" />}
            {view === 'board' ? 'Notes' : 'Board'}
          </Button>

          {/* Filter toggle */}
          {view === 'board' && (
            <Button
              variant="ghost"
              size="sm"
              className={`text-white hover:bg-white/20 gap-1 ${showFilters ? 'bg-white/20' : ''}`}
              onClick={() => setShowFilters((v) => !v)}
            >
              <Filter className="h-4 w-4" />
              Filter
              {activeFilterCount > 0 && (
                <span className="ml-1 bg-primary text-primary-foreground text-[10px] rounded-full px-1.5 py-0.5 leading-none">
                  {activeFilterCount}
                </span>
              )}
            </Button>
          )}

          {/* Board menu */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon" className="h-8 w-8 text-white hover:bg-white/20">
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-52">
              <DropdownMenuItem onClick={() => { setTitleDraft(board.title); setEditingTitle(true); }}>
                <Edit2 className="h-4 w-4 mr-2" />
                Rename Board
              </DropdownMenuItem>
              <DropdownMenuItem onClick={openBgPicker}>
                <Paintbrush className="h-4 w-4 mr-2" />
                Change Background
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={() => handleSetVisibility('private')}>
                <Lock className="h-4 w-4 mr-2" />
                Make Private
                {board.visibility === 'private' && <Check className="h-3 w-3 ml-auto" />}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleSetVisibility('team')}>
                <Users className="h-4 w-4 mr-2" />
                Make Team
                {board.visibility === 'team' && <Check className="h-3 w-3 ml-auto" />}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => handleSetVisibility('public')}>
                <Globe className="h-4 w-4 mr-2" />
                Make Public
                {board.visibility === 'public' && <Check className="h-3 w-3 ml-auto" />}
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={handleArchive}>
                <Archive className="h-4 w-4 mr-2" />
                {board.is_archived ? 'Unarchive Board' : 'Archive Board'}
              </DropdownMenuItem>
              <DropdownMenuItem
                className="text-destructive focus:text-destructive"
                onClick={handleDelete}
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Delete Board
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      {/* Filter bar */}
      {view === 'board' && showFilters && (
        <div className="flex items-center gap-2 px-4 py-2 bg-black/30 backdrop-blur-sm flex-wrap">
          <Input
            placeholder="Search cards..."
            value={filters.search}
            onChange={(e) => setFilters({ ...filters, search: e.target.value })}
            className="h-7 w-40 bg-white/20 border-white/30 text-white placeholder:text-white/50 text-xs"
          />

          <select
            value={filters.priority}
            onChange={(e) => setFilters({ ...filters, priority: e.target.value as CardPriority | '' })}
            className="h-7 rounded-md border border-white/30 bg-white/20 text-white text-xs px-2 focus:outline-none"
          >
            <option value="" className="bg-background text-foreground">Any Priority</option>
            <option value="urgent" className="bg-background text-foreground">Urgent</option>
            <option value="high" className="bg-background text-foreground">High</option>
            <option value="medium" className="bg-background text-foreground">Medium</option>
            <option value="low" className="bg-background text-foreground">Low</option>
            <option value="none" className="bg-background text-foreground">None</option>
          </select>

          {board.labels.length > 0 && (
            <select
              value={filters.labelId}
              onChange={(e) => setFilters({ ...filters, labelId: e.target.value })}
              className="h-7 rounded-md border border-white/30 bg-white/20 text-white text-xs px-2 focus:outline-none"
            >
              <option value="" className="bg-background text-foreground">Any Label</option>
              {board.labels.map((l) => (
                <option key={l.id} value={l.id} className="bg-background text-foreground">{l.name}</option>
              ))}
            </select>
          )}

          {board.members.length > 0 && (
            <select
              value={filters.assigneeId}
              onChange={(e) => setFilters({ ...filters, assigneeId: e.target.value })}
              className="h-7 rounded-md border border-white/30 bg-white/20 text-white text-xs px-2 focus:outline-none"
            >
              <option value="" className="bg-background text-foreground">Any Assignee</option>
              {board.members.map((m) => (
                <option key={m.user_id} value={m.user_id} className="bg-background text-foreground">{m.display_name}</option>
              ))}
            </select>
          )}

          <label className="flex items-center gap-1.5 text-white text-xs cursor-pointer">
            <input
              type="checkbox"
              checked={filters.dueSoon}
              onChange={(e) => setFilters({ ...filters, dueSoon: e.target.checked })}
              className="rounded"
            />
            Due soon
          </label>

          <label className="flex items-center gap-1.5 text-white text-xs cursor-pointer">
            <input
              type="checkbox"
              checked={filters.overdue}
              onChange={(e) => setFilters({ ...filters, overdue: e.target.checked })}
              className="rounded"
            />
            Overdue
          </label>

          {hasActiveFilters(filters) && (
            <Button
              variant="ghost"
              size="sm"
              className="h-7 text-white/70 hover:text-white hover:bg-white/10 text-xs"
              onClick={() => setFilters(DEFAULT_FILTERS)}
            >
              <X className="h-3 w-3 mr-1" />
              Clear
            </Button>
          )}

          <div className="ml-auto">
            <Settings2 className="h-4 w-4 text-white/50" />
          </div>
        </div>
      )}

      {/* Kanban board / Notes */}
      <div className={`flex-1 overflow-hidden ${view === 'notes' ? 'bg-background' : ''}`}>
        {view === 'board' ? (
          <KanbanBoard board={board} filters={filters} onRefresh={refetch} />
        ) : (
          <NotesPanel boardId={board.id} />
        )}
      </div>

      {/* Background picker dialog */}
      <Dialog open={showBgPicker} onOpenChange={setShowBgPicker}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Change Background</DialogTitle>
            <DialogDescription>
              Choose a color, gradient, or image for this board.
            </DialogDescription>
          </DialogHeader>
          <BackgroundPicker
            value={bgDraft}
            onChange={setBgDraft}
            previewTitle={board.title}
          />
          <div className="flex justify-end gap-2 mt-2">
            <Button variant="outline" onClick={() => setShowBgPicker(false)}>
              Cancel
            </Button>
            <Button onClick={handleSaveBg} disabled={updateBoard.isPending}>
              {updateBoard.isPending ? 'Saving...' : 'Save'}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
