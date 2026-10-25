<script setup lang="ts">
import { useDialog, useMessage } from "naive-ui";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";

import {
  assetUrl,
  checkEnvironment,
  createProject,
  deleteProject,
  isTauri,
  isVideoFile,
  listProjects,
  pickVideos,
  type EnvironmentStatus,
} from "../api";
import { formatDuration, greeting, todayLabel } from "../format";
import type { ProjectSummary } from "../types";
import AppRail from "../components/AppRail.vue";
import Icon from "../components/Icon.vue";
import ProjectCard from "../components/ProjectCard.vue";

const router = useRouter();
const route = useRoute();
const message = useMessage();
const dialog = useDialog();

const projects = ref<ProjectSummary[]>([]);
const loading = ref(true);
const importing = ref(false);
const query = ref("");
const env = ref<EnvironmentStatus>({ ffmpeg: false, ffprobe: false, ffmpegVersion: "检测中" });

const isHome = computed(() => route.name === "home");
const featured = computed(() => projects.value[0]);
const filtered = computed(() => {
  const keyword = query.value.trim().toLowerCase();
  if (!keyword) return projects.value;
  return projects.value.filter((p) => p.name.toLowerCase().includes(keyword));
});
const totalDuration = computed(() => projects.value.reduce((sum, p) => sum + p.duration, 0));
const totalCues = computed(() => projects.value.reduce((sum, p) => sum + p.subtitleCount, 0));
const weekCount = computed(
  () =>
    projects.value.filter(
      (p) => Date.now() - new Date(p.updatedAt).getTime() < 7 * 24 * 3600 * 1000,
    ).length,
);

const heroChars = computed(() => {
  const raw = featured.value?.name.replace(/[《》\s]/g, "") ?? "幕语";
  return Array.from(raw).slice(0, 2);
});

const heroCover = computed(() => assetUrl(featured.value?.coverPath));

const heroStyle = computed(() => {
  if (!featured.value) return {};
  const palettes = [
    ["#c08434", "#3d2f6b", "#0d2f3a"],
    ["#2f6f7a", "#193a57", "#0a1a24"],
    ["#7a5a2c", "#2f4a3d", "#101c18"],
  ];
  let hash = 0;
  for (const char of featured.value.id) hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
  const [p1, p2, p3] = palettes[hash % palettes.length];
  return { "--p1": p1, "--p2": p2, "--p3": p3 };
});

