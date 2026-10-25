<script setup lang="ts">
import { useMessage } from "naive-ui";
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";

import {
  analyzePlot,
  generateCopy,
  generateItems,
  getProject,
  getSettings,
  saveProject,
} from "../api";
import { formatDuration, formatShortTime, modeLabel } from "../format";
import type { LlmSettings, Project, ScriptItem, ScriptParams } from "../types";
import AppRail from "../components/AppRail.vue";
import Icon from "../components/Icon.vue";
import StepperBar from "../components/StepperBar.vue";

const route = useRoute();
const router = useRouter();
const message = useMessage();
const id = String(route.params.id);

const project = ref<Project | null>(null);
const settings = ref<LlmSettings | null>(null);
const loading = ref(true);
const error = ref("");
const stage = ref<2 | 3 | 4>(2);
const busy = ref("");
const plotDraft = ref("");
const copyDrafts = ref<string[]>([]);
const params = ref<ScriptParams>({
  genre: "悬疑 / 犯罪",
  language: "中文（简体）",
  targetWords: 600,
  density: "standard",
  originalRatio: 30,
});

const genres = ["悬疑 / 犯罪", "剧情 / 情感", "动作 / 冒险", "喜剧 / 轻松", "科幻 / 奇幻", "历史 / 战争", "恐怖 / 惊悚"];
const languages = ["中文（简体）", "中文（繁體）", "English", "日本語", "한국어"];
const densities = [
  { key: "dense", label: "更密" },
  { key: "standard", label: "标准" },
  { key: "loose", label: "更疏" },
] as const;

const demo = computed(() => !settings.value?.apiKey || settings.value?.demoMode);
const copyText = computed(() => copyDrafts.value.join(""));
const charCount = computed(() => copyText.value.replace(/\s/g, "").length);
const estimatedSeconds = computed(() => Math.round(charCount.value / 5));
const totalCues = computed(() => project.value?.subtitles.length ?? 0);
const items = computed(() => project.value?.items ?? []);
const ostCount = computed(() => items.value.filter((item) => item.OST === 1).length);
const ostRatio = computed(() =>
  items.value.length ? Math.round((ostCount.value / items.value.length) * 100) : 0,
);
const duration = computed(() => project.value?.videos[0]?.info.duration ?? 0);

