import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { useServerStore } from '@/store/server';
import { useAuthStore } from '@/store/auth';
import type { Board, BoardSummary, Card, Team, TeamMember, CalendarEvent, UserPreferences, User, Note } from '@/types';

// ====== SERVER INFO ======
export function useServerInfo() {
  const activeServerId = useServerStore((s) => s.activeServerId);
  return useQuery({
    queryKey: ['server-info', activeServerId],
    queryFn: async () => (await api.get('/server-info')).data,
    enabled: !!activeServerId,
    staleTime: 5 * 60 * 1000, // cache for 5 min
  });
}

// ====== BOARDS ======
export function useBoards(params?: Record<string, string>) {
  const activeServerId = useServerStore((s) => s.activeServerId);
  return useQuery<BoardSummary[]>({
    queryKey: ['boards', activeServerId, params],
    queryFn: async () => (await api.get('/boards', { params })).data,
    enabled: !!activeServerId,
  });
}

export function useBoard(id: string | undefined) {
  const activeServerId = useServerStore((s) => s.activeServerId);
  return useQuery<Board>({
    queryKey: ['board', activeServerId, id],
    queryFn: async () => (await api.get(`/boards/${id}`)).data,
    enabled: !!id && !!activeServerId,
  });
}

export function useCreateBoard() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (data: { title: string; description?: string; visibility?: string; team_id?: string; background_color?: string; background_image_url?: string }) =>
      (await api.post('/boards', data)).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['boards'] }),
  });
}

export function useUpdateBoard(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (data: Partial<Board>) => (await api.put(`/boards/${id}`, data)).data,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['board'] });
      qc.invalidateQueries({ queryKey: ['boards'] });
    },
  });
}

export function useDeleteBoard() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => (await api.delete(`/boards/${id}`)).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['boards'] }),
  });
}

export function useToggleStar(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async () => (await api.post(`/boards/${id}/star`)).data,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['boards'] });
      qc.invalidateQueries({ queryKey: ['board'] });
    },
  });
}

// ====== LISTS ======
export function useCreateList() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (data: { board_id: string; title: string }) =>
      (await api.post('/lists', data)).data,
    onSuccess: (_data, _variables) =>
      qc.invalidateQueries({ queryKey: ['board'] }),
  });
}

export function useMoveList() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ listId, position }: { listId: string; position: number; boardId: string }) =>
      (await api.post(`/lists/${listId}/move`, { position })).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['board'] }),
  });
}

export function useDeleteList() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ listId }: { listId: string; boardId: string }) =>
      (await api.delete(`/lists/${listId}`)).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['board'] }),
  });
}

export function useUpdateList() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ listId, title }: { listId: string; title: string }) =>
      (await api.put(`/lists/${listId}`, { title })).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['board'] }),
  });
}

// ====== CARDS ======
export function useCard(id: string | undefined) {
  const activeServerId = useServerStore((s) => s.activeServerId);
  return useQuery<Card>({
    queryKey: ['card', activeServerId, id],
    queryFn: async () => (await api.get(`/cards/${id}`)).data,
    enabled: !!id && !!activeServerId,
  });
}

export function useCreateCard() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (data: { list_id: string; title: string; description?: string; priority?: string; due_date?: string }) =>
      (await api.post('/cards', data)).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['board'] }),
  });
}

export function useMoveCard() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ cardId, list_id, position }: { cardId: string; list_id: string; position: number }) =>
      (await api.post(`/cards/${cardId}/move`, { list_id, position })).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['board'] }),
  });
}

export function useUpdateCard(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (data: Partial<Card>) => (await api.put(`/cards/${id}`, data)).data,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['card'] });
      qc.invalidateQueries({ queryKey: ['board'] });
    },
  });
}

export function useDeleteCard() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => (await api.delete(`/cards/${id}`)).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['board'] }),
  });
}

export function useAddComment(cardId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (content: string) => (await api.post(`/cards/${cardId}/comments`, { content })).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['card'] }),
  });
}

