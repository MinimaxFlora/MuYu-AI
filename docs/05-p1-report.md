# P1 交付报告

日期：2026-10-25
范围：文本模型接入 + 剧情理解 + 逐句文案 + 分镜脚本（items）

---

## 1. 交付物

| 路径 | 内容 |
|---|---|
| `muyu/src-tauri/src/llm.rs` | OpenAI 兼容客户端、连接测试、JSON 模式降级 |
| `muyu/src-tauri/src/pipeline.rs` | 三段流水线、提示词、时间码归一化、演示规则 |
| `muyu/src-tauri/src/settings.rs` | 模型设置持久化 |
| `muyu/src/views/WriteView.vue` | AI 工作台（剧情 / 文案 / 分镜） |
| `muyu/src/views/SettingsView.vue` | 模型配置表单 |
| `design/screens/p1-*.png` | 真实桌面端截图 |

## 2. 已实现

### 2.1 模型接入
- OpenAI 兼容协议：填 Base URL、模型、Key、温度即可。
- 默认 `https://api.deepseek.com/v1` + `deepseek-chat`。
- 「测试连接」走真实请求（未配置 Key 时返回演示提示）。
- JSON 模式失败（400/422）自动去掉 `response_format` 重试。
- 设置落盘 `%APPDATA%\com.muyu.studio\settings.json`。

### 2.2 三段流水线
```
字幕（带时间码） → 剧情理解 → 逐句文案 → 分镜 items
```
| 步骤 | 输入 | 输出 | 失败策略 |
|---|---|---|---|
| 剧情理解 | 字幕 + 类型 | 一句话故事 / 人物 / 主线 / 钩子 | 回退本地规则，并写入 `pipelineNote` |
| 逐句文案 | 剧情 + 字幕 + 类型/字数/密度 | 8–24 字短句正文 | 回退本地规则 |
| 分镜脚本 | 审核后文案 + 字幕 | `items[]`（含 OST） | JSON 解析失败 / 空数组时回退本地规则 |

### 2.3 数据模型
项目新增：
- `plotAnalysis`：可编辑的剧情理解
- `narrationCopy`：逐句文案正文
- `items`：分镜数组（`_id / videoId / videoName / timestamp / picture / narration / OST`）
- `params`：类型、语言、目标字数、密度、原片占比
- `pipelineNote`：降级/回退原因

### 2.4 质量闸门
- 字幕上下文超过 60k 字符自动截断，避免爆上下文。
- LLM 返回的 `timestamp` **吸附到真实字幕区间**，不接受虚构时间码。
- `_id` 重新连续编号，`OST` 限制在 0–2。
- 空 narration / 「播放原片」条目丢弃。
- 最多 300 镜，防止异常输出撑爆项目。
- 任何模型失败都会保留一个可继续编辑的本地结果，而不是把流程卡死。

### 2.5 演示模式
未配置 Key（或手动打开演示模式）时不发网络请求：
- 剧情：从字幕提炼结构化摘要
- 文案：钩子 + 字幕句子，控制在字幕条数内
- 分镜：一句一镜，最后一个镜头设为原声（约 33%）
这样首次安装即可完整走通三步，也便于截图与验收。

### 2.6 UI
- 新路由 `/studio/:id/write`，三步 Tab：剧情理解 / 逐句文案 / 分镜脚本。
- 文案以「句卡」呈现，可逐句改、删、补，实时显示字数与预计秒数。
- 分镜表可改解说、切 OST、删镜头、复制 items JSON。
- 右侧参数与统计随步骤切换；演示模式有明确横幅。
- 素材工作台的「进入剧情理解」已接通（无字幕时禁用）。

## 3. 验证记录

| 检查 | 结果 |
|---|---|
| `cargo test` | 2 passed / 1 ignored：导入链路 + 演示流水线 |
| 演示流水线断言 | 剧情含「一句话故事」；文案 > 20 字；items 非空、`_id` 连续、时间码合法 |
| `pnpm build` | vue-tsc + vite 通过 |
| 桌面端 UI 流程 | 真实 WebView：剧情生成 → 3 句文案 → 3 个镜头，OST 0/0/1（33%） |
| 设置页 | Base URL 回填、测试连接返回演示提示、保存写入 settings.json |
| 截图 | `design/screens/p1-plot.png`、`p1-copy.png`、`p1-items.png`、`p1-settings.png` |

## 4. 与设计的偏差

- **规划与生成合并为一次调用**：NarratoAI 先规划 `segments` 再写脚本；P1 为降低失败率，让模型直接输出 `items`，再用确定性规则把时间码吸附到字幕。P2 做长视频 / 多集时再拆回「规划 → 绑定」两段。
- **Gemini 原生 / Tavily / 视觉未接入**：P2 再做；当前只走 OpenAI 兼容文本接口。
- **Key 明文落盘**：P1 先保证可用；P2 迁移 Windows 凭据管理器。
- **未做真实模型调用验收**：本机没有可用 Key，真实请求路径只做了代码与降级逻辑验证；配置 Key 后需要一次真机联调。

## 5. 下一步（P2）

1. Edge TTS 逐句配音 + 时长回写 + 试听缓存。
2. 分镜时间轴：OST 切换、音画对齐检查、单镜重配。
3. 长字幕分块 / 多集规划，恢复「规划 → 绑定」两段式。
4. 任务队列、断点续跑、取消与重试。
5. Windows 凭据库、Gemini 原生、Tavily、视觉抽帧（纪录片）。
