# 幕语 · 架构与项目结构

版本：0.9.5 · P4（收尾）
日期：2026-10-25

---

## 1. 产品主线

```
项目库
  → 素材（视频 / ffprobe / 封面 / SRT / 截取范围）
  → 剧情理解（LLM 或演示规则）
  → 逐句文案（8–24 字句卡）
  → 分镜 items（时间码吸附 / OST 0/1/2）
  → 配音（Edge TTS / SAPI，时长回写）
  → 四轨时间轴（视频 / 原声 / 配音 / 字幕）
  → 成片导出（MP4 / SRT / items）
  → 剪映五轨草稿（视频 / 原声 / 配音 / 字幕 / BGM）
  → 批量队列
```

## 2. 技术栈

| 层 | 技术 |
|---|---|
| 桌面壳 | Tauri 2.12 / Rust 1.98 |
| 前端 | Vue 3.5 / TypeScript / Vite 7 / Naive UI 2.45 / Pinia 3 |
| 后端 | Rust：serde / reqwest / encoding_rs / msedge-tts / rustls |
| 媒体 | 系统 FFmpeg / FFprobe |
| 存储 | JSON 项目目录（原子写入） |

## 3. 目录结构

```
AI/
├── docs/                     产品、拆解、阶段报告、架构
├── design/
│   ├── mockups/              早期高保真稿
│   └── screens/              真实桌面端截图
├── _research/NarratoAI/      MIT 参考源码（只读）
└── muyu/
    ├── brand/                logo 源图 + 1024 图标
    ├── public/               logo / favicon
    ├── samples/              8 秒测试视频 + SRT
    ├── scripts/              图标、截图、CDP
    ├── src/
    │   ├── views/
    │   │   ├── HomeView.vue      首页 / 项目库
    │   │   ├── StudioView.vue    素材工作台
    │   │   ├── WriteView.vue     剧情 / 文案 / 分镜
    │   │   ├── VoiceView.vue     配音 + 四轨时间轴
    │   │   ├── ExportView.vue    成片 / BGM / 草稿
    │   │   ├── BatchView.vue     批量队列
    │   │   └── SettingsView.vue  模型 / 环境 / 声音
    │   ├── components/       AppRail / StepperBar / Icon / ProjectCard
    │   ├── api.ts            invoke 封装 + 浏览器演示数据
    │   ├── types.ts          前端类型
    │   ├── format.ts         时长 / 字节 / 模式
    │   ├── demo.ts           浏览器预览数据
    │   ├── router.ts         路由
    │   └── styles.css        Cinema Noir 设计系统
    ├── src-tauri/
    │   ├── src/
    │   │   ├── lib.rs        模块注册 + Tauri handler
    │   │   ├── commands.rs   所有命令 + 测试
    │   │   ├── model.rs      Project / ScriptItem / RenderResult / DraftResult
    │   │   ├── store.rs      项目目录 / 原子 save / list
    │   │   ├── media.rs      ffprobe / ffmpeg / 封面 / 环境
    │   │   ├── subtitle.rs   SRT 解码（UTF-8/16/GBK）与解析
    │   │   ├── settings.rs   LLM 设置 JSON
    │   │   ├── llm.rs        OpenAI 兼容客户端
    │   │   ├── pipeline.rs   剧情 → 文案 → items + 演示规则
    │   │   ├── tts.rs        Edge TTS / Windows SAPI
    │   │   ├── render.rs     裁剪 / 拼接 / BGM / 字幕 / 编码器
    │   │   └── jianying.rs   剪映明文草稿
    │   ├── icons/            全平台图标
    │   └── tauri.conf.json   窗口 / 资源协议 / bundle
    └── tools/nasm/           NASM 2.16.03（构建用）
```

## 4. 前端页面

| 路由 | 作用 |
|---|---|
| `/` | 首页、最近项目、任务卡、环境状态 |
| `/projects` | 全部项目 |
| `/studio/:id` | 素材、字幕、截取、元数据 |
| `/studio/:id/write` | 剧情 / 逐句文案 / items |
| `/studio/:id/voice` | Edge/SAPI 配音、进度、四轨时间轴 |
| `/studio/:id/export` | 成片设置、BGM、字幕、草稿、预览 |
| `/tasks` | 多项目串行批量渲染 |
| `/settings` | 模型、环境、声音、关于 |

