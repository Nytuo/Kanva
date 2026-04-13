import { useState } from 'react';
import {
  SortableContext,
  verticalListSortingStrategy,
  useSortable,
} from '@dnd-kit/sortable';
import { useDroppable } from '@dnd-kit/core';
import { CSS } from '@dnd-kit/utilities';
import { MoreHorizontal, Plus, X, Trash2, GripVertical } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import BoardCard from './BoardCard';
import type { List as ListType, Label } from '@/types';

interface BoardListProps {
  list: ListType;
  labels: Label[];
  onAddCard: (listId: string, title: string) => void;
  onCardClick: (cardId: string) => void;
  onDeleteList: (listId: string) => void;
  onRenameList: (listId: string, title: string) => void;
}

export default function BoardList({
  list,
  labels,
  onAddCard,
  onCardClick,
  onDeleteList,
  onRenameList,
}: BoardListProps) {
  const [isAdding, setIsAdding] = useState(false);
  const [newCardTitle, setNewCardTitle] = useState('');
  const [isEditing, setIsEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(list.title);
  const [showMenu, setShowMenu] = useState(false);

  const {
    attributes,
    listeners,
    setNodeRef: setSortableRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: `list-${list.id}`,
    data: { type: 'list', list },
  });

  const { setNodeRef: setDroppableRef } = useDroppable({
    id: `list-drop-${list.id}`,
    data: { type: 'list', listId: list.id },
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  const handleAddCard = () => {
    if (!newCardTitle.trim()) return;
    onAddCard(list.id, newCardTitle.trim());
    setNewCardTitle('');
    setIsAdding(false);
  };

  const handleRename = () => {
    if (editTitle.trim() && editTitle !== list.title) {
      onRenameList(list.id, editTitle.trim());
    }
    setIsEditing(false);
  };

  const cardIds = list.cards.map((c) => c.id);

  return (
    <div
      ref={setSortableRef}
      style={style}
      className={cn(
        'flex-shrink-0 w-72 bg-muted/50 rounded-xl flex flex-col max-h-full',
        isDragging && 'opacity-50',
      )}
    >
      {/* List header */}
      <div className="flex items-center justify-between px-3 py-2">
        <div className="flex items-center gap-1 flex-1 min-w-0">
          <button
            {...attributes}
            {...listeners}
            className="cursor-grab active:cursor-grabbing p-0.5 text-muted-foreground hover:text-foreground"
          >
            <GripVertical className="h-4 w-4" />
          </button>

          {isEditing ? (
            <Input
              value={editTitle}
              onChange={(e) => setEditTitle(e.target.value)}
              onBlur={handleRename}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleRename();
                if (e.key === 'Escape') setIsEditing(false);
              }}
              className="h-7 text-sm font-semibold"
              autoFocus
            />
          ) : (
            <button
              onClick={() => setIsEditing(true)}
              className="text-sm font-semibold truncate text-left px-1 hover:bg-accent rounded"
            >
              {list.title}
            </button>
          )}

          <span className="text-xs text-muted-foreground ml-1">
            {list.cards.length}
          </span>
        </div>

        <div className="relative">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => setShowMenu(!showMenu)}
          >
            <MoreHorizontal className="h-4 w-4" />
          </Button>

          {showMenu && (
            <>
              <div className="fixed inset-0 z-10" onClick={() => setShowMenu(false)} />
              <div className="absolute right-0 top-8 z-20 w-48 bg-popover border rounded-md shadow-lg p-1">
                <button
                  onClick={() => {
                    setShowMenu(false);
                    setIsAdding(true);
                  }}
                  className="flex items-center gap-2 w-full rounded-sm px-2 py-1.5 text-sm hover:bg-accent"
                >
                  <Plus className="h-4 w-4" />
                  Add Card
                </button>
                <button
                  onClick={() => {
                    setShowMenu(false);
                    onDeleteList(list.id);
                  }}
                  className="flex items-center gap-2 w-full rounded-sm px-2 py-1.5 text-sm hover:bg-accent text-destructive"
                >
                  <Trash2 className="h-4 w-4" />
                  Delete List
                </button>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Cards */}
      <div
        ref={setDroppableRef}
        className="flex-1 overflow-y-auto px-2 pb-2 space-y-2 min-h-[2rem]"
      >
        <SortableContext items={cardIds} strategy={verticalListSortingStrategy}>
          {list.cards.map((card) => (
            <BoardCard
              key={card.id}
              card={card}
              labels={labels}
              onClick={() => onCardClick(card.id)}
            />
          ))}
        </SortableContext>
      </div>

      {/* Add card */}
      <div className="px-2 pb-2">
        {isAdding ? (
          <div className="space-y-2">
            <textarea
              value={newCardTitle}
              onChange={(e) => setNewCardTitle(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  handleAddCard();
                }
                if (e.key === 'Escape') {
                  setIsAdding(false);
                  setNewCardTitle('');
                }
              }}
              placeholder="Enter a title for this card..."
              className="w-full rounded-md border bg-card p-2 text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary"
              rows={3}
              autoFocus
            />
            <div className="flex items-center gap-2">
              <Button size="sm" onClick={handleAddCard} disabled={!newCardTitle.trim()}>
                Add Card
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8"
                onClick={() => {
                  setIsAdding(false);
                  setNewCardTitle('');
                }}
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
          </div>
        ) : (
          <Button
            variant="ghost"
            className="w-full justify-start text-muted-foreground h-8"
            onClick={() => setIsAdding(true)}
          >
            <Plus className="h-4 w-4 mr-1" />
            Add a card
          </Button>
        )}
      </div>
    </div>
  );
}
