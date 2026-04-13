import { useState, useRef } from 'react';
import {
  Settings as SettingsIcon,
  User,
  Bell,
  Palette,
  Shield,
  Save,
  Moon,
  Sun,
  Monitor,
  Check,
  AlertCircle,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { usePreferences, useUpdatePreferences, useUpdateProfile, useChangePassword, useUploadAvatar, useDeleteAccount } from '@/hooks/useApi';
import { useAuthStore } from '@/store/auth';
import { useThemeStore } from '@/store/theme';
import { cn, getInitials } from '@/lib/utils';

type SettingsTab = 'profile' | 'appearance' | 'notifications' | 'account';

const tabs: { id: SettingsTab; label: string; icon: typeof User }[] = [
  { id: 'profile', label: 'Profile', icon: User },
  { id: 'appearance', label: 'Appearance', icon: Palette },
  { id: 'notifications', label: 'Notifications', icon: Bell },
  { id: 'account', label: 'Account', icon: Shield },
];

export default function SettingsPage() {
  const user = useAuthStore((s) => s.user);
  const { data: preferences } = usePreferences();
  const updatePreferences = useUpdatePreferences();
  const updateProfile = useUpdateProfile();
  const changePassword = useChangePassword();
  const uploadAvatar = useUploadAvatar();
  const deleteAccount = useDeleteAccount();
  const avatarInputRef = useRef<HTMLInputElement>(null);
  const { theme, setTheme } = useThemeStore();

  const [activeTab, setActiveTab] = useState<SettingsTab>('profile');
  const [profile, setProfile] = useState({
    display_name: user?.display_name || '',
    username: user?.username || '',
    bio: user?.bio || '',
  });
  const [profileSaved, setProfileSaved] = useState(false);

  const [notifSettings, setNotifSettings] = useState({
    email_notifications: preferences?.email_notifications ?? true,
    push_notifications: preferences?.push_notifications ?? true,
  });

  const [passwordForm, setPasswordForm] = useState({
    current_password: '',
    new_password: '',
    confirm_password: '',
  });
  const [passwordError, setPasswordError] = useState('');
  const [passwordSaved, setPasswordSaved] = useState(false);

  const handleSaveProfile = () => {
    updateProfile.mutate(
      { display_name: profile.display_name, bio: profile.bio },
      {
        onSuccess: () => {
          setProfileSaved(true);
          setTimeout(() => setProfileSaved(false), 2000);
        },
      },
    );
  };

  const handleSaveNotifications = () => {
    updatePreferences.mutate({
      email_notifications: notifSettings.email_notifications,
      push_notifications: notifSettings.push_notifications,
    });
  };

  const handleChangePassword = () => {
    setPasswordError('');
    setPasswordSaved(false);
    if (passwordForm.new_password.length < 8) {
      setPasswordError('New password must be at least 8 characters');
      return;
    }
    if (passwordForm.new_password !== passwordForm.confirm_password) {
      setPasswordError('New passwords do not match');
      return;
    }
    changePassword.mutate(
      { current_password: passwordForm.current_password, new_password: passwordForm.new_password },
      {
        onSuccess: () => {
          setPasswordForm({ current_password: '', new_password: '', confirm_password: '' });
          setPasswordSaved(true);
          setTimeout(() => setPasswordSaved(false), 3000);
        },
        onError: (err: unknown) => {
          const msg = (err as { response?: { data?: { error?: { message?: string } } } })?.response?.data?.error?.message;
          setPasswordError(msg || 'Failed to change password');
        },
      },
    );
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold flex items-center gap-2 mb-6">
        <SettingsIcon className="h-6 w-6" />
        Settings
      </h1>

      <div className="flex gap-6">
        {/* Sidebar tabs */}
        <div className="w-48 flex-shrink-0">
          <nav className="space-y-1">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={cn(
                  'flex items-center gap-2 w-full rounded-md px-3 py-2 text-sm transition-colors',
                  activeTab === tab.id
                    ? 'bg-primary/10 text-primary font-medium'
                    : 'text-muted-foreground hover:bg-accent hover:text-foreground',
                )}
              >
                <tab.icon className="h-4 w-4" />
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          {/* Profile */}
          {activeTab === 'profile' && (
            <div className="space-y-6">
              <Card>
                <CardContent className="p-6">
                  <h2 className="text-lg font-semibold mb-4">Profile</h2>
                  <div className="flex items-start gap-6 mb-6">
                    <Avatar className="h-20 w-20">
                      {user?.avatar_url && <AvatarImage src={user.avatar_url} />}
                      <AvatarFallback className="text-2xl">
                        {user ? getInitials(user.display_name) : 'U'}
                      </AvatarFallback>
                    </Avatar>
                    <div>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => avatarInputRef.current?.click()}
                        disabled={uploadAvatar.isPending}
                      >
                        {uploadAvatar.isPending ? 'Uploading...' : 'Change Avatar'}
                      </Button>
                      <input
                        ref={avatarInputRef}
                        type="file"
                        accept="image/*"
                        className="hidden"
                        onChange={(e) => {
                          const file = e.target.files?.[0];
                          if (file) uploadAvatar.mutate(file);
                          e.target.value = '';
                        }}
                      />
                      <p className="text-xs text-muted-foreground mt-1">
                        JPG, PNG or GIF. Max 2MB.
                      </p>
                    </div>
                  </div>
                  <div className="space-y-4">
                    <div>
                      <label className="text-sm font-medium">Display Name</label>
                      <Input
                        value={profile.display_name}
                        onChange={(e) =>
                          setProfile({ ...profile, display_name: e.target.value })
                        }
                      />
                    </div>
                    <div>
                      <label className="text-sm font-medium">Username</label>
                      <Input
                        value={profile.username}
                        onChange={(e) =>
                          setProfile({ ...profile, username: e.target.value })
                        }
                      />
                    </div>
                    <div>
                      <label className="text-sm font-medium">Bio</label>
                      <textarea
                        value={profile.bio}
                        onChange={(e) =>
                          setProfile({ ...profile, bio: e.target.value })
                        }
                        className="w-full rounded-md border bg-background p-2 text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary"
                        rows={3}
                        placeholder="Tell us about yourself..."
                      />
                    </div>
                    <div>
                      <label className="text-sm font-medium">Email</label>
                      <Input value={user?.email || ''} disabled />
                      <p className="text-xs text-muted-foreground mt-1">
                        Email cannot be changed here.
                      </p>
                    </div>
                    <Button onClick={handleSaveProfile} disabled={updateProfile.isPending}>
                      {profileSaved ? (
                        <><Check className="h-4 w-4 mr-1" />Saved</>
                      ) : (
                        <><Save className="h-4 w-4 mr-1" />Save Profile</>
                      )}
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </div>
          )}

          {/* Appearance */}
          {activeTab === 'appearance' && (
            <div className="space-y-6">
              <Card>
                <CardContent className="p-6">
                  <h2 className="text-lg font-semibold mb-4">Theme</h2>
                  <div className="grid grid-cols-3 gap-3">
                    {[
                      { id: 'light' as const, label: 'Light', icon: Sun },
                      { id: 'dark' as const, label: 'Dark', icon: Moon },
                      { id: 'system' as const, label: 'System', icon: Monitor },
                    ].map((t) => (
                      <button
                        key={t.id}
                        onClick={() => setTheme(t.id)}
                        className={cn(
                          'flex flex-col items-center gap-2 rounded-lg border p-4 transition-colors',
                          theme === t.id
                            ? 'border-primary bg-primary/5'
                            : 'hover:bg-accent',
                        )}
                      >
                        <t.icon className="h-6 w-6" />
                        <span className="text-sm font-medium">{t.label}</span>
                      </button>
                    ))}
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardContent className="p-6">
                  <h2 className="text-lg font-semibold mb-4">Board Defaults</h2>
                  <div className="space-y-4">
                    <div>
                      <label className="text-sm font-medium">Default Board View</label>
                      <select className="w-full rounded-md border p-2 text-sm mt-1 bg-background">
                        <option value="kanban">Kanban Board</option>
                        <option value="list">List View</option>
                        <option value="calendar">Calendar View</option>
                      </select>
                    </div>
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm font-medium">Compact Mode</p>
                        <p className="text-xs text-muted-foreground">
                          Show smaller cards with less detail
                        </p>
                      </div>
                      <Switch
                        checked={!!preferences?.compact_mode}
                        onCheckedChange={(checked) =>
                          updatePreferences.mutate({ compact_mode: checked })
                        }
                      />
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          )}

          {/* Notifications */}
          {activeTab === 'notifications' && (
            <Card>
              <CardContent className="p-6">
                <h2 className="text-lg font-semibold mb-4">Notification Preferences</h2>
                <div className="space-y-4">
                  <div className="flex items-center justify-between py-2 border-b">
                    <div>
                      <p className="text-sm font-medium">Email Notifications</p>
                      <p className="text-xs text-muted-foreground">
                        Receive email alerts for board activity
                      </p>
                    </div>
                    <Switch
                      checked={notifSettings.email_notifications}
                      onCheckedChange={(checked) =>
                        setNotifSettings({
                          ...notifSettings,
                          email_notifications: checked,
                        })
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between py-2 border-b">
                    <div>
                      <p className="text-sm font-medium">Push Notifications</p>
                      <p className="text-xs text-muted-foreground">
                        Receive browser push notifications
                      </p>
                    </div>
                    <Switch
                      checked={notifSettings.push_notifications}
                      onCheckedChange={(checked) =>
                        setNotifSettings({
                          ...notifSettings,
                          push_notifications: checked,
                        })
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between py-2 border-b">
                    <div>
                      <p className="text-sm font-medium">Card Assignments</p>
                      <p className="text-xs text-muted-foreground">
                        Notify when assigned to a card
                      </p>
                    </div>
                    <Switch checked disabled />
                  </div>

                  <div className="flex items-center justify-between py-2 border-b">
                    <div>
                      <p className="text-sm font-medium">Due Date Reminders</p>
                      <p className="text-xs text-muted-foreground">
                        Remind before cards are due
                      </p>
                    </div>
                    <Switch checked disabled />
                  </div>

                  <div className="flex items-center justify-between py-2">
                    <div>
                      <p className="text-sm font-medium">Comments & Mentions</p>
                      <p className="text-xs text-muted-foreground">
                        Notify on new comments and @mentions
                      </p>
                    </div>
                    <Switch checked disabled />
                  </div>

                  <Button onClick={handleSaveNotifications}>
                    <Save className="h-4 w-4 mr-1" />
                    Save Preferences
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Account */}
          {activeTab === 'account' && (
            <div className="space-y-6">
              <Card>
                <CardContent className="p-6">
                  <h2 className="text-lg font-semibold mb-4">Change Password</h2>
                  <div className="space-y-4">
                    <div>
                      <label className="text-sm font-medium">Current Password</label>
                      <Input
                        type="password"
                        value={passwordForm.current_password}
                        onChange={(e) => setPasswordForm({ ...passwordForm, current_password: e.target.value })}
                      />
                    </div>
                    <div>
                      <label className="text-sm font-medium">New Password</label>
                      <Input
                        type="password"
                        value={passwordForm.new_password}
                        onChange={(e) => setPasswordForm({ ...passwordForm, new_password: e.target.value })}
                      />
                    </div>
                    <div>
                      <label className="text-sm font-medium">Confirm New Password</label>
                      <Input
                        type="password"
                        value={passwordForm.confirm_password}
                        onChange={(e) => setPasswordForm({ ...passwordForm, confirm_password: e.target.value })}
                      />
                    </div>
                    {passwordError && (
                      <div className="flex items-center gap-2 text-sm text-destructive">
                        <AlertCircle className="h-4 w-4" />
                        {passwordError}
                      </div>
                    )}
                    {passwordSaved && (
                      <div className="flex items-center gap-2 text-sm text-green-600">
                        <Check className="h-4 w-4" />
                        Password changed successfully
                      </div>
                    )}
                    <Button
                      onClick={handleChangePassword}
                      disabled={changePassword.isPending || !passwordForm.current_password || !passwordForm.new_password}
                    >
                      Change Password
                    </Button>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardContent className="p-6">
                  <h2 className="text-lg font-semibold mb-4">Connected Accounts</h2>
                  <div className="space-y-3">
                    {['GitHub', 'GitLab', 'Atlassian'].map((provider) => (
                      <div
                        key={provider}
                        className="flex items-center justify-between py-2 border-b last:border-0"
                      >
                        <span className="text-sm font-medium">{provider}</span>
                        <Button variant="outline" size="sm">
                          Connect
                        </Button>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>

              <Card className="border-destructive/50">
                <CardContent className="p-6">
                  <h2 className="text-lg font-semibold text-destructive mb-2">
                    Danger Zone
                  </h2>
                  <p className="text-sm text-muted-foreground mb-4">
                    Once you delete your account, there is no going back. Please be
                    certain.
                  </p>
                  <Button
                    variant="destructive"
                    disabled={deleteAccount.isPending}
                    onClick={() => {
                      if (window.confirm('Are you sure you want to delete your account? This action cannot be undone.')) {
                        deleteAccount.mutate();
                      }
                    }}
                  >
                    {deleteAccount.isPending ? 'Deleting...' : 'Delete Account'}
                  </Button>
                </CardContent>
              </Card>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
