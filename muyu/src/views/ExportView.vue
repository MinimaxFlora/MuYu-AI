<script setup lang="ts">
import { useMessage } from "naive-ui";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";

import {
  assetUrl,
  cancelRender,
  detectJianyingRoot,
  exportItems,
  exportJianying,
  exportSrt,
  getProject,
  isTauri,
  openPath,
  renderVideo,
  revealProject,
  saveProject,
} from "../api";
import { formatBytes, formatDuration, modeLabel } from "../format";
import type { DraftResult, Project, RenderOptions, RenderResult } from "../types";
import AppRail from "../components/AppRail.vue";
import Icon from "../components/Icon.vue";
import StepperBar from "../components/StepperBar.vue";

const route = useRoute();
const router = useRouter();
const message = useMessage();
const id = String(route.params.id);

const project = ref<Project | null>(null);
const loading = ref(true);
const error = ref("");
const busy = ref(false);
const result = ref<RenderResult | null>(null);
const draft = ref<DraftResult | null>(null);
const draftBusy = ref(false);
const draftRoot = ref("");
const progress = ref({ percent: 0, message: "准备中…" });
const options = ref<RenderOptions>({
  aspect: "original",
  resolution: "1080p",
  burnSubtitles: true,
  outputName: "",
  bgmPath: null,
  bgmVolume: 0.18,
  bgmDucking: true,
  bgmFade: true,
  duckingStrength: "medium",
  encoder: "auto",
});

let unlisten: (() => void) | undefined;

const aspects = [
  { key: "original", label: "原始" },
  { key: "16:9", label: "16:9" },
  { key: "9:16", label: "9:16" },
  { key: "1:1", label: "1:1" },
];

const items = computed(() => project.value?.items ?? []);
const targets = computed(() =>
  items.value.filter((item) => (item.OST === 0 || item.OST === 2) && item.narration.trim()),
);
const readyCount = computed(() => targets.value.filter((item) => item.audioDuration).length);
const ready = computed(() => targets.value.length > 0 && readyCount.value === targets.value.length);
const missing = computed(() => Math.max(0, targets.value.length - readyCount.value));
const totalDuration = computed(() =>
  items.value.reduce((sum, item) => {
    if (item.audioDuration && (item.OST === 0 || item.OST === 2)) return sum + item.audioDuration;
    const [start, end] = item.timestamp.split("-");
    const parse = (raw: string) => {
      const parts = (raw ?? "").trim().replace(",", ".").split(":");
      if (parts.length !== 3) return 0;
      return Number(parts[0]) * 3600 + Number(parts[1]) * 60 + Number(parts[2]);
    };
    return sum + Math.max(0, parse(end) - parse(start));
  }, 0),
);
const coverUrl = computed(() => assetUrl(project.value?.coverPath));
const videoUrl = computed(() => (result.value ? assetUrl(result.value.path) : ""));
const outputPath = computed(() => result.value?.path ?? "");
const resolutionText = computed(() => {
  if (!result.value) return "—";
  return `${result.value.width}×${result.value.height}`;
});
const encoderLabel = computed(() => {
  const raw = result.value?.encoder ?? "";
  if (raw === "h264_nvenc") return "NVIDIA NVENC";
  if (raw === "h264_qsv") return "Intel QSV";
  if (raw === "h264_amf") return "AMD AMF";
  if (raw === "libx264") return "x264";
  return raw || "—";
});

function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

function errorText(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    const loaded = await getProject(id);
    project.value = loaded;
    result.value = loaded.lastRender ?? null;
    draft.value = loaded.lastDraft ?? null;
    options.value.outputName = loaded.name;
    options.value.bgmPath = loaded.bgmPath ?? null;
    options.value.bgmVolume = loaded.bgmVolume ?? 0.18;
    options.value.bgmDucking = loaded.bgmDucking ?? true;
    options.value.bgmFade = loaded.bgmFade ?? true;
    options.value.duckingStrength = loaded.duckingStrength ?? "medium";
    if (isTauri()) {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<{ percent: number; message: string; done: boolean }>(
        "render://progress",
        (event) => {
          progress.value = {
            percent: event.payload.percent,
            message: event.payload.message,
          };
          if (event.payload.done) busy.value = false;
        },
      );
    }
  } catch (err) {
    error.value = errorText(err);
  } finally {
    loading.value = false;
  }
}

