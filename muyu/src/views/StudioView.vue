<script setup lang="ts">
import { useDialog, useMessage } from "naive-ui";
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";

import {
  allowAssetPath,
  assetUrl,
  attachSubtitle,
  getProject,
  isTauri,
  pickSubtitle,
  removeSubtitle,
  revealProject,
  saveProject,
} from "../api";
import { formatBytes, formatCueTime, formatDuration, formatShortTime, modeLabel } from "../format";
import type { Project } from "../types";
import AppRail from "../components/AppRail.vue";
import Icon from "../components/Icon.vue";
import StepperBar from "../components/StepperBar.vue";

const route = useRoute();
const router = useRouter();
const message = useMessage();
const dialog = useDialog();

const desktop = isTauri();

const project = ref<Project | null>(null);
const loading = ref(true);
const error = ref("");
const activeIndex = ref(0);
const trim = ref<[number, number]>([0, 0]);
const savedState = ref<"saved" | "saving" | "error">("saved");
const coverFailed = ref(false);
const videoFailed = ref(false);
const previewPaths = ref<Record<string, string>>({});

let saveTimer: ReturnType<typeof setTimeout> | undefined;

const activeVideo = computed(() => project.value?.videos[activeIndex.value]);
const duration = computed(() => activeVideo.value?.info.duration ?? 0);
const coverUrl = computed(() => assetUrl(project.value?.coverPath));
const videoUrl = computed(() => {
  const path = activeVideo.value?.path;
  return assetUrl(path ? previewPaths.value[path] ?? path : null);
});
const posterStyle = computed(() => {
  const palettes = [
    ["#c08434", "#3d2f6b", "#0d2f3a"],
    ["#2f6f7a", "#193a57", "#0a1a24"],
    ["#7a5a2c", "#2f4a3d", "#101c18"],
  ];
  const id = project.value?.id ?? "muyu";
  let hash = 0;
  for (const char of id) hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
  const [p1, p2, p3] = palettes[hash % palettes.length];
  return { "--p1": p1, "--p2": p2, "--p3": p3 };
});
const titleChars = computed(() => {
  const raw = project.value?.name.replace(/[《》\s]/g, "") ?? "幕语";
  return Array.from(raw).slice(0, 2);
});
const subtitleName = computed(() => {
  const path = project.value?.subtitlePath;
  if (!path) return "";
  return path.split(/[\\/]/).pop() ?? path;
});
const visibleCues = computed(() => project.value?.subtitles.slice(0, 500) ?? []);

function errorText(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

function syncTrimFromProject() {
  const video = project.value?.videos[activeIndex.value];
  trim.value = video ? [video.trimStart, video.trimEnd] : [0, 0];
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    project.value = await getProject(String(route.params.id));
    activeIndex.value = 0;
    videoFailed.value = false;
    syncTrimFromProject();
    if (isTauri()) {
      await Promise.all(
        project.value.videos.map(async (video) => {
          try {
            previewPaths.value[video.path] = await allowAssetPath(video.path);
          } catch {
            // 回退到原始路径，播放失败时显示封面
          }
        }),
      );
    }
  } catch (err) {
    error.value = errorText(err);
  } finally {
    loading.value = false;
  }
}

function onTrim(value: number | [number, number]) {
  if (!Array.isArray(value)) return;
  trim.value = [value[0], value[1]];
  const video = project.value?.videos[activeIndex.value];
  if (!video) return;
  video.trimStart = value[0];
  video.trimEnd = value[1];
  scheduleSave();
}

function scheduleSave() {
  if (!isTauri()) return;
  savedState.value = "saving";
  clearTimeout(saveTimer);
  saveTimer = setTimeout(async () => {
    if (!project.value) return;
    try {
      project.value = await saveProject(project.value);
      savedState.value = "saved";
    } catch (err) {
      savedState.value = "error";
      message.error(`保存失败：${errorText(err)}`);
    }
  }, 600);
}

async function importSubtitle() {
  if (!isTauri()) {
    message.info("浏览器预览模式无法导入字幕");
    return;
  }
  if (!project.value) return;
  const path = await pickSubtitle();
  if (!path) return;
  try {
    project.value = await attachSubtitle(project.value.id, path);
    message.success(`已挂载字幕，共 ${project.value.subtitles.length} 条`);
  } catch (err) {
    message.error(`解析字幕失败：${errorText(err)}`);
  }
}

