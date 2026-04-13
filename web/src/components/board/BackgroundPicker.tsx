import { useState } from 'react';
import { ImageIcon, Palette } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';
import {
  bgColors,
  bgGradients,
  bgImages,
  boardBgStyle,
  type BgTab,
} from './board-backgrounds';

export interface BackgroundPickerValue {
  bgColor: string;
  bgImage: string;
}

interface BackgroundPickerProps {
  value: BackgroundPickerValue;
  onChange: (value: BackgroundPickerValue) => void;
  previewTitle?: string;
}

export default function BackgroundPicker({ value, onChange, previewTitle }: BackgroundPickerProps) {
  const [bgTab, setBgTab] = useState<BgTab>(() => {
    if (value.bgImage) return 'images';
    if (value.bgColor.startsWith('linear-gradient')) return 'gradients';
    return 'colors';
  });
  const [customImageUrl, setCustomImageUrl] = useState('');

  return (
    <div>
      <label className="text-sm font-medium">Background</label>
      <div className="flex gap-1 mt-2 mb-3">
        {([
          ['colors', 'Colors', Palette],
          ['gradients', 'Gradients', Palette],
          ['images', 'Images', ImageIcon],
        ] as const).map(([tab, label, Icon]) => (
          <Button
            key={tab}
            variant={bgTab === tab ? 'default' : 'outline'}
            size="sm"
            className="text-xs h-7 px-2"
            onClick={() => setBgTab(tab)}
          >
            <Icon className="h-3 w-3 mr-1" />
            {label}
          </Button>
        ))}
      </div>

      {bgTab === 'colors' && (
        <div className="flex gap-2 flex-wrap">
          {bgColors.map((c) => (
            <button
              key={c}
              className={cn(
                'h-8 w-8 rounded-md transition-all',
                value.bgColor === c && !value.bgImage && 'ring-2 ring-primary ring-offset-2',
              )}
              style={{ backgroundColor: c }}
              onClick={() => onChange({ bgColor: c, bgImage: '' })}
            />
          ))}
        </div>
      )}

      {bgTab === 'gradients' && (
        <div className="flex gap-2 flex-wrap">
          {bgGradients.map((g) => (
            <button
              key={g}
              className={cn(
                'h-8 w-8 rounded-md transition-all',
                value.bgColor === g && !value.bgImage && 'ring-2 ring-primary ring-offset-2',
              )}
              style={{ backgroundImage: g }}
              onClick={() => onChange({ bgColor: g, bgImage: '' })}
            />
          ))}
        </div>
      )}

      {bgTab === 'images' && (
        <div className="space-y-3">
          <div className="grid grid-cols-3 gap-2">
            {bgImages.map((url) => (
              <button
                key={url}
                className={cn(
                  'h-14 rounded-md bg-cover bg-center transition-all',
                  value.bgImage === url && 'ring-2 ring-primary ring-offset-2',
                )}
                style={{ backgroundImage: `url(${url})` }}
                onClick={() => onChange({ ...value, bgImage: url })}
              />
            ))}
          </div>
          <div>
            <Input
              value={customImageUrl}
              onChange={(e) => setCustomImageUrl(e.target.value)}
              placeholder="Or paste an image URL..."
              className="text-xs h-8"
              onKeyDown={(e) => {
                if (e.key === 'Enter' && customImageUrl.trim()) {
                  onChange({ ...value, bgImage: customImageUrl.trim() });
                }
              }}
            />
            {customImageUrl.trim() && value.bgImage !== customImageUrl.trim() && (
              <Button
                variant="outline"
                size="sm"
                className="mt-1 text-xs h-7"
                onClick={() => onChange({ ...value, bgImage: customImageUrl.trim() })}
              >
                Use this image
              </Button>
            )}
          </div>
        </div>
      )}

      {/* Preview */}
      <div
        className="mt-3 h-16 rounded-md flex items-end p-2"
        style={boardBgStyle(value.bgColor, value.bgImage || undefined)}
      >
        <span className="text-white text-xs font-medium drop-shadow truncate">
          {previewTitle || 'Board Preview'}
        </span>
      </div>
    </div>
  );
}
