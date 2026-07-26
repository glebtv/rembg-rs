import React from 'react';
import { Card, Button } from '@heroui/react';
import { ImagePreview } from './ImagePreview';

interface ResultViewProps {
  originalUrl: string;
  resultUrl: string | null;
  maskUrl: string | null;
  isProcessing: boolean;
  resultFilename: string;
  maskFilename: string;
  onDownload: (url: string, filename: string) => void;
  onReset: () => void;
}

export function ResultView({
  originalUrl,
  resultUrl,
  maskUrl,
  isProcessing,
  resultFilename,
  maskFilename,
  onDownload,
  onReset,
}: ResultViewProps) {
  if (!originalUrl && !resultUrl) return null;

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <ImagePreview src={originalUrl} title="Original" />
        <div className="flex flex-col gap-2">
          <span className="text-xs font-medium text-slate-300 uppercase tracking-wide">
            Result
          </span>
          <div className="relative rounded-xl overflow-hidden bg-slate-800/60 border border-slate-700/50 aspect-square flex items-center justify-center">
            {isProcessing ? (
              <div className="flex flex-col items-center gap-2">
                <div className="w-8 h-8 border-2 border-violet-500/30 border-t-violet-500 rounded-full animate-spin" />
                <span className="text-xs text-slate-300">Processing...</span>
              </div>
            ) : resultUrl ? (
              <img
                src={resultUrl}
                alt="Result"
                className="w-full h-full object-contain"
                data-testid="result-image"
              />
            ) : (
              <span className="text-xs text-slate-500">No result yet</span>
            )}
          </div>
        </div>
      </div>

      {maskUrl && <ImagePreview src={maskUrl} title="Mask" />}

      <div className="flex gap-2 flex-wrap">
        {resultUrl && (
          <Button
            size="sm"
            color="violet"
            radius="full"
            onPress={() => onDownload(resultUrl, resultFilename)}
            data-testid="download-result"
          >
            Download Result
          </Button>
        )}
        {maskUrl && (
          <Button
            size="sm"
            variant="bordered"
            border="border-violet-500/30 text-violet-300"
            radius="full"
            onPress={() => onDownload(maskUrl, maskFilename)}
            data-testid="download-mask"
          >
            Download Mask
          </Button>
        )}
        <Button
          size="sm"
          variant="flat"
          color="slate"
          radius="full"
          onPress={onReset}
        >
          New Image
        </Button>
      </div>
    </div>
  );
}
