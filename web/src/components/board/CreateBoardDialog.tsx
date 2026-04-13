import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { AlertCircle, Layout, ChevronLeft } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog';
import {
  useTeams,
  useCreateBoard,
  useBoardTemplates,
  useCreateFromTemplate,
  type BoardTemplate,
} from '@/hooks/useApi';
import { Users, Globe, Lock } from 'lucide-react';
import BackgroundPicker from './BackgroundPicker';
import { bgColors, swatchStyle } from './board-backgrounds';

type CreateStep = 'template' | 'details';

const BLANK_TEMPLATE_ID = '00000000-0000-0000-0000-000000000008';

interface VisibilityConfig {
  /** Fixed visibility (e.g. 'team' for team pages). If set, visibility selector is hidden. */
  locked?: 'private' | 'team' | 'public';
  /** Fixed team_id (e.g. for team pages). If set, team selector is hidden. */
  teamId?: string;
}

interface CreateBoardDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Override dialog title for the details step */
  detailsTitle?: string;
  /** Override dialog description for the details step */
  detailsDescription?: string;
  /** Visibility configuration */
  visibility?: VisibilityConfig;
}

const VisibilityIcon = ({ v }: { v: string }) => {
  if (v === 'public') return <Globe className="h-3 w-3" />;
  if (v === 'team') return <Users className="h-3 w-3" />;
  return <Lock className="h-3 w-3" />;
};

