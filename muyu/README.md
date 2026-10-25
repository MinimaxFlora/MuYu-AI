# 幕语 Muyu Studio

AI 影视解说工作台 · Tauri 2 + Rust + Vue 3 + Naive UI

当前进度：**P4 完成（Authenticode 未签名）**（素材 → 剧情 → 文案 → 分镜 → 配音 → 成片 MP4 → 剪映草稿 + BGM）。
剪映草稿已生成五轨（视频/原声/配音/字幕/BGM）并复制素材；已在剪映 11.6 识别并由官方引擎导出验证，GUI 编辑器显示仍需人工确认。BGM 支持闪避与淡入淡出，编码器支持硬件探测/回退，批量队列、系统托盘/单实例、日志、开机自启、GitHub updater 与 NSIS 安装包（Authenticode 未签名）已完成。

---

## 已实现

### P0 · 素材与项目库
- 项目库：`projects/<id>/project.json` + `cover.jpg`，原子写入。
- 视频导入：多选 / 拖拽；ffprobe 读时长、分辨率、帧率、编码、码率、大小。
- 封面帧：ffmpeg 在 10% 处抽帧，960px JPEG。
- 字幕：同名 `.srt` 自动挂载、手动导入/移除、UTF-8 / UTF-16 / GBK。
- 截取范围、环境检测（FFmpeg / FFprobe）。

### P1 · AI 流水线
- OpenAI 兼容模型接入、测试连接、JSON 模式降级。
- 剧情理解、逐句文案、分镜 `items`（时间码吸附、OST 0/1/2）。
- 未配 Key 自动演示模式；模型失败回退本地规则并写 `pipelineNote`。

### P2 · 配音与时间轴
- Edge TTS（31 个中英音色）+ Windows SAPI 离线回退。
- 逐句配音、ffprobe 时长回写、`tts://progress`、继续补配、软取消。
- 视频 / 原声 / 配音 / 字幕四轨时间轴，对齐误差统计。

### P3 · 成片导出
- 按配音重剪：OST=0 去原声、OST=1 保原声、OST=2 混音。
- 统一 H.264 / AAC / 25fps，逐镜裁剪后 concat。
- 字幕按新时间轴烧录（Microsoft YaHei），失败自动回退。
- 原始 / 16:9 / 9:16 / 1:1，720p / 1080p。
- 成片 MP4、SRT、items JSON，应用内播放。

### P4 · 剪映草稿 + BGM
- 明文草稿包：`draft_info.json`、`draft_meta_info.json`、`draft_settings`、`template*.tmp`、`root_meta_info.json`。
- 五轨：视频轨、原声轨、配音轨、字幕轨、BGM 轨。
- 素材复制到 `assets/video`、`assets/audio`，草稿不依赖原项目路径。
- 草稿根索引更新，重名自动加后缀。
- 草稿结果持久化到项目，重开仍可看到上次路径。
- **原声独立轨**：OST=1/2 的原声抽成 wav，视频轨静音，草稿单独一轨。
- **BGM 混音**：MP4 在拼接后、字幕烧录前混入 BGM，自动循环并裁到总长；草稿生成 BGM 轨，按实际时长循环切段。
- 导出页可选 BGM、调音量（默认 18%）、移除；设置写回项目。
- **配音闪避**：`sidechaincompress` 以正片音轨为侧链，解说进入时 BGM 自动压低。
- **淡入淡出**：BGM 默认 1.2s 淡入/淡出，循环接缝与片尾不再硬切。
- **硬件编码探测**：自动检测 NVENC/QSV/AMF，64×64 空跑通过才用；失败回退 x264，结果回传实际编码器。
- **批量队列**：`/tasks` 逐项目串行渲染，按项目读取 BGM/ducking，单个失败不阻断后续。
- **系统托盘**：左键显示主窗口；菜单含项目库、设置、退出；关闭窗口隐藏到托盘。
- **单实例**：重复启动只聚焦已有窗口。
- 未安装剪映时 UI 明确警告「结构已生成，未做打开验证」。

## 运行

前置：Node 20+、pnpm、Rust stable、Windows 10/11 + WebView2、`ffmpeg`/`ffprobe` 在 PATH。

