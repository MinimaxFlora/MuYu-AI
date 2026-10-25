# P4 增量报告：BGM 与混音

日期：2026-10-25
范围：背景音乐选择、MP4 混音、剪映草稿 BGM 轨（ducking/淡入淡出见 `10-p4-audio-report.md`）

---

## 1. 已实现

### 1.1 项目设置
- 项目新增 `bgmPath` / `bgmVolume`（默认 18%）。
- 导出页可选 BGM、调音量、移除；设置立即写回项目。
- 支持 mp3 / wav / m4a / aac / flac / ogg。

### 1.2 MP4 混音
在逐镜拼接之后、字幕烧录之前混入 BGM：

```
concat.mp4 + BGM
  → BGM volume + atrim(total) + apad(total)
  → 与正片音轨 amix(duration=first)
  → mixed.mp4
  → 字幕烧录 / 输出
```

- BGM 自动循环（`-stream_loop -1`），裁到成片总长。
- 原声/配音音轨保持 1.0，BGM 按用户音量衰减。
- BGM 文件不存在时直接报错，不静默忽略。

### 1.3 剪映草稿 BGM 轨
- BGM 复制为 `assets/audio/bgm.<ext>`。
- 新增第 4 轨「BGM」，按 BGM 实际时长切段并循环铺满全片。
- 每段 `volume = project.bgmVolume`。
- 草稿结果从 3 轨变为 4 轨，后续原声独立轨再变为 5 轨。

### 1.4 UI
- 输出设置新增「背景音乐」行：选择/更换/移除、文件名、音量滑杆、百分比。
- 顶栏出现 `BGM` 状态 chip。
- 渲染完成后仍在结果区显示 BGM 配置。

## 2. 验证记录

| 检查 | 结果 |
|---|---|
| MP4 混音 | 760,569 B · 11.8s · 1280×720 |
| 流信息 | H.264 + AAC |
| BGM 存在性 | 220Hz 窄带对比 4.84×（邻频中位数），确认正弦 BGM 进入成片 |
| 草稿 | 首版 4 轨 / 4 素材；原声独立轨后为 5 轨 / 5 素材 |
| 草稿 BGM | `assets/audio/bgm.wav`，循环段铺满 11.759s |
| UI | 背景音乐行显示 `bgm.wav`、15%、顶栏 BGM chip |
| `cargo test` | 2 passed / 4 ignored（含 BGM 渲染与草稿） |
| `pnpm build` | vue-tsc + vite 通过 |

## 3. 限制

- **首版没有 ducking / sidechain**：已在 `10-p4-audio-report.md` 增量补齐。
- **首版没有淡入淡出**：已在 `10-p4-audio-report.md` 增量补齐。
- **没有响度归一化**：不同 BGM 的响度差异要靠手动音量补。
- **草稿 BGM 是循环切段**：剪映里看到的是多个首尾相接的 BGM 片段，不是单段循环素材。
- 原声仍没有独立音轨，靠视频 segment 的 volume 控制。
- 未做硬件编码、多集批量、安装包。

## 4. 下一步

1. 原声独立轨 + 可调 ducking 参数。
2. 硬件编码（NVENC/QSV/AMF）探测与回退。
3. 多集批量、渲染队列、临时目录清理。
4. 安装包 / 签名 / 自动更新。
5. 有剪映环境后做草稿打开回归。
