<script setup lang="ts">
import { useMessage } from "naive-ui";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useRouter } from "vue-router";

import {
  assetUrl,
  cancelRender,
  getProject,
  isTauri,
  listProjects,
  renderVideo,
} from "../api";
import { formatDuration, modeLabel } from "../format";
import type { Project, RenderOptions } from "../types";
import AppRail from "../components/AppRail.vue";
import Icon from "../components/Icon.vue";

interface BatchItem {
  project: Project;
  ready: boolean;
  targets: number;
  missing: number;
}

interface BatchResult {
  id: string;
  name: string;
  ok: boolean;
  path?: string;
  error?: string;
}

const router = useRouter();
const message = useMessage();

const loading = ref(true);
const items = ref<BatchItem[]>([]);
const selected = ref<string[]>([]);
const running = ref(false);
const cancelled = ref(false);
const currentId = ref("");
const progress = ref({ percent: 0, message: "准备中…" });
const results = ref<BatchResult[]>([]);
const options = ref({
  aspect: "original",
  resolution: "1080p",
  encoder: "auto",
  burnSubtitles: true,
});

let unlisten: (() => void) | undefined;

const readyItems = computed(() => items.value.filter((item) => item.ready));
const selectedItems = computed(() =>
  items.value.filter((item) => selected.value.includes(item.project.id) && item.ready),
);
const currentItem = computed(() =>
  items.value.find((item) => item.project.id === currentId.value),
);
const progressPercent = computed(() => Math.min(100, Math.max(0, progress.value.percent)));
const doneCount = computed(() => results.value.filter((item) => item.ok).length);
const failedCount = computed(() => results.value.filter((item) => !item.ok).length);

