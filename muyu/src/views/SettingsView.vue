<script setup lang="ts">
import { useMessage } from "naive-ui";
import { onMounted, onUnmounted, reactive, ref } from "vue";

import {
  checkEnvironment,
  checkUpdate,
  getSettings,
  installUpdate,
  isTauri,
  openLogDir,
  saveSettings,
  testLlm,
  type EnvironmentStatus,
} from "../api";
import AppRail from "../components/AppRail.vue";
import Icon from "../components/Icon.vue";
import type { LlmSettings, UpdateInfo, UpdateProgress } from "../types";

const message = useMessage();
const desktop = isTauri();
const env = ref<EnvironmentStatus>({ ffmpeg: false, ffprobe: false, ffmpegVersion: "检测中" });
const autoStart = ref(false);
const autoStartBusy = ref(false);
const llm = reactive<LlmSettings>({
  baseUrl: "https://api.deepseek.com/v1",
  apiKey: "",
  model: "deepseek-chat",
  temperature: 0.8,
  demoMode: false,
  updateRepo: "MinimaxFlora/MuYu-AI",
  autoCheckUpdate: true,
});
const saving = ref(false);
const testing = ref(false);
const testResult = ref("");
const saved = ref(false);
const checkingUpdate = ref(false);
const updating = ref(false);
const updateInfo = ref<UpdateInfo | null>(null);
const updateProgress = ref<UpdateProgress | null>(null);
let unlistenUpdate: (() => void) | undefined;

onMounted(async () => {
  try {
    env.value = await checkEnvironment();
  } catch (error) {
    message.error(`环境检测失败：${error instanceof Error ? error.message : String(error)}`);
  }
  try {
    Object.assign(llm, await getSettings());
  } catch (error) {
    message.error(`读取模型设置失败：${error instanceof Error ? error.message : String(error)}`);
  }
  if (isTauri()) {
    try {
      const { isEnabled } = await import("@tauri-apps/plugin-autostart");
      autoStart.value = await isEnabled();
    } catch {
      autoStart.value = false;
    }
    const { listen } = await import("@tauri-apps/api/event");
    unlistenUpdate = await listen<UpdateProgress>("updater://progress", (event) => {
      updateProgress.value = event.payload;
    });
  }
});

onUnmounted(() => unlistenUpdate?.());

async function checkForUpdates() {
  if (!isTauri()) return;
  if (!llm.updateRepo.trim()) {
    message.warning("先填写 GitHub 仓库，例如 owner/MuYu");
    return;
  }
  checkingUpdate.value = true;
  updateInfo.value = null;
  updateProgress.value = null;
  try {
    Object.assign(llm, await saveSettings({ ...llm }));
    updateInfo.value = await checkUpdate();
    if (!updateInfo.value) message.success("当前已经是最新版本");
    else message.info(`发现新版本 v${updateInfo.value.version}`);
  } catch (error) {
    message.error(`检查更新失败：${error instanceof Error ? error.message : String(error)}`);
  } finally {
    checkingUpdate.value = false;
  }
}

async function startUpdate() {
  if (!updateInfo.value || updating.value) return;
  updating.value = true;
  updateProgress.value = null;
  try {
    await installUpdate();
    message.success("更新安装程序已启动");
  } catch (error) {
    message.error(`安装更新失败：${error instanceof Error ? error.message : String(error)}`);
  } finally {
    updating.value = false;
  }
}

async function toggleAutoUpdate() {
  llm.autoCheckUpdate = !llm.autoCheckUpdate;
  try {
    await saveSettings({ ...llm });
  } catch {
    // 下次保存时再落盘
  }
}

async function toggleAutostart() {
  if (!isTauri() || autoStartBusy.value) return;
  autoStartBusy.value = true;
  try {
    const { enable, disable, isEnabled } = await import("@tauri-apps/plugin-autostart");
    if (autoStart.value) await disable();
    else await enable();
    autoStart.value = await isEnabled();
    message.success(autoStart.value ? "已开启开机自启" : "已关闭开机自启");
  } catch (error) {
    message.error(`设置开机自启失败：${error instanceof Error ? error.message : String(error)}`);
  } finally {
    autoStartBusy.value = false;
  }
}

async function openLogs() {
  try {
    await openLogDir();
  } catch (error) {
    message.error(`打开日志目录失败：${error instanceof Error ? error.message : String(error)}`);
  }
}

async function persist() {
  saving.value = true;
  saved.value = false;
  try {
    Object.assign(llm, await saveSettings({ ...llm }));
    saved.value = true;
    message.success("模型设置已保存");
  } catch (error) {
    message.error(`保存失败：${error instanceof Error ? error.message : String(error)}`);
  } finally {
    saving.value = false;
  }
}

async function test() {
  testing.value = true;
  testResult.value = "";
  try {
    testResult.value = await testLlm({ ...llm });
  } catch (error) {
    testResult.value = `连接失败：${error instanceof Error ? error.message : String(error)}`;
  } finally {
    testing.value = false;
  }
}

