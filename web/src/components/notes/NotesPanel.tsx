import { useEffect, useMemo, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { FileText, Pin, Plus, Trash2, Eye, Pencil } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import { cn, formatRelativeDate } from '@/lib/utils';
import { useNotes, useCreateNote, useUpdateNote, useDeleteNote } from '@/hooks/useApi';
import type { Note } from '@/types';

/** Debounce a fast-changing value so we don't fire a save request per keystroke. */
function useDebounced<T>(value: T, delayMs: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delayMs);
    return () => clearTimeout(timer);
  }, [value, delayMs]);
  return debounced;
}

export interface NotesPanelProps {
  /** Scope notes to a board (shared with board members). Omit for the caller's private global notes. */
  boardId?: string;
}

export default function NotesPanel({ boardId }: NotesPanelProps) {
  const { data: notes, isLoading } = useNotes(boardId);
  const createNote = useCreateNote();
  const updateNote = useUpdateNote();
  const deleteNote = useDeleteNote();

  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [mode, setMode] = useState<'edit' | 'preview'>('edit');

  const sorted = useMemo(
    () => [...(notes ?? [])].sort((a, b) => Number(b.is_pinned) - Number(a.is_pinned) || a.position - b.position),
    [notes],
  );

  const selected = sorted.find((n) => n.id === selectedId) ?? null;

  // Keep a selection once notes load; drop it if the note was deleted elsewhere.
  useEffect(() => {
    if (!notes) return;
    if (selectedId && notes.some((n) => n.id === selectedId)) return;
    setSelectedId(sorted[0]?.id ?? null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [notes]);

  useEffect(() => {
    setTitle(selected?.title ?? '');
    setContent(selected?.content ?? '');
    setMode('edit');
  }, [selected?.id]);

  const debouncedTitle = useDebounced(title, 600);
  const debouncedContent = useDebounced(content, 600);

  // Autosave — Notion/Obsidian-style, no explicit save button.
  useEffect(() => {
    if (!selected) return;
    if (debouncedTitle === selected.title && debouncedContent === selected.content) return;
    updateNote.mutate({ id: selected.id, title: debouncedTitle, content: debouncedContent });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedTitle, debouncedContent]);

  const handleCreate = () => {
    createNote.mutate(
      { board_id: boardId, title: 'Untitled', content: '' },
      { onSuccess: (note: Note) => setSelectedId(note.id) },
    );
  };

  const handleDelete = (note: Note) => {
    if (!window.confirm(`Delete note "${note.title}"? This cannot be undone.`)) return;
    deleteNote.mutate({ id: note.id, boardId }, {
      onSuccess: () => {
        if (selectedId === note.id) setSelectedId(null);
      },
    });
  };

  const togglePin = (note: Note) => {
    updateNote.mutate({ id: note.id, is_pinned: !note.is_pinned });
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin h-6 w-6 border-4 border-primary border-t-transparent rounded-full" />
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0">
      {/* Notes list */}
      <div className="w-64 flex-shrink-0 border-r flex flex-col min-h-0">
        <div className="flex items-center justify-between p-3 border-b">
          <span className="text-xs font-semibold text-muted-foreground uppercase">
            {boardId ? 'Project Notes' : 'My Notes'}
          </span>
          <Button variant="ghost" size="icon" className="h-6 w-6" onClick={handleCreate} disabled={createNote.isPending}>
            <Plus className="h-4 w-4" />
          </Button>
        </div>
        <div className="flex-1 overflow-y-auto">
          {sorted.length === 0 && (
            <div className="p-4 text-sm text-muted-foreground text-center">
              No notes yet. Click + to create one.
            </div>
          )}
          {sorted.map((note) => (
            <button
              key={note.id}
              onClick={() => setSelectedId(note.id)}
              className={cn(
                'w-full text-left px-3 py-2 border-b hover:bg-accent transition-colors group',
                selectedId === note.id && 'bg-accent',
              )}
            >
              <div className="flex items-center gap-1.5">
                {note.is_pinned && <Pin className="h-3 w-3 text-primary flex-shrink-0" />}
                <span className="text-sm font-medium truncate flex-1">{note.title || 'Untitled'}</span>
              </div>
              <div className="flex items-center justify-between mt-0.5">
                <span className="text-xs text-muted-foreground">{formatRelativeDate(note.updated_at)}</span>
                <span className="text-xs text-muted-foreground truncate max-w-[100px] opacity-0 group-hover:opacity-100">
                  {note.content.replace(/[#*_`>-]/g, '').trim().slice(0, 40)}
                </span>
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Editor */}
      <div className="flex-1 flex flex-col min-h-0">
        {!selected ? (
          <div className="flex-1 flex flex-col items-center justify-center gap-2 text-muted-foreground">
            <FileText className="h-8 w-8" />
            <p className="text-sm">Select a note or create a new one</p>
          </div>
        ) : (
          <>
            <div className="flex items-center gap-2 p-3 border-b">
              <Input
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Untitled"
                className="h-8 border-none shadow-none text-base font-semibold focus-visible:ring-0 px-0"
              />
              <Button variant="ghost" size="icon" className="h-7 w-7 flex-shrink-0" onClick={() => togglePin(selected)}>
                <Pin className={cn('h-4 w-4', selected.is_pinned && 'fill-primary text-primary')} />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 flex-shrink-0 text-destructive hover:text-destructive"
                onClick={() => handleDelete(selected)}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>

            <Tabs value={mode} onValueChange={(v) => setMode(v as 'edit' | 'preview')} className="flex-1 flex flex-col min-h-0">
              <div className="px-3 pt-2">
                <TabsList className="h-8">
                  <TabsTrigger value="edit" className="h-6 gap-1 text-xs">
                    <Pencil className="h-3 w-3" /> Write
                  </TabsTrigger>
                  <TabsTrigger value="preview" className="h-6 gap-1 text-xs">
                    <Eye className="h-3 w-3" /> Preview
                  </TabsTrigger>
                </TabsList>
              </div>
              <TabsContent value="edit" className="flex-1 min-h-0 p-3 pt-2 mt-0">
                <Textarea
                  value={content}
                  onChange={(e) => setContent(e.target.value)}
                  placeholder="Write in Markdown…"
                  className="h-full resize-none font-mono text-sm"
                />
              </TabsContent>
              <TabsContent value="preview" className="flex-1 min-h-0 overflow-y-auto p-4 pt-2 mt-0">
                {content.trim() ? (
                  <div className="prose prose-sm dark:prose-invert max-w-none">
                    <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground italic">Nothing to preview yet.</p>
                )}
              </TabsContent>
            </Tabs>
          </>
        )}
      </div>
    </div>
  );
}