// ====== NOTES ======
// Pass a boardId to scope to a project's shared notes; omit for the caller's
// private global notes.
export function useNotes(boardId?: string) {
  const activeServerId = useServerStore((s) => s.activeServerId);
  return useQuery<Note[]>({
    queryKey: ['notes', activeServerId, boardId ?? 'global'],
    queryFn: async () => (await api.get('/notes', { params: boardId ? { board_id: boardId } : {} })).data,
    enabled: !!activeServerId,
  });
}

export function useNote(id: string | undefined) {
  const activeServerId = useServerStore((s) => s.activeServerId);
  return useQuery<Note>({
    queryKey: ['note', activeServerId, id],
    queryFn: async () => (await api.get(`/notes/${id}`)).data,
    enabled: !!id && !!activeServerId,
  });
}

export function useCreateNote() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (data: { board_id?: string; title?: string; content?: string }) =>
      (await api.post('/notes', data)).data as Note,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['notes'] }),
  });
}

export function useUpdateNote() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ id, ...data }: Partial<Pick<Note, 'title' | 'content' | 'position' | 'is_pinned'>> & { id: string }) =>
      (await api.put(`/notes/${id}`, data)).data as Note,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['note'] });
      qc.invalidateQueries({ queryKey: ['notes'] });
    },
  });
}

export function useDeleteNote() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ id }: { id: string; boardId?: string }) => (await api.delete(`/notes/${id}`)).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['notes'] }),
  });
}

// ====== TEAMS ======
// Teams hooks gracefully return empty data when teams are disabled on the server.
export function useTeams() {
  const activeServerId = useServerStore((s) => s.activeServerId);
  const teamsEnabled = useServerStore((s) => s.isTeamsEnabled());

  return useQuery<Team[]>({
    queryKey: ['teams', activeServerId],
    queryFn: async () => {
      if (!teamsEnabled) return [];
      return (await api.get('/teams')).data;
    },
    enabled: !!activeServerId,
  });
}

export function useTeam(id: string | undefined) {
  const activeServerId = useServerStore((s) => s.activeServerId);
  const teamsEnabled = useServerStore((s) => s.isTeamsEnabled());

  return useQuery<Team>({
    queryKey: ['team', activeServerId, id],
    queryFn: async () => (await api.get(`/teams/${id}`)).data,
    enabled: !!id && !!activeServerId && teamsEnabled,
  });
}

export function useCreateTeam() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (data: { name: string; description?: string }) =>
      (await api.post('/teams', data)).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['teams'] }),
  });
}

export function useTeamMembers(teamId: string | undefined) {
  const activeServerId = useServerStore((s) => s.activeServerId);
  const teamsEnabled = useServerStore((s) => s.isTeamsEnabled());
  return useQuery<TeamMember[]>({
    queryKey: ['team-members', activeServerId, teamId],
    queryFn: async () => (await api.get(`/teams/${teamId}/members`)).data,
    enabled: !!teamId && !!activeServerId && teamsEnabled,
  });
}

export function useInviteTeamMember() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ teamId, email, role }: { teamId: string; email: string; role?: string }) =>
      (await api.post(`/teams/${teamId}/invite`, { email, role })).data,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['team-members'] });
      qc.invalidateQueries({ queryKey: ['team'] });
    },
  });
}

export function useRemoveTeamMember() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ teamId, userId }: { teamId: string; userId: string }) =>
      (await api.delete(`/teams/${teamId}/members/${userId}`)).data,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['team-members'] });
      qc.invalidateQueries({ queryKey: ['team'] });
    },
  });
}

// ====== CALENDAR ======
export function useCalendarEvents(start: string, end: string, boardId?: string) {
  const activeServerId = useServerStore((s) => s.activeServerId);
  return useQuery<CalendarEvent[]>({
    queryKey: ['calendar', activeServerId, start, end, boardId],
    queryFn: async () =>
      (await api.get('/calendar/events', { params: { start, end, board_id: boardId } })).data,
    enabled: !!activeServerId,
  });
}