async function start() {
  if (!ready.value) {
    message.warning(missing.value ? `还有 ${missing.value} 个解说镜未配音` : "没有可渲染的镜头");
    return;
  }
  busy.value = true;
  progress.value = { percent: 0, message: "准备中…" };
  try {
    const rendered = await renderVideo(id, { ...options.value });
    result.value = rendered;
    if (project.value) project.value.lastRender = rendered;
    message.success(`成片已输出：${baseName(rendered.path)}`);
  } catch (err) {
    message.error(`渲染失败：${errorText(err)}`);
  } finally {
    busy.value = false;
  }
}

async function stop() {
  await cancelRender();
  message.info("已请求停止，当前镜头完成后终止");
}

async function pickBgm() {
  const selected = await open({
    multiple: false,
    directory: false,
    title: "选择背景音乐",
    filters: [{ name: "Audio", extensions: ["mp3", "wav", "m4a", "aac", "flac", "ogg"] }],
  });
  if (!selected || typeof selected !== "string") return;
  options.value.bgmPath = selected;
  await persistBgm();
}

async function clearBgm() {
  options.value.bgmPath = null;
  await persistBgm();
}

async function toggleBgm(kind: "ducking" | "fade") {
  if (kind === "ducking") options.value.bgmDucking = !options.value.bgmDucking;
  else options.value.bgmFade = !options.value.bgmFade;
  await persistBgm(true);
}

async function persistBgm(silent = false) {
  if (!project.value) return;
  try {
    const saved = await saveProject({
      ...project.value,
      bgmPath: options.value.bgmPath,
      bgmVolume: options.value.bgmVolume,
      bgmDucking: options.value.bgmDucking,
      bgmFade: options.value.bgmFade,
      duckingStrength: options.value.duckingStrength,
    });
    project.value = saved;
    if (!silent) message.success("BGM 设置已保存");
  } catch (err) {
    message.error(`保存 BGM 设置失败：${errorText(err)}`);
  }
}

async function exportSubtitle() {
  try {
    const path = await exportSrt(id);
    message.success(`SRT 已导出：${baseName(path)}`);
  } catch (err) {
    message.error(`导出 SRT 失败：${errorText(err)}`);
  }
}

async function exportJson() {
  try {
    const path = await exportItems(id);
    message.success(`items 已导出：${baseName(path)}`);
  } catch (err) {
    message.error(`导出 items 失败：${errorText(err)}`);
  }
}

async function exportDraft() {
  if (!isTauri()) {
    message.info("浏览器预览模式无法导出剪映草稿");
    return;
  }
  if (!draftRoot.value) {
    await chooseDraftRoot();
  }
  if (!draftRoot.value) return;
  draftBusy.value = true;
  try {
    draft.value = await exportJianying(id, draftRoot.value, project.value?.name);
    message.success(`剪映草稿已生成：${draft.value.name}`);
  } catch (err) {
    message.error(`草稿导出失败：${errorText(err)}`);
  } finally {
    draftBusy.value = false;
  }
}

async function chooseDraftRoot() {
  const root = await open({
    directory: true,
    multiple: false,
    title: "选择剪映草稿目录（JianyingPro 的 draft 文件夹）",
  });
  if (typeof root === "string") draftRoot.value = root;
}

async function openDraftFolder() {
  if (draft.value) await openPath(draft.value.path);
}

async function openFolder() {
  try {
    await revealProject(id);
  } catch (err) {
    message.error(`打开目录失败：${errorText(err)}`);
  }
}

onMounted(load);
onMounted(async () => {
  if (!isTauri()) return;
  try {
    draftRoot.value = (await detectJianyingRoot()) ?? "";
  } catch {
    draftRoot.value = "";
  }
});
onUnmounted(() => unlisten?.());
</script>