function clearSubtitle() {
  if (!project.value) return;
  dialog.warning({
    title: "移除字幕",
    content: "只移除项目里的字幕记录，不会删除 SRT 文件。",
    positiveText: "移除",
    negativeText: "取消",
    onPositiveClick: async () => {
      try {
        project.value = await removeSubtitle(project.value!.id);
        message.success("字幕已移除");
      } catch (err) {
        message.error(`移除失败：${errorText(err)}`);
      }
    },
  });
}

async function openFolder() {
  if (!project.value) return;
  try {
    await revealProject(project.value.id);
  } catch (err) {
    message.error(`打开目录失败：${errorText(err)}`);
  }
}

watch(
  () => route.params.id,
  () => void load(),
);

onMounted(load);
</script>

<template>
  <div class="app">
    <AppRail />

    <div class="main studio">
      <header class="studio-top">
        <button class="back" title="返回" @click="router.push('/')">
          <Icon name="arrow-l" :size="16" />
        </button>
        <div class="proj">
          <b>{{ project?.name ?? "加载中…" }}</b>
          <span>
            {{ project ? modeLabel(project.mode) : "素材" }}
            <template v-if="activeVideo">
              · {{ formatDuration(duration) }} · {{ activeVideo.info.width }}×{{ activeVideo.info.height }}
            </template>
          </span>
        </div>

        <div class="studio-grow" />

        <span v-if="!desktop" class="chip violet">预览模式</span>
        <span v-else class="save-state" :class="savedState">
          <span class="dot" />
          {{ savedState === "saving" ? "保存中…" : savedState === "error" ? "保存失败" : "已自动保存" }}
        </span>

        <button class="btn ghost" :disabled="!project" @click="openFolder">
          <Icon name="folder-open" :size="15" />
          项目文件夹
        </button>
        <button class="btn primary" disabled title="P1 开放">
          导出
          <Icon name="download" :size="15" />
        </button>
      </header>

      <StepperBar :current="1">
        <template #right>
          <span class="muted mono stepper-note">P0 · 素材阶段</span>
        </template>
      </StepperBar>

      <div v-if="loading" class="studio-loading">
        <n-spin size="large" />
        <span>读取项目中…</span>
      </div>

      <div v-else-if="error" class="studio-error">
        <Icon name="alert" :size="22" />
        <b>项目打不开</b>
        <p>{{ error }}</p>
        <button class="btn" @click="router.push('/')">返回首页</button>
      </div>

      <div v-else-if="project" class="studio-body">
        <!-- Left: assets -->
        <aside class="studio-col left">
          <div class="col-head">
            <b>素材</b>
            <span class="mini">{{ project.videos.length }}</span>
            <div class="col-grow" />
            <Icon name="more" :size="14" class="dim" />
          </div>

          <div class="video-list">
            <button
              v-for="(video, index) in project.videos"
              :key="video.path"
              class="video-item"
              :class="{ active: index === activeIndex }"
              @click="activeIndex = index; syncTrimFromProject()"
            >
              <div class="v-thumb poster" :style="posterStyle">
                <span>{{ String(index + 1).padStart(2, "0") }}</span>
              </div>
              <div class="v-body">
                <div class="v-name">{{ video.name }}</div>
                <div class="v-meta">{{ formatDuration(video.info.duration) }} · {{ video.info.width }}×{{ video.info.height }}</div>
              </div>
              <Icon v-if="index === activeIndex" name="wave" :size="14" class="v-active" />
            </button>

            <n-tooltip trigger="hover">
              <template #trigger>
                <button class="add-video" disabled>
                  <Icon name="plus" :size="14" />
                  再导入一集
                </button>
              </template>
              P1 支持在项目中追加素材
            </n-tooltip>
          </div>

          <div class="left-divider" />

          <div class="col-head">
            <b>字幕</b>
            <span class="mini">{{ project.subtitles.length }}</span>
            <div class="col-grow" />
            <span v-if="project.subtitlePath" class="chip green">已挂载</span>
            <span v-else class="chip">未挂载</span>
          </div>

          <div class="subtitle-box">
            <div v-if="subtitleName" class="sub-file" :title="project.subtitlePath ?? ''">
              <Icon name="text" :size="14" />
              <span>{{ subtitleName }}</span>
            </div>
            <p v-else class="sub-empty">
              没有找到同名字幕。导入 SRT 后，文案阶段才能读取原始对白。
            </p>
            <div class="sub-actions">
              <button class="btn soft-gold sm" @click="importSubtitle">
                <Icon name="upload" :size="13" />
                {{ project.subtitlePath ? "换一份字幕" : "导入 SRT" }}
              </button>
              <button v-if="project.subtitlePath" class="btn ghost sm" @click="clearSubtitle">
                移除
              </button>
            </div>
          </div>

          <div class="asset-foot">
            <button class="btn ghost sm" @click="openFolder">
              <Icon name="folder-open" :size="13" />
              在资源管理器中打开
            </button>
          </div>
        </aside>

        <!-- Center -->
        <section class="studio-col center">
          <div class="stage-wrap">
            <div class="stage">
              <div class="stage-preview">
                <video
                  v-if="videoUrl && !videoFailed"
                  class="stage-video"
                  :src="videoUrl"
                  :poster="coverUrl || undefined"
                  controls
                  playsinline
                  preload="metadata"
                  @error="videoFailed = true"
                />
                <img
                  v-else-if="coverUrl && !coverFailed"
                  :src="coverUrl"
                  alt=""
                  @error="coverFailed = true"
                />
                <div v-else class="poster stage-poster" :style="posterStyle">
                  <div class="stage-title">
                    <span v-for="char in titleChars" :key="char">{{ char }}</span>
                  </div>
                  <div class="stage-sub">MATERIAL · {{ activeVideo ? formatDuration(duration) : "--:--" }}</div>
                </div>

                <div class="stage-badges">
                  <span>原片</span>
                  <span class="on">素材预览</span>
                </div>
                <div class="stage-resolution">
                  <span>{{ activeVideo?.info.width ?? "—" }}×{{ activeVideo?.info.height ?? "—" }}</span>
                  <span>{{ activeVideo ? Math.round(activeVideo.info.fps) : "—" }} fps</span>
                </div>

                <button v-if="!videoUrl || videoFailed" class="stage-play" disabled title="无法预览原片">
                  <Icon name="play" :size="20" />
                </button>

                <div v-if="!videoUrl || videoFailed" class="stage-caption">
                  {{ project.name }} · 封面取自原片第 {{ Math.max(1, Math.round(duration * 0.1)) }} 秒
                </div>
              </div>
            </div>

            <div class="meta-row">
              <span class="chip">{{ activeVideo?.info.codec ?? "—" }}</span>
              <span class="chip">{{ formatBytes(activeVideo?.info.size ?? 0) }}</span>
              <span class="chip">{{ activeVideo?.info.bitRate ? `${(activeVideo.info.bitRate / 1_000_000).toFixed(1)} Mbps` : "—" }}</span>
              <span class="chip">{{ project.videos.length }} 个视频</span>
              <div class="meta-grow" />
              <button class="btn ghost sm" @click="openFolder">
                <Icon name="folder-open" :size="13" />
                打开目录
              </button>
            </div>

            <div class="trim-card">
              <div class="trim-head">
                <div>
                  <b>截取范围</b>
                  <span class="muted">只处理选中区间，后面所有步骤都会沿用</span>
                </div>
                <div class="trim-values mono">
                  <span>{{ formatCueTime(trim[0]) }}</span>
                  <Icon name="arrow-r" :size="12" />
                  <span>{{ formatCueTime(trim[1]) }}</span>
                  <span class="trim-dur">{{ formatDuration(trim[1] - trim[0]) }}</span>
                </div>
              </div>
              <n-slider
                :value="trim"
                range
                :min="0"
                :max="Math.max(duration, 1)"
                :step="0.1"
                :tooltip="false"
                @update:value="onTrim"
              />
              <div class="trim-hint">
                <span>起点 {{ formatDuration(trim[0]) }}</span>
                <span>终点 {{ formatDuration(trim[1]) }}</span>
              </div>
            </div>
          </div>

          <div class="subtitle-panel">
            <div class="section-head compact">
              <div class="section-title">字幕预览</div>
              <div class="section-count">{{ project.subtitles.length }}</div>
              <div class="section-grow" />
              <span v-if="project.subtitles.length > visibleCues.length" class="mini">
                仅显示前 {{ visibleCues.length }} 条
              </span>
              <button class="btn ghost sm" @click="importSubtitle">
                <Icon name="upload" :size="13" />
                导入
              </button>
            </div>

            <div v-if="!project.subtitles.length" class="sub-empty-state">
              <Icon name="text" :size="20" />
              <b>还没有字幕</b>
              <p>导入 SRT 或与视频同名的 .srt 文件，P1 才能分析剧情并写文案。</p>
              <button class="btn soft-gold sm" @click="importSubtitle">导入 SRT</button>
            </div>

            <div v-else class="cue-list">
              <div v-for="cue in visibleCues" :key="cue.index" class="cue-row">
                <span class="cue-index">{{ String(cue.index).padStart(3, "0") }}</span>
                <span class="cue-time mono">{{ formatShortTime(cue.start) }}</span>
                <span class="cue-text">{{ cue.text }}</span>
              </div>
            </div>
          </div>
        </section>

        <!-- Right: next + info -->
        <aside class="studio-col right">
          <div class="col-head">
            <b>下一步</b>
            <div class="col-grow" />
            <span class="chip gold">P0</span>
          </div>

          <div class="inspector-scroll">
            <div class="check-card">
              <div class="check-item">
                <span class="check-dot done"><Icon name="check" :size="11" /></span>
                <div><b>素材已导入</b><span>{{ project.videos.length }} 个视频 · {{ formatDuration(duration) }}</span></div>
              </div>
              <div class="check-item">
                <span class="check-dot" :class="project.subtitles.length ? 'done' : 'todo'">
                  <Icon v-if="project.subtitles.length" name="check" :size="11" />
                  <template v-else>!</template>
                </span>
                <div>
                  <b>字幕{{ project.subtitles.length ? "已挂载" : "待导入" }}</b>
                  <span>{{ project.subtitles.length ? `${project.subtitles.length} 条 · 已解析时间轴` : "SRT 是剧情理解的前提" }}</span>
                </div>
              </div>
              <div class="check-item">
                <span class="check-dot todo">3</span>
                <div><b>剧情理解</b><span>P1 · 读取字幕，梳理人物与主线</span></div>
              </div>
            </div>

            <button
              class="btn primary lg full"
              :disabled="!project.subtitles.length"
              @click="router.push(`/studio/${project.id}/write`)"
            >
              <Icon name="spark" :size="16" />
              进入剧情理解
            </button>
            <div class="muted center-note">
              {{ project.subtitles.length ? "P1 · 模型或演示模式" : "先导入字幕，才能分析剧情" }}
            </div>

            <div class="hr" />

            <div class="field-block">
              <label>项目信息</label>
              <div class="info-row"><span>模式</span><b>{{ modeLabel(project.mode) }}</b></div>
              <div class="info-row"><span>创建</span><b>{{ new Date(project.createdAt).toLocaleString("zh-CN") }}</b></div>
              <div class="info-row"><span>更新</span><b>{{ new Date(project.updatedAt).toLocaleString("zh-CN") }}</b></div>
              <div class="info-row"><span>项目 ID</span><b class="mono small">{{ project.id.slice(0, 12) }}</b></div>
            </div>

            <div class="field-block">
              <label>视频信息</label>
              <div class="info-row"><span>编码</span><b>{{ activeVideo?.info.codec ?? "—" }}</b></div>
              <div class="info-row"><span>帧率</span><b>{{ activeVideo ? activeVideo.info.fps.toFixed(2) : "—" }} fps</b></div>
              <div class="info-row"><span>码率</span><b>{{ activeVideo?.info.bitRate ? `${(activeVideo.info.bitRate / 1_000_000).toFixed(1)} Mbps` : "—" }}</b></div>
              <div class="info-row"><span>大小</span><b>{{ formatBytes(activeVideo?.info.size ?? 0) }}</b></div>
            </div>

            <div class="tip small-tip">
              <div class="t-kicker">P0 NOTE</div>
              <b>这一步只做“素材就绪”</b>
              <p>视频元数据、封面帧、字幕解析与截取范围都会存进项目。关掉应用再打开，进度还在。</p>
            </div>
          </div>
        </aside>
      </div>
    </div>
  </div>
</template>
