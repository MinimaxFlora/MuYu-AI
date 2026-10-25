<script setup lang="ts">
import { useMessage } from "naive-ui";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";

import {
  assetUrl,
  cancelTts,
  getProject,
  isTauri,
  listTtsVoices,
  previewVoice,
  synthesizeItems,
} from "../api";
import { formatDuration, modeLabel } from "../format";
import type { Project, ScriptItem, TtsProgress, TtsVoice } from "../types";
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
const engine = ref("edge");
const voice = ref("zh-CN-YunxiNeural");
const rate = ref(1);
const voices = ref<TtsVoice[]>([]);
const voicesLoading = ref(false);
const busy = ref(false);
const previewing = ref(false);
const progress = ref<TtsProgress | null>(null);
const selectedIndex = ref(0);
const playingId = ref<number | null>(null);

let audioEl: HTMLAudioElement | null = null;
let unlisten: (() => void) | undefined;

const items = computed(() => project.value?.items ?? []);
const selected = computed<ScriptItem | undefined>(() => items.value[selectedIndex.value]);
const targets = computed(() =>
  items.value.filter((item) => (item.OST === 0 || item.OST === 2) && item.narration.trim()),
);
const synthesized = computed(() => targets.value.filter((item) => item.audioDuration));
const allSynth = computed(() => targets.value.length > 0 && synthesized.value.length === targets.value.length);
const coverUrl = computed(() => assetUrl(project.value?.coverPath));
const voiceLabel = computed(
  () => voices.value.find((item) => item.id === voice.value)?.label ?? voice.value,
);
const posterStyle = computed(() => {
  const palettes = [
    ["#c08434", "#3d2f6b", "#0d2f3a"],
    ["#2f6f7a", "#193a57", "#0a1a24"],
    ["#7a5a2c", "#2f4a3d", "#101c18"],
  ];
  let hash = 0;
  for (const char of project.value?.id ?? "muyu") hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
  const [p1, p2, p3] = palettes[hash % palettes.length];
  return { "--p1": p1, "--p2": p2, "--p3": p3 };
});

function errorText(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

function slotDuration(item: ScriptItem): number {
  const [start, end] = item.timestamp.split("-");
  const parse = (raw: string) => {
    const parts = (raw ?? "").trim().replace(",", ".").split(":");
    if (parts.length !== 3) return 0;
    return Number(parts[0]) * 3600 + Number(parts[1]) * 60 + Number(parts[2]);
  };
  return Math.max(0, parse(end) - parse(start));
}

function effective(item: ScriptItem): number {
  return item.audioDuration && item.audioDuration > 0 ? item.audioDuration : slotDuration(item);
}

interface Block {
  index: number;
  item: ScriptItem;
  start: number;
  duration: number;
  left: number;
  width: number;
}

const blocks = computed<Block[]>(() => {
  const total = items.value.reduce((sum, item) => sum + effective(item), 0) || 1;
  let cursor = 0;
  return items.value.map((item, index) => {
    const duration = effective(item);
    const block = {
      index,
      item,
      start: cursor,
      duration,
      left: (cursor / total) * 100,
      width: Math.max(0.6, (duration / total) * 100),
    };
    cursor += duration;
    return block;
  });
});

const totalDuration = computed(() => items.value.reduce((sum, item) => sum + effective(item), 0));
const selectedBlock = computed(() => blocks.value[selectedIndex.value]);
const drift = computed(() => {
  const measured = items.value.filter((item) => item.audioDuration);
  if (!measured.length) return null;
  const sum = measured.reduce(
    (acc, item) => acc + Math.abs((item.audioDuration ?? 0) - slotDuration(item)),
    0,
  );
  return Math.round((sum / measured.length) * 1000);
});

const progressPercent = computed(() => {
  if (progress.value) return Math.min(100, Math.max(0, progress.value.percent));
  return 0;
});

function ostLabel(ost: number): string {
  return ost === 1 ? "原声" : ost === 2 ? "混合" : "解说";
}

function ostClass(ost: number): string {
  return ost === 1 ? "cyan" : ost === 2 ? "violet" : "gold";
}

function fmt(seconds: number): string {
  return formatDuration(seconds);
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    const loaded = await getProject(id);
    project.value = loaded;
    engine.value = loaded.ttsEngine || "edge";
    voice.value = loaded.voice || "zh-CN-YunxiNeural";
    rate.value = loaded.voiceRate || 1;
    selectedIndex.value = 0;
    loading.value = false;
    await loadVoices();
    if (isTauri()) {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<TtsProgress>("tts://progress", (event) => {
        progress.value = event.payload;
        if (event.payload.done) busy.value = false;
      });
    }
  } catch (err) {
    error.value = errorText(err);
    loading.value = false;
  }
}

