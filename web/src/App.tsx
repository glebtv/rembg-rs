import React, { useState, useCallback, useEffect } from 'react';
import { UploadArea } from './components/UploadArea';
import { OptionsPanel } from './components/OptionsPanel';
import { ResultView } from './components/ResultView';
import { removeBackground, checkHealth, getConfig, validateApiKey, RemoveBgParams } from './api/client';
import { Spinner, Card, Divider } from '@heroui/react';

interface ProcessingState {
  isProcessing: boolean;
  error: string | null;
}

const initialState: ProcessingState = {
  isProcessing: false,
  error: null,
};

function App() {
  const [file, setFile] = useState<File | null>(null);
  const [originalUrl, setOriginalUrl] = useState<string | null>(null);
  const [resultUrl, setResultUrl] = useState<string | null>(null);
  const [maskUrl, setMaskUrl] = useState<string | null>(null);
  const [resultFilename, setResultFilename] = useState('result.png');
  const [maskFilename, setMaskFilename] = useState('mask.png');
  const [processing, setProcessing] = useState<ProcessingState>(initialState);
  const [serverHealthy, setServerHealthy] = useState<boolean | null>(null);
  const [authRequired, setAuthRequired] = useState(false);
  const [apiKey, setApiKey] = useState('');
  const [apiKeyInput, setApiKeyInput] = useState('');
  const [authError, setAuthError] = useState<string | null>(null);

  const [threshold, setThreshold] = useState(160);
  const [binary, setBinary] = useState(false);
  const [sticker, setSticker] = useState(false);
  const [compress, setCompress] = useState(false);
  const [format, setFormat] = useState<'png' | 'jpeg' | 'webp'>('png');
  const [includeMask, setIncludeMask] = useState(false);

  useEffect(() => {
    checkHealth().then(setServerHealthy);
    getConfig().then((config) => setAuthRequired(config.authRequired)).catch(() => {});
  }, []);

  const handleApiKeySubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setAuthError(null);
    if (await validateApiKey(apiKeyInput)) {
      setApiKey(apiKeyInput);
      setApiKeyInput('');
    } else {
      setAuthError('Invalid API key');
    }
  };

  const handleFileSelect = useCallback((selectedFile: File) => {
    setFile(selectedFile);
    setResultUrl(null);
    setMaskUrl(null);
    setProcessing(initialState);
    if (originalUrl) URL.revokeObjectURL(originalUrl);
    setOriginalUrl(URL.createObjectURL(selectedFile));
  }, [originalUrl]);

  const handleProcess = useCallback(async () => {
    if (!file) return;

    setProcessing({ isProcessing: true, error: null });

    try {
      const params: RemoveBgParams = {
        threshold,
        binary,
        sticker,
        compress,
        format,
        includeMask,
      };

      const result = await removeBackground(file, params, apiKey);

      if (resultUrl) URL.revokeObjectURL(resultUrl);
      setResultUrl(URL.createObjectURL(result.image));
      setResultFilename(result.imageFilename);

      if (maskUrl) URL.revokeObjectURL(maskUrl);
      setMaskUrl(result.mask ? URL.createObjectURL(result.mask) : null);
      setMaskFilename(result.maskFilename ?? 'mask.png');

      setProcessing({ isProcessing: false, error: null });
    } catch (err) {
      setProcessing({
        isProcessing: false,
        error: err instanceof Error ? err.message : 'Failed to process image',
      });
    }
  }, [file, threshold, binary, sticker, compress, format, includeMask, resultUrl, maskUrl, apiKey]);

  const handleDownload = useCallback((url: string, filename: string) => {
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
  }, []);

  const handleReset = useCallback(() => {
    setFile(null);
    if (originalUrl) URL.revokeObjectURL(originalUrl);
    setOriginalUrl(null);
    if (resultUrl) URL.revokeObjectURL(resultUrl);
    setResultUrl(null);
    if (maskUrl) URL.revokeObjectURL(maskUrl);
    setMaskUrl(null);
    setProcessing(initialState);
  }, [originalUrl, resultUrl, maskUrl]);

  return (
    <div className="min-h-screen">
      <div className="max-w-4xl mx-auto px-4 py-8 space-y-8">
        <header className="text-center space-y-2">
          <h1 className="text-3xl font-bold bg-gradient-to-r from-violet-400 via-cyan-400 to-violet-400 bg-clip-text text-transparent">
            rembg-rs
          </h1>
          <p className="text-sm text-slate-400">
            AI-powered background removal
          </p>
          {serverHealthy === false && (
            <Card className="bg-red-500/10 border-red-500/30 inline-flex items-center gap-2 px-4 py-2 text-xs text-red-300">
              Server unavailable
            </Card>
          )}
        </header>

        {authRequired && !apiKey ? (
          <Card className="max-w-md mx-auto bg-slate-900/80 border-slate-700 p-6">
            <form className="space-y-4" onSubmit={handleApiKeySubmit}>
              <div>
                <h2 className="text-lg font-semibold text-white">API key required</h2>
                <p className="text-sm text-slate-400 mt-1">Enter the server API key to process images.</p>
              </div>
              <input
                type="password"
                value={apiKeyInput}
                onChange={(event) => setApiKeyInput(event.target.value)}
                placeholder="API key"
                autoComplete="current-password"
                data-testid="api-key-input"
                className="w-full rounded-xl border border-slate-700 bg-slate-950 px-4 py-3 text-white outline-none focus:border-cyan-500"
              />
              {authError && <p className="text-sm text-red-300">{authError}</p>}
              <button
                type="submit"
                data-testid="api-key-submit"
                className="w-full rounded-xl bg-gradient-to-r from-violet-500 to-cyan-500 px-4 py-3 font-medium text-white"
              >
                Unlock
              </button>
            </form>
          </Card>
        ) : <main className="space-y-6">
          {!file ? (
            <UploadArea onFileSelect={handleFileSelect} />
          ) : (
            <ResultView
              originalUrl={originalUrl!}
              resultUrl={resultUrl}
              maskUrl={maskUrl}
              isProcessing={processing.isProcessing}
              resultFilename={resultFilename}
              maskFilename={maskFilename}
              onDownload={handleDownload}
              onReset={handleReset}
            />
          )}

          <div className="flex gap-6 flex-wrap">
            <div className="flex-1 min-w-0">
              <OptionsPanel
                threshold={threshold}
                setThreshold={setThreshold}
                binary={binary}
                setBinary={setBinary}
                sticker={sticker}
                setSticker={setSticker}
                compress={compress}
                setCompress={setCompress}
                format={format}
                setFormat={setFormat}
                includeMask={includeMask}
                setIncludeMask={setIncludeMask}
              />
            </div>
            <div className="flex flex-col justify-end">
              <button
                onClick={handleProcess}
                data-testid="process-image"
                disabled={!file || processing.isProcessing}
                className={`flex items-center gap-2 px-6 py-3 rounded-full font-medium text-sm transition-all ${
                  file && !processing.isProcessing
                    ? 'bg-gradient-to-r from-violet-500 to-cyan-500 text-white shadow-lg shadow-violet-500/25 hover:shadow-violet-500/40 hover:scale-105 active:scale-95'
                    : 'bg-slate-800 text-slate-500 cursor-not-allowed'
                }`}
              >
                {processing.isProcessing ? (
                  <>
                    <Spinner size="sm" color="white" />
                    Processing...
                  </>
                ) : (
                  <>
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                    Remove Background
                  </>
                )}
              </button>
            </div>
          </div>

          {processing.error && (
            <Card className="bg-red-500/10 border-red-500/30 text-red-300 text-sm px-4 py-3">
              {processing.error}
            </Card>
          )}
        </main>}

        <footer className="text-center text-xs text-slate-500 pt-4">
          <p>Powered by ONNX Runtime</p>
        </footer>
      </div>
    </div>
  );
}

export default App;
