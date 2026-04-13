import { useState, useMemo, useCallback, useEffect, useRef } from 'react';
import {
  ChevronLeft,
  ChevronRight,
  Plus,
  Calendar as CalendarIcon,
  CreditCard,
  Trash2,
} from 'lucide-react';
import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  type DragStartEvent,
  type DragEndEvent,
} from '@dnd-kit/core';
import { useDroppable, useDraggable } from '@dnd-kit/core';
import {
  addMonths,
  subMonths,
  startOfMonth,
  endOfMonth,
  startOfWeek,
  endOfWeek,
  eachDayOfInterval,
  format,
  isSameMonth,
  isSameDay,
  isToday,
  addDays,
  differenceInCalendarDays,
} from 'date-fns';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog';
import {
  useCalendarEvents,
  useCreateCalendarEvent,
  useUpdateCalendarEvent,
  useDeleteCalendarEvent,
} from '@/hooks/useApi';
import { cn } from '@/lib/utils';
import type { CalendarEvent } from '@/types';

const eventColors = [
  '#3b82f6',
  '#ef4444',
  '#22c55e',
  '#f97316',
  '#8b5cf6',
  '#06b6d4',
  '#ec4899',
];

// Draggable event chip
function DraggableEvent({ event, onEdit }: { event: CalendarEvent; onEdit: (event: CalendarEvent) => void }) {
  const isCard = !!event.card_id;
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: event.id,
    data: { event },
    disabled: isCard, // card-based events can't be dragged (managed via card due_date)
  });

  return (
    <div
      ref={setNodeRef}
      {...listeners}
      {...attributes}
      className={cn(
        'text-[10px] px-1.5 py-0.5 rounded truncate font-medium flex items-center gap-0.5',
        isDragging && 'opacity-40',
        isCard
          ? 'text-blue-700 dark:text-blue-300 bg-blue-100 dark:bg-blue-900/40 border border-blue-200 dark:border-blue-800'
          : 'text-white cursor-grab active:cursor-grabbing',
      )}
      style={isCard ? undefined : { backgroundColor: event.color || '#3b82f6' }}
      onClick={(e) => {
        e.stopPropagation();
        onEdit(event);
      }}
    >
      {isCard && <CreditCard className="h-2.5 w-2.5 flex-shrink-0" />}
      <span className="truncate">{event.title}</span>
    </div>
  );
}

// Droppable day cell
function DayCell({
  day,
  events,
  isCurrentMonth,
  onClick,
  onEditEvent,
}: {
  day: Date;
  events: CalendarEvent[];
  isCurrentMonth: boolean;
  onClick: () => void;
  onEditEvent: (event: CalendarEvent) => void;
}) {
  const today = isToday(day);
  const { setNodeRef, isOver } = useDroppable({
    id: `day-${day.toISOString()}`,
    data: { day },
  });

  return (
    <div
      ref={setNodeRef}
      className={cn(
        'min-h-[100px] border-b border-r p-1 cursor-pointer hover:bg-accent/30 transition-colors',
        !isCurrentMonth && 'bg-muted/20 text-muted-foreground',
        isOver && 'bg-primary/10 ring-1 ring-inset ring-primary/40',
      )}
      onClick={onClick}
    >
      <div className="flex items-center justify-between mb-1">
        <span
          className={cn(
            'text-xs font-medium h-6 w-6 flex items-center justify-center rounded-full',
            today && 'bg-primary text-primary-foreground',
          )}
        >
          {format(day, 'd')}
        </span>
      </div>
      <div className="space-y-0.5">
        {events.slice(0, 3).map((event) => (
          <DraggableEvent key={event.id} event={event} onEdit={onEditEvent} />
        ))}
        {events.length > 3 && (
          <div className="text-[10px] text-muted-foreground px-1">
            +{events.length - 3} more
          </div>
        )}
      </div>
    </div>
  );
}

