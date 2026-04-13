import { useState, useCallback, useMemo, useRef, useEffect } from 'react';
import {
  DndContext,
  DragOverlay,
  closestCorners,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragStartEvent,
  type DragEndEvent,
  type DragOverEvent,
} from '@dnd-kit/core';
import {
  SortableContext,
  horizontalListSortingStrategy,
  sortableKeyboardCoordinates,
  arrayMove,
} from '@dnd-kit/sortable';
import { Plus, X, GripVertical } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import BoardList from './BoardList';
import BoardCard from './BoardCard';
import CardDetailDialog from './CardDetailDialog';
import {
  useCreateList,
  useCreateCard,
  useMoveCard,
  useMoveList,
  useDeleteList,
  useUpdateList,
} from '@/hooks/useApi';
import { useWebSocket } from '@/hooks/useWebSocket';
import type { Board, CardSummary, List } from '@/types';
import type { BoardFilters } from '@/pages/BoardPage';

interface KanbanBoardProps {
  board: Board;
  filters: BoardFilters;
  onRefresh: () => void;
}

function filterCards(cards: CardSummary[], filters: BoardFilters): CardSummary[] {
  const now = new Date();
  const threeDaysFromNow = new Date(now.getTime() + 3 * 24 * 60 * 60 * 1000);

  return cards.filter((card) => {
    if (filters.search) {
      if (!card.title.toLowerCase().includes(filters.search.toLowerCase())) return false;
    }
    if (filters.priority) {
      if (card.priority !== filters.priority) return false;
    }
    if (filters.labelId) {
      if (!card.label_ids.includes(filters.labelId)) return false;
    }
    // assigneeId filter requires full card data; CardSummary only has assignee_count
    if (filters.dueSoon) {
      if (!card.due_date) return false;
      const due = new Date(card.due_date);
      if (due < now || due > threeDaysFromNow) return false;
    }
    if (filters.overdue) {
      if (!card.due_date) return false;
      const due = new Date(card.due_date);
      if (due >= now) return false;
    }
    return true;
  });
}

/** Find which list a card belongs to */
function findListByCardId(lists: List[], cardId: string): List | undefined {
  return lists.find((l) => l.cards.some((c) => c.id === cardId));
}