export default function CreateBoardDialog({
  open,
  onOpenChange,
  detailsTitle = 'Create New Board',
  detailsDescription = 'Configure your board settings',
  visibility: visConfig,
}: CreateBoardDialogProps) {
  const navigate = useNavigate();
  const { data: teams } = useTeams();
  const { data: templates = [] } = useBoardTemplates();
  const createBoard = useCreateBoard();
  const createFromTemplate = useCreateFromTemplate();

  const [createStep, setCreateStep] = useState<CreateStep>('template');
  const [selectedTemplate, setSelectedTemplate] = useState<BoardTemplate | null>(null);
  const [newTitle, setNewTitle] = useState('');
  const [bg, setBg] = useState({ bgColor: bgColors[0], bgImage: '' });
  const [newVisibility, setNewVisibility] = useState<string>(visConfig?.locked || 'private');
  const [newTeamId, setNewTeamId] = useState<string>(visConfig?.teamId || '');
  const [createError, setCreateError] = useState('');

  const resetForm = () => {
    setCreateStep('template');
    setSelectedTemplate(null);
    setNewTitle('');
    setBg({ bgColor: bgColors[0], bgImage: '' });
    setNewVisibility(visConfig?.locked || 'private');
    setNewTeamId(visConfig?.teamId || '');
    setCreateError('');
  };

  const selectTemplate = (tmpl: BoardTemplate) => {
    setSelectedTemplate(tmpl);
    setNewTitle(tmpl.name === 'Blank Board' ? '' : tmpl.name);
    if (tmpl.background_color) {
      setBg({
        bgColor: tmpl.background_color,
        bgImage: '',
      });
    }
    setCreateStep('details');
  };

  const handleCreate = () => {
    if (!newTitle.trim()) return;
    setCreateError('');

    const bgColor = bg.bgImage ? undefined : bg.bgColor;
    const bgImage = bg.bgImage || undefined;
    const teamId = visConfig?.teamId || newTeamId || undefined;
    const vis = visConfig?.locked || newVisibility;

    const onSuccess = (data: { id: string }) => {
      onOpenChange(false);
      resetForm();
      if (data?.id) navigate(`/board/${data.id}`);
    };
    const onError = (err: unknown) => {
      const message = err instanceof Error ? err.message : 'Failed to create board. Please try again.';
      setCreateError(message);
    };

    if (selectedTemplate && selectedTemplate.id !== BLANK_TEMPLATE_ID) {
      createFromTemplate.mutate(
        {
          templateId: selectedTemplate.id,
          title: newTitle,
          visibility: vis,
          team_id: teamId,
          background_color: bgColor,
          background_image_url: bgImage,
        },
        { onSuccess, onError },
      );
    } else {
      createBoard.mutate(
        {
          title: newTitle,
          visibility: vis,
          background_color: bgColor,
          background_image_url: bgImage,
          team_id: teamId,
        },
        { onSuccess, onError },
      );
    }
  };

  const isPending = createBoard.isPending || createFromTemplate.isPending;

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        onOpenChange(o);
        if (!o) resetForm();
      }}
    >
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <div className="flex items-center gap-2">
            {createStep === 'details' && (
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 -ml-1"
                onClick={() => setCreateStep('template')}
              >
                <ChevronLeft className="h-4 w-4" />
              </Button>
            )}
            <div>
              <DialogTitle>
                {createStep === 'template' ? 'Choose a Template' : detailsTitle}
              </DialogTitle>
              <DialogDescription>
                {createStep === 'template'
                  ? 'Start with a pre-built template or a blank board'
                  : detailsDescription}
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        {createStep === 'template' ? (
          /* ---- Template step ---- */
          <div className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Choose a template to get started quickly, or start from scratch.
            </p>
            <div className="grid grid-cols-2 gap-3 max-h-[400px] overflow-y-auto pr-1">
              {templates.map((tmpl) => (
                <button
                  key={tmpl.id}
                  className="text-left rounded-lg border hover:border-primary hover:shadow-md transition-all overflow-hidden"
                  onClick={() => selectTemplate(tmpl)}
                >
                  <div
                    className="h-16 flex items-end p-2"
                    style={swatchStyle(tmpl.background_color || '#3b82f6')}
                  >
                    <span className="text-white text-xs font-semibold drop-shadow truncate">
                      {tmpl.name}
                    </span>
                  </div>
                  <div className="p-2">
                    {tmpl.description && (
                      <p className="text-[10px] text-muted-foreground line-clamp-2">
                        {tmpl.description}
                      </p>
                    )}
                    {tmpl.lists && tmpl.lists.length > 0 && (
                      <div className="flex gap-1 mt-1.5 flex-wrap">
                        {tmpl.lists.slice(0, 4).map((l) => (
                          <span key={l} className="text-[9px] bg-muted px-1.5 py-0.5 rounded">
                            {l}
                          </span>
                        ))}
                        {tmpl.lists.length > 4 && (
                          <span className="text-[9px] text-muted-foreground">
                            +{tmpl.lists.length - 4}
                          </span>
                        )}
                      </div>
                    )}
                  </div>
                </button>
              ))}
            </div>
          </div>
        ) : (
          /* ---- Details step ---- */
          <div className="space-y-4">
            {createError && (
              <div className="flex items-center gap-2 rounded-md bg-destructive/10 p-3 text-sm text-destructive">
                <AlertCircle className="h-4 w-4 flex-shrink-0" />
                <span>{createError}</span>
              </div>
            )}

            {selectedTemplate && (
              <div className="flex items-center gap-2 text-xs text-muted-foreground bg-muted/50 rounded-md px-3 py-2">
                <Layout className="h-3.5 w-3.5" />
                <span>
                  Template: <strong>{selectedTemplate.name}</strong>
                </span>
                {selectedTemplate.lists && selectedTemplate.lists.length > 0 && (
                  <span className="ml-auto">{selectedTemplate.lists.length} lists</span>
                )}
              </div>
            )}

            <div>
              <label className="text-sm font-medium">Board Title</label>
              <Input
                value={newTitle}
                onChange={(e) => setNewTitle(e.target.value)}
                placeholder="e.g., Project Alpha"
                autoFocus
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleCreate();
                }}
              />
            </div>

            <BackgroundPicker
              value={bg}
              onChange={setBg}
              previewTitle={newTitle}
            />

            {/* Visibility — hidden when locked */}
            {!visConfig?.locked && (
              <div>
                <label className="text-sm font-medium">Visibility</label>
                <div className="flex gap-2 mt-2">
                  {['private', 'team', 'public'].map((v) => (
                    <Button
                      key={v}
                      variant={newVisibility === v ? 'default' : 'outline'}
                      size="sm"
                      onClick={() => setNewVisibility(v)}
                    >
                      <VisibilityIcon v={v} />
                      <span className="ml-1 capitalize">{v}</span>
                    </Button>
                  ))}
                </div>
              </div>
            )}

            {/* Team selector — hidden when teamId is locked */}
            {!visConfig?.teamId && newVisibility === 'team' && teams && teams.length > 0 && (
              <div>
                <label className="text-sm font-medium">Team</label>
                <select
                  className="w-full rounded-md border p-2 text-sm mt-1 bg-background"
                  value={newTeamId}
                  onChange={(e) => setNewTeamId(e.target.value)}
                >
                  <option value="">Select a team</option>
                  {teams.map((t) => (
                    <option key={t.id} value={t.id}>
                      {t.name}
                    </option>
                  ))}
                </select>
              </div>
            )}

            <Button
              onClick={handleCreate}
              className="w-full"
              disabled={!newTitle.trim() || isPending}
            >
              {isPending ? 'Creating...' : 'Create Board'}
            </Button>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