<template>
  <div class="app">
    <AppRail />

    <div class="main studio">
      <header class="studio-top">
        <button class="back" title="返回配音" @click="router.push(`/studio/${id}/voice`)">
          <Icon name="arrow-l" :size="16" />
        </button>
        <div class="proj">
          <b>{{ project?.name ?? "加载中…" }}</b>
          <span>
            {{ project ? `${modeLabel(project.mode)} · ${items.length} 镜 · ${formatDuration(totalDuration)}` : "成片导出" }}
          </span>
        </div>
        <div class="studio-grow" />
        <span class="chip violet">P4</span>
        <span v-if="options.bgmPath" class="chip green">BGM</span>
        <span class="chip">{{ options.burnSubtitles ? "字幕烧录" : "无字幕" }}</span>
        <button class="btn ghost" @click="openFolder">
          <Icon name="folder-open" :size="15" />
          打开项目目录
        </button>
        <button class="btn ghost" @click="router.push(`/studio/${id}/voice`)">
          <Icon name="wave" :size="15" />
          返回配音
        </button>
      </header>

      <StepperBar :current="5">
        <template #right>
          <span class="muted mono stepper-note">P3/P4 · 成片与草稿</span>
        </template>
      </StepperBar>

      <div v-if="loading" class="studio-loading">
        <n-spin size="large" />
        <span>读取项目中…</span>
      </div>

      <div v-else-if="error" class="studio-error">
        <Icon name="alert" :size="22" />
        <b>导出页打不开</b>
        <p>{{ error }}</p>
        <button class="btn" @click="router.push(`/studio/${id}/voice`)">返回配音</button>
      </div>

      <div v-else-if="project" class="export-scroll">
        <div class="export-grid">
          <div class="export-main">
            <section class="panel">
              <div class="panel-head">
                <b>输出设置</b>
                <div class="panel-grow" />
                <span class="chip">H.264 · AAC · 25fps</span>
              </div>

              <div class="export-row">
                <label>画面比例</label>
                <div class="aspect-seg">
                  <button
                    v-for="aspect in aspects"
                    :key="aspect.key"
                    :class="{ on: options.aspect === aspect.key }"
                    :disabled="busy"
                    @click="options.aspect = aspect.key"
                  >
                    {{ aspect.label }}
                  </button>
                </div>
              </div>

              <div class="export-row">
                <label>分辨率</label>
                <div class="aspect-seg narrow">
                  <button
                    :class="{ on: options.resolution === '720p' }"
                    :disabled="busy"
                    @click="options.resolution = '720p'"
                  >
                    720p
                  </button>
                  <button
                    :class="{ on: options.resolution === '1080p' }"
                    :disabled="busy"
                    @click="options.resolution = '1080p'"
                  >
                    1080p
                  </button>
                </div>
                <span class="muted" style="font-size: 9px">原始比例时按源视频尺寸</span>
              </div>

              <div class="export-row">
                <label>编码器</label>
                <select v-model="options.encoder" class="mini-select" :disabled="busy" style="max-width: 220px">
                  <option value="auto">自动（优先硬件）</option>
                  <option value="h264_nvenc">NVIDIA NVENC</option>
                  <option value="h264_qsv">Intel QSV</option>
                  <option value="h264_amf">AMD AMF</option>
                  <option value="libx264">x264（兼容）</option>
                </select>
                <span class="muted" style="font-size: 9px">不可用时自动回退 x264</span>
              </div>

              <div class="export-row">
                <label>字幕</label>
                <span
                  class="switch"
                  :class="{ on: options.burnSubtitles }"
                  @click="!busy && (options.burnSubtitles = !options.burnSubtitles)"
                />
                <span class="muted" style="font-size: 9px">
                  {{ options.burnSubtitles ? "烧录进画面（Microsoft YaHei）" : "不烧录，仍会导出 SRT" }}
                </span>
              </div>

              <div class="export-row">
                <label>背景音乐</label>
                <button class="btn ghost sm" :disabled="busy" @click="pickBgm">
                  <Icon name="volume" :size="13" />
                  {{ options.bgmPath ? "更换 BGM" : "选择 BGM" }}
                </button>
                <span v-if="options.bgmPath" class="bgm-name" :title="options.bgmPath">{{ baseName(options.bgmPath) }}</span>
                <button v-if="options.bgmPath" class="link-btn" :disabled="busy" @click="clearBgm">移除</button>
                <div class="export-row-grow" />
                <template v-if="options.bgmPath">
                  <input
                    v-model.number="options.bgmVolume"
                    type="range"
                    min="0"
                    max="0.6"
                    step="0.01"
                    class="bgm-range"
                    :disabled="busy"
                    @change="persistBgm(true)"
                  />
                  <span class="mono muted" style="font-size: 9px">{{ Math.round(options.bgmVolume * 100) }}%</span>
                </template>
              </div>

              <div v-if="options.bgmPath" class="export-row">
                <label>BGM 处理</label>
                <button class="bgm-toggle" :class="{ on: options.bgmDucking }" :disabled="busy" @click="toggleBgm('ducking')">
                  <Icon name="volume" :size="12" />
                  配音闪避
                </button>
                <button class="bgm-toggle" :class="{ on: options.bgmFade }" :disabled="busy" @click="toggleBgm('fade')">
                  <Icon name="wave" :size="12" />
                  淡入淡出
                </button>
                <select
                  v-if="options.bgmDucking"
                  v-model="options.duckingStrength"
                  class="mini-select"
                  style="width: 64px; height: 26px"
                  :disabled="busy"
                  @change="persistBgm(true)"
                >
                  <option value="light">轻</option>
                  <option value="medium">中</option>
                  <option value="strong">强</option>
                </select>
                <span class="muted" style="font-size: 9px">闪避约 -12dB，淡入淡出 1.2s</span>
              </div>

              <div class="export-row">
                <label>输出名称</label>
                <div class="export-name">
                  <n-input v-model:value="options.outputName" :disabled="busy" placeholder="默认使用项目名" />
                  <span class="muted mono">.mp4</span>
                </div>
              </div>

              <div class="export-note">
                <Icon name="alert" :size="13" />
                <span>渲染在本机用 FFmpeg 完成：逐镜裁剪 → 拼接 → 字幕烧录。硬件编码不可用时自动回退 x264，取消会在当前镜头结束后生效。</span>
              </div>
            </section>

            <section class="panel export-preview">
              <div class="panel-head">
                <b>成片预览</b>
                <span v-if="result" class="mini">{{ result.width }}×{{ result.height }} · {{ formatDuration(result.duration) }}</span>
                <div class="panel-grow" />
                <span v-if="result" class="chip" :class="result.subtitleBurned ? 'green' : 'gold'">
                  {{ result.subtitleBurned ? "字幕已烧录" : "未烧录字幕" }}
                </span>
              </div>

              <div v-if="result" class="export-result">
                <video v-if="videoUrl" class="export-video" :src="videoUrl" controls />
                <div class="export-result-meta">
                  <div><span>文件</span><b class="mono">{{ baseName(result.path) }}</b></div>
                  <div><span>大小</span><b class="mono">{{ formatBytes(result.size) }}</b></div>
                  <div><span>时长</span><b class="mono">{{ formatDuration(result.duration) }}</b></div>
                  <div><span>镜头</span><b class="mono">{{ result.segments }}</b></div>
                  <div><span>编码</span><b class="mono">{{ encoderLabel }}</b></div>
                  <div><span>SRT</span><b class="mono">{{ baseName(result.srtPath) }}</b></div>
                </div>
                <div v-if="result.note" class="ai-note">
                  <Icon name="alert" :size="13" />
                  <span>{{ result.note }}</span>
                </div>
                <div class="export-path">{{ result.path }}</div>
              </div>

              <div v-else class="export-placeholder">
                <img v-if="coverUrl" :src="coverUrl" alt="" />
                <div v-else class="poster" style="--p1: #c08434; --p2: #3d2f6b; --p3: #0d2f3a" />
                <div class="export-placeholder-mask">
                  <div class="export-placeholder-ico"><Icon name="film" :size="22" /></div>
                  <b>还没有成片</b>
                  <p>确认右侧的镜头与配音，点「开始渲染」。</p>
                </div>
              </div>
            </section>

            <section class="panel">
              <div class="panel-head"><b>导出文件</b><div class="panel-grow" /><span class="mini">存于项目 /output</span></div>
              <div class="export-files">
                <button class="export-file" :disabled="busy" @click="exportSubtitle">
                  <Icon name="text" :size="18" />
                  <div><b>SRT 字幕</b><span>按配音时长重新计时</span></div>
                  <Icon name="download" :size="14" />
                </button>
                <button class="export-file" :disabled="busy" @click="exportJson">
                  <Icon name="layers" :size="18" />
                  <div><b>items JSON</b><span>含 OST / 时间码 / 解说</span></div>
                  <Icon name="download" :size="14" />
                </button>
                <button class="export-file" :disabled="busy || draftBusy" @click="exportDraft">
                  <Icon name="scissor" :size="18" />
                  <div>
                    <b>{{ draftBusy ? "生成草稿中…" : "剪映草稿" }}</b>
                    <span>视频 / 配音 / 字幕轨 + 素材复制</span>
                  </div>
                  <Icon name="download" :size="14" />
                </button>
              </div>
              <div v-if="draftRoot" class="muted mono" style="margin-top: 7px; font-size: 9px; display: flex; gap: 8px; align-items: center">
                <span>草稿目录</span>
                <span :title="draftRoot" style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 330px">{{ draftRoot }}</span>
                <button class="btn ghost sm" style="padding: 2px 8px" @click="chooseDraftRoot">更改</button>
              </div>

              <div v-if="draft" class="draft-result">
                <div class="draft-result-head">
                  <b>草稿已生成</b>
                  <span class="mini">{{ draft.tracks }} 轨 · {{ draft.assets }} 素材 · {{ formatDuration(draft.duration) }}</span>
                  <div class="panel-grow" />
                  <button class="btn ghost sm" @click="openDraftFolder">打开目录</button>
                </div>
                <div class="export-path">{{ draft.path }}</div>
                <div v-for="warning in draft.warnings" :key="warning" class="ai-note" style="margin-top: 6px">
                  <Icon name="alert" :size="13" />
                  <span>{{ warning }}</span>
                </div>
              </div>
            </section>
          </div>

          <aside class="export-side">
            <section class="panel">
              <div class="panel-head"><b>本次导出</b><div class="panel-grow" /><span class="chip" :class="ready ? 'green' : 'gold'">{{ ready ? "就绪" : "待配音" }}</span></div>
              <div class="export-cover">
                <img v-if="coverUrl" :src="coverUrl" alt="" />
                <div v-else class="poster" style="--p1: #c08434; --p2: #3d2f6b; --p3: #0d2f3a" />
                <div class="export-cover-shade" />
                <div class="export-cover-title">{{ project.name }}</div>
              </div>
              <div class="export-stat">
                <div><span>镜头</span><b>{{ items.length }}</b></div>
                <div><span>解说镜</span><b>{{ targets.length }}</b></div>
                <div><span>已配音</span><b>{{ readyCount }}/{{ targets.length }}</b></div>
                <div><span>成片时长</span><b>{{ formatDuration(totalDuration) }}</b></div>
                <div><span>画面</span><b>{{ resolutionText }}</b></div>
              </div>

              <div v-if="!ready" class="ai-note" style="margin-top: 10px">
                <Icon name="alert" :size="13" />
                <span>还有 {{ missing }} 个解说镜未配音，先去配音页补齐。</span>
              </div>

              <div class="voice-progress" v-if="busy || progress.percent > 0">
                <div class="vp-top">
                  <span>{{ progress.message }}</span>
                  <span class="mono">{{ Math.round(progress.percent) }}%</span>
                </div>
                <div class="vp-bar"><i :style="{ width: `${progress.percent}%` }" /></div>
              </div>

              <button class="btn primary lg full" style="margin-top: 12px" :disabled="busy || !ready" @click="start">
                <Icon name="film" :size="16" />
                {{ busy ? "渲染中…" : result ? "重新渲染" : "开始渲染" }}
              </button>
              <button v-if="busy" class="btn ghost full stop-btn" style="margin-top: 8px" @click="stop">停止渲染</button>
              <button v-if="!ready" class="btn ghost full" style="margin-top: 8px" @click="router.push(`/studio/${id}/voice`)">
                去配音
              </button>
              <div class="muted center-note">{{ result ? `上次渲染 ${new Date(result.renderedAt).toLocaleTimeString("zh-CN")}` : "首次渲染会创建临时片段目录" }}</div>
            </section>

            <section v-if="draft" class="panel">
              <div class="panel-head">
                <b>剪映草稿</b>
                <div class="panel-grow" />
                <span class="chip green">已生成</span>
              </div>
              <div class="export-stat">
                <div><span>轨道</span><b>{{ draft.tracks }}</b></div>
                <div><span>素材</span><b>{{ draft.assets }}</b></div>
                <div><span>时长</span><b>{{ formatDuration(draft.duration) }}</b></div>
                <div><span>镜头</span><b>{{ draft.segments }}</b></div>
              </div>
              <div class="export-path" style="margin-top: 8px">{{ draft.path }}</div>
              <button class="btn ghost sm" style="margin-top: 8px" @click="openDraftFolder">
                <Icon name="folder-open" :size="13" />
                打开草稿目录
              </button>
              <div v-if="draft.warnings.length" class="ai-note" style="margin-top: 8px">
                <Icon name="alert" :size="13" />
                <span>{{ draft.warnings[0] }}</span>
              </div>
            </section>

            <section class="panel">
              <div class="panel-head"><b>输出目录</b><div class="panel-grow" /><span class="mini">项目内</span></div>
              <div class="export-path">{{ outputPath || "渲染后显示输出文件" }}</div>
              <button class="btn ghost sm" style="margin-top: 8px" @click="openFolder">
                <Icon name="folder-open" :size="13" />
                在资源管理器中打开
              </button>
            </section>

            <section class="tip">
              <div class="t-kicker">P3 NOTE</div>
              <b>画面对齐配音</b>
              <p>OST=0 的镜头会按配音时长裁剪并去掉原声；OST=1 保留原声；OST=2 混合。渲染完成后可以直接播放检查。</p>
            </section>
          </aside>
        </div>
      </div>
    </div>
  </div>
</template>