function errorText(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

async function refresh() {
  loading.value = true;
  try {
    projects.value = await listProjects();
  } catch (error) {
    message.error(`读取项目失败：${errorText(error)}`);
  } finally {
    loading.value = false;
  }
}

async function importVideos(paths?: string[]) {
  if (!isTauri()) {
    message.info("浏览器预览模式无法导入素材，请用 pnpm tauri dev 启动桌面端");
    return;
  }
  const files = paths && paths.length ? paths : await pickVideos();
  if (!files.length) return;

  importing.value = true;
  try {
    const project = await createProject(files);
    message.success(`已创建「${project.name}」`);
    await refresh();
    router.push(`/studio/${project.id}`);
  } catch (error) {
    message.error(`导入失败：${errorText(error)}`);
  } finally {
    importing.value = false;
  }
}

function confirmRemove(project: ProjectSummary) {
  if (!isTauri()) {
    message.info("预览模式下的示例项目不能删除");
    return;
  }
  dialog.warning({
    title: "删除项目",
    content: `确定删除「${project.name}」？项目记录和封面会被移除，源视频不会删除。`,
    positiveText: "删除",
    negativeText: "取消",
    onPositiveClick: async () => {
      try {
        await deleteProject(project.id);
        await refresh();
        message.success("已删除");
      } catch (error) {
        message.error(`删除失败：${errorText(error)}`);
      }
    },
  });
}

function onDropped(event: Event) {
  const paths = (event as CustomEvent<string[]>).detail ?? [];
  if (!paths.length) return;
  const videos = paths.filter(isVideoFile);
  if (videos.length) {
    void importVideos(videos);
  } else {
    message.info("请拖入视频文件（mp4 / mov / mkv / webm）");
  }
}

onMounted(async () => {
  window.addEventListener("muyu:files-dropped", onDropped);
  await refresh();
  try {
    env.value = await checkEnvironment();
  } catch {
    env.value = { ffmpeg: false, ffprobe: false, ffmpegVersion: "检测失败" };
  }
});

onUnmounted(() => window.removeEventListener("muyu:files-dropped", onDropped));
</script>

<template>
  <div class="app">
    <AppRail />

    <div class="main">
      <header class="topbar">
        <div class="crumb">
          <b>{{ isHome ? "首页" : "项目" }}</b>
          <span class="sep">/</span>
          <span>{{ isHome ? "工作台" : "全部项目" }}</span>
        </div>
        <div class="topbar-grow" />

        <n-input
          v-model:value="query"
          class="search-input"
          size="small"
          placeholder="搜索项目"
          clearable
        >
          <template #prefix>
            <Icon name="search" :size="15" />
          </template>
          <template #suffix>
            <kbd class="search-kbd">⌘K</kbd>
          </template>
        </n-input>

        <button class="icon-btn" title="通知">
          <Icon name="bell" :size="17" />
        </button>

        <n-tooltip trigger="hover">
          <template #trigger>
            <button class="btn ghost" disabled>
              <Icon name="link" :size="15" />
              导入链接
            </button>
          </template>
          链接解析将在 P1 开放
        </n-tooltip>

        <button class="btn primary" :disabled="importing" @click="importVideos()">
          <Icon name="plus" :size="16" />
          {{ importing ? "导入中…" : "新建解说" }}
        </button>

        <div class="avatar">林</div>
      </header>

      <main class="page">
        <div class="page-head">
          <div>
            <div class="page-title">
              {{ isHome ? `${greeting()}，林屿` : "全部项目" }}
            </div>
            <div class="page-sub">
              <template v-if="isHome">
                {{ projects.length ? `${projects.length} 个项目，${weekCount} 个本周有更新。` : "还没有项目，导入一个视频开始。" }}
              </template>
              <template v-else>
                共 {{ projects.length }} 个项目 · 合计 {{ formatDuration(totalDuration) }}
              </template>
            </div>
          </div>
          <div class="page-head-grow" />
          <div class="muted mono date-label">{{ todayLabel() }}</div>
        </div>

        <div class="home-grid">
          <div class="home-main">
            <section v-if="isHome" class="hero" :style="heroStyle">
              <img v-if="heroCover" class="hero-bg" :src="heroCover" alt="" />
              <div v-else class="poster" :style="heroStyle" />
              <div v-if="featured" class="hero-art">
                <div class="ha-title">
                  <span v-for="char in heroChars" :key="char">{{ char }}</span>
                </div>
                <div class="ha-num">01</div>
                <div class="ha-line" />
              </div>
              <div class="hero-shade" />

              <template v-if="featured">
                <div class="corner-badge">
                  <span class="dot green" />
                  自动保存 · {{ new Date(featured.updatedAt).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" }) }}
                </div>
                <div class="hero-content">
                  <div class="kicker"><span class="kicker-line" />CONTINUE · 继续创作</div>
                  <h2>{{ featured.name }}</h2>
                  <p>
                    {{ formatDuration(featured.duration) }} 素材 ·
                    {{ featured.subtitleCount }} 条字幕 ·
                    {{ featured.videoCount }} 个视频
                  </p>
                  <div class="hero-meta">
                    <span class="chip gold">短句解说</span>
                    <span class="chip">素材已就绪</span>
                    <span class="chip" :class="featured.subtitleCount ? 'green' : ''">
                      {{ featured.subtitleCount ? "字幕已挂载" : "待导入字幕" }}
                    </span>
                    <span class="chip">P0 · 素材阶段</span>
                  </div>
                  <div class="hero-actions">
                    <button class="btn primary lg" @click="router.push(`/studio/${featured.id}`)">
                      继续创作
                      <Icon name="arrow-r" :size="16" />
                    </button>
                    <button class="btn ghost lg" @click="router.push('/projects')">全部项目</button>
                    <div class="progress-mini">
                      <i class="on" /><i /><i /><i /><i />
                      <span>1 / 5 · 下一步：剧情理解</span>
                    </div>
                  </div>
                </div>
              </template>

              <template v-else>
                <div class="hero-content">
                  <div class="kicker"><span class="kicker-line" />START HERE</div>
                  <h2>把第一部长片交给幕语</h2>
                  <p>导入视频，自动识别同名字幕，抽帧、元数据与项目存档一次完成。</p>
                  <div class="hero-actions">
                    <button class="btn primary lg" :disabled="importing" @click="importVideos()">
                      <Icon name="upload" :size="16" />
                      导入视频
                    </button>
                    <span class="muted" style="font-size: 11px">支持 mp4 / mov / mkv / webm · 可多选</span>
                  </div>
                </div>
              </template>
            </section>

            <div class="stat-row">
              <div class="stat">
                <div class="ico"><Icon name="film" :size="16" /></div>
                <div>
                  <div class="num">{{ projects.length }}<small>个</small></div>
                  <div class="lbl">项目</div>
                </div>
              </div>
              <div class="stat">
                <div class="ico"><Icon name="clock" :size="16" /></div>
                <div>
                  <div class="num">{{ formatDuration(totalDuration) }}</div>
                  <div class="lbl">素材总时长</div>
                </div>
              </div>
              <div class="stat">
                <div class="ico"><Icon name="text" :size="16" /></div>
                <div>
                  <div class="num">{{ totalCues.toLocaleString() }}<small>条</small></div>
                  <div class="lbl">已挂载字幕</div>
                </div>
              </div>
              <div class="stat">
                <div class="ico"><Icon name="spark" :size="16" /></div>
                <div>
                  <div class="num">{{ weekCount }}<small>个</small></div>
                  <div class="lbl">本周更新</div>
                </div>
              </div>
            </div>

            <div class="section-head">
              <div class="section-title">{{ isHome ? "最近项目" : "项目" }}</div>
              <div class="section-count">{{ projects.length.toString().padStart(2, "0") }}</div>
              <div class="section-grow" />
              <span class="chip">全部</span>
              <span class="chip">进行中</span>
              <span class="chip">已完成</span>
            </div>

            <div v-if="loading" class="project-grid">
              <div v-for="n in 4" :key="n" class="project-skeleton" />
            </div>
            <div v-else class="project-grid">
              <ProjectCard
                v-for="project in filtered"
                :key="project.id"
                :project="project"
                @open="router.push(`/studio/${project.id}`)"
                @remove="confirmRemove(project)"
              />
              <button class="new-card" :disabled="importing" @click="importVideos()">
                <span class="plus"><Icon name="plus" :size="20" /></span>
                <b>{{ importing ? "正在导入…" : "新建解说" }}</b>
                <span>拖入视频，或点击选择</span>
              </button>
            </div>

            <template v-if="isHome">
              <div class="section-head">
                <div class="section-title">爆款结构</div>
                <div class="section-count">PRESETS</div>
                <div class="section-grow" />
                <span class="link">全部模板<Icon name="chev-r" :size="12" /></span>
              </div>
              <div class="template-row">
                <div class="template-card">
                  <div class="t-ico gold"><Icon name="spark" :size="16" /></div>
                  <div><b>黄金 3 秒钩子</b><span>困境 + 反常 + 悬念</span></div>
                  <span class="chip">P1</span>
                </div>
                <div class="template-card">
                  <div class="t-ico violet"><Icon name="refresh" :size="16" /></div>
                  <div><b>倒叙悬念结构</b><span>高潮前置 · 回到起点</span></div>
                  <span class="chip">P1</span>
                </div>
                <div class="template-card">
                  <div class="t-ico cyan"><Icon name="scissor" :size="16" /></div>
                  <div><b>短剧反转节奏</b><span>30 秒一钩 · 90 秒一翻</span></div>
                  <span class="chip">P1</span>
                </div>
              </div>
            </template>

            <footer class="page-foot">
              <Icon name="film" :size="12" />
              <span>项目保存在应用数据目录</span>
              <span class="dot-sep">·</span>
              <span>源视频不会被复制</span>
              <div class="foot-grow" />
              <span>幕语 0.9.7 · P4</span>
            </footer>
          </div>

          <aside class="side-col">
            <section class="panel">
              <div class="panel-head">
                <b>任务队列</b>
                <span class="mini">0 个进行中</span>
                <div class="panel-grow" />
                <Icon name="chev-r" :size="14" class="dim" />
              </div>
              <div class="task-empty">
                <div class="task-empty-ico"><Icon name="layers" :size="18" /></div>
                <b>暂无任务</b>
                <p>P1 接入转写与配音后，进度会出现在这里，可暂停、可重试。</p>
              </div>
            </section>

            <section class="panel">
              <div class="panel-head">
                <b>引擎状态</b>
                <div class="panel-grow" />
                <span class="mini">{{ env.ffmpeg ? "本机就绪" : "需检查" }}</span>
              </div>
              <div class="engine">
                <span class="status" :class="env.ffmpeg ? 'ok' : 'warn'"><span class="dot" /></span>
                <span class="name">FFmpeg</span>
                <span class="val">{{ env.ffmpeg ? "已检测到" : "未检测到" }}</span>
              </div>
              <div class="engine">
                <span class="status" :class="env.ffprobe ? 'ok' : 'warn'"><span class="dot" /></span>
                <span class="name">FFprobe</span>
                <span class="val">{{ env.ffprobe ? "已检测到" : "未检测到" }}</span>
              </div>
              <div class="engine">
                <span class="status"><span class="dot" /></span>
                <span class="name">文本模型</span>
                <span class="val">P1 接入</span>
              </div>
              <div class="engine">
                <span class="status"><span class="dot" /></span>
                <span class="name">语音合成</span>
                <span class="val">P1 接入</span>
              </div>
              <div class="hr" />
              <div class="engine">
                <span class="name">检测提示</span>
                <span class="val" style="max-width: 150px; text-align: right; line-height: 1.5">
                  找不到 FFmpeg 时，可在设置里指定路径
                </span>
              </div>
            </section>

            <section class="tip">
              <div class="t-kicker">SHORT SENTENCE</div>
              <b>短句解说的秘密</b>
              <p>一句一镜，8–24 字。观众的眼睛跟着画面走，耳朵才愿意跟你的声音走。</p>
              <span class="t-link">P1 开放写法模板<Icon name="chev-r" :size="12" /></span>
            </section>
          </aside>
        </div>
      </main>
    </div>
  </div>
</template>