export default function CalendarPage() {
  const [currentMonth, setCurrentMonth] = useState(new Date());
  const [selectedDate, setSelectedDate] = useState<Date | null>(null);
  const [showCreate, setShowCreate] = useState(false);
  const [activeEvent, setActiveEvent] = useState<CalendarEvent | null>(null);
  const [editingEvent, setEditingEvent] = useState<CalendarEvent | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [newEvent, setNewEvent] = useState({
    title: '',
    description: '',
    date: '',
    start_time: '',
    end_time: '',
    all_day: true,
    color: eventColors[0],
  });
  const [editForm, setEditForm] = useState({
    title: '',
    description: '',
    date: '',
    start_time: '',
    end_time: '',
    all_day: true,
    color: eventColors[0],
  });

  // Optimistic local events state (mirrors localLists pattern from KanbanBoard)
  const [localEvents, setLocalEvents] = useState<CalendarEvent[] | null>(null);
  const isDraggingRef = useRef(false);

  const monthStart = startOfMonth(currentMonth);
  const monthEnd = endOfMonth(currentMonth);
  const calStart = startOfWeek(monthStart);
  const calEnd = endOfWeek(monthEnd);

  const { data: events = [] } = useCalendarEvents(
    calStart.toISOString(),
    calEnd.toISOString(),
  );

  const createEvent = useCreateCalendarEvent();
  const updateEvent = useUpdateCalendarEvent();
  const deleteEvent = useDeleteCalendarEvent();

  // Clear optimistic localEvents once server data arrives (events changes) and not dragging
  useEffect(() => {
    if (localEvents && !isDraggingRef.current) {
      setLocalEvents(null);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [events]);

  // Use local events when available (during/after drag), otherwise server data
  const activeEvents = localEvents ?? events;

  const days = useMemo(
    () => eachDayOfInterval({ start: calStart, end: calEnd }),
    [calStart.getTime(), calEnd.getTime()],
  );

  const getEventsForDay = useCallback(
    (day: Date): CalendarEvent[] =>
      activeEvents.filter((e) => {
        const eventDate = new Date(e.start_time);
        return isSameDay(eventDate, day);
      }),
    [activeEvents],
  );

  // DnD sensors
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
  );

  const handleDragStart = (event: DragStartEvent) => {
    const data = event.active.data.current;
    if (data?.event) {
      setActiveEvent(data.event as CalendarEvent);
      isDraggingRef.current = true;
      // Snapshot events for optimistic updates
      setLocalEvents([...events]);
    }
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    setActiveEvent(null);
    isDraggingRef.current = false;

    if (!over) {
      setLocalEvents(null);
      return;
    }

    const draggedEvent = active.data.current?.event as CalendarEvent | undefined;
    const targetDay = over.data.current?.day as Date | undefined;
    if (!draggedEvent || !targetDay) {
      setLocalEvents(null);
      return;
    }
    if (draggedEvent.card_id) {
      setLocalEvents(null);
      return;
    }

    const oldStart = new Date(draggedEvent.start_time);
    const oldEnd = new Date(draggedEvent.end_time);
    if (isSameDay(oldStart, targetDay)) {
      setLocalEvents(null);
      return;
    }

    const dayDiff = differenceInCalendarDays(targetDay, oldStart);
    const newStart = addDays(oldStart, dayDiff);
    const newEnd = addDays(oldEnd, dayDiff);

    // Optimistically update localEvents so the event renders in the new day immediately
    setLocalEvents((prev) =>
      (prev ?? events).map((e) =>
        e.id === draggedEvent.id
          ? { ...e, start_time: newStart.toISOString(), end_time: newEnd.toISOString() }
          : e,
      ),
    );

    // Fire the mutation — localEvents will be cleared when React Query refetches
    updateEvent.mutate({
      id: draggedEvent.id,
      start_time: newStart.toISOString(),
      end_time: newEnd.toISOString(),
    });
  };

  const openCreateDialog = (day: Date) => {
    setSelectedDate(day);
    setNewEvent({
      title: '',
      description: '',
      date: format(day, 'yyyy-MM-dd'),
      start_time: '',
      end_time: '',
      all_day: true,
      color: eventColors[0],
    });
    setShowCreate(true);
  };

  const handleCreateEvent = () => {
    if (!newEvent.title.trim()) return;
    const eventDate = newEvent.date || (selectedDate ? format(selectedDate, 'yyyy-MM-dd') : '');
    if (!eventDate) return;

    const startTime = newEvent.all_day
      ? new Date(`${eventDate}T00:00:00`).toISOString()
      : new Date(`${eventDate}T${newEvent.start_time || '09:00'}`).toISOString();
    const endTime = newEvent.all_day
      ? new Date(`${eventDate}T23:59:59`).toISOString()
      : new Date(`${eventDate}T${newEvent.end_time || '10:00'}`).toISOString();

    createEvent.mutate({
      title: newEvent.title,
      description: newEvent.description || undefined,
      start_time: startTime,
      end_time: endTime,
      all_day: newEvent.all_day,
      color: newEvent.color,
    });

    setShowCreate(false);
  };

  // Edit dialog handlers
  const openEditDialog = (event: CalendarEvent) => {
    setEditingEvent(event);
    const startDt = new Date(event.start_time);
    const endDt = new Date(event.end_time);
    setEditForm({
      title: event.title,
      description: event.description || '',
      date: format(startDt, 'yyyy-MM-dd'),
      start_time: format(startDt, 'HH:mm'),
      end_time: format(endDt, 'HH:mm'),
      all_day: event.all_day,
      color: event.color || eventColors[0],
    });
    setShowDeleteConfirm(false);
  };

  const handleUpdateEvent = () => {
    if (!editingEvent || !editForm.title.trim() || !editForm.date) return;
    const isCard = !!editingEvent.card_id;
    if (isCard) return; // card events not editable here

    const startTime = editForm.all_day
      ? new Date(`${editForm.date}T00:00:00`).toISOString()
      : new Date(`${editForm.date}T${editForm.start_time || '09:00'}`).toISOString();
    const endTime = editForm.all_day
      ? new Date(`${editForm.date}T23:59:59`).toISOString()
      : new Date(`${editForm.date}T${editForm.end_time || '10:00'}`).toISOString();

    updateEvent.mutate({
      id: editingEvent.id,
      title: editForm.title,
      description: editForm.description || undefined,
      start_time: startTime,
      end_time: endTime,
      color: editForm.color,
    });

    setEditingEvent(null);
  };

  const handleDeleteEvent = () => {
    if (!editingEvent) return;
    deleteEvent.mutate(editingEvent.id);
    setEditingEvent(null);
    setShowDeleteConfirm(false);
  };

  const weekDays = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

  return (
    <div className="flex flex-col h-full p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <CalendarIcon className="h-6 w-6" />
            Calendar
          </h1>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="icon"
              className="h-8 w-8"
              onClick={() => setCurrentMonth(subMonths(currentMonth, 1))}
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <h2 className="text-lg font-semibold min-w-[180px] text-center">
              {format(currentMonth, 'MMMM yyyy')}
            </h2>
            <Button
              variant="outline"
              size="icon"
              className="h-8 w-8"
              onClick={() => setCurrentMonth(addMonths(currentMonth, 1))}
            >
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setCurrentMonth(new Date())}
          >
            Today
          </Button>
        </div>
        <Button
          size="sm"
          onClick={() => openCreateDialog(new Date())}
        >
          <Plus className="h-4 w-4 mr-1" />
          New Event
        </Button>
      </div>

      {/* Calendar grid with DnD */}
      <DndContext
        sensors={sensors}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
      >
        <div className="flex-1 border rounded-lg overflow-hidden">
          {/* Day headers */}
          <div className="grid grid-cols-7 border-b bg-muted/50">
            {weekDays.map((day) => (
              <div
                key={day}
                className="px-2 py-2 text-xs font-semibold text-muted-foreground text-center"
              >
                {day}
              </div>
            ))}
          </div>

          {/* Day cells */}
          <div className="grid grid-cols-7 flex-1" style={{ gridAutoRows: '1fr' }}>
            {days.map((day) => (
              <DayCell
                key={day.toISOString()}
                day={day}
                events={getEventsForDay(day)}
                isCurrentMonth={isSameMonth(day, currentMonth)}
                onClick={() => openCreateDialog(day)}
                onEditEvent={openEditDialog}
              />
            ))}
          </div>
        </div>

        {/* Drag overlay */}
        <DragOverlay>
          {activeEvent && (
            <div
              className="text-[10px] px-1.5 py-0.5 rounded truncate text-white font-medium shadow-lg"
              style={{ backgroundColor: activeEvent.color || '#3b82f6' }}
            >
              {activeEvent.title}
            </div>
          )}
        </DragOverlay>
      </DndContext>

      {/* Create event dialog */}
      <Dialog open={showCreate} onOpenChange={setShowCreate}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create Event</DialogTitle>
            <DialogDescription>
              Add a new calendar event
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium">Title</label>
              <Input
                value={newEvent.title}
                onChange={(e) =>
                  setNewEvent({ ...newEvent, title: e.target.value })
                }
                placeholder="Event title"
                autoFocus
              />
            </div>

            <div>
              <label className="text-sm font-medium">Description</label>
              <textarea
                value={newEvent.description}
                onChange={(e) =>
                  setNewEvent({ ...newEvent, description: e.target.value })
                }
                placeholder="Optional description..."
                className="w-full rounded-md border bg-background p-2 text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary"
                rows={3}
              />
            </div>

            <div>
              <label className="text-sm font-medium">Date</label>
              <Input
                type="date"
                value={newEvent.date}
                onChange={(e) =>
                  setNewEvent({ ...newEvent, date: e.target.value })
                }
              />
            </div>

            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="allDay"
                checked={newEvent.all_day}
                onChange={(e) =>
                  setNewEvent({ ...newEvent, all_day: e.target.checked })
                }
                className="rounded"
              />
              <label htmlFor="allDay" className="text-sm">
                All day
              </label>
            </div>

            {!newEvent.all_day && (
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-sm font-medium">Start Time</label>
                  <Input
                    type="time"
                    value={newEvent.start_time}
                    onChange={(e) =>
                      setNewEvent({ ...newEvent, start_time: e.target.value })
                    }
                  />
                </div>
                <div>
                  <label className="text-sm font-medium">End Time</label>
                  <Input
                    type="time"
                    value={newEvent.end_time}
                    onChange={(e) =>
                      setNewEvent({ ...newEvent, end_time: e.target.value })
                    }
                  />
                </div>
              </div>
            )}

            <div>
              <label className="text-sm font-medium">Color</label>
              <div className="flex gap-2 mt-2">
                {eventColors.map((c) => (
                  <button
                    key={c}
                    className={cn(
                      'h-7 w-7 rounded-full',
                      newEvent.color === c &&
                        'ring-2 ring-primary ring-offset-2',
                    )}
                    style={{ backgroundColor: c }}
                    onClick={() => setNewEvent({ ...newEvent, color: c })}
                  />
                ))}
              </div>
            </div>

            <Button
              onClick={handleCreateEvent}
              className="w-full"
              disabled={!newEvent.title.trim() || !newEvent.date}
            >
              Create Event
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      {/* Edit event dialog */}
      <Dialog open={!!editingEvent} onOpenChange={(open) => { if (!open) setEditingEvent(null); }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {editingEvent?.card_id ? 'Card Event' : 'Edit Event'}
            </DialogTitle>
            <DialogDescription>
              {editingEvent?.card_id
                ? 'This event is linked to a card. Edit it from the board.'
                : 'Update or delete this calendar event'}
            </DialogDescription>
          </DialogHeader>

          {editingEvent?.card_id ? (
            // Card event — read-only view
            <div className="space-y-3">
              <div>
                <label className="text-sm font-medium text-muted-foreground">Title</label>
                <p className="text-sm flex items-center gap-1.5">
                  <CreditCard className="h-3.5 w-3.5 text-blue-500" />
                  {editingEvent.title}
                </p>
              </div>
              <div>
                <label className="text-sm font-medium text-muted-foreground">Date</label>
                <p className="text-sm">{format(new Date(editingEvent.start_time), 'PPP')}</p>
              </div>
              <Button variant="outline" className="w-full" onClick={() => setEditingEvent(null)}>
                Close
              </Button>
            </div>
          ) : (
            // Regular event — editable
            <div className="space-y-4">
              <div>
                <label className="text-sm font-medium">Title</label>
                <Input
                  value={editForm.title}
                  onChange={(e) => setEditForm({ ...editForm, title: e.target.value })}
                  autoFocus
                />
              </div>

              <div>
                <label className="text-sm font-medium">Description</label>
                <textarea
                  value={editForm.description}
                  onChange={(e) => setEditForm({ ...editForm, description: e.target.value })}
                  placeholder="Optional description..."
                  className="w-full rounded-md border bg-background p-2 text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary"
                  rows={3}
                />
              </div>

              <div>
                <label className="text-sm font-medium">Date</label>
                <Input
                  type="date"
                  value={editForm.date}
                  onChange={(e) => setEditForm({ ...editForm, date: e.target.value })}
                />
              </div>

              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="editAllDay"
                  checked={editForm.all_day}
                  onChange={(e) => setEditForm({ ...editForm, all_day: e.target.checked })}
                  className="rounded"
                />
                <label htmlFor="editAllDay" className="text-sm">All day</label>
              </div>

              {!editForm.all_day && (
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="text-sm font-medium">Start Time</label>
                    <Input
                      type="time"
                      value={editForm.start_time}
                      onChange={(e) => setEditForm({ ...editForm, start_time: e.target.value })}
                    />
                  </div>
                  <div>
                    <label className="text-sm font-medium">End Time</label>
                    <Input
                      type="time"
                      value={editForm.end_time}
                      onChange={(e) => setEditForm({ ...editForm, end_time: e.target.value })}
                    />
                  </div>
                </div>
              )}

              <div>
                <label className="text-sm font-medium">Color</label>
                <div className="flex gap-2 mt-2">
                  {eventColors.map((c) => (
                    <button
                      key={c}
                      className={cn(
                        'h-7 w-7 rounded-full',
                        editForm.color === c && 'ring-2 ring-primary ring-offset-2',
                      )}
                      style={{ backgroundColor: c }}
                      onClick={() => setEditForm({ ...editForm, color: c })}
                    />
                  ))}
                </div>
              </div>

              <div className="flex gap-2">
                <Button
                  onClick={handleUpdateEvent}
                  className="flex-1"
                  disabled={!editForm.title.trim() || !editForm.date}
                >
                  Save Changes
                </Button>
                {!showDeleteConfirm ? (
                  <Button
                    variant="destructive"
                    size="icon"
                    onClick={() => setShowDeleteConfirm(true)}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                ) : (
                  <Button
                    variant="destructive"
                    onClick={handleDeleteEvent}
                  >
                    Confirm Delete
                  </Button>
                )}
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
