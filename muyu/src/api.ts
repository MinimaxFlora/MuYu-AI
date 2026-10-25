import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import { demoProject, demoProjects } from "./demo";
import type {
  DraftResult,
  LlmSettings,
  Project,
  ProjectSummary,
  RenderOptions,
  RenderResult,
  ScriptItem,
  ScriptParams,
  TtsVoice,
  UpdateInfo,
} from "./types";

let demoState: Project = JSON.parse(JSON.stringify(demoProject)) as Project;

const DEFAULT_LLM: LlmSettings = {
  baseUrl: "https://api.deepseek.com/v1",
  apiKey: "",
  model: "deepseek-chat",
  temperature: 0.8,
  demoMode: false,
  updateRepo: "MinimaxFlora/MuYu-AI",
  autoCheckUpdate: true,
};

function cloneDemo(): Project {
  return JSON.parse(JSON.stringify(demoState)) as Project;
}

export const isTauri = (): boolean =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

const VIDEO_EXTENSIONS = ["mp4", "mov", "mkv", "webm", "avi", "m4v"];

export function isVideoFile(path: string): boolean {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  return VIDEO_EXTENSIONS.includes(ext);
}

export async function listProjects(): Promise<ProjectSummary[]> {
  if (!isTauri()) return demoProjects;
  return invoke<ProjectSummary[]>("list_projects");
}

export async function getProject(id: string): Promise<Project> {
  if (!isTauri()) {
    if (id.startsWith("demo")) return cloneDemo();
    throw new Error("预览模式下没有这个项目");
  }
  return invoke<Project>("get_project", { id });
}

export async function createProject(
  paths: string[],
  name?: string,
  mode = "shortline",
): Promise<Project> {
  if (!isTauri()) {
    throw new Error("浏览器预览模式无法导入素材，请用 pnpm tauri dev 启动桌面端");
  }
  return invoke<Project>("create_project", { paths, name, mode });
}

export async function saveProject(project: Project): Promise<Project> {
  if (!isTauri()) {
    demoState = { ...project, updatedAt: new Date().toISOString() };
    return demoState;
  }
  return invoke<Project>("save_project", { project });
}

export async function deleteProject(id: string): Promise<void> {
  if (!isTauri()) return;
  await invoke("delete_project", { id });
}

export async function attachSubtitle(id: string, path: string): Promise<Project> {
  if (!isTauri()) throw new Error("浏览器预览模式无法导入字幕");
  return invoke<Project>("attach_subtitle", { id, path });
}

export async function removeSubtitle(id: string): Promise<Project> {
  if (!isTauri()) throw new Error("浏览器预览模式无法移除字幕");
  return invoke<Project>("remove_subtitle", { id });
}

export async function revealProject(id: string): Promise<void> {
  if (!isTauri()) return;
  await invoke("reveal_project", { id });
}

export interface EnvironmentStatus {
  ffmpeg: boolean;
  ffprobe: boolean;
  ffmpegVersion: string;
}

export async function checkEnvironment(): Promise<EnvironmentStatus> {
  if (!isTauri()) {
    return { ffmpeg: false, ffprobe: false, ffmpegVersion: "预览模式" };
  }
  return invoke<EnvironmentStatus>("check_environment");
}

export async function pickVideos(): Promise<string[]> {
  const result = await open({
    multiple: true,
    directory: false,
    title: "选择视频素材",
    filters: [{ name: "视频", extensions: VIDEO_EXTENSIONS }],
  });
  if (!result) return [];
  return Array.isArray(result) ? result : [result];
}

export async function pickSubtitle(): Promise<string | null> {
  const result = await open({
    multiple: false,
    directory: false,
    title: "选择 SRT 字幕",
    filters: [{ name: "字幕", extensions: ["srt"] }],
  });
  return typeof result === "string" ? result : null;
}

export function assetUrl(path?: string | null): string {
  if (!path || !isTauri()) return "";
  try {
    return convertFileSrc(path);
  } catch {
    return "";
  }
}

/* ---------- P1: settings + AI pipeline ---------- */

export async function getSettings(): Promise<LlmSettings> {
  if (!isTauri()) {
    const raw = localStorage.getItem("muyu.llm.settings");
    return raw ? (JSON.parse(raw) as LlmSettings) : { ...DEFAULT_LLM };
  }
  return invoke<LlmSettings>("get_settings");
}

