import { useState } from 'react';
import {
  AlignLeft,
  Calendar,
  CheckSquare,
  MessageSquare,
  Paperclip,
  Tag,
  Trash2,
  X,
} from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { useCard, useUpdateCard, useAddComment, useDeleteCard } from '@/hooks/useApi';
import { cn, priorityColors, formatDate, formatRelativeDate, getInitials } from '@/lib/utils';
import type { Card, CardPriority, Label } from '@/types';

interface CardDetailDialogProps {
  cardId: string | null;
  boardLabels: Label[];
  onClose: () => void;
}

const priorities: CardPriority[] = ['none', 'low', 'medium', 'high', 'urgent'];

export default function CardDetailDialog({
  cardId,
  boardLabels: _boardLabels,
  onClose,
}: CardDetailDialogProps) {
  const { data: card, isLoading } = useCard(cardId ?? undefined);
  const updateCard = useUpdateCard(cardId ?? '');
  const addComment = useAddComment(cardId ?? '');
  const deleteCard = useDeleteCard();

  const [editingTitle, setEditingTitle] = useState(false);
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [editingDesc, setEditingDesc] = useState(false);
  const [newComment, setNewComment] = useState('');
  const [showPriority, setShowPriority] = useState(false);

  const handleTitleSave = () => {
    if (title.trim() && title !== card?.title) {
      updateCard.mutate({ title: title.trim() } as Partial<Card>);
    }
    setEditingTitle(false);
  };

  const handleDescSave = () => {
    updateCard.mutate({ description } as Partial<Card>);
    setEditingDesc(false);
  };

  const handleAddComment = () => {
    if (!newComment.trim()) return;
    addComment.mutate(newComment.trim());
    setNewComment('');
  };

  const handlePriorityChange = (priority: CardPriority) => {
    updateCard.mutate({ priority } as Partial<Card>);
    setShowPriority(false);
  };

  const handleDelete = () => {
    if (cardId && window.confirm('Are you sure you want to delete this card?')) {
      deleteCard.mutate(cardId);
      onClose();
    }
  };

  return (
    <Dialog open={!!cardId} onOpenChange={() => onClose()}>
      <DialogContent className="max-w-2xl max-h-[85vh] overflow-y-auto">
        {isLoading || !card ? (
          <div className="flex items-center justify-center h-40">
            <div className="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full" />
          </div>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle className="sr-only">Card Details</DialogTitle>
              <DialogDescription className="sr-only">
                View and edit card details
              </DialogDescription>
            </DialogHeader>

            {/* Cover */}
            {card.cover_color && (
              <div
                className="h-24 -mx-6 -mt-6 mb-4 rounded-t-lg"
                style={{ backgroundColor: card.cover_color }}
              />
            )}

            {/* Title */}
            <div className="mb-4">
              {editingTitle ? (
                <Input
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  onBlur={handleTitleSave}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') handleTitleSave();
                    if (e.key === 'Escape') setEditingTitle(false);
                  }}
                  className="text-lg font-semibold"
                  autoFocus
                />
              ) : (
                <h2
                  className="text-lg font-semibold cursor-pointer hover:bg-accent/50 rounded px-1 -mx-1"
                  onClick={() => {
                    setTitle(card.title);
                    setEditingTitle(true);
                  }}
                >
                  {card.title}
                </h2>
              )}
            </div>

            <div className="grid grid-cols-[1fr_180px] gap-6">
              {/* Main content */}
              <div className="space-y-6 min-w-0">
                {/* Labels */}
                {card.labels.length > 0 && (
                  <div>
                    <h3 className="flex items-center gap-2 text-xs font-semibold text-muted-foreground uppercase mb-2">
                      <Tag className="h-3 w-3" />
                      Labels
                    </h3>
                    <div className="flex flex-wrap gap-1.5">
                      {card.labels.map((label) => (
                        <span
                          key={label.id}
                          className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium text-white"
                          style={{ backgroundColor: label.color }}
                        >
                          {label.name}
                        </span>
                      ))}
                    </div>
                  </div>
                )}

                {/* Description */}
                <div>
                  <h3 className="flex items-center gap-2 text-xs font-semibold text-muted-foreground uppercase mb-2">
                    <AlignLeft className="h-3 w-3" />
                    Description
                  </h3>
                  {editingDesc ? (
                    <div className="space-y-2">
                      <textarea
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        className="w-full min-h-[120px] rounded-md border bg-background p-3 text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary"
                        placeholder="Add a more detailed description..."
                        autoFocus
                      />
                      <div className="flex items-center gap-2">
                        <Button size="sm" onClick={handleDescSave}>
                          Save
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => setEditingDesc(false)}
                        >
                          Cancel
                        </Button>
                      </div>
                    </div>
                  ) : (
                    <div
                      className="min-h-[60px] rounded-md bg-accent/50 p-3 text-sm cursor-pointer hover:bg-accent"
                      onClick={() => {
                        setDescription(card.description || '');
                        setEditingDesc(true);
                      }}
                    >
                      {card.description || (
                        <span className="text-muted-foreground">
                          Add a more detailed description...
                        </span>
                      )}
                    </div>
                  )}
                </div>

                {/* Checklists */}
                {card.checklists.length > 0 && (
                  <div className="space-y-4">
                    {card.checklists.map((checklist) => {
                      const done = checklist.items.filter((i) => i.is_checked).length;
                      const total = checklist.items.length;
                      const pct = total > 0 ? Math.round((done / total) * 100) : 0;

                      return (
                        <div key={checklist.id}>
                          <h3 className="flex items-center gap-2 text-xs font-semibold text-muted-foreground uppercase mb-2">
                            <CheckSquare className="h-3 w-3" />
                            {checklist.title}
                            <span className="text-[10px] ml-auto">
                              {done}/{total}
                            </span>
                          </h3>
                          {/* Progress bar */}
                          <div className="h-1.5 bg-muted rounded-full mb-2 overflow-hidden">
                            <div
                              className={cn(
                                'h-full rounded-full transition-all',
                                pct === 100 ? 'bg-green-500' : 'bg-primary',
                              )}
                              style={{ width: `${pct}%` }}
                            />
                          </div>
                          <div className="space-y-1">
                            {checklist.items.map((item) => (
                              <label
                                key={item.id}
                                className="flex items-center gap-2 py-1 px-1 rounded hover:bg-accent/50 cursor-pointer"
                              >
                                <input
                                  type="checkbox"
                                  checked={item.is_checked}
                                  readOnly
                                  className="rounded"
                                />
                                <span
                                  className={cn(
                                    'text-sm',
                                    item.is_checked && 'line-through text-muted-foreground',
                                  )}
                                >
                                  {item.title}
                                </span>
                              </label>
                            ))}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}

                {/* Attachments */}
                {card.attachments.length > 0 && (
                  <div>
                    <h3 className="flex items-center gap-2 text-xs font-semibold text-muted-foreground uppercase mb-2">
                      <Paperclip className="h-3 w-3" />
                      Attachments
                    </h3>
                    <div className="space-y-2">
                      {card.attachments.map((att) => (
                        <a
                          key={att.id}
                          href={att.file_url}
                          target="_blank"
                          rel="noreferrer"
                          className="flex items-center gap-3 p-2 rounded-md border hover:bg-accent/50 transition-colors"
                        >
                          <div className="h-10 w-10 rounded bg-muted flex items-center justify-center text-xs font-medium">
                            {att.filename.split('.').pop()?.toUpperCase() || 'FILE'}
                          </div>
                          <div className="flex-1 min-w-0">
                            <p className="text-sm font-medium truncate">{att.filename}</p>
                            <p className="text-xs text-muted-foreground">
                              {(att.file_size / 1024).toFixed(1)} KB
                            </p>
                          </div>
                        </a>
                      ))}
                    </div>
                  </div>
                )}

                {/* Comments */}
                <div>
                  <h3 className="flex items-center gap-2 text-xs font-semibold text-muted-foreground uppercase mb-3">
                    <MessageSquare className="h-3 w-3" />
                    Activity
                  </h3>
                  <div className="flex gap-3 mb-4">
                    <Avatar className="h-8 w-8">
                      <AvatarFallback>U</AvatarFallback>
                    </Avatar>
                    <div className="flex-1">
                      <textarea
                        value={newComment}
                        onChange={(e) => setNewComment(e.target.value)}
                        placeholder="Write a comment..."
                        className="w-full rounded-md border bg-background p-2 text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary"
                        rows={2}
                      />
                      {newComment.trim() && (
                        <Button
                          size="sm"
                          className="mt-2"
                          onClick={handleAddComment}
                        >
                          Save
                        </Button>
                      )}
                    </div>
                  </div>
                  <div className="space-y-4">
                    {card.comments.map((comment) => (
                      <div key={comment.id} className="flex gap-3">
                        <Avatar className="h-8 w-8">
                          {comment.avatar_url && (
                            <AvatarImage src={comment.avatar_url} />
                          )}
                          <AvatarFallback>
                            {getInitials(comment.display_name)}
                          </AvatarFallback>
                        </Avatar>
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-1">
                            <span className="text-sm font-medium">
                              {comment.display_name}
                            </span>
                            <span className="text-xs text-muted-foreground">
                              {formatRelativeDate(comment.created_at)}
                            </span>
                          </div>
                          <div className="text-sm bg-accent/50 rounded-md p-2">
                            {comment.content}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>

              {/* Sidebar actions */}
              <div className="space-y-2">
                <p className="text-xs font-semibold text-muted-foreground uppercase mb-1">
                  Details
                </p>

                {/* Priority */}
                <div className="relative">
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full justify-start"
                    onClick={() => setShowPriority(!showPriority)}
                  >
                    <span
                      className={cn(
                        'px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase mr-2',
                        priorityColors[card.priority],
                      )}
                    >
                      {card.priority}
                    </span>
                    Priority
                  </Button>
                  {showPriority && (
                    <>
                      <div
                        className="fixed inset-0 z-10"
                        onClick={() => setShowPriority(false)}
                      />
                      <div className="absolute right-0 top-9 z-20 w-40 bg-popover border rounded-md shadow-lg p-1">
                        {priorities.map((p) => (
                          <button
                            key={p}
                            className={cn(
                              'flex items-center gap-2 w-full rounded-sm px-2 py-1.5 text-sm hover:bg-accent',
                              card.priority === p && 'bg-accent',
                            )}
                            onClick={() => handlePriorityChange(p)}
                          >
                            <span
                              className={cn(
                                'px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase',
                                priorityColors[p],
                              )}
                            >
                              {p}
                            </span>
                          </button>
                        ))}
                      </div>
                    </>
                  )}
                </div>

                {/* Due date picker */}
                <div>
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full justify-start"
                    onClick={() => {
                      // Open native date picker via hidden input
                      const input = document.getElementById('card-due-date-input') as HTMLInputElement;
                      input?.showPicker?.();
                      input?.focus();
                    }}
                  >
                    <Calendar className="h-4 w-4 mr-2 text-muted-foreground" />
                    {card.due_date ? (
                      <span className="flex-1 text-left">{formatDate(card.due_date)}</span>
                    ) : (
                      <span className="flex-1 text-left text-muted-foreground">Due Date</span>
                    )}
                    {card.due_date && (
                      <span
                        role="button"
                        className="ml-1 hover:text-destructive"
                        onClick={(e) => {
                          e.stopPropagation();
                          updateCard.mutate({ due_date: '' } as Partial<Card>);
                        }}
                      >
                        <X className="h-3 w-3" />
                      </span>
                    )}
                  </Button>
                  <input
                    id="card-due-date-input"
                    type="date"
                    className="sr-only"
                    value={card.due_date ? card.due_date.slice(0, 10) : ''}
                    onChange={(e) => {
                      const val = e.target.value;
                      if (val) {
                        updateCard.mutate({ due_date: val + 'T00:00:00Z' } as Partial<Card>);
                      }
                    }}
                  />
                </div>

                {/* Assignees */}
                {card.assignees.length > 0 && (
                  <div>
                    <p className="text-xs text-muted-foreground mb-1">Assignees</p>
                    <div className="space-y-1">
                      {card.assignees.map((a) => (
                        <div key={a.user_id} className="flex items-center gap-2 text-sm">
                          <Avatar className="h-6 w-6">
                            {a.avatar_url && <AvatarImage src={a.avatar_url} />}
                            <AvatarFallback className="text-[10px]">
                              {getInitials(a.display_name)}
                            </AvatarFallback>
                          </Avatar>
                          <span className="truncate">{a.display_name}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                <hr className="my-2" />

                {/* Hours */}
                {(card.estimated_hours || card.actual_hours) && (
                  <div className="text-sm space-y-1">
                    {card.estimated_hours && (
                      <p className="text-muted-foreground">
                        Est: {card.estimated_hours}h
                      </p>
                    )}
                    {card.actual_hours && (
                      <p className="text-muted-foreground">
                        Actual: {card.actual_hours}h
                      </p>
                    )}
                  </div>
                )}

                <hr className="my-2" />

                <Button
                  variant="destructive"
                  size="sm"
                  className="w-full"
                  onClick={handleDelete}
                >
                  <Trash2 className="h-4 w-4 mr-1" />
                  Delete Card
                </Button>

                <p className="text-[10px] text-muted-foreground mt-4">
                  Created {formatRelativeDate(card.created_at)}
                </p>
              </div>
            </div>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