async function loadVoices() {
  voicesLoading.value = true;
  try {
    const list = await listTtsVoices(engine.value);
    voices.value = list;
    const matched = list.find(
      (item) => item.id === voice.value || item.id.includes(voice.value),
    );
    voice.value = matched?.id ?? list[0]?.id ?? "";
  } catch (err) {
    if (engine.value === "edge") {
      message.warning(`Edge 音色获取失败，已切到 Windows SAPI：${errorText(err)}`);
      engine.value = "sapi";
      return loadVoices();
    }
    message.error(`音色获取失败：${errorText(err)}`);
  } finally {
    voicesLoading.value = false;
  }
}

watch(engine, () => {
  if (loading.value) return;
  voice.value = "";
  void loadVoices();
});

async function preview() {
  if (!voice.value) {
    message.warning("先选择一个音色");
    return;
  }
  previewing.value = true;
  try {
    const path = await previewVoice(engine.value, voice.value, rate.value);
    const url = assetUrl(path);
    if (!url) throw new Error("无法读取试听音频");
    audioEl?.pause();
    audioEl = new Audio(url);
    audioEl.onended = () => (previewing.value = false);
    audioEl.onerror = () => (previewing.value = false);
    await audioEl.play();
  } catch (err) {
    message.error(`试听失败：${errorText(err)}`);
    previewing.value = false;
  }
}

function playItem(item?: ScriptItem) {
  if (!item) return;
  if (!item.audioPath) {
    message.info("这一镜还没有配音");
    return;
  }
  const url = assetUrl(item.audioPath);
  if (!url) return;
  audioEl?.pause();
  audioEl = new Audio(url);
  playingId.value = item._id;
  audioEl.onended = () => {
    playingId.value = null;
  };
  audioEl.play().catch((err) => {
    playingId.value = null;
    message.error(`播放失败：${errorText(err)}`);
  });
}

async function synthesize() {
  if (!project.value || busy.value) return;
  busy.value = true;
  progress.value = {
    current: 0,
    total: targets.value.length,
    percent: 0,
    message: "准备中…",
    done: false,
  };
  try {
    const updated = await synthesizeItems(
      id,
      engine.value,
      voice.value,
      rate.value,
      allSynth.value,
    );
    project.value = updated;
    message.success(updated.ttsNote ?? "配音完成");
  } catch (err) {
    message.error(`配音失败：${errorText(err)}`);
  } finally {
    busy.value = false;
  }
}

async function stop() {
  await cancelTts();
  message.info("已请求停止");
}

function select(index: number) {
  selectedIndex.value = index;
}

onMounted(load);
onUnmounted(() => {
  audioEl?.pause();
  unlisten?.();
});
</script>