export async function saveSettings(settings: LlmSettings): Promise<LlmSettings> {
  if (!isTauri()) {
    localStorage.setItem("muyu.llm.settings", JSON.stringify(settings));
    return settings;
  }
  return invoke<LlmSettings>("save_settings", { settings });
}

export async function testLlm(settings: LlmSettings): Promise<string> {
  if (!isTauri()) {
    return settings.apiKey
      ? "浏览器预览模式不会发起真实请求"
      : "演示模式：未配置 API Key";
  }
  return invoke<string>("test_llm", { settings });
}

export async function analyzePlot(id: string): Promise<Project> {
  if (!isTauri()) {
    demoState.plotAnalysis = demoPlot(demoState);
    demoState.pipelineNote = "演示模式：剧情理解为本地规则生成。";
    return cloneDemo();
  }
  return invoke<Project>("analyze_plot", { id });
}

export async function generateCopy(id: string, params: ScriptParams): Promise<Project> {
  if (!isTauri()) {
    demoState.params = params;
    demoState.narrationCopy = demoCopy(demoState);
    demoState.items = [];
    demoState.pipelineNote = "演示模式：文案为本地规则生成。";
    return cloneDemo();
  }
  return invoke<Project>("generate_copy", { id, params });
}

export async function generateItems(id: string): Promise<Project> {
  if (!isTauri()) {
    demoState.items = demoItems(demoState);
    demoState.pipelineNote = "演示模式：分镜为本地规则生成。";
    return cloneDemo();
  }
  return invoke<Project>("generate_items", { id });
}

function demoPlot(project: Project): string {
  const cues = project.subtitles.slice(0, 6).map((cue) => cue.text.trim());
  const first = cues[0] ?? "故事从一个不起眼的细节开始。";
  const lines = [
    `一句话故事：${first.slice(0, 60)}`,
    "人物与关系：以字幕里的关键人物为核心，围绕一次意外与隐藏证据展开；表面关系之下还有未说出口的动机。",
    "主线脉络：",
    ...cues.map((cue, index) => `${index + 1}. ${cue.slice(0, 42)}`),
    "可用钩子：",
    "- 这件关键物件为什么会出现？",
    "- 谁在说谎，谁又在替谁隐瞒？",
    "- 当证据指向最信任的人，他会怎么选？",
    "注意事项：本结果由本地规则生成，仅用于打通流程；配置模型后会得到更完整的人物动机与因果链。",
  ];
  return lines.join("\n");
}

function demoCopy(project: Project): string {
  const limit = Math.min(project.subtitles.length, 6);
  const sentences: string[] = [];
  if (limit >= 2) sentences.push("开场，先别急着看画面，这件事远没有表面那么简单。");
  for (const cue of project.subtitles.slice(0, limit - sentences.length)) {
    const text = cue.text.replace(/\n/g, " ").trim().slice(0, 30);
    if (text) sentences.push(/[。！？!?]$/.test(text) ? text : `${text}。`);
  }
  return sentences.join("");
}

function splitSentences(text: string): string[] {
  return text
    .split(/(?<=[。！？!?；;])/)
    .map((sentence) => sentence.trim())
    .filter(Boolean);
}

function fmtTs(seconds: number): string {
  const total = Math.max(0, seconds);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = Math.floor(total % 60);
  const ms = Math.round((total % 1) * 1000);
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")},${String(ms).padStart(3, "0")}`;
}

function demoItems(project: Project): ScriptItem[] {
  if (!project.subtitles.length) return [];
  const sentences = splitSentences(project.narrationCopy || demoCopy(project));
  const videoName = project.videos[0]?.name ?? "video.mp4";
  const items = sentences.slice(0, 120).map((narration, index) => {
    const cue = project.subtitles[Math.min(index, project.subtitles.length - 1)];
    return {
      _id: index + 1,
      videoId: 1,
      videoName,
      timestamp: `${fmtTs(cue.start)}-${fmtTs(cue.end)}`,
      picture: cue.text.replace(/\n/g, " ").slice(0, 40),
      narration,
      OST: 0,
    };
  });
  if (items.length >= 2) items[items.length - 1].OST = 1;
  return items;
}

