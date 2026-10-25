# P0 交付报告

日期：2026-10-25
范围：可运行的 Tauri 桌面壳 + 项目库 + 视频/字幕素材链路

---

## 1. 交付物

| 路径 | 内容 |
|---|---|
| `muyu/` | Tauri 2 + Rust + Vue 3 + Naive UI 工程 |
| `muyu/src-tauri/src/` | 命令、ffprobe/ffmpeg、SRT、项目存储 |
| `muyu/samples/` | 8 秒测试视频 + 同名 SRT |
| `design/screens/p0-home.png` | 真实桌面端首页截图 |
| `design/screens/p0-studio.png` | 真实桌面端素材工作台截图 |

## 2. 已实现

- 项目库：每个项目 `projects/<id>/project.json` + `cover.jpg`，原子写入。
- 视频导入：多选/拖拽，ffprobe 元数据（时长、宽高、fps、编码、码率、大小）。
- 封面帧：ffmpeg 抽帧（10% 处，上限 3s），960px JPEG；失败自动回退到 0s。
- 字幕：同名字幕自动挂载、手动导入/移除、UTF-8/UTF-16/GBK 解码、时间轴解析。
- 截取范围：双滑块 + 防抖自动保存 + 时间显示。
- 设计系统：深色影院风、Rail、顶栏、项目卡、步骤条、面板、Naive UI 主题覆盖。
- 环境检测：FFmpeg/FFprobe 可用性与版本，在设置页与侧栏展示。
- 原生窗口：最大化、深色、最小尺寸、拖拽导入、打开项目目录。

## 3. 验证记录

| 检查 | 结果 |
|---|---|
| `cargo check` | 通过（仅 Windows 链接器提示） |
| `cargo test` | `imports_sample_project_end_to_end` 通过 |
| `pnpm build`（vue-tsc + vite） | 通过 |
| `pnpm tauri dev` | 窗口「幕语」启动，无崩溃 |
| 真实导入链路 | 测试断言：时长 ≈ 8s、1280×720、封面存在、3 条字幕、可 list/load |
| 环境检测 | 桌面端显示 FFmpeg/FFprobe「已检测到」 |
| 资产加载 | 封面经 `asset://` 正常加载（naturalWidth 960） |

`cargo test` 覆盖：ffprobe 探测 → ffmpeg 抽帧 → SRT 解码解析 → project.json 落盘 → list/load。

## 4. 实现中的三个修正

1. **窗口尺寸**：在 125% DPI、1536×960 的工作区里，1440 逻辑宽度会被系统裁切；改为 `maximized` + 1280×820 默认值。
2. **封面协议**：`assetProtocol.scope` 从 `$APPDATA/com.muyu.studio/**` 改为 `$APPDATA/**`，因为 `$APPDATA` 已指向应用数据目录。
3. **Vite watcher**：Windows 上编辑文件会生成 `.tmpdir`，Vite 监听时报 `EBUSY`；已在 `server.watch.ignored` 排除。

## 5. 与计划的偏差

- **项目库用 JSON，不是 SQLite**：P0 项目量小，JSON 便于排查、迁移与手动恢复；P1 引入任务/版本表时再迁移 SQLite。
- **FFmpeg 不随包分发**：P0 使用系统 PATH；P1 打安装包时再捆绑静态构建。
- **UI 导入未做自动化点击**：文件对话框为原生窗口；后端 `create_project` 链路已由测试覆盖，前端调用路径与 studio 展示已用种子项目验证。

## 6. 已知限制

- 链接下载、任务队列、播放器、剧情理解、文案、配音、时间轴、渲染、剪映导出未实现。
- 浏览器预览模式（`pnpm dev`）只有示例数据，不能写盘。
- 项目 JSON 含全量字幕，超大项目（数万条）后续应拆分存储。

## 7. 下一步（P1）

1. LLM Provider（OpenAI 兼容 + Gemini）+ 设置页密钥管理。
2. 剧情理解 → 解说文案（逐句）→ 片段规划 → `items` 脚本。
3. 文案工作台（逐句编辑、改写、字数/时长预估）。
4. 任务队列与断点续跑。