function errorText(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

function cover(item: BatchItem): string | null {
  return assetUrl(item.project.coverPath);
}

function toggle(item: BatchItem) {
  if (!item.ready || running.value) return;
  const index = selected.value.indexOf(item.project.id);
  if (index >= 0) selected.value.splice(index, 1);
  else selected.value.push(item.project.id);
}

function selectAll() {
  selected.value = readyItems.value.map((item) => item.project.id);
}

function clearAll() {
  selected.value = [];
}

async function load() {
  loading.value = true;
  try {
    const summaries = await listProjects();
    const loaded = await Promise.all(
      summaries.map(async (summary) => {
        try {
          return await getProject(summary.id);
        } catch {
          return null;
        }
      }),
    );
    items.value = loaded.filter((project): project is Project => !!project).map((project) => {
      const targets = project.items.filter(
        (item) => (item.OST === 0 || item.OST === 2) && item.narration.trim(),
      );
      const missing = targets.filter((item) => !item.audioDuration).length;
      return {
        project,
        targets: targets.length,
        missing,
        ready: targets.length > 0 && missing === 0,
      };
    });
    selected.value = items.value.filter((item) => item.ready).map((item) => item.project.id);
    if (isTauri()) {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<{ percent: number; message: string; done: boolean }>(
        "render://progress",
        (event) => {
          progress.value = { percent: event.payload.percent, message: event.payload.message };
        },
      );
    }
  } catch (err) {
    message.error(`读取项目失败：${errorText(err)}`);
  } finally {
    loading.value = false;
  }
}

async function start() {
  if (!selectedItems.value.length) {
    message.warning("先选择至少一个已配音项目");
    return;
  }
  running.value = true;
  cancelled.value = false;
  results.value = [];
  for (const item of selectedItems.value) {
    if (cancelled.value) break;
    const project = item.project;
    currentId.value = project.id;
    progress.value = { percent: 0, message: "准备中…" };
    try {
      const result = await renderVideo(project.id, {
        aspect: options.value.aspect,
        resolution: options.value.resolution,
        encoder: options.value.encoder,
        burnSubtitles: options.value.burnSubtitles,
        outputName: project.name,
        bgmPath: project.bgmPath ?? null,
        bgmVolume: project.bgmVolume ?? 0.18,
        bgmDucking: project.bgmDucking ?? true,
        bgmFade: project.bgmFade ?? true,
        duckingStrength: project.duckingStrength ?? "medium",
      });
      project.lastRender = result;
      results.value.push({
        id: project.id,
        name: project.name,
        ok: true,
        path: result.path,
      });
    } catch (err) {
      results.value.push({
        id: project.id,
        name: project.name,
        ok: false,
        error: errorText(err),
      });
    }
  }
  currentId.value = "";
  running.value = false;
  if (cancelled.value) message.info("批量渲染已停止");
  else message.success(`批量完成：成功 ${doneCount.value}，失败 ${failedCount.value}`);
}

async function stop() {
  cancelled.value = true;
  await cancelRender();
  message.info("已请求停止，当前镜头结束后终止");
}

onMounted(load);
onUnmounted(() => unlisten?.());
</script>

<template>
  <div class="app">
    <AppRail />

    <div class="main studio">
      <header class="studio-top">
        <button class="back" title="返回首页" @click="router.push('/')">
          <Icon name="arrow-l" :size="16" />
        </button>
        <div class="proj">
          <b>批量渲染</b>
          <span>{{ readyItems.length }} 个项目就绪 · 串行队列</span>
        </div>
        <div class="studio-grow" />
        <span class="chip violet">P4</span>
        <span class="chip">{{ running ? "队列运行中" : "本机编码" }}</span>
        <button class="btn ghost" @click="load" :disabled="running">
          <Icon name="refresh" :size="15" />
          刷新项目
        </button>
      </header>

      <div class="tasks-scroll">
        <div v-if="loading" class="studio-loading">
          <n-spin size="large" />
          <span>读取项目…</span>
        </div>

        <div v-else class="tasks-grid">
          <section class="tasks-main">
            <div class="tasks-head">
              <div>
                <b>待渲染项目</b>
                <span class="muted">已配音的项目才会进入队列</span>
              </div>
              <div class="tasks-head-actions">
                <button class="btn ghost sm" :disabled="running" @click="selectAll">全选就绪</button>
                <button class="btn ghost sm" :disabled="running" @click="clearAll">清空</button>
                <span class="chip">{{ selected.length }}/{{ items.length }}</span>
              </div>
            </div>

            <div v-if="!items.length" class="tasks-empty">
              <div class="tasks-empty-ico"><Icon name="film" :size="24" /></div>
              <b>还没有项目</b>
              <p>先导入素材、生成分镜并完成配音，再回来批量渲染。</p>
              <button class="btn primary" @click="router.push('/')">去首页</button>
            </div>

            <div v-else class="task-list">
              <article
                v-for="item in items"
                :key="item.project.id"
                class="task-card"
                :class="{ selected: selected.includes(item.project.id), disabled: !item.ready }"
                @click="toggle(item)"
              >
                <span class="task-check" :class="{ on: selected.includes(item.project.id) }">
                  <Icon v-if="selected.includes(item.project.id)" name="check" :size="12" />
                </span>
                <div class="task-cover">
                  <img v-if="cover(item)" :src="cover(item) ?? undefined" alt="" />
                  <div v-else class="poster" style="--p1: #c08434; --p2: #3d2f6b; --p3: #0d2f3a" />
                  <span class="task-cover-dur">{{ formatDuration(item.project.videos[0]?.info.duration ?? 0) }}</span>
                </div>
                <div class="task-body">
                  <div class="task-title">
                    <b>{{ item.project.name }}</b>
                    <span class="chip">{{ modeLabel(item.project.mode) }}</span>
                  </div>
                  <div class="task-meta">
                    <span>{{ item.project.items.length }} 镜</span>
                    <span>{{ item.targets }} 条配音</span>
                    <span v-if="item.project.lastRender">上次 {{ formatDuration(item.project.lastRender.duration) }}</span>
                  </div>
                  <div class="task-status">
                    <span class="state" :class="item.ready ? 'ok' : 'warn'">
                      {{ item.ready ? "就绪" : `还差 ${item.missing} 镜配音` }}
                    </span>
                    <span v-if="item.project.bgmPath" class="mini">BGM</span>
                    <span v-if="item.project.lastRender" class="mini">已渲染过</span>
                  </div>
                </div>
                <button
                  v-if="!item.ready"
                  class="btn ghost sm"
                  @click.stop="router.push(`/studio/${item.project.id}/voice`)"
                >
                  去配音
                </button>
              </article>
            </div>
          </section>

          <aside class="tasks-side">
            <section class="panel">
              <div class="panel-head"><b>队列设置</b><div class="panel-grow" /><span class="chip">共用</span></div>
              <div class="export-row">
                <label>画面比例</label>
                <div class="aspect-seg">
                  <button
                    v-for="aspect in [
                      { key: 'original', label: '原始' },
                      { key: '16:9', label: '16:9' },
                      { key: '9:16', label: '9:16' },
                      { key: '1:1', label: '1:1' },
                    ]"
                    :key="aspect.key"
                    :class="{ on: options.aspect === aspect.key }"
                    :disabled="running"
                    @click="options.aspect = aspect.key"
                  >
                    {{ aspect.label }}
                  </button>
                </div>
              </div>
              <div class="export-row">
                <label>分辨率</label>
                <div class="aspect-seg narrow">
                  <button :class="{ on: options.resolution === '720p' }" :disabled="running" @click="options.resolution = '720p'">720p</button>
                  <button :class="{ on: options.resolution === '1080p' }" :disabled="running" @click="options.resolution = '1080p'">1080p</button>
                </div>
              </div>
              <div class="export-row">
                <label>编码器</label>
                <select v-model="options.encoder" class="mini-select" :disabled="running" style="max-width: 100%">
                  <option value="auto">自动（优先硬件）</option>
                  <option value="h264_nvenc">NVIDIA NVENC</option>
                  <option value="h264_qsv">Intel QSV</option>
                  <option value="h264_amf">AMD AMF</option>
                  <option value="libx264">x264（兼容）</option>
                </select>
              </div>
              <div class="export-row">
                <label>字幕</label>
                <span
                  class="switch"
                  :class="{ on: options.burnSubtitles }"
                  @click="!running && (options.burnSubtitles = !options.burnSubtitles)"
                />
                <span class="muted" style="font-size: 9px">{{ options.burnSubtitles ? "烧录" : "只导出 SRT" }}</span>
              </div>
              <button class="btn primary lg full" style="margin-top: 12px" :disabled="running || !selectedItems.length" @click="start">
                <Icon name="film" :size="16" />
                {{ running ? "队列运行中…" : `开始批量渲染（${selectedItems.length}）` }}
              </button>
              <button v-if="running" class="btn ghost full stop-btn" style="margin-top: 8px" @click="stop">停止队列</button>
            </section>

            <section v-if="running || progress.percent > 0" class="panel">
              <div class="panel-head"><b>当前进度</b><div class="panel-grow" /><span class="mini mono">{{ Math.round(progressPercent) }}%</span></div>
              <div class="task-current">
                <b>{{ currentItem?.project.name ?? (running ? "准备中…" : "上一项") }}</b>
                <span>{{ progress.message }}</span>
              </div>
              <div class="vp-bar" style="margin-top: 8px"><i :style="{ width: `${progressPercent}%` }" /></div>
            </section>

            <section v-if="results.length" class="panel">
              <div class="panel-head">
                <b>队列结果</b>
                <div class="panel-grow" />
                <span class="chip green">{{ doneCount }} 成功</span>
                <span v-if="failedCount" class="chip gold">{{ failedCount }} 失败</span>
              </div>
              <div class="task-results">
                <div v-for="result in results" :key="result.id" class="task-result" :class="{ fail: !result.ok }">
                  <Icon :name="result.ok ? 'check' : 'alert'" :size="13" />
                  <div>
                    <b>{{ result.name }}</b>
                    <span v-if="result.ok">{{ result.path }}</span>
                    <span v-else>{{ result.error }}</span>
                  </div>
                </div>
              </div>
            </section>

            <section class="tip">
              <div class="t-kicker">BATCH</div>
              <b>串行，不抢显卡</b>
              <p>队列逐个项目渲染，每个项目使用自己的 BGM、ducking 与字幕设置；完成后可回项目里播放检查。</p>
            </section>
          </aside>
        </div>
      </div>
    </div>
  </div>
</template>
