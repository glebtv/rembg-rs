import React from 'react';

interface ImagePreviewProps {
  src: string;
  title: string;
}

export function ImagePreview({ src, title }: ImagePreviewProps) {
  return (
    <div className="flex flex-col gap-2">
      <span className="text-xs font-medium text-slate-300 uppercase tracking-wide">
        {title}
      </span>
      <div className="relative rounded-xl overflow-hidden bg-slate-800/60 border border-slate-700/50 aspect-square">
        <img
          src={src}
          alt={title}
          data-testid={title === 'Mask' ? 'mask-image' : undefined}
          className="w-full h-full object-contain"
        />
      </div>
    </div>
  );
}