export function useCreateCalendarEvent() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (data: Partial<CalendarEvent>) => (await api.post('/calendar/events', data)).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['calendar'] }),
  });
}

export function useUpdateCalendarEvent() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ id, ...data }: Partial<CalendarEvent> & { id: string }) =>
      (await api.put(`/calendar/events/${id}`, data)).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['calendar'] }),
  });
}

export function useDeleteCalendarEvent() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => (await api.delete(`/calendar/events/${id}`)).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['calendar'] }),
  });
}

// ====== USER ======
export function usePreferences() {
  const activeServerId = useServerStore((s) => s.activeServerId);
  return useQuery<UserPreferences>({
    queryKey: ['preferences', activeServerId],
    queryFn: async () => (await api.get('/users/preferences')).data,
    enabled: !!activeServerId,
  });
}

export function useUpdatePreferences() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (data: Partial<UserPreferences>) => (await api.put('/users/preferences', data)).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['preferences'] }),
  });
}

export function useUpdateProfile() {
  const qc = useQueryClient();
  const setUser = useAuthStore((s) => s.setUser);
  return useMutation({
    mutationFn: async (data: { display_name?: string; bio?: string; avatar_url?: string }) =>
      (await api.put('/users/profile', data)).data as User,
    onSuccess: (updatedUser) => {
      setUser(updatedUser);
      qc.invalidateQueries({ queryKey: ['profile'] });
    },
  });
}

export function useChangePassword() {
  return useMutation({
    mutationFn: async (data: { current_password: string; new_password: string }) =>
      (await api.put('/users/password', data)).data,
  });
}

export function useNotifications() {
  const activeServerId = useServerStore((s) => s.activeServerId);
  return useQuery({
    queryKey: ['notifications', activeServerId],
    queryFn: async () => (await api.get('/users/notifications')).data,
    enabled: !!activeServerId,
  });
}

export function useSearchUsers(query: string) {
  return useQuery({
    queryKey: ['search-users', query],
    queryFn: async () => (await api.get('/users/search', { params: { q: query } })).data,
    enabled: query.length >= 2,
  });
}

export function useUploadAvatar() {
  const qc = useQueryClient();
  const setUser = useAuthStore((s) => s.setUser);
  return useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData();
      formData.append('file', file);
      return (await api.post('/users/avatar', formData, {
        headers: { 'Content-Type': 'multipart/form-data' },
      })).data as { avatar_url: string };
    },
    onSuccess: (data) => {
      qc.invalidateQueries({ queryKey: ['profile'] });
      // Update the user in auth store with new avatar
      const user = useAuthStore.getState().user;
      if (user) setUser({ ...user, avatar_url: data.avatar_url });
    },
  });
}

export function useDeleteAccount() {
  const logout = useAuthStore((s) => s.logout);
  return useMutation({
    mutationFn: async () => (await api.delete('/users/account')).data,
    onSuccess: () => logout(),
  });
}

// ====== INTEGRATIONS ======
export function useIntegrations() {
  const activeServerId = useServerStore((s) => s.activeServerId);
  return useQuery({
    queryKey: ['integrations', activeServerId],
    queryFn: async () => (await api.get('/integrations')).data,
    enabled: !!activeServerId,
  });
}

// ====== BOARD TEMPLATES ======
export interface BoardTemplate {
  id: string;
  name: string;
  description?: string;
  is_builtin?: boolean;
  background_color?: string;
  lists?: string[];
}

export function useBoardTemplates() {
  const activeServerId = useServerStore((s) => s.activeServerId);
  return useQuery<BoardTemplate[]>({
    queryKey: ['board-templates', activeServerId],
    queryFn: async () => (await api.get('/boards/templates')).data,
    enabled: !!activeServerId,
  });
}

export function useCreateFromTemplate() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ templateId, ...data }: { templateId: string; title: string; description?: string; visibility?: string; team_id?: string; background_color?: string; background_image_url?: string }) =>
      (await api.post(`/boards/templates/${templateId}`, data)).data,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['boards'] }),
  });
}
