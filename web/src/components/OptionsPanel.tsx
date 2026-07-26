import React from 'react';
import { Card, Slider, Switch, Select, SelectItem } from '@heroui/react';

interface OptionsPanelProps {
  threshold: number;
  setThreshold: (v: number) => void;
  binary: boolean;
  setBinary: (v: boolean) => void;
  sticker: boolean;
  setSticker: (v: boolean) => void;
  compress: boolean;
  setCompress: (v: boolean) => void;
  format: 'png' | 'jpeg' | 'webp';
  setFormat: (v: 'png' | 'jpeg' | 'webp') => void;
  includeMask: boolean;
  setIncludeMask: (v: boolean) => void;
}

export function OptionsPanel({
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
  return (
    <Card className="bg-slate-800/40 border-slate-700/40 backdrop-blur-sm">
      <div className="space-y-5 p-1">
        <div>
          <Slider
            label="Threshold"
            value={threshold}
            onChange={(v) => setThreshold(v as number)}
            minValue={0}
            maxValue={255}
            step={1}
            size="sm"
            color="violet"
            className="max-w-md"
          />
          <p className="text-xs text-slate-400 mt-1">
            Higher = more aggressive removal
          </p>
        </div>

        <div className="flex items-center justify-between">
          <span className="text-sm text-slate-200">Binary mode</span>
          <Switch
            isSelected={binary}
            onValueChange={setBinary}
            size="sm"
            color="violet"
          >
            Hard edges
          </Switch>
        </div>

        <div className="flex items-center justify-between">
          <span className="text-sm text-slate-200">Sticker mode</span>
          <Switch
            isSelected={sticker}
            onValueChange={setSticker}
            size="sm"
            color="violet"
          >
            Outline border
          </Switch>
        </div>

        <div className="flex items-center justify-between">
          <span className="text-sm text-slate-200">Compress PNG</span>
          <Switch
            isSelected={compress}
            onValueChange={setCompress}
            size="sm"
            color="violet"
          >
            Smaller file
          </Switch>
        </div>

        <div>
          <Select
            label="Format"
            selectedKeys={[format]}
            onSelectionChange={(keys) => {
              const val = Array.from(keys)[0] as 'png' | 'jpeg' | 'webp';
              if (val) setFormat(val);
            }}
            size="sm"
            className="max-w-xs"
          >
            <SelectItem key="png">PNG</SelectItem>
            <SelectItem key="jpeg">JPEG</SelectItem>
            <SelectItem key="webp">WebP</SelectItem>
          </Select>
        </div>

        <div className="flex items-center justify-between">
          <span className="text-sm text-slate-200">Include mask</span>
          <Switch
            data-testid="include-mask"
            isSelected={includeMask}
            onValueChange={setIncludeMask}
            size="sm"
            color="violet"
          >
            Return mask image
          </Switch>
        </div>
      </div>
    </Card>
  );
}