function soon() {
  message.info("这一项将在 P2 接入");
}
</script>

<template>
  <div class="app">
    <AppRail />

    <div class="main">
      <header class="topbar">
        <div class="crumb">
          <b>设置</b>
          <span class="sep">/</span>
          <span>模型与引擎</span>
        </div>
        <div class="topbar-grow" />
        <span class="chip">P4</span>
        <div class="avatar">林</div>
      </header>

      <main class="page settings-page">
        <div class="page-head">
          <div>
            <div class="page-title">设置</div>
            <div class="page-sub">P4 已接入剪映草稿、BGM 混音/闪避、硬件编码探测与批量队列；安装包已构建（未签名）。</div>
          </div>
        </div>

        <div class="settings-grid">
          <section class="panel wide">
            <div class="panel-head">
              <b>文本模型</b>
              <span class="mini">OpenAI 兼容协议</span>
              <div class="panel-grow" />
              <span class="chip" :class="llm.demoMode || !llm.apiKey ? 'gold' : 'green'">
                {{ llm.demoMode || !llm.apiKey ? "演示模式" : "已配置" }}
              </span>
            </div>
            <div class="llm-form">
              <label class="form-field">
                <span>API Base URL</span>
                <n-input v-model:value="llm.baseUrl" placeholder="https://api.deepseek.com/v1" />
              </label>
              <label class="form-field">
                <span>模型名称</span>
                <n-input v-model:value="llm.model" placeholder="deepseek-chat" />
              </label>
              <label class="form-field full">
                <span>API Key</span>
                <n-input
                  v-model:value="llm.apiKey"
                  type="password"
                  show-password-on="click"
                  placeholder="sk-..."
                />
              </label>
            </div>

            <div class="llm-row">
              <span class="llm-label">温度</span>
              <n-slider v-model:value="llm.temperature" :min="0" :max="1.5" :step="0.1" style="flex: 1" />
              <span class="mono llm-value">{{ llm.temperature.toFixed(1) }}</span>
            </div>
            <div class="llm-row">
              <span class="llm-label">演示模式</span>
              <span
                class="switch"
                :class="{ on: llm.demoMode || !llm.apiKey }"
                @click="llm.demoMode = !llm.demoMode"
              />
              <span class="muted">不调用模型，用本地规则跑通全流程；未配置 Key 时自动启用</span>
            </div>

            <div class="llm-actions">
              <button class="btn ghost" :disabled="testing" @click="test">
                <Icon name="wave" :size="14" />
                {{ testing ? "测试中…" : "测试连接" }}
              </button>
              <button class="btn primary" :disabled="saving" @click="persist">
                <Icon name="check" :size="14" />
                {{ saving ? "保存中…" : "保存设置" }}
              </button>
              <span v-if="testResult" class="test-result" :class="{ bad: testResult.includes('失败') }">{{ testResult }}</span>
              <span v-else-if="saved" class="test-result">已保存</span>
            </div>
            <p class="settings-note">
              Key 保存在 Windows 凭据管理器（服务 <code>com.muyu.studio</code>），<code>settings.json</code> 不落明文；旧版本明文 Key 会在首次读取时自动迁移。
              默认走 OpenAI 兼容接口，DeepSeek / 硅基流动 / OpenAI / 本地 Ollama 等都可直接填 Base URL。
            </p>
          </section>

          <section class="panel">
            <div class="panel-head"><b>视频引擎</b><div class="panel-grow" /><span class="chip" :class="env.ffmpeg ? 'green' : 'red'">{{ env.ffmpeg ? "可用" : "缺失" }}</span></div>
            <div class="engine"><span class="status" :class="env.ffmpeg ? 'ok' : 'warn'"><span class="dot" /></span><span class="name">FFmpeg</span><span class="val">{{ env.ffmpeg ? "PATH 中已找到" : "未找到" }}</span></div>
            <div class="engine"><span class="status" :class="env.ffprobe ? 'ok' : 'warn'"><span class="dot" /></span><span class="name">FFprobe</span><span class="val">{{ env.ffprobe ? "PATH 中已找到" : "未找到" }}</span></div>
            <div class="env-version mono">{{ env.ffmpegVersion }}</div>
            <p class="settings-note">
              找不到时，把 ffmpeg 加入系统 PATH，或设置环境变量 <code>MUYU_FFMPEG</code>、<code>MUYU_FFPROBE</code>。
            </p>
            <button class="btn ghost sm" @click="soon">指定自定义路径（P2）</button>
          </section>

          <section class="panel">
            <div class="panel-head"><b>视觉与联网</b><div class="panel-grow" /><span class="chip">P4</span></div>
            <div class="engine"><span class="status"><span class="dot" /></span><span class="name">视觉模型</span><span class="val">未接入</span></div>
            <div class="engine"><span class="status"><span class="dot" /></span><span class="name">Tavily 搜索</span><span class="val">未接入</span></div>
            <div class="engine"><span class="status"><span class="dot" /></span><span class="name">Gemini 原生</span><span class="val">未接入</span></div>
            <p class="settings-note">P2 用于纪录片抽帧理解、短剧联网补剧情，以及原生 Gemini 渠道。</p>
          </section>

          <section class="panel">
            <div class="panel-head"><b>声音</b><div class="panel-grow" /><span class="chip">P4</span></div>
            <div class="engine"><span class="status ok"><span class="dot" /></span><span class="name">Edge TTS</span><span class="val">已接入 · 配音工作台</span></div>
            <div class="engine"><span class="status"><span class="dot" /></span><span class="name">IndexTTS</span><span class="val">待接入</span></div>
            <div class="engine"><span class="status"><span class="dot" /></span><span class="name">豆包 / 腾讯 / Azure</span><span class="val">待接入</span></div>
            <p class="settings-note">本地克隆引擎会以 HTTP Pack 的方式接入，不塞进主进程。</p>
            <button class="btn ghost sm" @click="soon">音色库（P2）</button>
          </section>

          <section class="panel">
            <div class="panel-head"><b>存储</b><div class="panel-grow" /></div>
            <div class="engine"><span class="name">项目数据</span><span class="val">应用数据目录</span></div>
            <div class="engine"><span class="name">源视频</span><span class="val">只引用，不复制</span></div>
            <div class="engine"><span class="name">封面帧</span><span class="val">项目目录 / cover.jpg</span></div>
            <p class="settings-note">项目 JSON 与封面存放在 <code>%APPDATA%\com.muyu.studio\projects</code>。</p>
            <button class="btn ghost sm" :disabled="!desktop" @click="soon">清理缓存（P2）</button>
          </section>

          <section class="panel">
            <div class="panel-head"><b>启动与日志</b><div class="panel-grow" /><span class="chip">P4</span></div>
            <div class="engine">
              <span class="status" :class="{ ok: autoStart }"><span class="dot" /></span>
              <span class="name">开机自启</span>
              <span class="val">{{ autoStart ? "已开启" : "已关闭" }}</span>
              <span class="switch" :class="{ on: autoStart }" style="margin-left: 8px" @click="toggleAutostart" />
            </div>
            <div class="engine">
              <span class="status ok"><span class="dot" /></span>
              <span class="name">系统托盘</span>
              <span class="val">关闭时隐藏，不退出</span>
            </div>
            <p class="settings-note">启动、崩溃与前端错误写入日志目录；托盘退出才会真正结束进程。</p>
            <button class="btn ghost sm" :disabled="!desktop" @click="openLogs">打开日志目录</button>
          </section>

          <section class="panel">
            <div class="panel-head"><b>更新</b><div class="panel-grow" /><span class="chip">GitHub</span></div>
            <div class="engine">
              <span class="name">更新仓库</span>
              <n-input
                v-model:value="llm.updateRepo"
                size="small"
                placeholder="owner/MuYu"
                style="max-width: 190px"
              />
            </div>
            <div class="engine">
              <span class="name">启动自动检查</span>
              <span class="switch" :class="{ on: llm.autoCheckUpdate }" @click="toggleAutoUpdate" />
              <span class="val">{{ llm.autoCheckUpdate ? "已开启" : "已关闭" }}</span>
            </div>
            <button class="btn ghost sm" :disabled="checkingUpdate || !desktop" @click="checkForUpdates">
              <Icon name="refresh" :size="13" />
              {{ checkingUpdate ? "检查中…" : "检查更新" }}
            </button>

            <div v-if="updateInfo" class="update-box">
              <div class="update-box-head">
                <b>v{{ updateInfo.version }}</b>
                <span>当前 v{{ updateInfo.currentVersion }}</span>
              </div>
              <p>{{ updateInfo.body || "暂无更新说明。" }}</p>
              <button class="btn primary sm" :disabled="updating" @click="startUpdate">
                {{ updating ? "安装中…" : "下载并安装" }}
              </button>
            </div>

            <div v-if="updateProgress" class="voice-progress">
              <div class="vp-top">
                <span>{{ updateProgress.message }}</span>
                <span class="mono">{{ Math.round(updateProgress.percent) }}%</span>
              </div>
              <div class="vp-bar"><i :style="{ width: `${Math.min(100, updateProgress.percent)}%` }" /></div>
            </div>

            <p class="settings-note">使用 GitHub Releases 的 latest.json + Tauri updater 签名校验；安装后自动重启。</p>
          </section>

          <section class="panel about">
            <div class="panel-head"><b>关于</b><div class="panel-grow" /></div>
            <div class="about-row"><span>版本</span><b>0.9.7 · P4</b></div>
            <div class="about-row"><span>技术</span><b>Tauri 2 + Rust + Vue 3 + Naive UI</b></div>
            <div class="about-row"><span>设计</span><b>幕语 · Cinema Noir</b></div>
            <p class="settings-note">P4 范围：剪映草稿、素材复制、BGM 混音、ducking、淡入淡出、硬件编码探测。</p>
          </section>
        </div>
      </main>
    </div>
  </div>
</template>
