# 幕语 · 高保真界面稿 v0.1

四张关键界面，覆盖「首页 → 文案 → 分镜 → 导出」的主链路。

| 文件 | 界面 | 说明 |
|---|---|---|
| `mockups/home.html` | 首页 | 继续创作、项目库、任务队列、引擎状态 |
| `mockups/script.html` | 文案工作台 | 逐句解说、单句工具条、参数与实时统计 |
| `mockups/timeline.html` | 分镜与时间轴 | 预览、分镜表、五轨时间轴、镜头检查器 |
| `mockups/export.html` | 导出中心 | 三种产物、输出设置、队列、摘要 |
| `mockups/style.css` | 设计系统 | 色彩、字体、间距、组件、暗色主题 |

截图：`screens/*.png`（设计稿 3200 × 2000，2× 缩放）。

## 真实桌面端截图（P0 / P1）

| 文件 | 界面 |
|---|---|
| `screens/p0-home.png` | 首页（真实 Tauri 窗口） |
| `screens/p0-studio.png` | 素材工作台 |
| `screens/p1-plot.png` | 剧情理解 |
| `screens/p1-copy.png` | 逐句文案 |
| `screens/p1-items.png` | 分镜脚本（items） |
| `screens/p1-settings.png` | 模型设置 |
| `screens/p2-voice.png` | 配音与时间轴 |
| `screens/p3-export.png` | 成片导出 |
| `screens/p4-jianying.png` | 成片与剪映草稿 |
| `screens/p4-bgm.png` | BGM 混音与成片 |
| `screens/p4-ducking.png` | BGM 闪避与淡入淡出 |
| `screens/p4-encoder.png` | 编码器与自动回退 |
| `screens/p4-batch.png` | 批量渲染队列 |
| [`app-icon.png`](app-icon.png) | 应用图标（2026-10 新品牌） |

> 真实截图里的《雾港》是 8 秒测试素材，用于验证抽帧、字幕、AI 流程与落库。

## 查看

直接双击任意 HTML（需要与 `style.css` 同目录）。

## 重新生成截图

```powershell
$edge = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
$base = "D:\AI 软件开发\AI\design"
foreach ($s in @("home","script","timeline","export")) {
  $uri = ([uri]"$base\mockups\$s.html").AbsoluteUri
  & $edge --headless=new --disable-gpu --hide-scrollbars `
    --force-device-scale-factor=2 --window-size=1600,1000 --virtual-time-budget=2000 `
    --screenshot="$base\screens\$s.png" $uri
}
```

## 设计约定

- 基准画布 1600 × 1000；应用最小窗口 1280 × 800。
- 影院暗色：`#08090D` 底、`#0F1219` 面板、金色 `#E9B949` 主行动。
- OST 色码：金 = 解说、青 = 原声、紫 = 混合；颜色 + 文字双编码。
- 字体：MiSans → HarmonyOS Sans SC → 微软雅黑；数字用 Bahnschrift。
- 全部图标为 24 网格、1.5px 线性；不用彩色插画图标。
- 界面稿中的影片、人物、数据均为虚构示例。

> 这是设计稿，不是可运行的前端工程；实现时以 `docs/02-product-design.md` 为准，并按 Naive UI 主题映射落地。
