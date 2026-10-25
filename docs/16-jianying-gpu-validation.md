# 剪映 11.6 草稿与 GPU 编码验证

版本：0.9.8
日期：2026-10-25

---

## 1. GPU 硬件编码

本机编码器探测：

| 编码器 | 结果 |
|---|---|
| `h264_nvenc` | 失败（无 NVIDIA） |
| `h264_qsv` | 失败（无可用 Intel QSV） |
| `h264_amf` | 成功（AMD） |
| `libx264` | 成功（软件回退） |

### 发现并修复的问题

原探测使用 `64x64` 测试画面，AMD AMF 在该尺寸下拒绝编码，导致自动探测误判为不可用。

修复：探测改为 `320x240`、1 帧。

### 验证结果

自动编码器选择：

```text
enc=h264_amf
```

输出：

- 1280×720
- 时长 11.8s
- 589,090 B
- `ffprobe` 视频流 `encoder=Lavc63.1.101 h264_amf`
- 抽帧正常，字幕已烧录
- 音频 AAC 正常

说明：硬件编码真实走通，不再只测“回退 x264”。

## 2. 剪映 11.6 草稿

本机版本：

```text
剪映专业版 11.6.0.14552
安装目录 D:\JianyingPro
```

### 检测方式

已删除硬编码 `C:/D:/E:/JianyingPro` 猜测，改为：

1. 遍历 HKCU/HKLM 卸载项（64/32 位视图），匹配 `DisplayName` 含「剪映 / Jianying」。
2. 读取 `InstallLocation`、`DisplayIcon`、`UninstallString` 推导安装目录。
3. 查询 `App Paths\JianyingPro.exe`。
4. 回退到系统登记的 `%LOCALAPPDATA%\JianyingPro`。

验证：`export_jianying` 返回 `warnings: []`，不再误报“未检测到剪映”。

### 草稿内容

`MuYu_Regression`：

- 4 条轨道：视频 / 原声 / 配音 / 字幕（该项目未配置 BGM）
- 4 个素材
- 时长 11.759s
- 剪映首页识别到草稿卡片：824.5K，00:12
- 剪映将结构从 169.0.0 迁移到 189.0.0，并生成加密 `draft_content.json`

### 官方引擎导出

使用 `jy-draftc` 11.6（MIT，仅用于验证，不随 MuYu 分发）调用本机 `videoeditor.dll` 无头导出：

```text
export: jianying-export.mp4 (1,578,484 bytes, 3.7 s)
duration: 11.77s
video: H.264 1280×720
audio: AAC，mean -27.4 dB / max -7.0 dB
抽帧：背景画面 + 中文字幕正常
```

这说明草稿的时间线、素材路径、字幕和音频能被剪映官方引擎正确还原并渲染。

### 仍需确认

用自动化点击打开草稿后，剪映编辑器窗口出现过**空白白屏**；但官方引擎无头导出成功。

可能原因：

- 自动化恢复/最小化窗口导致编辑器不重绘；
- 169.0.0 → 189.0.0 迁移后，GUI 对 `app_version`/元数据的要求比导出引擎更严格。

结论：

- **草稿数据与渲染已通过官方引擎验证。**
- **GUI 编辑器显示尚未完全确认**，需要在正常人工双击打开的路径下再看一次；若仍白屏，下一步应生成 189.0.0 加密草稿或补齐 `app_version` 后再打开。

## 3. 相关代码

- `muyu/src-tauri/src/render.rs`：AMF 探测尺寸修复。
- `muyu/src-tauri/src/jianying.rs`：注册表/App Paths 检测；忽略测试 `detects_jianying_from_registry`。
- 0.9.8 已发布，含以上修复。

## 4. 边界

- `jy-draftc` 依赖剪映自带的 `videoeditor.dll`，只作为验证工具，不进入 MuYu 安装包。
- MuYu 当前仍生成 169.0.0 明文草稿，由剪映 11.6 自动迁移；若 GUI 白屏复现，需要增加官方加密/版本迁移步骤。
- Authenticode 证书未提供，安装包仍为未签名。