设计系统：深色 Cinema Noir；金色 `#e9b949`、青色原声、紫色混合；OST 颜色贯穿时间轴与草稿。

## 5. 后端命令面

### 项目 / 素材
`create_project`、`list_projects`、`get_project`、`save_project`、`delete_project`、`attach_subtitle`、`remove_subtitle`、`probe_media`、`check_environment`、`reveal_project`、`open_path`。

### 设置 / AI
`get_settings`、`save_settings`、`test_llm`、`analyze_plot`、`generate_copy`、`generate_items`。

### 配音
`list_tts_voices`、`preview_voice`、`synthesize_items`、`cancel_tts`；进度事件 `tts://progress`。

### 渲染 / 导出
`render_video`、`cancel_render`、`export_srt`、`export_items`、`export_jianying`；进度事件 `render://progress`。

批量队列复用 `render_video`，由前端顺序调用，不新增后端命令。

## 6. 数据模型

`Project` 关键字段：

```
id / name / mode / status / createdAt / updatedAt
videos[] (path, name, info, trimStart, trimEnd)
subtitlePath / subtitles[]
coverPath
plotAnalysis / narrationCopy / items[]
params (genre, language, targetWords, density, originalRatio)
voice / ttsEngine / voiceRate / ttsNote
bgmPath / bgmVolume / bgmDucking / bgmFade / duckingStrength
lastRender / lastDraft
```

`ScriptItem`：

```
_id / videoId / videoName / timestamp / picture / narration / OST
audioPath / audioDuration
```

`OST`：`0` 解说（去原声）、`1` 原声、`2` 混合。

## 7. 媒体处理

### 7.1 渲染（render.rs）
1. 逐镜裁剪：
   - OST=0：按 `audioDuration` 裁画面，贴配音，去原声。
   - OST=1：按原时间码裁，保原声。
   - OST=2：按配音时长裁，原声 + 配音 `amix`。
2. 统一编码：H.264 / AAC / 48k / 25fps / yuv420p / 90k timescale。
3. concat demuxer `-c copy` 拼接。
4. BGM：循环、裁总长、`volume`、`afade`、`sidechaincompress` 闪避，再 `amix`。
5. 字幕：按新时间轴生成 SRT，`subtitles` 滤镜烧录；失败回退无字幕。
6. 编码器：NVENC → QSV → AMF → x264；64×64 空跑通过才用。

### 7.2 剪映草稿（jianying.rs）
- 明文结构：`draft_info.json`、`draft_meta_info.json`、`draft_settings`、`template*.tmp`、`root_meta_info.json`。
- 五轨：视频 / 原声 / 配音 / 字幕 / BGM。
- 素材复制到 `assets/video`、`assets/audio`。
- 原声用 ffmpeg 抽 wav；BGM 按实际时长循环切段。
- 未安装剪映时 UI 明确警告，不声称已验证。

## 8. 数据落盘

```
%APPDATA%/com.muyu.studio/
├── settings.json                 LLM 设置（明文，后续迁 keyring）
└── projects/<id>/
    ├── project.json
    ├── cover.jpg
    ├── audio/001.mp3 ...
    ├── render/<uuid>/           临时片段
    └── output/                  成片 MP4 / SRT / items
```

草稿写到用户选择的剪映 draft 根目录。

## 9. Tauri 规范与桌面集成

- **identifier**：`com.muyu.studio`（反向域名）。
- **productName**：`MuYu`（ASCII，安装包/可执行文件名）；窗口标题仍是「幕语 MuYu」。
- **版本**：semver `0.9.5`。
- **bundle**：publisher / copyright / category / shortDescription / longDescription 全部填写；Windows 目标为 NSIS。
- **NSIS**：`installMode=currentUser`，简体中文 + English 双语安装器，`startMenuFolder=MuYu`，安装器图标使用 `icons/icon.ico`。
- **托盘**：`TrayIconBuilder` 常驻；左键显示主窗口；菜单「显示主窗口 / 打开项目库 / 设置 / 退出 MuYu」。
- **关闭行为**：点窗口 X 隐藏到托盘，不退出进程；托盘「退出 MuYu」才真正退出。
- **单实例**：`tauri-plugin-single-instance`，重复启动只聚焦已有窗口。
- **CSP**：生产环境非 null；允许 `asset:` 图片/音频、`ipc:`、内联样式；开发环境单独 `devCsp` 允许 Vite HMR。
- **capabilities**：按 main 窗口最小授权，`core:default` + `dialog:default` + `log:default` + `autostart:default` + `updater:default`。
- **资源协议**：`assetProtocol` 开启，scope 限定 `$APPDATA/**`。
- **日志**：`tauri-plugin-log`，`%LOCALAPPDATA%\com.muyu.studio\logs\muyu.log`，Rust panic / 前端错误都会记录。
- **开机自启**：`tauri-plugin-autostart`，设置页开关。
- **自动更新**：`tauri-plugin-updater` + GitHub Release 的 `latest.json`，minisign 验签；自动检查 + 设置页手动检查/下载/安装。构建流程见 `.github/workflows/release.yml` 与 `docs/15-updater.md`。

