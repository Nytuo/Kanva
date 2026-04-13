import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Plus, Users, ChevronRight } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog';
import { useTeams, useCreateTeam } from '@/hooks/useApi';
import { getInitials } from '@/lib/utils';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';

export default function TeamsPage() {
  const { data: teams, isLoading } = useTeams();
  const createTeam = useCreateTeam();
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState('');
  const [newDescription, setNewDescription] = useState('');

  const handleCreate = () => {
    if (!newName.trim()) return;
    createTeam.mutate({
      name: newName.trim(),
      description: newDescription.trim() || undefined,
    });
    setShowCreate(false);
    setNewName('');
    setNewDescription('');
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full" />
      </div>
    );
  }

  return (
    <div className="p-6 max-w-5xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold flex items-center gap-2">
          <Users className="h-6 w-6" />
          Teams
        </h1>
        <Button onClick={() => setShowCreate(true)} size="sm">
          <Plus className="h-4 w-4 mr-1" />
          New Team
        </Button>
      </div>

      {teams && teams.length === 0 ? (
        <div className="text-center py-20">
          <Users className="h-16 w-16 mx-auto text-muted-foreground mb-4" />
          <h2 className="text-lg font-semibold mb-2">No teams yet</h2>
          <p className="text-muted-foreground mb-4">
            Create a team to collaborate with others on boards.
          </p>
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="h-4 w-4 mr-1" />
            Create Your First Team
          </Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {teams?.map((team) => (
            <Link key={team.id} to={`/teams/${team.id}`}>
              <Card className="hover:shadow-md transition-shadow cursor-pointer">
                <CardContent className="p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <Avatar className="h-10 w-10">
                        <AvatarFallback className="bg-primary/10 text-primary font-bold">
                          {getInitials(team.name)}
                        </AvatarFallback>
                      </Avatar>
                      <div>
                        <h3 className="font-semibold">{team.name}</h3>
                        {team.description && (
                          <p className="text-sm text-muted-foreground line-clamp-1">
                            {team.description}
                          </p>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-4 text-sm text-muted-foreground">
                      <span>{team.member_count} members</span>
                      <span>{team.board_count} boards</span>
                      <ChevronRight className="h-4 w-4" />
                    </div>
                  </div>
                </CardContent>
              </Card>
            </Link>
          ))}
        </div>
      )}

      {/* Create team dialog */}
      <Dialog open={showCreate} onOpenChange={setShowCreate}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create New Team</DialogTitle>
            <DialogDescription>
              Set up a team to collaborate with others
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium">Team Name</label>
              <Input
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                placeholder="e.g., Engineering"
                autoFocus
              />
            </div>
            <div>
              <label className="text-sm font-medium">Description</label>
              <textarea
                value={newDescription}
                onChange={(e) => setNewDescription(e.target.value)}
                placeholder="What does this team work on?"
                className="w-full rounded-md border bg-background p-2 text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary"
                rows={3}
              />
            </div>
            <Button
              onClick={handleCreate}
              className="w-full"
              disabled={!newName.trim()}
            >
              Create Team
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
