export interface MediaInfo {
  duration: number;
  width: number;
  height: number;
  fps: number;
  codec: string;
  bitRate: number;
  size: number;
}

export interface VideoAsset {
  path: string;
  name: string;
  info: MediaInfo;
  trimStart: number;
  trimEnd: number;
}

export interface SubtitleCue {
  index: number;
  start: number;
  end: number;
  text: string;
}

export interface ScriptParams {
  genre: string;
  language: string;
  targetWords: number;
  density: "dense" | "standard" | "loose";
  originalRatio: number;
}

export interface ScriptItem {
  _id: number;
  videoId: number;
  videoName: string;
  timestamp: string;
  picture: string;
  narration: string;
  OST: number;
  audioPath?: string | null;
  audioDuration?: number | null;
}

export interface Project {
  id: string;
  name: string;
  mode: string;
  status: string;
  createdAt: string;
  updatedAt: string;
  videos: VideoAsset[];
  subtitlePath?: string | null;
  subtitles: SubtitleCue[];
  coverPath?: string | null;
  plotAnalysis?: string | null;
  narrationCopy?: string | null;
  items: ScriptItem[];
  params: ScriptParams;
  pipelineNote?: string | null;
  voice: string;
  ttsEngine: string;
  voiceRate: number;
  ttsNote?: string | null;
  lastRender?: RenderResult | null;
  lastDraft?: DraftResult | null;
  bgmPath?: string | null;
  bgmVolume: number;
  bgmDucking: boolean;
  bgmFade: boolean;
  duckingStrength: string;
}

export interface TtsVoice {
  id: string;
  label: string;
  locale: string;
  gender: string;
}

export interface TtsProgress {
  current: number;
  total: number;
  itemId?: number | null;
  percent: number;
  message: string;
  done: boolean;
}

export interface RenderOptions {
  aspect: string;
  resolution: string;
  burnSubtitles: boolean;
  outputName: string;
  bgmPath?: string | null;
  bgmVolume: number;
  bgmDucking: boolean;
  bgmFade: boolean;
  duckingStrength: string;
  encoder: string;
}

export interface RenderResult {
  path: string;
  srtPath: string;
  duration: number;
  size: number;
  width: number;
  height: number;
  segments: number;
  subtitleBurned: boolean;
  note?: string | null;
  renderedAt: string;
  encoder?: string;
}

export interface DraftResult {
  path: string;
  name: string;
  duration: number;
  segments: number;
  tracks: number;
  assets: number;
  warnings: string[];
}

export interface LlmSettings {
  baseUrl: string;
  apiKey: string;
  model: string;
  temperature: number;
  demoMode: boolean;
  updateRepo: string;
  autoCheckUpdate: boolean;
}

export interface UpdateInfo {
  currentVersion: string;
  version: string;
  body?: string | null;
  date?: string | null;
  repo: string;
}

export interface UpdateProgress {
  downloaded: number;
  total?: number | null;
  percent: number;
  message: string;
}

export interface ProjectSummary {
  id: string;
  name: string;
  mode: string;
  status: string;
  updatedAt: string;
  coverPath?: string | null;
  duration: number;
  videoCount: number;
  subtitleCount: number;
}