实测：托盘创建成功；点 X 后进程仍在、窗口隐藏；再次启动只保留一个进程并重新显示窗口；生产 CSP 下首页/设置正常渲染；日志 `MuYu started` 已落盘。

> 安装包文件名不再使用中文：`MuYu_0.9.5_x64-setup.exe`。

## 10. 构建与发布

### 开发
```powershell
cd muyu
pnpm install
$env:PATH = "$PWD\tools\nasm\nasm-2.16.03;$env:PATH"
pnpm tauri dev
```

### 发布
```powershell
$env:PATH = "$PWD\tools\nasm\nasm-2.16.03;$env:PATH"
$env:CARGO_TARGET_DIR = "C:\muyu-target"   # NASM 不支持中文输出路径
pnpm tauri build --bundles nsis
```

原因：`aws-lc-sys` 的 NASM 汇编只能处理 ASCII 输出路径；项目路径含中文时必须把 target 放到 ASCII 盘。NASM 3.x 对 aws-lc 生成的 asm 不兼容，需用 2.16.03。

发布结果见文末。

## 11. 验证矩阵

| 阶段 | 验证 |
|---|---|
| P0 | 导入 / ffprobe / 封面 / SRT / 项目落盘 |
| P1 | 演示流水线 / 真实模型降级 / items 生成 |
| P2 | Edge TTS 合成 / 时长回写 / 进度 / 四轨 |
| P3 | 11.8s 成片 / 字幕烧录 / SRT / items |
| P4 草稿 | 5 轨 / 5 素材 / 原声 wav / 根索引 |
| P4 BGM | 混音 / 闪避 / 淡入淡出 / 220Hz 验证 |
| P4 编码 | NVENC/QSV/AMF 探测失败 → x264 回退 |
| P4 批量 | 2 项目 → 2 成功 / 0 失败 |
| 构建 | `cargo test` 2 passed / 4 ignored；`pnpm build` 通过 |

## 12. 已知边界

- 剪映草稿未在剪映内打开验证（本机未安装）。
- 硬件编码未在真实 GPU 验证。
- 队列不持久化、串行、共用设置。
- 没有链接下载 / ASR / 本地克隆 TTS。
- 没有签名证书；安装包未签名。
- FFmpeg 依赖系统 PATH。

## 13. 发布结果

- **GitHub Release**：`https://github.com/MinimaxFlora/MuYu-AI/releases/tag/v0.9.5`
- **NSIS 安装包**：`muyu/release/MuYu_0.9.5_x64-setup.exe`（GitHub Actions 构建）
  - 大小：4,699,108 B
  - SHA256：`576AD10AE30D15A18EE8EC4CDAC768DB284851751BCF4DE174F8BAFEBC69C464`
  - Authenticode 未签名（本机无代码签名证书）
- **updater 签名**：`MuYu_0.9.5_x64-setup.exe.sig`（minisign）
- **更新清单**：`latest.json` 1,383 B，包含 `windows-x86_64` 与 `windows-x86_64-nsis`
- **Portable exe**：`muyu/release/MuYu_0.9.5_portable.exe`
  - 大小：9,094,656 B
- 构建环境：NASM 2.16.03 + `CARGO_TARGET_DIR=C:\muyu-target`（规避 NASM 中文路径问题）
- 安装包内 exe 版本：0.9.5

### 运行前置

- Windows 10/11 + WebView2
- FFmpeg / FFprobe 在 PATH
- 安装包不携带 FFmpeg；未装 FFmpeg 时配音与渲染不可用。