function errorText(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

function splitSentences(text: string): string[] {
  return text
    .split(/(?<=[。！？!?；;])/)
    .map((sentence) => sentence.trim())
    .filter(Boolean);
}

function syncDrafts(copy: string) {
  copyDrafts.value = splitSentences(copy);
}

async function load() {
  loading.value = true;
  error.value = "";
  try {
    const loaded = await getProject(id);
    project.value = loaded;
    params.value = { ...loaded.params };
    plotDraft.value = loaded.plotAnalysis ?? "";
    syncDrafts(loaded.narrationCopy ?? "");
    if (loaded.items.length) stage.value = 4;
    else if (loaded.narrationCopy) stage.value = 3;
    else stage.value = 2;
    settings.value = await getSettings();
  } catch (err) {
    error.value = errorText(err);
  } finally {
    loading.value = false;
  }
}

function canGo(target: 2 | 3 | 4): boolean {
  if (target === 3) return Boolean(project.value?.plotAnalysis);
  if (target === 4) return Boolean(project.value?.narrationCopy);
  return true;
}

function goStage(target: 2 | 3 | 4) {
  if (!canGo(target)) {
    message.warning(target === 3 ? "先生成剧情理解" : "先生成逐句文案");
    return;
  }
  stage.value = target;
}

async function runPlot() {
  busy.value = "正在梳理剧情…";
  try {
    const updated = await analyzePlot(id);
    project.value = updated;
    plotDraft.value = updated.plotAnalysis ?? "";
    message.success("剧情理解已生成");
  } catch (err) {
    message.error(`剧情分析失败：${errorText(err)}`);
  } finally {
    busy.value = "";
  }
}

async function savePlot() {
  if (!project.value) return;
  project.value.plotAnalysis = plotDraft.value;
  project.value = await saveProject(project.value);
  message.success("剧情理解已保存");
}

async function runCopy() {
  busy.value = "正在写逐句文案…";
  try {
    const updated = await generateCopy(id, { ...params.value });
    project.value = updated;
    params.value = { ...updated.params };
    syncDrafts(updated.narrationCopy ?? "");
    message.success("逐句文案已生成");
  } catch (err) {
    message.error(`文案生成失败：${errorText(err)}`);
  } finally {
    busy.value = "";
  }
}

async function saveCopy(silent = false) {
  if (!project.value) return;
  project.value.narrationCopy = copyText.value;
  project.value.params = { ...params.value };
  project.value = await saveProject(project.value);
  if (!silent) message.success("文案已保存");
}

async function runItems() {
  await saveCopy(true);
  busy.value = "正在匹配画面与时间码…";
  try {
    const updated = await generateItems(id);
    project.value = updated;
    stage.value = 4;
    message.success(`已生成 ${updated.items.length} 个镜头`);
  } catch (err) {
    message.error(`分镜生成失败：${errorText(err)}`);
  } finally {
    busy.value = "";
  }
}

async function saveItems() {
  if (!project.value) return;
  project.value.items = items.value.map((item, index) => ({ ...item, _id: index + 1 }));
  project.value = await saveProject(project.value);
  message.success("脚本已保存");
}

async function copyJson() {
  try {
    await navigator.clipboard.writeText(JSON.stringify(items.value, null, 2));
    message.success("items JSON 已复制");
  } catch (err) {
    message.error(`复制失败：${errorText(err)}`);
  }
}

function removeItem(index: number) {
  project.value?.items.splice(index, 1);
}

function addSentence() {
  copyDrafts.value.push("补一句新的解说。");
}

function removeSentence(index: number) {
  copyDrafts.value.splice(index, 1);
}

function onSentenceInput(index: number, event: Event) {
  copyDrafts.value[index] = (event.target as HTMLTextAreaElement).value;
}

function shortTimestamp(timestamp: string): string {
  const [start, end] = timestamp.split("-");
  return `${(start ?? "").slice(0, 8)} → ${(end ?? "").slice(0, 8)}`;
}

onMounted(load);
</script>

<template>
  <div class="app">
    <AppRail />

    <div class="main studio">
      <header class="studio-top">
        <button class="back" title="返回素材" @click="router.push(`/studio/${id}`)">
          <Icon name="arrow-l" :size="16" />
        </button>
        <div class="proj">
          <b>{{ project?.name ?? "加载中…" }}</b>
          <span>{{ project ? `${modeLabel(project.mode)} · ${formatDuration(duration)} · ${totalCues} 条字幕` : "AI 工作台" }}</span>
        </div>
        <div class="studio-grow" />
        <span class="chip violet">P1</span>
        <span v-if="demo" class="chip gold">演示模式</span>
        <button class="btn ghost" :disabled="!project" @click="stage === 2 ? savePlot() : stage === 3 ? saveCopy() : saveItems()">
          <Icon name="save" :size="15" />
          保存
        </button>
        <button class="btn ghost" @click="router.push('/settings')">
          <Icon name="gear" :size="15" />
          模型设置
        </button>
      </header>

      <StepperBar :current="stage">
        <template #right>
          <span class="muted mono stepper-note">P1 · {{ demo ? "本地演示" : "模型生成" }}</span>
        </template>
      </StepperBar>

      <div v-if="loading" class="studio-loading">
        <n-spin size="large" />
        <span>读取项目中…</span>
      </div>

      <div v-else-if="error" class="studio-error">
        <Icon name="alert" :size="22" />
        <b>AI 工作台打不开</b>
        <p>{{ error }}</p>
        <button class="btn" @click="router.push(`/studio/${id}`)">返回素材</button>
      </div>

      <div v-else-if="project" class="studio-body write-body">
        <aside class="studio-col left">
          <div class="col-head">
            <b>原始字幕</b>
            <span class="mini">{{ totalCues }}</span>
            <div class="col-grow" />
            <span class="chip green">已解析</span>
          </div>

          <div class="write-asset">
            <div class="write-asset-title">{{ project.name }}</div>
            <div class="write-asset-meta">
              {{ formatDuration(duration) }} · {{ project.videos[0]?.info.width ?? "—" }}×{{ project.videos[0]?.info.height ?? "—" }} · {{ project.videos.length }} 集
            </div>
          </div>

          <div class="write-sub-list">
            <div v-for="cue in project.subtitles.slice(0, 500)" :key="cue.index" class="write-sub-row">
              <span class="ws-time mono">{{ formatShortTime(cue.start) }}</span>
              <span class="ws-text">{{ cue.text }}</span>
            </div>
          </div>
        </aside>

        <section class="studio-col center write-center">
          <div class="write-tabs">
            <button :class="{ on: stage === 2 }" @click="goStage(2)">
              <span>1</span> 剧情理解
            </button>
            <button :class="{ on: stage === 3 }" :disabled="!canGo(3)" @click="goStage(3)">
              <span>2</span> 逐句文案
            </button>
            <button :class="{ on: stage === 4 }" :disabled="!canGo(4)" @click="goStage(4)">
              <span>3</span> 分镜脚本
            </button>
            <div class="write-tabs-grow" />
            <span v-if="busy" class="busy-pill"><span class="spin" />{{ busy }}</span>
            <span
              v-else-if="project.pipelineNote && !project.pipelineNote.startsWith('演示模式')"
              class="chip gold"
            >{{ demo ? "本地规则" : "已降级" }}</span>
          </div>

          <div
            v-if="project.pipelineNote && !project.pipelineNote.startsWith('演示模式')"
            class="ai-note"
          >
            <Icon name="spark" :size="14" />
            <span>{{ project.pipelineNote }}</span>
          </div>

          <div v-if="demo" class="ai-banner">
            <Icon name="alert" :size="14" />
            <span>演示模式：不会调用模型，适合先跑通流程。配置 API Key 后可生成真实文案。</span>
            <button class="link" @click="router.push('/settings')">去设置</button>
          </div>

          <!-- Stage 2: plot -->
          <template v-if="stage === 2">
            <div class="write-head">
              <div>
                <h3>剧情理解</h3>
                <p>先把人物、动机、因果链梳理清楚，再写文案。结果可以手动修改。</p>
              </div>
              <div class="write-head-grow" />
              <span class="chip" :class="project.plotAnalysis ? 'green' : ''">{{ project.plotAnalysis ? "已生成" : "待生成" }}</span>
            </div>

            <div v-if="!project.plotAnalysis" class="stage-empty">
              <div class="stage-empty-ico"><Icon name="spark" :size="22" /></div>
              <b>让模型先读懂这部片</b>
              <p>读取 {{ totalCues }} 条字幕，输出一句话故事、人物关系、主线脉络与可用钩子。</p>
              <button class="btn primary" :disabled="!!busy" @click="runPlot">分析剧情</button>
            </div>

            <template v-else>
              <n-input
                v-model:value="plotDraft"
                type="textarea"
                :autosize="{ minRows: 13, maxRows: 26 }"
                placeholder="剧情理解结果"
              />
              <div class="write-foot">
                <button class="btn ghost sm" @click="savePlot">保存修改</button>
                <button class="btn ghost sm" :disabled="!!busy" @click="runPlot">重新分析</button>
                <div class="write-foot-grow" />
                <button class="btn primary sm" @click="goStage(3)">下一步：逐句文案</button>
              </div>
            </template>
          </template>

          <!-- Stage 3: copy -->
          <template v-else-if="stage === 3">
            <div class="write-head">
              <div>
                <h3>逐句文案</h3>
                <p>一句一镜。点任意一句就能改，改完再生成分镜。</p>
              </div>
              <div class="write-head-grow" />
              <span class="chip">{{ copyDrafts.length }} 句</span>
              <span class="chip" :class="charCount ? 'gold' : ''">{{ charCount }} 字</span>
            </div>

            <div v-if="!copyDrafts.length" class="stage-empty">
              <div class="stage-empty-ico"><Icon name="text" :size="22" /></div>
              <b>还没有文案</b>
              <p>按「{{ params.density === "dense" ? "更密" : params.density === "loose" ? "更疏" : "标准" }}」密度生成，目标 {{ params.targetWords }} 字。</p>
              <button class="btn primary" :disabled="!!busy" @click="runCopy">生成逐句文案</button>
            </div>

            <template v-else>
              <div class="sentence-list">
                <article v-for="(sentence, index) in copyDrafts" :key="index" class="sentence-card">
                  <div class="sentence-side">
                    <span class="sentence-idx mono">{{ String(index + 1).padStart(2, "0") }}</span>
                    <button class="sentence-del" title="删除此句" @click="removeSentence(index)">
                      <Icon name="trash" :size="12" />
                    </button>
                  </div>
                  <div class="sentence-body">
                    <textarea
                      class="sentence-input"
                      :value="sentence"
                      rows="1"
                      spellcheck="false"
                      @input="onSentenceInput(index, $event)"
                    />
                    <div class="sentence-meta">
                      <span>{{ sentence.replace(/\s/g, "").length }} 字</span>
                      <span>≈ {{ (sentence.replace(/\s/g, "").length / 5).toFixed(1) }}s</span>
                      <span v-if="sentence.replace(/\s/g, '').length > 32" class="warn">偏长 · 建议拆分</span>
                    </div>
                  </div>
                </article>
                <button class="sentence-add" @click="addSentence">
                  <Icon name="plus" :size="14" />
                  补一句
                </button>
              </div>

              <div class="write-foot">
                <button class="btn ghost sm" @click="saveCopy()">保存文案</button>
                <button class="btn ghost sm" :disabled="!!busy" @click="runCopy">重新生成</button>
                <div class="write-foot-grow" />
                <button class="btn primary sm" :disabled="!!busy" @click="runItems">生成分镜</button>
              </div>
            </template>
          </template>

          <!-- Stage 4: items -->
          <template v-else>
            <div class="write-head">
              <div>
                <h3>分镜脚本</h3>
                <p>一句文案对应一个镜头；OST 控制原声与解说。保存后进入 P2 配音。</p>
              </div>
              <div class="write-head-grow" />
              <span class="chip gold">{{ items.length }} 镜</span>
              <span class="chip cyan">原声 {{ ostRatio }}%</span>
            </div>

            <div v-if="!items.length" class="stage-empty">
              <div class="stage-empty-ico"><Icon name="film" :size="22" /></div>
              <b>还没有分镜</b>
              <p>把 {{ copyDrafts.length }} 句文案匹配到字幕时间码，生成可剪辑的 items。</p>
              <button class="btn primary" :disabled="!!busy" @click="runItems">生成分镜</button>
            </div>

            <template v-else>
              <div class="items-table">
                <div class="items-head">
                  <span>#</span><span>时间码</span><span>画面</span><span>解说</span><span>OST</span><span></span>
                </div>
                <div v-for="(item, index) in items" :key="item._id" class="items-row">
                  <span class="idx mono">{{ String(index + 1).padStart(2, "0") }}</span>
                  <span class="tc mono">{{ shortTimestamp(item.timestamp) }}</span>
                  <span class="pic">{{ item.picture }}</span>
                  <textarea v-model="item.narration" class="narr-input" rows="1" spellcheck="false" />
                  <select v-model.number="item.OST" class="ost-select" :class="{ orig: item.OST === 1, mix: item.OST === 2 }">
                    <option :value="0">解说</option>
                    <option :value="1">原声</option>
                    <option :value="2">混合</option>
                  </select>
                  <button class="row-del" title="删除镜头" @click="removeItem(index)">
                    <Icon name="trash" :size="12" />
                  </button>
                </div>
              </div>

              <div class="write-foot">
                <button class="btn ghost sm" @click="saveItems">保存脚本</button>
                <button class="btn ghost sm" @click="copyJson">复制 items JSON</button>
                <button class="btn ghost sm" :disabled="!!busy" @click="runItems">重新生成</button>
                <div class="write-foot-grow" />
                <button class="btn primary sm" @click="router.push(`/studio/${id}/voice`)">
                  配音与时间轴
                  <Icon name="arrow-r" :size="13" />
                </button>
              </div>
            </template>
          </template>
        </section>

        <aside class="studio-col right">
          <div class="col-head">
            <b>{{ stage === 2 ? "理解" : stage === 3 ? "文案参数" : "脚本检查" }}</b>
            <div class="col-grow" />
            <span class="chip gold">P1</span>
          </div>

          <div class="inspector-scroll">
            <template v-if="stage === 2">
              <div class="check-card">
                <div class="check-item">
                  <span class="check-dot done"><Icon name="check" :size="11" /></span>
                  <div><b>字幕已就绪</b><span>{{ totalCues }} 条 · 时间轴已解析</span></div>
                </div>
                <div class="check-item">
                  <span class="check-dot" :class="project.plotAnalysis ? 'done' : 'todo'">
                    <Icon v-if="project.plotAnalysis" name="check" :size="11" />
                    <template v-else>2</template>
                  </span>
                  <div><b>剧情理解</b><span>{{ project.plotAnalysis ? "可编辑、可重新生成" : "人物 / 动机 / 主线 / 钩子" }}</span></div>
                </div>
                <div class="check-item">
                  <span class="check-dot todo">3</span>
                  <div><b>逐句文案</b><span>下一步</span></div>
                </div>
              </div>

              <button v-if="!project.plotAnalysis" class="btn primary lg full" :disabled="!!busy" @click="runPlot">
                <Icon name="spark" :size="16" />
                {{ busy ? "分析中…" : "分析剧情" }}
              </button>
              <template v-else>
                <button class="btn primary lg full" @click="goStage(3)">
                  下一步：逐句文案
                  <Icon name="arrow-r" :size="15" />
                </button>
                <button class="btn ghost full" style="margin-top: 8px" :disabled="!!busy" @click="runPlot">
                  {{ busy ? "分析中…" : "重新分析剧情" }}
                </button>
              </template>
              <div class="muted center-note">{{ demo ? "演示模式：本地规则，不消耗 Token" : "将调用一次文本模型" }}</div>

              <div class="hr" />
              <div class="field-block">
                <label>输入摘要</label>
                <div class="info-row"><span>字幕条数</span><b>{{ totalCues }}</b></div>
                <div class="info-row"><span>素材时长</span><b>{{ formatDuration(duration) }}</b></div>
                <div class="info-row"><span>上下文</span><b>{{ totalCues > 500 ? "已截断至 500 条" : "全量" }}</b></div>
              </div>
            </template>

            <template v-else-if="stage === 3">
              <div class="param-field">
                <label>内容类型</label>
                <select v-model="params.genre" class="mini-select">
                  <option v-for="genre in genres" :key="genre" :value="genre">{{ genre }}</option>
                </select>
              </div>
              <div class="param-field">
                <label>解说语言</label>
                <select v-model="params.language" class="mini-select">
                  <option v-for="language in languages" :key="language" :value="language">{{ language }}</option>
                </select>
              </div>
              <div class="param-field">
                <label>目标字数</label>
                <input v-model.number="params.targetWords" class="mini-input mono" type="number" min="120" max="6000" step="50" />
              </div>
              <div class="param-field">
                <label>短句密度</label>
                <div class="density-seg">
                  <button
                    v-for="density in densities"
                    :key="density.key"
                    :class="{ on: params.density === density.key }"
                    @click="params.density = density.key"
                  >
                    {{ density.label }}
                  </button>
                </div>
              </div>
              <div class="param-field">
                <label>原片占比 <span class="mono muted">{{ params.originalRatio }}%</span></label>
                <input v-model.number="params.originalRatio" type="range" min="0" max="60" step="5" class="mini-range" />
              </div>

              <div class="stat-box">
                <div class="big"><b>{{ charCount }}</b><span>字</span><div style="flex: 1" /><span>≈ {{ Math.max(1, Math.round(estimatedSeconds / 60)) }} 分钟</span></div>
                <div class="rows">
                  <div class="row"><span>目标</span><div class="track"><i :style="{ width: Math.min(100, (charCount / params.targetWords) * 100) + '%', background: 'var(--gold)' }" /></div><span class="v">{{ params.targetWords }}</span></div>
                  <div class="row"><span>句数</span><div class="track"><i :style="{ width: Math.min(100, copyDrafts.length * 4) + '%', background: 'var(--cyan)' }" /></div><span class="v">{{ copyDrafts.length }}</span></div>
                </div>
              </div>

              <button
                v-if="!copyDrafts.length"
                class="btn primary lg full"
                style="margin-top: 12px"
                :disabled="!!busy"
                @click="runCopy"
              >
                <Icon name="text" :size="16" />
                {{ busy ? "生成中…" : "生成逐句文案" }}
              </button>
              <template v-else>
                <button class="btn primary lg full" style="margin-top: 12px" :disabled="!!busy" @click="runItems">
                  <Icon name="film" :size="16" />
                  生成分镜
                </button>
                <button class="btn ghost full" style="margin-top: 8px" :disabled="!!busy" @click="runCopy">
                  {{ busy ? "生成中…" : "重新生成文案" }}
                </button>
                <button class="btn ghost full" style="margin-top: 8px" @click="saveCopy()">保存文案</button>
              </template>
              <div class="muted center-note">{{ demo ? "演示模式：本地规则" : `目标 ${params.targetWords} 字，模型可能浮动 10%` }}</div>
            </template>

            <template v-else>
              <div class="check-card">
                <div class="check-item">
                  <span class="check-dot done"><Icon name="check" :size="11" /></span>
                  <div><b>分镜已生成</b><span>{{ items.length }} 个镜头 · 每句一镜</span></div>
                </div>
                <div class="check-item">
                  <span class="check-dot" :class="ostRatio <= params.originalRatio + 10 ? 'done' : 'todo'">
                    <Icon v-if="ostRatio <= params.originalRatio + 10" name="check" :size="11" />
                    <template v-else>!</template>
                  </span>
                  <div><b>原声占比 {{ ostRatio }}%</b><span>目标 {{ params.originalRatio }}% · 允许 ±10%</span></div>
                </div>
                <div class="check-item">
                  <span class="check-dot todo">4</span>
                  <div><b>配音与时间轴</b><span>P2 开放</span></div>
                </div>
              </div>

              <div class="stat-box">
                <div class="big"><b>{{ items.length }}</b><span>镜</span><div style="flex: 1" /><span>{{ ostCount }} 个原声</span></div>
                <div class="rows">
                  <div class="row"><span>解说镜</span><div class="track"><i :style="{ width: (100 - ostRatio) + '%', background: 'var(--gold)' }" /></div><span class="v">{{ items.length - ostCount }}</span></div>
                  <div class="row"><span>原声镜</span><div class="track"><i :style="{ width: ostRatio + '%', background: 'var(--cyan)' }" /></div><span class="v">{{ ostCount }}</span></div>
                </div>
              </div>

              <button class="btn primary lg full" style="margin-top: 12px" @click="router.push(`/studio/${id}/voice`)">
                <Icon name="wave" :size="16" />
                下一步：配音与时间轴
              </button>
              <button class="btn ghost full" style="margin-top: 8px" @click="saveItems">保存脚本</button>
              <button class="btn ghost full" style="margin-top: 8px" :disabled="!!busy" @click="runItems">重新生成分镜</button>
              <div class="muted center-note">P2 会把 items 接到 Edge TTS 与时间轴</div>
            </template>

            <div class="tip small-tip">
              <div class="t-kicker">PIPELINE</div>
              <b>三步，都是可逆的</b>
              <p>理解可以改，文案可以逐句改，分镜可以换 OST。任何一步重新生成都不会动素材。</p>
            </div>
          </div>
        </aside>
      </div>
    </div>
  </div>
</template>
