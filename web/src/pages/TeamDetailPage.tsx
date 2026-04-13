import { useState } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { ArrowLeft, Plus, Mail, Layout } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import type { TeamMember } from '@/types';
import {
  useTeam,
  useBoards,
  useTeamMembers,
  useInviteTeamMember,
  useRemoveTeamMember,
} from '@/hooks/useApi';
import { cn, getInitials } from '@/lib/utils';
import { boardBgStyle } from '@/components/board/board-backgrounds';
import CreateBoardDialog from '@/components/board/CreateBoardDialog';

export default function TeamDetailPage() {
  const { teamId } = useParams<{ teamId: string }>();
  const navigate = useNavigate();
  const { data: team, isLoading } = useTeam(teamId);
  const { data: allBoards } = useBoards({ team_id: teamId ?? '' });
  const { data: members = [], isLoading: membersLoading } = useTeamMembers(teamId);
  const inviteMember = useInviteTeamMember();
  const removeMember = useRemoveTeamMember();

  const [showInvite, setShowInvite] = useState(false);
  const [inviteEmail, setInviteEmail] = useState('');
  const [activeTab, setActiveTab] = useState<'boards' | 'members'>('boards');
  const [showCreate, setShowCreate] = useState(false);

  const teamBoards = allBoards?.filter((b) => b.team_id === teamId) || [];

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full" />
      </div>
    );
  }

  if (!team) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <p className="text-muted-foreground">Team not found</p>
        <Button variant="outline" onClick={() => navigate('/teams')}>
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back to Teams
        </Button>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-5xl mx-auto">
      {/* Header */}
      <div className="flex items-center gap-3 mb-6">
        <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => navigate('/teams')}>
          <ArrowLeft className="h-4 w-4" />
        </Button>
        <Avatar className="h-12 w-12">
          <AvatarFallback className="bg-primary/10 text-primary font-bold text-lg">
            {getInitials(team.name)}
          </AvatarFallback>
        </Avatar>
        <div>
          <h1 className="text-2xl font-bold">{team.name}</h1>
          {team.description && <p className="text-sm text-muted-foreground">{team.description}</p>}
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 border-b mb-6">
        <button
          className={cn('px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors', activeTab === 'boards' ? 'border-primary text-primary' : 'border-transparent text-muted-foreground hover:text-foreground')}
          onClick={() => setActiveTab('boards')}
        >
          Boards ({teamBoards.length})
        </button>
        <button
          className={cn('px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors', activeTab === 'members' ? 'border-primary text-primary' : 'border-transparent text-muted-foreground hover:text-foreground')}
          onClick={() => setActiveTab('members')}
        >
          Members ({team.member_count})
        </button>
      </div>

      {/* Boards tab */}
      {activeTab === 'boards' && (
        <div>
          <div className="flex justify-end mb-4">
            <Button size="sm" onClick={() => setShowCreate(true)}>
              <Plus className="h-4 w-4 mr-1" />
              New Board
            </Button>
          </div>
          {teamBoards.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 gap-4">
              <div className="h-14 w-14 rounded-2xl bg-primary/10 flex items-center justify-center">
                <Layout className="h-7 w-7 text-primary" />
              </div>
              <div className="text-center">
                <p className="font-medium">No boards yet</p>
                <p className="text-sm text-muted-foreground mt-1">Create a board to start collaborating with your team.</p>
              </div>
              <Button onClick={() => setShowCreate(true)}>
                <Plus className="h-4 w-4 mr-2" />
                Create Board
              </Button>
            </div>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">
              {teamBoards.map((board) => (
                <Link key={board.id} to={`/board/${board.id}`}>
                  <Card className="group overflow-hidden hover:shadow-lg transition-shadow cursor-pointer">
                    <div className="h-24 flex items-end p-3" style={boardBgStyle(board.background_color, board.background_image_url)}>
                      <h3 className="text-white font-bold text-sm truncate drop-shadow">{board.title}</h3>
                    </div>
                    <CardContent className="p-3">
                      <div className="flex items-center justify-between text-xs text-muted-foreground">
                        <span>{board.card_count} cards</span>
                        <span>{board.member_count} members</span>
                      </div>
                    </CardContent>
                  </Card>
                </Link>
              ))}
              <Card
                className="flex items-center justify-center h-36 cursor-pointer hover:bg-accent transition-colors border-dashed"
                onClick={() => setShowCreate(true)}
              >
                <div className="text-center text-muted-foreground">
                  <Plus className="h-8 w-8 mx-auto mb-2" />
                  <span className="text-sm">Create board</span>
                </div>
              </Card>
            </div>
          )}
        </div>
      )}

      {/* Members tab */}
      {activeTab === 'members' && (
        <div>
          <div className="flex justify-end mb-4">
            <Button size="sm" onClick={() => setShowInvite(!showInvite)}>
              <Plus className="h-4 w-4 mr-1" />
              Invite Member
            </Button>
          </div>

          {showInvite && (
            <Card className="mb-4">
              <CardContent className="p-4">
                <div className="flex gap-2">
                  <Input
                    value={inviteEmail}
                    onChange={(e) => setInviteEmail(e.target.value)}
                    placeholder="Enter email address..."
                    type="email"
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && inviteEmail.trim() && teamId) {
                        inviteMember.mutate(
                          { teamId, email: inviteEmail.trim() },
                          { onSuccess: () => { setInviteEmail(''); setShowInvite(false); } },
                        );
                      }
                    }}
                  />
                  <Button
                    disabled={!inviteEmail.trim() || inviteMember.isPending}
                    onClick={() => {
                      if (!teamId) return;
                      inviteMember.mutate(
                        { teamId, email: inviteEmail.trim() },
                        { onSuccess: () => { setInviteEmail(''); setShowInvite(false); } },
                      );
                    }}
                  >
                    <Mail className="h-4 w-4 mr-1" />
                    {inviteMember.isPending ? 'Sending...' : 'Send Invite'}
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}

          {membersLoading ? (
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin h-6 w-6 border-4 border-primary border-t-transparent rounded-full" />
            </div>
          ) : members.length === 0 ? (
            <div className="text-sm text-muted-foreground text-center py-8">
              No members yet. Invite someone to get started.
            </div>
          ) : (
            <div className="space-y-2">
              {members.map((member: TeamMember) => (
                <div key={member.user_id} className="flex items-center gap-3 rounded-md border p-3">
                  <Avatar className="h-9 w-9">
                    {member.avatar_url && <AvatarImage src={member.avatar_url} />}
                    <AvatarFallback className="text-xs">{getInitials(member.display_name)}</AvatarFallback>
                  </Avatar>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium truncate">{member.display_name}</p>
                    <p className="text-xs text-muted-foreground">@{member.username}</p>
                  </div>
                  <span className="text-xs font-medium bg-muted px-2 py-1 rounded capitalize">{member.role}</span>
                  {member.role !== 'owner' && teamId && (
                    <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => removeMember.mutate({ teamId, userId: member.user_id })}>
                      Remove
                    </Button>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Create board dialog — team-scoped */}
      <CreateBoardDialog
        open={showCreate}
        onOpenChange={setShowCreate}
        detailsTitle={`New Board for ${team.name}`}
        detailsDescription="This board will be shared with your team"
        visibility={{ locked: 'team', teamId }}
      />
    </div>
  );
}