export default function KanbanBoard({ board, filters, onRefresh }: KanbanBoardProps) {
  const [activeCard, setActiveCard] = useState<CardSummary | null>(null);
  const [activeList, setActiveList] = useState<List | null>(null);
  const [selectedCardId, setSelectedCardId] = useState<string | null>(null);
  const [addingList, setAddingList] = useState(false);
  const [newListTitle, setNewListTitle] = useState('');

  // Local list state for optimistic DnD reordering
  const [localLists, setLocalLists] = useState<List[] | null>(null);
  // Track the original card position so we know what to send on dragEnd
  const dragOriginRef = useRef<{ cardId: string; listId: string; position: number } | null>(null);
  // Track whether we're mid-drag so we don't clear localLists from the effect
  const isDraggingRef = useRef(false);

  const createList = useCreateList();
  const createCard = useCreateCard();
  const moveCard = useMoveCard();
  const moveList = useMoveList();
  const deleteList = useDeleteList();
  const updateList = useUpdateList();

  // WebSocket for real-time updates
  const handleWsMessage = useCallback(
    (msg: { event: string; data: unknown }) => {
      if (
        [
          'card_created',
          'card_moved',
          'card_updated',
          'card_deleted',
          'list_created',
          'list_moved',
          'list_deleted',
        ].includes(msg.event)
      ) {
        onRefresh();
      }
    },
    [onRefresh],
  );

  useWebSocket(board.id, handleWsMessage);

  // Clear optimistic localLists once server data arrives (board.lists changes)
  // but only if we're not currently mid-drag
  useEffect(() => {
    if (localLists && !isDraggingRef.current) {
      setLocalLists(null);
    }
  }, [board.lists]);

  // Use local lists during drag, board.lists otherwise
  const activeLists = localLists ?? board.lists;

  // Apply filters to cards in each list
  const isFiltering = !!(
    filters.search || filters.priority || filters.labelId ||
    filters.assigneeId || filters.dueSoon || filters.overdue
  );
  const filteredLists = useMemo<List[]>(() => {
    if (!isFiltering) return activeLists;
    return activeLists.map((list) => ({
      ...list,
      cards: filterCards(list.cards, filters),
    }));
  }, [activeLists, filters, isFiltering]);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 8 },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );

  const handleDragStart = (event: DragStartEvent) => {
    const { active } = event;
    const data = active.data.current;
    isDraggingRef.current = true;

    if (data?.type === 'card') {
      setActiveCard(data.card);
      // Snapshot lists for optimistic reorder
      setLocalLists(board.lists.map((l) => ({ ...l, cards: [...l.cards] })));
      const sourceList = findListByCardId(board.lists, data.card.id);
      if (sourceList) {
        const idx = sourceList.cards.findIndex((c) => c.id === data.card.id);
        dragOriginRef.current = { cardId: data.card.id, listId: sourceList.id, position: idx };
      }
    } else if (data?.type === 'list') {
      setActiveList(data.list);
      setLocalLists(board.lists.map((l) => ({ ...l, cards: [...l.cards] })));
    }
  };

  const handleDragOver = (event: DragOverEvent) => {
    const { active, over } = event;
    if (!over || !localLists) return;

    const activeData = active.data.current;
    const overData = over.data.current;

    // Only handle card-over-card or card-over-list movement
    if (activeData?.type !== 'card') return;

    const activeCardId = active.id as string;

    // Find source list in localLists
    const sourceList = localLists.find((l) => l.cards.some((c) => c.id === activeCardId));
    if (!sourceList) return;

    let targetList: List | undefined;
    let targetIdx: number;

    if (overData?.type === 'card') {
      const overCardId = over.id as string;
      targetList = localLists.find((l) => l.cards.some((c) => c.id === overCardId));
      if (!targetList) return;
      targetIdx = targetList.cards.findIndex((c) => c.id === overCardId);
    } else if (overData?.type === 'list') {
      const overListId = overData.listId as string;
      targetList = localLists.find((l) => l.id === overListId);
      if (!targetList) return;
      targetIdx = targetList.cards.length; // Append to end
    } else {
      return;
    }

    // Same list, same position — no-op
    if (sourceList.id === targetList.id) {
      const sourceIdx = sourceList.cards.findIndex((c) => c.id === activeCardId);
      if (sourceIdx === targetIdx) return;
      // Reorder within same list
      setLocalLists((prev) =>
        prev!.map((l) => {
          if (l.id !== sourceList.id) return l;
          return { ...l, cards: arrayMove(l.cards, sourceIdx, targetIdx) };
        }),
      );
      return;
    }

    // Moving between lists
    const cardToMove = sourceList.cards.find((c) => c.id === activeCardId)!;
    setLocalLists((prev) =>
      prev!.map((l) => {
        if (l.id === sourceList.id) {
          return { ...l, cards: l.cards.filter((c) => c.id !== activeCardId) };
        }
        if (l.id === targetList!.id) {
          const newCards = [...l.cards];
          newCards.splice(targetIdx, 0, cardToMove);
          return { ...l, cards: newCards };
        }
        return l;
      }),
    );
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    const wasCard = !!activeCard;
    const wasList = !!activeList;

    setActiveCard(null);
    setActiveList(null);
    isDraggingRef.current = false;

    if (!over) {
      setLocalLists(null);
      dragOriginRef.current = null;
      return;
    }

    const activeData = active.data.current;

    if (wasCard && activeData?.type === 'card' && localLists) {
      // Find where the card ended up in localLists
      const finalList = localLists.find((l) => l.cards.some((c) => c.id === active.id));
      if (finalList) {
        const finalIdx = finalList.cards.findIndex((c) => c.id === active.id);
        // Only call API if something actually changed
        const origin = dragOriginRef.current;
        if (origin && (origin.listId !== finalList.id || origin.position !== finalIdx)) {
          // Keep localLists alive — the useEffect will clear it when board.lists refreshes
          moveCard.mutate({
            cardId: active.id as string,
            list_id: finalList.id,
            position: finalIdx,
          });
        } else {
          // Nothing changed, clear immediately
          setLocalLists(null);
        }
      } else {
        setLocalLists(null);
      }
    } else if (wasList && activeData?.type === 'list') {
      const activeId = (active.id as string).replace('list-', '');
      const overId = (over.id as string).replace('list-', '');

      if (activeId !== overId) {
        const overIndex = board.lists.findIndex((l) => l.id === overId);
        if (overIndex !== -1) {
          moveList.mutate({
            listId: activeId,
            position: overIndex,
            boardId: board.id,
          });
        }
      }
      setLocalLists(null);
    } else {
      setLocalLists(null);
    }

    dragOriginRef.current = null;
  };

  const handleCreateList = () => {
    if (!newListTitle.trim()) return;
    createList.mutate({ board_id: board.id, title: newListTitle.trim() });
    setNewListTitle('');
    setAddingList(false);
  };

  const handleAddCard = (listId: string, title: string) => {
    createCard.mutate({ list_id: listId, title });
  };

  const handleDeleteList = (listId: string) => {
    if (window.confirm('Delete this list and all its cards?')) {
      deleteList.mutate({ listId, boardId: board.id });
    }
  };

  const handleRenameList = (listId: string, title: string) => {
    if (!title.trim()) return;
    updateList.mutate({ listId, title: title.trim() });
  };

  const listIds = filteredLists.map((l) => `list-${l.id}`);

  return (
    <>
      <DndContext
        sensors={sensors}
        collisionDetection={closestCorners}
        onDragStart={handleDragStart}
        onDragOver={handleDragOver}
        onDragEnd={handleDragEnd}
      >
        <div className="flex gap-4 p-4 overflow-x-auto h-full items-start">
          <SortableContext items={listIds} strategy={horizontalListSortingStrategy}>
            {filteredLists.map((list) => (
              <BoardList
                key={list.id}
                list={list}
                labels={board.labels}
                onAddCard={handleAddCard}
                onCardClick={(cardId) => setSelectedCardId(cardId)}
                onDeleteList={handleDeleteList}
                onRenameList={handleRenameList}
              />
            ))}
          </SortableContext>

          {/* Add list */}
          <div className="flex-shrink-0 w-72">
            {addingList ? (
              <div className="bg-muted/50 rounded-xl p-3 space-y-2">
                <Input
                  value={newListTitle}
                  onChange={(e) => setNewListTitle(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') handleCreateList();
                    if (e.key === 'Escape') {
                      setAddingList(false);
                      setNewListTitle('');
                    }
                  }}
                  placeholder="Enter list title..."
                  autoFocus
                />
                <div className="flex items-center gap-2">
                  <Button size="sm" onClick={handleCreateList} disabled={!newListTitle.trim()}>
                    Add List
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8"
                    onClick={() => {
                      setAddingList(false);
                      setNewListTitle('');
                    }}
                  >
                    <X className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            ) : (
              <Button
                variant="outline"
                className="w-full justify-start bg-muted/30 border-dashed"
                onClick={() => setAddingList(true)}
              >
                <Plus className="h-4 w-4 mr-2" />
                Add another list
              </Button>
            )}
          </div>
        </div>

        {/* Drag overlay */}
        <DragOverlay dropAnimation={null}>
          {activeCard && (
            <div className="rotate-3 shadow-xl">
              <BoardCard
                card={activeCard}
                labels={board.labels}
                onClick={() => {}}
              />
            </div>
          )}
          {activeList && (
            <div className="rotate-2 shadow-xl opacity-90 w-72 bg-muted/50 rounded-xl p-3">
              <div className="flex items-center gap-1 mb-2">
                <GripVertical className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-semibold">{activeList.title}</span>
                <span className="text-xs text-muted-foreground ml-1">
                  {activeList.cards.length}
                </span>
              </div>
              <div className="space-y-2 max-h-40 overflow-hidden">
                {activeList.cards.slice(0, 3).map((card) => (
                  <div key={card.id} className="bg-card border rounded-lg p-2">
                    <p className="text-xs font-medium truncate">{card.title}</p>
                  </div>
                ))}
                {activeList.cards.length > 3 && (
                  <p className="text-xs text-muted-foreground text-center">
                    +{activeList.cards.length - 3} more
                  </p>
                )}
              </div>
            </div>
          )}
        </DragOverlay>
      </DndContext>

      {/* Card detail dialog */}
      <CardDetailDialog
        cardId={selectedCardId}
        boardLabels={board.labels}
        onClose={() => setSelectedCardId(null)}
      />
    </>
  );
}
