import axios from 'axios';

const api = axios.create({
  baseURL: '/',
});

export interface RemoveBgParams {
  model?: string;
  threshold?: number;
  binary?: boolean;
  sticker?: boolean;
  compress?: boolean;
  format?: 'png' | 'jpeg' | 'webp';
  includeMask?: boolean;
}

export interface RemoveBgResult {
  image: Blob;
  imageFilename: string;
  mask: Blob | null;
  maskFilename: string | null;
}

interface RemoveBgJsonResponse {
  imageData: string;
  imageFormat: string;
  filename: string;
  maskData: string | null;
  maskFormat: string | null;
  maskFilename: string | null;
}

export async function removeBackground(file: File, params: RemoveBgParams, apiKey?: string): Promise<RemoveBgResult> {
  const formData = new FormData();
  formData.append('image', file);
  if (params.model !== undefined) formData.append('model', params.model);
  if (params.threshold !== undefined) formData.append('threshold', String(params.threshold));
  if (params.binary !== undefined) formData.append('binary', String(params.binary));
  if (params.sticker !== undefined) formData.append('sticker', String(params.sticker));
  if (params.compress !== undefined) formData.append('compress', String(params.compress));
  if (params.format !== undefined) formData.append('format', params.format);
  if (params.includeMask !== undefined) formData.append('includeMask', String(params.includeMask));

  const response = await api.post('/api/remove-bg', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
      ...(apiKey ? { 'X-API-Key': apiKey } : {}),
    },
    responseType: 'blob',
  });

  if (response.data.type === 'application/json') {
    const payload = JSON.parse(await response.data.text()) as RemoveBgJsonResponse;
    return {
      image: base64ToBlob(payload.imageData, payload.imageFormat),
      imageFilename: payload.filename,
      mask: payload.maskData ? base64ToBlob(payload.maskData, payload.maskFormat ?? 'image/png') : null,
      maskFilename: payload.maskFilename,
    };
  }

  return {
    image: response.data,
    imageFilename: contentDispositionFilename(response.headers['content-disposition']) ?? `result.${params.format ?? 'png'}`,
    mask: null,
    maskFilename: null,
  };
}

function base64ToBlob(base64: string, mimeType: string): Blob {
  const binary = atob(base64);
  const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
  return new Blob([bytes], { type: mimeType });
}

function contentDispositionFilename(value: string | undefined): string | null {
  return value?.match(/filename="([^"]+)"/)?.[1] ?? null;
}

export interface ConfigResponse {
  modelPath: string;
  defaultModel: string;
  models: ModelOption[];
  host: string;
  port: number;
  authRequired: boolean;
}

export interface ModelOption {
  id: string;
  name: string;
  description: string;
}

export async function validateApiKey(apiKey: string): Promise<boolean> {
  try {
    await api.get('/api/auth/validate', { headers: { 'X-API-Key': apiKey } });
    return true;
  } catch {
    return false;
  }
}

export async function getConfig(): Promise<ConfigResponse> {
  const response = await api.get<ConfigResponse>('/api/config');
  return response.data;
}

export async function checkHealth(): Promise<boolean> {
  try {
    await api.get('/healthz');
    return true;
  } catch {
    return false;
  }
}