/* ---------- P2: TTS ---------- */

const DEMO_EDGE_VOICES: TtsVoice[] = [
  { id: "zh-CN-YunxiNeural", label: "zh-CN-YunxiNeural · 男", locale: "zh-CN", gender: "男" },
  { id: "zh-CN-XiaoxiaoNeural", label: "zh-CN-XiaoxiaoNeural · 女", locale: "zh-CN", gender: "女" },
  { id: "zh-CN-YunyangNeural", label: "zh-CN-YunyangNeural · 男", locale: "zh-CN", gender: "男" },
  { id: "zh-CN-XiaoyiNeural", label: "zh-CN-XiaoyiNeural · 女", locale: "zh-CN", gender: "女" },
];

export async function listTtsVoices(engine: string): Promise<TtsVoice[]> {
  if (!isTauri()) {
    return engine === "sapi"
      ? [{ id: "Microsoft Huihui Desktop", label: "Microsoft Huihui Desktop · zh-CN", locale: "zh-CN", gender: "" }]
      : DEMO_EDGE_VOICES;
  }
  return invoke<TtsVoice[]>("list_tts_voices", { engine });
}

export async function previewVoice(engine: string, voice: string, rate: number): Promise<string> {
  if (!isTauri()) throw new Error("浏览器预览模式无法合成音频");
  return invoke<string>("preview_voice", { engine, voice, rate });
}

export async function synthesizeItems(
  id: string,
  engine: string,
  voice: string,
  rate: number,
  force: boolean,
): Promise<Project> {
  if (!isTauri()) {
    demoState.items = demoState.items.map((item) => ({
      ...item,
      audioDuration: Math.max(1, item.narration.replace(/\s/g, "").length / 5),
      audioPath: null,
    }));
    demoState.voice = voice;
    demoState.ttsEngine = engine;
    demoState.voiceRate = rate;
    demoState.ttsNote = "浏览器预览模式：仅计算时长，不生成音频。";
    return cloneDemo();
  }
  return invoke<Project>("synthesize_items", { id, engine, voice, rate, force });
}

export async function cancelTts(): Promise<void> {
  if (!isTauri()) return;
  await invoke("cancel_tts");
}

/* ---------- P3: render & export ---------- */

export async function renderVideo(id: string, options: RenderOptions): Promise<RenderResult> {
  if (!isTauri()) throw new Error("浏览器预览模式无法渲染成片，请用 pnpm tauri dev");
  return invoke<RenderResult>("render_video", { id, options });
}

export async function cancelRender(): Promise<void> {
  if (!isTauri()) return;
  await invoke("cancel_render");
}

export async function exportSrt(id: string): Promise<string> {
  if (!isTauri()) throw new Error("浏览器预览模式无法导出字幕");
  return invoke<string>("export_srt", { id });
}

export async function exportItems(id: string): Promise<string> {
  if (!isTauri()) throw new Error("浏览器预览模式无法导出 items");
  return invoke<string>("export_items", { id });
}

export async function exportJianying(
  id: string,
  root: string,
  name?: string,
): Promise<DraftResult> {
  if (!isTauri()) throw new Error("浏览器预览模式无法导出剪映草稿");
  return invoke<DraftResult>("export_jianying", { id, root, name });
}

export async function detectJianyingRoot(): Promise<string | null> {
  if (!isTauri()) return null;
  return invoke<string | null>("detect_jianying_root");
}

export async function allowAssetPath(path: string): Promise<string> {
  if (!isTauri()) return path;
  return invoke<string>("allow_asset_path", { path });
}

export async function openPath(path: string): Promise<void> {
  if (!isTauri()) return;
  await invoke("open_path", { path });
}

export async function openLogDir(): Promise<string> {
  if (!isTauri()) throw new Error("浏览器预览模式没有日志目录");
  return invoke<string>("open_log_dir");
}

export async function checkUpdate(): Promise<UpdateInfo | null> {
  if (!isTauri()) throw new Error("浏览器预览模式不能检查更新");
  return invoke<UpdateInfo | null>("check_update");
}

export async function installUpdate(): Promise<void> {
  if (!isTauri()) throw new Error("浏览器预览模式不能安装更新");
  await invoke("install_update");
}
