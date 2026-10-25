# P2 交付报告

日期：2026-10-25
范围：Edge TTS 逐句配音 + 时长回写 + 音画对齐 + 分镜时间轴

---

## 1. 交付物

| 路径 | 内容 |
|---|---|
| `muyu/src-tauri/src/tts.rs` | Edge TTS 会话、Windows SAPI 回退、音色列表 |
| `muyu/src-tauri/src/commands.rs` | 批量配音、进度事件、取消、试听 |
| `muyu/src-tauri/.cargo/config.toml` | `AWS_LC_SYS_NO_ASM=1`，免 NASM 构建 |
| `muyu/src/views/VoiceView.vue` | 配音工作台 + 四轨时间轴 |
| `design/screens/p2-voice.png` | 真实桌面端截图 |

## 2. 已实现

### 2.1 配音引擎
- **Edge TTS（默认，在线）**：`msedge-tts` 0.4，微软神经音色，免 Key。
- **Windows SAPI（离线回退）**：`System.Speech`，系统已装音色。
- 音色列表：Edge 返回中英 31 个音色；SAPI 返回本机音色。
- 语速 0.5–1.5x，映射到 Edge `rate%` / SAPI `Rate`。
- 试听：合成一句固定文案到缓存目录并播放。

### 2.2 批量配音
- 只处理 `OST=0` / `OST=2` 的解说镜头。
- 逐条合成到 `projects/<id>/audio/<镜头号>.mp3|wav`。
- 每条完成后用 ffprobe 读真实时长，回写 `audioDuration`，并立即保存项目。
- Tauri 事件 `tts://progress` 推送 `current/total/percent/message/done`。
- 支持「继续配音（仅缺失）」与「重新配音」。
- 支持取消：当前条完成后停止，已完成的音频保留。
- 失败不中断整批：记录失败条数，写入 `ttsNote`。

### 2.3 时间轴与对齐
- 四轨：视频 / 原声 / 配音 / 字幕。
- 镜头宽度按**有效时长**（有配音用配音时长，否则用原槽时长）计算。
- 颜色：金 = 解说、青 = 原声、紫 = 混合。
- 选中镜头有播放头；点击任意块切换镜头。
- 顶部显示成片总时长；右侧显示「已配音 X/Y」「对齐误差 ms」。
- 配音时长与原槽差异 >500ms 时，误差数字标金提示。

### 2.4 试听与播放
- 每镜「播放本镜」通过 `asset://` 播放本地 mp3/wav。
- 音色「试听」合成临时文件播放，不写入项目。
- 浏览器预览模式只计算时长，不生成音频。

## 3. 验证记录

| 检查 | 结果 |
|---|---|
| Edge TTS 冒烟 | `YunxiNeural`，18,720 bytes MP3，耗时 4.3s |
| 桌面端批量配音 | 3 镜 → 2 条目标；进度 100%；`ttsNote=配音完成 2/2` |
| 时长回写 | `5.736s`、`4.224s`；原槽 3.0s / 2.8s |
| 音频文件 | `001.mp3` 32KB、`002.mp3` 24KB |
| WebView 读取 | `asset://` 200，`audio/mpeg`，34,416 bytes |
| SAPI 回退 | 试听生成 `preview-*.wav`，236KB |
| 进度事件 | 0% → 100%，`2 / 2` |
| 取消 | 命令与静态标志已接，UI 有「停止」 |
| `cargo test` | 2 passed / 2 ignored（含 Edge 冒烟、种子项目） |
| `pnpm build` | vue-tsc + vite 通过 |

## 4. 构建说明

`msedge-tts` 依赖 `aws-lc-sys`，默认需要 NASM。项目在 `src-tauri/.cargo/config.toml` 设置了：

```toml
[env]
AWS_LC_SYS_NO_ASM = "1"
```

用 aws-lc 的预生成源码构建，**不需要安装 NASM**。代价是少量汇编优化缺失，对 TTS 无感。

## 5. 与设计的偏差

- **时间轴是 4 轨**（视频/原声/配音/字幕），设计稿是 5 轨（含 BGM）。BGM 混音在 P3 渲染时接。
- **没有连续播放**：目前是逐镜播放；整片预览要等 P3 渲染或播放器。
- **配音串行**：为了稳定与限流，逐条合成；38 镜大约 1–2 分钟。
- **取消是软取消**：当前条合成完才停。
- **画面还没按配音重剪**：P2 只回写时长并在时间轴展示，真正裁切在 P3。

## 6. 已知限制

- Edge TTS 依赖网络；微软接口变更会导致失败，已内置 SAPI 回退与提示。
- 音色/语速写入项目，但尚未做「按镜头单独音色」。
- 没有 BGM、原声压低、淡入淡出与响度归一化。
- 没有渲染与剪映导出（P3）。

## 7. 下一步（P3）

1. 按 `audioDuration` 重剪画面：OST=0 去原声、OST=1 保原声、OST=2 混音。
2. 视频拼接 + 配音/原声/BGM 混音 + 智能配平。
3. 字幕烧录（ASS）、原字幕遮盖、比例/分辨率。
4. 合成 MP4 + 剪映草稿 + SRT 导出。
5. 硬件编码与失败回退、导出队列。
