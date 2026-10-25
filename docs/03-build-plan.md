# 实施计划 v0.1

> 状态：**P0、P1、P2、P3 已落地，P4 进行中**（2026-10-25），验证记录见 `04-p0-report.md` ~ `12-p4-batch-report.md`。P4 已完成五轨剪映草稿、BGM 混音、ducking/淡入淡出、硬件编码探测与批量队列；安装包已构建（未签名），见 `13-architecture.md`。

> 目标：用 Tauri 2 + Rust + Vue 3 + Naive UI 复刻 NarratoAI 的完整能力，并把交互升级为可视化创作台。
> 原则：**功能对齐、数据协议对齐、代码自研**（NarratoAI 为 MIT，仅参考流程与字段语义，不复制其 UI/素材/源码；如复用片段将保留版权声明）。

---

## 1. 工程结构

```
muyu/
├── apps/desktop/                 # Vue 3 + TS + Naive UI + Pinia
│   ├── src/app/                  # 启动、路由、主题、i18n
│   ├── src/features/             # home / studio / projects / assets / tasks / settings
│   ├── src/components/           # 设计系统组件（Card, Stepper, OstChip, Track…）
│   └── src/stores/               # project / job / settings / player
├── src-tauri/
│   ├── src/                      # commands、state、events、window
│   └── crates/
│       ├── core/                 # 配置、SQLite、路径、日志、keyring
│       ├── media/                # ffprobe/ffmpeg、抽帧、裁剪、拼接、混音、字幕、硬件加速
│       ├── llm/                  # OpenAI 兼容、Gemini、TwelveLabs、Tavily、重试/降级
│       ├── asr/                  # SRT、Fun-ASR/FireRed/百炼、翻译、校准
│       ├── tts/                  # Edge/Azure/腾讯/豆包/Qwen3/IndexTTS/OmniVoice/VoxCPM
│       ├── pipeline/             # 5 种模式状态机、步骤产物、断点、队列
│       └── export/               # 剪映草稿、SRT、文案、成片
├── sidecars/                     # ffmpeg.exe / ffprobe.exe / yt-dlp.exe
├── design/                       # 本设计稿
└── docs/
```

运行时目录：`%APPDATA%/Muyu/`
```
config.json          # 非敏感配置
secrets.bin          # API Key（Windows Credential Manager / 加密）
projects/<id>/       # project.json + steps/ + media/ + output/
cache/               # 抽帧、裁剪片段、TTS 缓存（可清理）
logs/
```

---

## 2. 分阶段计划

### P0 骨架与素材（约 1–2 周）
- Tauri 2 工程、单实例、窗口状态、深色主题、Naive UI 覆盖。
- 左栏/顶栏/工作台壳、步骤条、项目库（SQLite）、最近项目。
- 导入视频（拖拽/选择/多选）、ffprobe 元数据、封面帧、截取范围。
- SRT 解析、字幕列表、同名自动挂载、项目自动保存。
- 设计系统组件：Button/Card/Chip/Stepper/Field/Track/OstChip。

**验收**：拖入 3 集视频 → 项目落库 → 重启后恢复 → 字幕可预览。

### P1 脚本流水线（约 2–3 周）
- LLM Provider 抽象（OpenAI 兼容 + Gemini 原生），Key 安全存储、测试连接。
- 5 种模式：影视解说 / 短剧解说 / 短句解说 / 短剧混剪 / 我有脚本。
- 剧情理解、解说文案、片段规划、脚本生成、格式校验、定向修复。
- 文案工作台：逐句卡片、编辑、拆分、AI 改写、字数/时长预估、覆盖统计。
- `items` JSON 落盘、版本历史、步骤重跑。

**验收**：同一部片生成结构合法的 `items`；`OST` 分布符合规则；文案可逐句改。

### P2 配音与分镜（约 2 周）
- Edge TTS 首发；Azure / 腾讯 / 豆包 / Qwen3 云端接入。
- 逐条合成、并发限流、重试、时长回写、试听缓存。
- 分镜表、五轨时间轴、OST 切换、单镜重配、音画对齐检查。
- 任务队列：进度事件、暂停/取消、失败重试、断点续跑。

