# P3 交付报告

日期：2026-10-25
范围：按配音重剪 + 音画合成 + 字幕烧录 + 成片 MP4 / SRT / items 导出

---

## 1. 交付物

| 路径 | 内容 |
|---|---|
| `muyu/src-tauri/src/render.rs` | 逐镜裁剪、拼接、字幕烧录、尺寸适配 |
| `muyu/src-tauri/src/commands.rs` | render / cancel / export_srt / export_items |
| `muyu/src/views/ExportView.vue` | 导出设置、渲染进度、成片预览、文件导出 |
| `design/screens/p3-export.png` | 真实桌面端成片导出截图 |

## 2. 已实现

### 2.1 渲染管线
```
items → 逐镜裁剪（OST 0/1/2）→ 统一编码 → concat → 字幕烧录 → MP4
```
- **OST=0**：按 `audioDuration` 裁画面，去掉原声，贴配音。
- **OST=1**：按原时间码裁画面，保留原声。
- **OST=2**：按配音时长裁画面，原声 + 配音 `amix`。
- 统一输出：H.264 / AAC 48kHz 立体声 / 25fps / `yuv420p` / 90k timescale，保证 concat 可 `-c copy`。
- 画面适配：`scale + pad + setsar`，支持原始 / 16:9 / 9:16 / 1:1；720p / 1080p。
- 短画面自动 `tpad` 冻结补足，避免音画长度不一致。
- 字幕：按新的累计时间轴生成 SRT，再用 `subtitles` 滤镜烧录（Microsoft YaHei，字号随分辨率）。
- 字幕烧录失败时自动回退为无字幕 MP4，并在结果里写 `note`。

### 2.2 导出
- **成片 MP4**：`projects/<id>/output/<名称>.mp4`，可在应用内直接播放。
- **SRT**：独立导出，时间轴与成片一致，OST=1 不入字幕。
- **items JSON**：含 `_id / timestamp / picture / narration / OST / audioPath / audioDuration`。
- 输出目录支持资源管理器打开。
- 渲染进度 `render://progress`、软取消（当前镜头结束后停止）。

### 2.3 UI
- 五步 Stepper 走到「成片导出」。
- 输出设置（比例 / 分辨率 / 字幕 / 名称）、成片预览、文件导出、输出路径。
- 右侧汇总：镜头、解说镜、配音完成度、成片时长、画面尺寸。
- 未完成配音时禁用渲染，并给出「去配音」。

## 3. 验证记录

| 检查 | 结果 |
|---|---|
| 后端端到端 | 3 镜 → 2 条配音 → 渲染成功 |
| 成片 | 747,485 B · 11.8s · 1280×720 |
| 流信息 | H.264 + AAC，时长 11.800s |
| 字幕 | `subtitleBurned=true`，播放器可见中文烧录字幕 |
| SRT | 197 B，按配音时长重新计时 |
| items JSON | 1,251 B |
| 应用内播放 | `<video>` readyState=4，duration=11.8 |
| `cargo test` | 2 passed / 3 ignored（Edge 冒烟、种子、渲染端到端） |
| `pnpm build` | vue-tsc + vite 通过 |

## 4. 与设计的偏差

- **剪映草稿未实现**：草稿 schema 与剪映版本强绑定，本机没有剪映可验收；P4 用 fixture + 导出后自检再做，避免给一个打不开的草稿。
- **BGM 未接**：P3 只有配音与原声；BGM 轨与智能配平在 P4。
- **只用了 libx264**：硬件编码（NVENC/QSV/AMF）与回退还没接。
- **单项目串行渲染**：没有批量队列，渲染时不能在应用内同时做别的重任务。
- **取消粒度是镜头**：FFmpeg 子进程没有中途 kill。

## 5. 已知限制

- 1080p 竖屏渲染在长视频上会比较慢，暂无进度条内的 ffmpeg 百分比。
- 字幕字体依赖系统安装的 Microsoft YaHei；缺失时 libass 会回退。
- 输出会覆盖同名文件，没有版本号后缀。
- 临时片段目录 `render/<uuid>` 暂不自动清理。
- 没有封面、片头片尾、水印、转场。

## 6. 下一步（P4）

1. 剪映 / CapCut 草稿：视频/配音/原声/BGM/字幕五轨、素材复制、版本探测与自检。
2. BGM 轨、智能配平、淡入淡出、响度归一化。
3. 硬件编码探测与多级回退、ffmpeg 进度解析。
4. 多集批量、渲染队列、临时目录清理。
5. 安装包、签名、自动更新、链接下载、本地克隆 TTS。
