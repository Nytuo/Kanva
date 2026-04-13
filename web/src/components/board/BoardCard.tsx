import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import {
  Clock,
  MessageSquare,
  CheckSquare,
  User,
} from 'lucide-react';
import { cn, priorityColors, formatDate } from '@/lib/utils';
import type { CardSummary, Label } from '@/types';

interface BoardCardProps {
  card: CardSummary;
  labels: Label[];
  onClick: () => void;
}

export default function BoardCard({ card, labels, onClick }: BoardCardProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: card.id,
    data: { type: 'card', card },
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  const cardLabels = labels.filter((l) => card.label_ids.includes(l.id));

  const isOverdue =
    card.due_date && new Date(card.due_date) < new Date();

  // Parse checklist progress "3/5" format
  const checklistParts = card.checklist_progress?.split('/');
  const checklistDone = checklistParts ? parseInt(checklistParts[0], 10) : 0;
  const checklistTotal = checklistParts ? parseInt(checklistParts[1], 10) : 0;

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      onClick={onClick}
      className={cn(
        'group bg-card border rounded-lg p-3 cursor-pointer hover:ring-2 hover:ring-primary/20 transition-all',
        isDragging && 'opacity-50 rotate-2 shadow-lg z-50',
      )}
    >
      {/* Cover color */}
      {card.cover_color && (
        <div
          className="h-8 -mx-3 -mt-3 mb-2 rounded-t-lg"
          style={{ backgroundColor: card.cover_color }}
        />
      )}

      {/* Labels */}
      {cardLabels.length > 0 && (
        <div className="flex flex-wrap gap-1 mb-2">
          {cardLabels.map((label) => (
            <span
              key={label.id}
              className="inline-block h-2 w-10 rounded-full"
              style={{ backgroundColor: label.color }}
              title={label.name}
            />
          ))}
        </div>
      )}

      {/* Title */}
      <p className="text-sm font-medium leading-snug">{card.title}</p>

      {/* Badges */}
      <div className="flex flex-wrap items-center gap-2 mt-2 text-xs text-muted-foreground">
        {/* Priority */}
        {card.priority !== 'none' && (
          <span
            className={cn(
              'px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase',
              priorityColors[card.priority],
            )}
          >
            {card.priority}
          </span>
        )}

        {/* Due date */}
        {card.due_date && (
          <span
            className={cn(
              'flex items-center gap-1',
              isOverdue && 'text-red-500 font-medium',
            )}
          >
            <Clock className="h-3 w-3" />
            {formatDate(card.due_date)}
          </span>
        )}

        {/* Comments */}
        {card.comment_count > 0 && (
          <span className="flex items-center gap-1">
            <MessageSquare className="h-3 w-3" />
            {card.comment_count}
          </span>
        )}

        {/* Checklist */}
        {card.checklist_progress && checklistTotal > 0 && (
          <span
            className={cn(
              'flex items-center gap-1',
              checklistDone === checklistTotal && 'text-green-600',
            )}
          >
            <CheckSquare className="h-3 w-3" />
            {card.checklist_progress}
          </span>
        )}

        {/* Assignees */}
        {card.assignee_count > 0 && (
          <span className="flex items-center gap-1 ml-auto">
            <User className="h-3 w-3" />
            {card.assignee_count}
          </span>
        )}
      </div>
    </div>
  );
}