<template>
  <div class="app">
    <AppRail />

    <div class="main studio">
      <header class="studio-top">
        <button class="back" title="返回分镜" @click="router.push(`/studio/${id}/write`)">
          <Icon name="arrow-l" :size="16" />
        </button>
        <div class="proj">
          <b>{{ project?.name ?? "加载中…" }}</b>
          <span>
            {{ project ? `${modeLabel(project.mode)} · ${items.length} 镜 · ${synthesized.length} 已配音` : "配音与时间轴" }}
          </span>
        </div>
        <div class="studio-grow" />
        <span class="chip violet">P2</span>
        <span class="chip gold">{{ engine === "sapi" ? "Windows SAPI" : "Edge TTS" }}</span>
        <button class="btn ghost" @click="router.push(`/studio/${id}/write`)">
          <Icon name="arrow-l" :size="15" />
          返回分镜
        </button>
        <button class="btn primary" disabled>
          导出
          <Icon name="download" :size="15" />
        </button>
      </header>

      <StepperBar :current="4">
        <template #right>
          <span class="muted mono stepper-note">P2 · 配音与时间轴</span>
        </template>
      </StepperBar>

      <div v-if="loading" class="studio-loading">
        <n-spin size="large" />
        <span>读取项目中…</span>
      </div>

      <div v-else-if="error" class="studio-error">
        <Icon name="alert" :size="22" />
        <b>配音工作台打不开</b>
        <p>{{ error }}</p>
        <button class="btn" @click="router.push(`/studio/${id}/write`)">返回分镜</button>
      </div>

      <div v-else-if="project" class="studio-body voice-body">
        <!-- Left: shots -->
        <aside class="studio-col left">
          <div class="col-head">
            <b>分镜</b>
            <span class="mini">{{ items.length }}</span>
            <div class="col-grow" />
            <span class="chip" :class="allSynth ? 'green' : ''">{{ synthesized.length }}/{{ targets.length }}</span>
          </div>
          <div class="voice-items">
            <button
              v-for="(item, index) in items"
              :key="item._id"
              class="voice-item"
              :class="{ active: index === selectedIndex }"
              @click="select(index)"
            >
              <span class="vi-idx mono">{{ String(item._id).padStart(2, "0") }}</span>
              <span class="chip" :class="ostClass(item.OST)">{{ ostLabel(item.OST) }}</span>
              <span class="vi-body">
                <span class="vi-text">{{ item.narration }}</span>
                <span class="vi-meta">
                  {{ fmt(item.audioDuration || slotDuration(item)) }}
                  <template v-if="item.audioPath"> · 已配音</template>
                </span>
              </span>
              <span class="vi-dot" :class="{ on: item.audioPath }" />
            </button>
          </div>
        </aside>

        <!-- Center -->
        <section class="studio-col center voice-center">
          <div class="voice-stage">
            <img v-if="coverUrl" :src="coverUrl" alt="" />
            <div v-else class="poster" :style="posterStyle">
              <div class="stage-sub">VOICE · TIMELINE</div>
            </div>
            <div class="stage-badges"><span>原片</span><span class="on">配音预览</span></div>
            <div class="stage-resolution">
              <span>{{ selectedBlock ? fmt(selectedBlock.start) : "--:--" }}</span>
              <span>{{ selected ? fmt(effective(selected)) : "--:--" }}</span>
            </div>
            <div class="voice-caption">{{ selected?.narration || "选择左侧镜头查看解说" }}</div>
            <button class="voice-play" :disabled="!selected?.audioPath" @click="playItem(selected)">
              <Icon :name="playingId === selected?._id ? 'pause' : 'play'" :size="18" />
            </button>
          </div>

          <div class="voice-meta">
            <span class="chip" :class="selected ? ostClass(selected.OST) : ''">{{ selected ? ostLabel(selected.OST) : "—" }}</span>
            <span class="chip">{{ selected?.timestamp ?? "—" }}</span>
            <span class="chip" v-if="selected?.audioDuration">配音 {{ selected?.audioDuration?.toFixed(2) }}s</span>
            <span class="chip" v-if="selected?.audioDuration">原槽 {{ selected ? slotDuration(selected).toFixed(2) : "0.00" }}s</span>
            <div class="meta-grow" />
            <button class="btn ghost sm" :disabled="!selected?.audioPath" @click="playItem(selected)">
              <Icon name="play" :size="12" />
              播放本镜
            </button>
          </div>

          <div class="voice-timeline">
            <div class="vt-head">
              <b>时间轴</b>
              <span class="muted">{{ items.length }} 镜 · 成片 {{ fmt(totalDuration) }}</span>
              <div class="vt-grow" />
              <span class="vt-legend"><i class="gold" />解说</span>
              <span class="vt-legend"><i class="cyan" />原声</span>
              <span class="vt-legend"><i class="violet" />混合</span>
            </div>
            <div class="vt-ruler">
              <span v-for="tick in [0, 25, 50, 75, 100]" :key="tick" :style="{ left: `${tick}%` }">
                {{ fmt((totalDuration * tick) / 100) }}
              </span>
            </div>
            <div class="vt-row">
              <span class="vt-label">视频</span>
              <div class="vt-lane">
                <button
                  v-for="block in blocks"
                  :key="`v-${block.item._id}`"
                  class="vt-block video"
                  :class="`ost${block.item.OST}`"
                  :style="{ left: `${block.left}%`, width: `${block.width}%` }"
                  :title="block.item.narration"
                  @click="select(block.index)"
                >
                  {{ block.item._id }}
                </button>
              </div>
            </div>
            <div class="vt-row">
              <span class="vt-label">原声</span>
              <div class="vt-lane">
                <button
                  v-for="block in blocks.filter((b) => b.item.OST === 1 || b.item.OST === 2)"
                  :key="`o-${block.item._id}`"
                  class="vt-block orig"
                  :style="{ left: `${block.left}%`, width: `${block.width}%` }"
                  @click="select(block.index)"
                />
              </div>
            </div>
            <div class="vt-row">
              <span class="vt-label">配音</span>
              <div class="vt-lane">
                <button
                  v-for="block in blocks.filter((b) => b.item.OST !== 1 && b.item.audioDuration)"
                  :key="`a-${block.item._id}`"
                  class="vt-block audio"
                  :style="{ left: `${block.left}%`, width: `${block.width}%` }"
                  @click="select(block.index)"
                >
                  {{ block.item.audioDuration?.toFixed(1) }}s
                </button>
              </div>
            </div>
            <div class="vt-row">
              <span class="vt-label">字幕</span>
              <div class="vt-lane">
                <button
                  v-for="block in blocks"
                  :key="`s-${block.item._id}`"
                  class="vt-block sub"
                  :style="{ left: `${block.left}%`, width: `${block.width}%` }"
                  @click="select(block.index)"
                >
                  {{ block.item.narration.slice(0, 10) }}
                </button>
                <div
                  v-if="selectedBlock"
                  class="vt-playhead"
                  :style="{ left: `${selectedBlock.left}%` }"
                />
              </div>
            </div>
          </div>

          <div class="voice-foot">
            <span>配音完成后，画面会按配音时长重新对齐（P3 渲染）</span>
            <div class="meta-grow" />
            <span class="mono">{{ project.ttsEngine === "sapi" ? "Windows SAPI" : "Edge TTS" }}</span>
            <span class="mono">{{ voiceLabel }}</span>
          </div>
        </section>

        <!-- Right -->
        <aside class="studio-col right">
          <div class="col-head">
            <b>配音设置</b>
            <div class="col-grow" />
            <span class="chip" :class="allSynth ? 'green' : ''">{{ synthesized.length }}/{{ targets.length }}</span>
          </div>

          <div class="inspector-scroll">
            <div class="param-field">
              <label>引擎</label>
              <select v-model="engine" class="mini-select" :disabled="busy">
                <option value="edge">Edge TTS · 在线神经音色</option>
                <option value="sapi">Windows SAPI · 离线</option>
              </select>
            </div>

            <div class="param-field">
              <label>音色 <span v-if="voicesLoading" class="muted">加载中…</span></label>
              <select v-model="voice" class="mini-select" :disabled="busy || voicesLoading">
                <option v-for="item in voices" :key="item.id" :value="item.id">{{ item.label }}</option>
              </select>
            </div>

            <div class="param-field">
              <label>语速 <span class="mono muted">{{ rate.toFixed(2) }}x</span></label>
              <input v-model.number="rate" type="range" min="0.5" max="1.5" step="0.05" class="mini-range" />
            </div>

            <div class="voice-actions">
              <button class="btn ghost sm" :disabled="previewing || busy" @click="preview">
                <Icon name="volume" :size="13" />
                {{ previewing ? "播放中…" : "试听音色" }}
              </button>
              <button v-if="busy" class="btn ghost sm stop-btn" @click="stop">
                <Icon name="pause" :size="13" />
                停止
              </button>
            </div>

            <button class="btn primary lg full" style="margin-top: 12px" :disabled="busy || !targets.length" @click="synthesize">
              <Icon name="wave" :size="16" />
              {{ busy ? "配音中…" : allSynth ? "重新配音" : synthesized.length ? "继续配音（仅缺失）" : "开始配音" }}
            </button>

            <div v-if="busy || progress" class="voice-progress">
              <div class="vp-top">
                <span>{{ progress?.message ?? "准备中…" }}</span>
                <span class="mono">{{ Math.round(progressPercent) }}%</span>
              </div>
              <div class="vp-bar"><i :style="{ width: `${progressPercent}%` }" /></div>
              <div class="vp-sub">{{ progress?.current ?? 0 }} / {{ progress?.total ?? targets.length }}</div>
            </div>

            <div class="stat-box" style="margin-top: 12px">
              <div class="big"><b>{{ synthesized.length }}</b><span>/ {{ targets.length }} 镜</span><div style="flex: 1" /><span>成片 {{ fmt(totalDuration) }}</span></div>
              <div class="rows">
                <div class="row"><span>已配音</span><div class="track"><i :style="{ width: `${targets.length ? (synthesized.length / targets.length) * 100 : 0}%`, background: 'var(--gold)' }" /></div><span class="v">{{ synthesized.length }}</span></div>
                <div class="row"><span>对齐误差</span><div class="track"><i :style="{ width: drift === null ? '0%' : `${Math.min(100, drift / 5)}%`, background: 'var(--cyan)' }" /></div><span class="v" :class="{ warn: drift !== null && drift > 500 }">{{ drift === null ? "—" : `${drift}ms` }}</span></div>
              </div>
            </div>

            <div v-if="project.ttsNote" class="ai-note" style="margin-top: 12px">
              <Icon name="check" :size="13" />
              <span>{{ project.ttsNote }}</span>
            </div>

            <button class="btn ghost lg full" style="margin-top: 12px" @click="router.push(`/studio/${id}/export`)">
              <Icon name="download" :size="15" />
              下一步：成片导出
            </button>

            <div class="tip small-tip">
              <div class="t-kicker">EDGE TTS</div>
              <b>免费、有神经音色</b>
              <p>Edge 走在线服务，需要网络；失败会自动提示切到 Windows SAPI。SAPI 完全离线，音色更少。</p>
            </div>
          </div>
        </aside>
      </div>
    </div>
  </div>
</template>
