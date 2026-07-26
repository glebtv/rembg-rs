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
    <div className="result-view">
      <div className="preview-grid">
        <ImagePreview src={originalUrl} title="Original" />
        <div className="preview-column">
          <span className="preview-label">
            Result
          </span>
          <div className="image-stage image-stage-result">
            {isProcessing ? (
              <div className="flex flex-col items-center gap-2">
                <div className="w-8 h-8 border-2 border-violet-500/30 border-t-violet-500 rounded-full animate-spin" />
                <span className="text-xs text-slate-300">Processing...</span>
              </div>
            ) : resultUrl ? (
              <img
                src={resultUrl}
                alt="Result"
                className="preview-image"
                data-testid="result-image"
              />
            ) : (
              <span className="text-xs text-slate-500">No result yet</span>
            )}
          </div>
        </div>
      </div>

      {maskUrl && <ImagePreview src={maskUrl} title="Mask" />}

      <div className="result-actions">
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
