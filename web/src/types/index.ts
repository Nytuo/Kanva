export interface User {
  id: string;
  email: string;
  username: string;
  display_name: string;
  avatar_url?: string;
  bio?: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
  user: User;
}

export interface Board {
  id: string;
  title: string;
  description?: string;
  visibility: 'private' | 'team' | 'public';
  background_color?: string;
  background_image_url?: string;
  is_starred: boolean;
  is_archived: boolean;
  owner_id: string;
  team_id?: string;
  lists: List[];
  labels: Label[];
  members: BoardMember[];
  created_at: string;
  updated_at: string;
}

export interface BoardSummary {
  id: string;
  title: string;
  description?: string;
  visibility: string;
  background_color?: string;
  background_image_url?: string;
  is_starred: boolean;
  is_archived: boolean;
  owner_id: string;
  team_id?: string;
  member_count: number;
  card_count: number;
  created_at: string;
}

export interface List {
  id: string;
  title: string;
  position: number;
  cards: CardSummary[];
}

export interface CardSummary {
  id: string;
  title: string;
  position: number;
  priority: CardPriority;
  due_date?: string;
  assignee_count: number;
  comment_count: number;
  checklist_progress?: string;
  label_ids: string[];
  cover_color?: string;
}

export interface Card {
  id: string;
  list_id: string;
  title: string;
  description?: string;
  position: number;
  priority: CardPriority;
  due_date?: string;
  start_date?: string;
  completed_at?: string;
  is_archived: boolean;
  cover_color?: string;
  cover_image_url?: string;
  estimated_hours?: number;
  actual_hours?: number;
  created_by: string;
  assignees: Assignee[];
  labels: Label[];
  checklists: Checklist[];
  comments: Comment[];
  attachments: Attachment[];
  custom_field_values: CustomFieldValue[];
  created_at: string;
  updated_at: string;
}

export type CardPriority = 'none' | 'low' | 'medium' | 'high' | 'urgent';

export interface Assignee {
  user_id: string;
  username: string;
  display_name: string;
  avatar_url?: string;
}

export interface Label {
  id: string;
  name: string;
  color: string;
}

export interface BoardMember {
  user_id: string;
  username: string;
  display_name: string;
  avatar_url?: string;
  role: string;
}

export interface Checklist {
  id: string;
  title: string;
  position: number;
  items: ChecklistItem[];
}

export interface ChecklistItem {
  id: string;
  title: string;
  is_checked: boolean;
  position: number;
  assigned_to?: string;
  due_date?: string;
}

export interface Comment {
  id: string;
  user_id: string;
  username: string;
  display_name: string;
  avatar_url?: string;
  content: string;
  edited_at?: string;
  created_at: string;
}

export interface Attachment {
  id: string;
  filename: string;
  file_url: string;
  file_size: number;
  mime_type?: string;
  created_at: string;
}

export interface CustomFieldValue {
  id: string;
  field_id: string;
  value: unknown;
}

export interface Team {
  id: string;
  name: string;
  slug: string;
  description?: string;
  avatar_url?: string;
  member_count: number;
  board_count: number;
  created_by: string;
  created_at: string;
}

export interface TeamMember {
  user_id: string;
  username: string;
  display_name: string;
  avatar_url?: string;
  role: string;
  joined_at: string;
}

export interface CalendarEvent {
  id: string;
  user_id: string;
  board_id?: string;
  card_id?: string;
  title: string;
  description?: string;
  start_time: string;
  end_time: string;
  all_day: boolean;
  color?: string;
  recurrence_rule?: string;
  created_at: string;
}

export interface Integration {
  id: string;
  board_id: string;
  provider: 'github' | 'gitlab' | 'atlassian';
  config: Record<string, unknown>;
  enabled: boolean;
  created_at: string;
}

export interface Notification {
  id: string;
  title: string;
  message: string;
  link?: string;
  is_read: boolean;
  created_at: string;
}

export interface UserPreferences {
  theme: string;
  language: string;
  timezone: string;
  email_notifications: boolean;
  push_notifications: boolean;
  default_board_view: string;
  compact_mode: boolean;
}