**验收**：38 镜全部配音；时长误差可见；时间轴可拖拽；中断后可续。

### P3 渲染与导出（约 2 周）
- OST 0/1/2 裁剪（TTS 时长驱动）、拼接、配音/原声/BGM 混音、智能配平。
- 字幕烧录（ASS + drawtext 回退）、字体/颜色/描边/位置、原字幕遮罩。
- 硬件加速探测（NVENC/QSV/AMF）与多级回退；代理预览。
- 成片 MP4、剪映草稿（四轨 + 素材复制）、SRT/文案导出。
- 导出中心、队列、完成通知、打开目录。

**验收**：30 秒样片端到端 < 2 分钟；成片音画误差 < 100ms；剪映草稿可打开且素材不丢。

### P4 增强（持续）
- 多集批量、跨集桥接、每集独立参数。
- Fun-ASR 本地/在线转录、字幕翻译/校准。
- 链接下载（yt-dlp sidecar：抖音/B站/YouTube）。
- 本地 TTS Pack（IndexTTS/OmniVoice/VoxCPM）HTTP 接入。
- 更新器、崩溃恢复、缓存清理、中英 i18n、快捷键面板。

---

## 3. 关键技术决策

| 议题 | 决策 | 理由 |
|---|---|---|
| 重活放哪 | Rust 负责编排/IO/ffmpeg/HTTP；前端只做状态与渲染 | 不阻塞 UI，Key 不进前端 |
| 长任务 | `job_id` + Tauri event 流 + 步骤产物落盘 | 可断点、可观测、可并发 |
| 本地模型 | 统一 HTTP 协议（端口 + 健康检查），不嵌入进程 | 避免包体爆炸与 CUDA 冲突 |
| FFmpeg | 随包分发静态构建 + 系统/自定义路径回退 | 用户零配置；企业环境可替换 |
| 字幕 | 优先 ASS（libass），失败回退 drawtext/PNG | 样式与兼容性平衡 |
| 剪映草稿 | 以「模板 + 轨道生成器」实现，版本适配层隔离 | 剪映 schema 会变 |
| 数据 | SQLite（项目/任务/版本）+ 文件（媒体/JSON） | 查询与可移植兼顾 |
| 安全 | Key 存 OS 凭据库；日志脱敏；URL 白名单提示 | 避免明文泄露 |

---

## 4. 风险与对策

| 风险 | 影响 | 对策 |
|---|---|---|
| 剪映草稿 schema 随版本变化 | 草稿打不开/素材丢失 | 固化 fixture、版本探测、导出后自检、失败降级为「素材包 + SRT」 |
| 抖音/B站解析不稳定 | 下载失败 | yt-dlp 随包 + 自动更新 + 明确失败提示；不承诺 100% |
| LLM 输出不合规 | 流水线中断 | 严格 schema + 确定性校验 + 定向修复 + 保留可编辑 JSON |
| 长视频渲染慢/占空间 | 体验差 | 代理预览、分段渲染、硬件编码、缓存裁剪片段 |
| 云端 TTS 限流 | 批量失败 | 并发池 + 退避重试 + 断点续配 + 本地引擎兜底 |
| 4K/多集素材 | 内存与磁盘压力 | 流式处理、临时目录配额、缓存清理 |
| 版权/合规 | 下架风险 | 仅提供工具能力；下载与素材使用提示用户自负合规 |

---

## 5. 验收口径（1:1）

对同一素材、同一模型、同一参数：

1. `items` 字段、顺序、`OST` 语义一致；`timestamp` 均为集内局部时间。
2. 成片时长与配音总时长误差 < 100ms；`OST=0` 无原声。
3. 剪映草稿包含视频/配音/原声/BGM/字幕五轨，素材可独立打开。
4. 五种模式均可产出；「短句解说」单句 8–24 字、一句一镜。
5. 任一失败可在当前步骤重试，不需要从头跑。

---

## 6. 下一步（待确认后执行）

1. 冻结设计方向与品牌名。
2. 初始化 `muyu/` 工程（Tauri 2 + Vue 3 + Naive UI + Rust workspace）。
3. 先交付 P0 的可运行壳 + 导入/字幕/项目库，再进 P1。
