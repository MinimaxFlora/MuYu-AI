# P4 交付报告（第一阶段：剪映草稿）

日期：2026-10-25
范围：剪映专业版草稿包导出（首版三轨 + 素材复制 + 结构自检；BGM 见 `09-p4-bgm-report.md`）

---

## 1. 交付物

| 路径 | 内容 |
|---|---|
| `muyu/src-tauri/src/jianying.rs` | 草稿模板、视频/配音/字幕轨、素材复制、根索引 |
| `muyu/src-tauri/src/commands.rs` | `export_jianying` / `open_path` |
| `muyu/src/views/ExportView.vue` | 草稿目录选择、结果卡片、警告 |
| `design/screens/p4-jianying.png` | 真实桌面端截图 |

## 2. 已实现

### 2.1 草稿结构
按剪映 10.6.0 / `new_version=169.0.0` 的明文结构生成：

```
<草稿根>/<名称>/
├── draft_info.json
├── draft_meta_info.json
├── draft_settings
├── draft_cover.jpg
├── template.tmp / template-2.tmp
├── root_meta_info.json（草稿根）
├── assets/video/源视频.mp4
├── assets/audio/audio_XXX.mp3
├── common_attachment/*
├── draft_agency_config.json / draft_biz_config.json
├── draft_virtual_store.json / performance_opt_info.json / timeline_layout.json
└── Resources / subdraft / matting / qr_upload / smart_crop / adjust_mask
```

### 2.2 轨道
- **视频轨**：每个 item 一个 segment，`source_timerange` 用原时间码，`target_timerange` 用累计时间；OST=0 音量为 0，OST=1/2 保留原声。
- **配音轨**：OST=0/2 的 mp3 按累计时间铺开，音量 1.0。
- **字幕轨**：按重新计时后的时间轴写 `text` 素材，底部居中，白色 + 黑色描边。
- **素材复制**：源视频与配音复制进 `assets/`，草稿不依赖原项目路径。
- **根索引**：更新 `root_meta_info.json` 的 `all_draft_store`，避免同名覆盖。

### 2.3 工程
- 草稿导出后写入 `project.lastDraft`，重开项目仍能看到上次草稿路径。
- UI 选择剪映草稿目录（原生文件夹选择器），生成后可打开目录。
- 未安装剪映时给出明确警告：结构已生成，但**未做剪映内打开验证**。

## 3. 验证记录

| 检查 | 结果 |
|---|---|
| 后端端到端 | 3 镜 → 2 条配音 → 草稿生成 |
| 草稿 | 3 轨 · 3 素材 · 11.759s |
| 轨道数 | 视频 / 配音 / 字幕 = 3 |
| 素材 | `assets/video` 1 个、`assets/audio` 2 个 |
| 文件 | `draft_info.json` / `draft_meta_info.json` / `draft_settings` / `template*.tmp` 均在 |
| 根索引 | `root_meta_info.json` 已写入 |
| UI | 草稿卡片显示 3 轨/3 素材/00:11，可打开目录 |
| `cargo test` | 2 passed / 4 ignored（Edge、种子、渲染、草稿） |
| `pnpm build` | vue-tsc + vite 通过 |

## 4. 关键限制

- **未在剪映里打开验证**：本机没有安装剪映专业版。结构按 NarratoAI 的 169.0.0 明文草稿移植，并做了 JSON/路径/轨道的自检；但剪映版本差异可能导致打不开或提示升级。
- **首版没有 BGM 轨**：已在 `09-p4-bgm-report.md` 增量补齐，当前草稿为四轨。
- **原声没有单独音轨**：靠视频 segment 的 `volume` 控制；如果在剪映里拆原声，需要额外音轨。
- **没有转场/特效/关键帧/封面布局**。
- **草稿目录需手动选择**：没有自动探测剪映安装路径（本机未安装，无法验证探测规则）。

## 5. 下一步（P4 剩余）

1. 原声独立轨、ducking、音量包络与淡入淡出。
2. 硬件编码探测（NVENC/QSV/AMF）与回退。
3. 多集批量、渲染队列、临时目录清理。
4. 安装包（NSIS/MSI）、签名、自动更新。
5. 有剪映环境后做草稿打开回归，按版本修正 schema。
