import type React from 'react';

export const bgColors = [
  '#3b82f6', '#ef4444', '#22c55e', '#f97316',
  '#8b5cf6', '#06b6d4', '#ec4899', '#6366f1',
];

export const bgGradients = [
  'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
  'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
  'linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)',
  'linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)',
  'linear-gradient(135deg, #fa709a 0%, #fee140 100%)',
  'linear-gradient(135deg, #a18cd1 0%, #fbc2eb 100%)',
  'linear-gradient(135deg, #fccb90 0%, #d57eeb 100%)',
  'linear-gradient(135deg, #0c3483 0%, #a2b6df 100%)',
];

export const bgImages = [
  'https://images.unsplash.com/photo-1506744038136-46273834b3fb?w=800&q=80',
  'https://images.unsplash.com/photo-1477346611705-65d1883cee1e?w=800&q=80',
  'https://images.unsplash.com/photo-1469474968028-56623f02e42e?w=800&q=80',
  'https://images.unsplash.com/photo-1470071459604-3b5ec3a7fe05?w=800&q=80',
  'https://images.unsplash.com/photo-1441974231531-c6227db76b6e?w=800&q=80',
  'https://images.unsplash.com/photo-1518173946687-a243cf18c407?w=800&q=80',
];

/** Returns inline style for background_color (solid or gradient) + optional image URL */
export function boardBgStyle(bgColor?: string | null, bgImage?: string | null): React.CSSProperties {
  if (bgImage) {
    return { backgroundImage: `url(${bgImage})`, backgroundSize: 'cover', backgroundPosition: 'center' };
  }
  if (bgColor?.startsWith('linear-gradient')) {
    return { backgroundImage: bgColor };
  }
  return { backgroundColor: bgColor || '#3b82f6' };
}

/** Swatch style for pickers — handles solid colors, gradients, and image URLs */
export function swatchStyle(value: string): React.CSSProperties {
  if (value.startsWith('linear-gradient')) {
    return { backgroundImage: value };
  }
  if (value.startsWith('http')) {
    return { backgroundImage: `url(${value})`, backgroundSize: 'cover', backgroundPosition: 'center' };
  }
  return { backgroundColor: value };
}

export type BgTab = 'colors' | 'gradients' | 'images';
