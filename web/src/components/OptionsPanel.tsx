import type { CSSProperties } from 'react';
import type { ModelOption } from '../api/client';

interface OptionsPanelProps {
  models: ModelOption[];
  model: string;
  setModel: (value: string) => void;
  threshold: number;
  setThreshold: (value: number) => void;
  binary: boolean;
  setBinary: (value: boolean) => void;
  sticker: boolean;
  setSticker: (value: boolean) => void;
  compress: boolean;
  setCompress: (value: boolean) => void;
  format: 'png' | 'jpeg' | 'webp';
  setFormat: (value: 'png' | 'jpeg' | 'webp') => void;
  includeMask: boolean;
  setIncludeMask: (value: boolean) => void;
}

interface ToggleProps {
  label: string;
  hint: string;
  checked: boolean;
  onChange: (value: boolean) => void;
  testId?: string;
}

function Toggle({ label, hint, checked, onChange, testId }: ToggleProps) {
  return (
    <label className="option-toggle">
      <span>
        <strong>{label}</strong>
        <small>{hint}</small>
      </span>
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
        data-testid={testId}
      />
      <span className="toggle-track" aria-hidden="true" />
    </label>
  );
}

export function OptionsPanel({
  models,
  model,
  setModel,
  threshold,
  setThreshold,
  binary,
  setBinary,
  sticker,
  setSticker,
  compress,
  setCompress,
  format,
  setFormat,
  includeMask,
  setIncludeMask,
}: OptionsPanelProps) {
  const selectedModel = models.find((option) => option.id === model);

  return (
    <section className="options-panel" aria-labelledby="settings-heading">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Processing</p>
          <h2 id="settings-heading">Output settings</h2>
        </div>
        <span className="settings-badge">Local inference</span>
      </div>

      <div className="select-grid">
        <label className="field-label">
          <span>Model</span>
          <select
            value={model}
            onChange={(event) => setModel(event.target.value)}
            data-testid="model-select"
          >
            {models.map((option) => (
              <option key={option.id} value={option.id}>{option.name}</option>
            ))}
          </select>
          <small>{selectedModel?.description ?? 'Server default model'}</small>
        </label>

        <label className="field-label">
          <span>Output format</span>
          <select
            value={format}
            onChange={(event) => setFormat(event.target.value as 'png' | 'jpeg' | 'webp')}
          >
            <option value="png">PNG, transparent</option>
            <option value="webp">WebP, transparent</option>
            <option value="jpeg">JPEG, white background</option>
          </select>
          <small>PNG preserves the cleanest transparent edge.</small>
        </label>
      </div>

      <label className="threshold-field">
        <span className="threshold-heading">
          <span>Removal threshold</span>
          <output>{threshold}</output>
        </span>
        <input
          type="range"
          value={threshold}
          onChange={(event) => setThreshold(Number(event.target.value))}
          min="0"
          max="255"
          step="1"
          style={{ '--range-progress': `${(threshold / 255) * 100}%` } as CSSProperties}
        />
        <small>Increase for more aggressive background removal.</small>
      </label>

      <div className="toggle-grid">
        <Toggle label="Binary edges" hint="Create a hard alpha edge" checked={binary} onChange={setBinary} />
        <Toggle label="Sticker outline" hint="Add a border around the subject" checked={sticker} onChange={setSticker} />
        <Toggle label="Compress PNG" hint="Reduce the final file size" checked={compress} onChange={setCompress} />
        <Toggle label="Include mask" hint="Generate a separate mask image" checked={includeMask} onChange={setIncludeMask} testId="include-mask" />
      </div>
    </section>
  );
}
