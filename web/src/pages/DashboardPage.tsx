import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Plus, Star, Clock, Users, Globe, Lock } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { useBoards } from '@/hooks/useApi';
import { boardBgStyle } from '@/components/board/board-backgrounds';
import CreateBoardDialog from '@/components/board/CreateBoardDialog';
import type { BoardSummary } from '@/types';

const VisibilityIcon = ({ v }: { v: string }) => {
  if (v === 'public') return <Globe className="h-3 w-3" />;
  if (v === 'team') return <Users className="h-3 w-3" />;
  return <Lock className="h-3 w-3" />;
};

const BoardCard = ({ board }: { board: BoardSummary }) => (
  <Link to={`/board/${board.id}`}>
    <Card className="group overflow-hidden hover:shadow-lg transition-shadow cursor-pointer">
      <div
        className="h-24 flex items-end p-3"
        style={boardBgStyle(board.background_color, board.background_image_url)}
      >
        <h3 className="text-white font-bold text-sm truncate drop-shadow">{board.title}</h3>
      </div>
      <CardContent className="p-3">
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <div className="flex items-center gap-1">
            <VisibilityIcon v={board.visibility} />
            <span>{board.visibility}</span>
          </div>
          <div className="flex items-center gap-3">
            <span>{board.card_count} cards</span>
            <span>{board.member_count} members</span>
          </div>
        </div>
      </CardContent>
    </Card>
  </Link>
);

export default function DashboardPage() {
  const { data: boards, isLoading } = useBoards();
  const [showCreate, setShowCreate] = useState(false);

  const starredBoards = boards?.filter((b) => b.is_starred) || [];
  const recentBoards = boards?.filter((b) => !b.is_archived) || [];

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full" />
      </div>
    );
  }

  const isEmpty = !boards || boards.length === 0;

  if (isEmpty) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-6 p-6">
        <div className="text-center">
          <div className="h-16 w-16 rounded-2xl bg-primary/10 flex items-center justify-center mx-auto mb-4">
            <Plus className="h-8 w-8 text-primary" />
          </div>
          <h2 className="text-xl font-semibold mb-2">No boards yet</h2>
          <p className="text-sm text-muted-foreground max-w-sm">
            Create your first board to start organizing tasks with lists and cards.
          </p>
        </div>
        <Button onClick={() => setShowCreate(true)}>
          <Plus className="h-4 w-4 mr-2" />
          Create Your First Board
        </Button>
        <CreateBoardDialog open={showCreate} onOpenChange={setShowCreate} />
      </div>
    );
  }

  return (
    <div className="p-6 max-w-7xl mx-auto">
      {/* Starred */}
      {starredBoards.length > 0 && (
        <section className="mb-8">
          <h2 className="flex items-center gap-2 text-lg font-semibold mb-4">
            <Star className="h-5 w-5 text-yellow-500" />
            Starred Boards
          </h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
            {starredBoards.map((board) => (
              <BoardCard key={board.id} board={board} />
            ))}
          </div>
        </section>
      )}

      {/* Recent */}
      <section>
        <div className="flex items-center justify-between mb-4">
          <h2 className="flex items-center gap-2 text-lg font-semibold">
            <Clock className="h-5 w-5" />
            Your Boards
          </h2>
          <Button onClick={() => setShowCreate(true)} size="sm">
            <Plus className="h-4 w-4 mr-1" />
            New Board
          </Button>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          {recentBoards.map((board) => (
            <BoardCard key={board.id} board={board} />
          ))}
          <Card
            className="flex items-center justify-center h-36 cursor-pointer hover:bg-accent transition-colors border-dashed"
            onClick={() => setShowCreate(true)}
          >
            <div className="text-center text-muted-foreground">
              <Plus className="h-8 w-8 mx-auto mb-2" />
              <span className="text-sm">Create new board</span>
            </div>
          </Card>
        </div>
      </section>

      <CreateBoardDialog open={showCreate} onOpenChange={setShowCreate} />
    </div>
  );
}