```powershell
cd muyu
pnpm install
$env:PATH = "$PWD\tools\nasm\nasm-2.16.03;$env:PATH"
$env:CARGO_TARGET_DIR = "C:\muyu-target"   # 路径含中文时必须用 ASCII target
pnpm tauri dev
```

浏览器预览（示例数据，不能写盘/合成/渲染/草稿）：

```powershell
pnpm dev
# http://127.0.0.1:1420
```

构建：

```powershell
pnpm build
pnpm tauri build
```

> 构建 `aws-lc` 需要 NASM 2.16。项目内已带 `tools/nasm/nasm-2.16.03`，构建前把它加入 PATH。项目路径含中文时，NASM 无法写中文输出路径，需要把 `CARGO_TARGET_DIR` 指到 ASCII 目录。

## 测试

```powershell
cd src-tauri
cargo test
```

- `imports_sample_project_end_to_end`：ffprobe / ffmpeg / SRT / 落库。
- `demo_pipeline_generates_items`：演示三段流水线。
- `edge_tts_smoke`（ignored）：真实 Edge TTS 合成。
- `render_sample_end_to_end`（ignored）：配音 + 渲染 + 字幕烧录。
- `jianying_draft_end_to_end`（ignored）：配音 + 草稿包 + 根索引 + 结构自检。
- `seed_sample_project_for_dev`（ignored）：写入完整示例项目，供 UI 联调。

## 目录

```
muyu/
├── src/
│   ├── views/            Home / Studio / Write / Voice / Export / Batch / Settings
│   ├── components/       Rail、ProjectCard、Stepper、Icon
│   ├── api.ts            Tauri invoke 封装 + 浏览器演示数据
│   └── styles.css        设计系统
├── src-tauri/src/
│   ├── commands.rs       项目 / 设置 / AI / 配音 / 渲染 / 草稿命令
│   ├── pipeline.rs       剧情 → 文案 → items
│   ├── tts.rs            Edge TTS / Windows SAPI
│   ├── render.rs         裁剪 / 拼接 / 字幕 / 成片
│   ├── jianying.rs       剪映明文草稿
│   ├── llm.rs            OpenAI 兼容客户端
│   ├── settings.rs       模型设置
│   ├── media.rs          ffprobe / ffmpeg
│   ├── subtitle.rs       SRT 解码与解析
│   └── store.rs          项目目录、原子写入
├── samples/              8 秒测试视频 + SRT
├── release/              NSIS 安装包 + portable exe
├── brand/                logo 源图 + 1024 应用图标
├── public/               logo.png / apple-touch-icon
├── scripts/              图标、窗口截图、CDP 调试
└── design/               设计稿（上级目录）
```

## 发布

- `release/MuYu_0.9.10_x64-setup.exe`：NSIS 安装包（Authenticode 未签名）
- `release/MuYu_0.9.10_x64-setup.exe.sig`：updater minisign 签名
- `release/MuYu_0.9.10_portable.exe`：免安装 exe

安装包不携带 FFmpeg；目标机需要 FFmpeg/FFprobe 在 PATH。

## 当前边界

- **剪映草稿已在 11.6 识别并导出**：官方引擎无头导出通过；GUI 编辑器显示仍需人工确认。schema 按 169.0.0 明文结构生成，由剪映迁移到 189.0.0。
- **草稿五轨已含原声与 BGM**：仍没有转场/特效/关键帧。
- **BGM 处理是固定参数**：闪避阈值/比率、淡入淡出时长暂不可调；没有响度归一化。
- **草稿 BGM 不做闪避**：剪映里仍是固定音量，需要手动关键帧。
- 硬件编码已真实验证 AMD AMF（`h264_amf`）；NVENC/QSV 在本机不可用，未验证。
- 批量队列是串行、纯前端、不持久化；临时片段目录不自动清理。
- 字幕依赖系统 Microsoft YaHei。
- FFmpeg 使用系统 PATH，不随包分发。
- 真实模型请求需要自备 Key；本机没有 Key，只验证了降级与演示链路。
- API Key 保存在 Windows 凭据管理器（`com.muyu.studio` / `llm-api-key`），`settings.json` 不落明文。

## 与 NarratoAI 的关系

流程与字段语义、剪映明文草稿 schema 参考 NarratoAI（MIT）的公开实现；界面、Rust 后端与流水线代码为独立实现，未复制其源码与素材。详见上级 `docs/`。
