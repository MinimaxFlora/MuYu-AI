# P4 增量报告：硬件编码探测与回退

日期：2026-10-25
范围：NVENC / QSV / AMF 探测、自动选择、x264 回退、UI 编码器选项

---

## 1. 已实现

### 1.1 探测
- `ffmpeg -encoders` 检查 `h264_nvenc` / `h264_qsv` / `h264_amf` / `libx264` 是否存在。
- 对每个硬件编码器做 **64×64 / 1 帧空跑**：
  - 只有真正能编码成功才选用；
  - 仅“列表里有”不算可用。
- `auto` 顺序：NVENC → QSV → AMF → x264。

### 1.2 编码参数
| 编码器 | 参数 |
|---|---|
| `h264_nvenc` | `-preset p4 -cq 20 -b:v 0` |
| `h264_qsv` | `-preset veryfast -global_quality 22` |
| `h264_amf` | `-quality balanced -rc cqp -qp_i 20 -qp_p 22` |
| `libx264` | `-preset veryfast -crf 20` |

- 所有编码器统一 `yuv420p`、25fps、90k timescale，保证 concat 可 `-c copy`。
- 字幕烧录优先用选中的硬件编码器；失败自动重试 x264。
- `RenderResult.encoder` 回传实际使用的编码器，UI 显示为 `x264` / `NVIDIA NVENC` 等。

### 1.3 UI
- 输出设置新增「编码器」：
  - 自动（优先硬件）
  - NVIDIA NVENC
  - Intel QSV
  - AMD AMF
  - x264（兼容）
- 文案提示「不可用时自动回退 x264」。
- 成片结果新增「编码」字段。

## 2. 验证记录

| 检查 | 结果 |
|---|---|
| 本机 FFmpeg 列表 | 有 `h264_nvenc` / `h264_qsv` / `h264_amf` / `libx264` |
| 实际空跑 | NVENC 失败、QSV 失败、AMF 失败（本机无对应硬件/驱动） |
| `auto` 选择 | 正确回退 `libx264` |
| 后端成片 | 760,812 B · 11.8s · 字幕烧录成功 · `enc=libx264` |
| UI | 编码器下拉 5 项；结果「编码 x264」 |
| UI 成片 | 741.8 KB · 11.8s |
| `cargo test` | 2 passed / 4 ignored |
| `pnpm build` | vue-tsc + vite 通过 |

## 3. 限制

- **本机没有可用硬件编码器**，NVENC/QSV/AMF 路径只验证了探测与回退，未在真实 GPU 上验证编码质量/速度。
- 如果“空跑”通过但正式滤镜链失败，目前会直接报错，不会在渲染中途切回 x264。
- 没有码率/质量滑杆；硬件编码的画质参数固定。
- 没有按项目保存编码器选择（每次导出页默认 auto）。
- 多集批量、安装包仍未做。

## 4. 下一步

1. 多集批量 + 渲染队列（同一套编码器选择）。
2. 安装包 / 签名 / 自动更新。
3. 原声独立轨 + 可调 ducking。
4. 有 GPU 的机器上做硬件编码画质/速度回归。
5. 有剪映环境后做草稿打开回归。
