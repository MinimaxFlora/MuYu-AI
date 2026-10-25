use serde_json::Value;

use crate::llm;
use crate::model::{Project, ScriptItem, ScriptParams};
use crate::settings::LlmSettings;

const MAX_CONTEXT_CHARS: usize = 60_000;

pub fn fmt_ts(seconds: f64) -> String {
    let total = seconds.max(0.0);
    let h = (total / 3600.0).floor() as u64;
    let m = ((total % 3600.0) / 60.0).floor() as u64;
    let s = (total % 60.0).floor() as u64;
    let ms = ((total - total.floor()) * 1000.0).round() as u64;
    format!("{h:02}:{m:02}:{s:02},{ms:03}")
}

fn parse_ts(raw: &str) -> Option<f64> {
    let cleaned = raw.trim().replace(',', ".");
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let h: f64 = parts[0].parse().ok()?;
    let m: f64 = parts[1].parse().ok()?;
    let s: f64 = parts[2].parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + s)
}

pub fn subtitle_context(project: &Project) -> String {
    let mut out = String::new();
    for cue in &project.subtitles {
        let text = cue.text.replace('\n', " ");
        let line = format!("[{} - {}] {}\n", fmt_ts(cue.start), fmt_ts(cue.end), text);
        if out.len() + line.len() > MAX_CONTEXT_CHARS {
            out.push_str("（字幕过长，后续内容已截断）\n");
            break;
        }
        out.push_str(&line);
    }
    out
}

fn video_meta(project: &Project) -> (u32, String, f64) {
    let name = project
        .videos
        .first()
        .map(|v| v.name.clone())
        .unwrap_or_else(|| "video.mp4".into());
    let end = project
        .videos
        .first()
        .map(|v| v.trim_end)
        .unwrap_or(0.0);
    (1, name, end)
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch == '\n' || ch == '\r' {
            continue;
        }
        current.push(ch);
        if "。！？!?；;".contains(ch) {
            let sentence = current.trim().trim_start_matches(|c: char| "，,、：: ".contains(c));
            if !sentence.is_empty() {
                out.push(sentence.to_string());
            }
            current.clear();
        }
    }
    let tail = current.trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    out
}

fn short(text: &str, limit: usize) -> String {
    let mut trimmed: String = text.chars().take(limit).collect();
    if text.chars().count() > limit {
        trimmed.push('…');
    }
    trimmed
}

fn demo_plot(project: &Project) -> String {
    let cues: Vec<String> = project
        .subtitles
        .iter()
        .map(|c| c.text.replace('\n', " ").trim().to_string())
        .filter(|t| !t.is_empty())
        .take(6)
        .collect();

    let first = cues
        .first()
        .map(|s| s.as_str())
        .unwrap_or("故事从一个不起眼的细节开始。");
    let mut lines = String::new();
    lines.push_str(&format!("一句话故事：{}\n", short(first, 60)));
    lines.push_str("人物与关系：以字幕中的关键人物为核心，围绕一次意外事件与隐藏证据展开；表面关系之下还有未说出口的动机。\n");
    lines.push_str("主线脉络：\n");
    for (index, cue) in cues.iter().enumerate() {
        lines.push_str(&format!("{}. {}\n", index + 1, short(cue, 42)));
    }
    if cues.is_empty() {
        lines.push_str("1. 素材缺少字幕，建议先补齐字幕后重新分析。\n");
    }
    lines.push_str("可用钩子：\n- 这件关键物件为什么会出现？\n- 谁在说谎，谁又在替谁隐瞒？\n- 当证据指向最信任的人，他会怎么选？\n");
    lines.push_str("注意事项：本结果由本地规则生成，仅用于打通流程；配置模型后会得到更完整的人物动机与因果链。\n");
    lines
}

fn demo_copy(project: &Project) -> Result<String, String> {
    if project.subtitles.is_empty() {
        return Err("没有字幕，无法生成文案".into());
    }
    let limit = project.subtitles.len().min(6);
    let mut sentences: Vec<String> = Vec::with_capacity(limit);
    if limit >= 2 {
        sentences.push("开场，先别急着看画面，这件事远没有表面那么简单。".into());
    }
    for cue in project
        .subtitles
        .iter()
        .take(limit.saturating_sub(sentences.len()))
    {
        let text = cue.text.replace('\n', " ");
        let sentence = short(text.trim(), 30);
        if !sentence.is_empty() {
            sentences.push(if sentence.ends_with('。')
                || sentence.ends_with('？')
                || sentence.ends_with('！')
            {
                sentence
            } else {
                format!("{sentence}。")
            });
        }
    }
    Ok(sentences.join(""))
}

fn demo_items(project: &Project) -> Result<Vec<ScriptItem>, String> {
    if project.subtitles.is_empty() {
        return Err("没有字幕，无法生成分镜".into());
    }
    let copy = project
        .narration_copy
        .clone()
        .unwrap_or_else(|| demo_copy(project).unwrap_or_default());
    let sentences = split_sentences(&copy);
    let (video_id, video_name, video_end) = video_meta(project);
    let last_cue = project.subtitles.last().unwrap();
    let mut items = Vec::new();
    let mut cursor_end = last_cue.end;

    for (index, sentence) in sentences.iter().take(120).enumerate() {
        let cue = project
            .subtitles
            .get(index.min(project.subtitles.len() - 1))
            .unwrap();
        let (start, end) = if index < project.subtitles.len() {
            (cue.start, cue.end.max(cue.start + 2.0))
        } else {
            let start = cursor_end + 0.2;
            let end = start + ((sentence.chars().count() as f64 / 5.0).max(2.0));
            (start, end)
        };
        cursor_end = end;
        if video_end > 0.0 && start >= video_end {
            break;
        }
        let end = if video_end > 0.0 { end.min(video_end) } else { end };
        items.push(ScriptItem {
            id: items.len() as u32 + 1,
            video_id,
            video_name: video_name.clone(),
            timestamp: format!("{}-{}", fmt_ts(start), fmt_ts(end)),
            picture: short(&cue.text.replace('\n', " "), 40),
            narration: sentence.clone(),
            ost: if index % 4 == 3 { 1 } else { 0 },
            audio_path: None,
            audio_duration: None,
        });
    }

    if items.is_empty() {
        return Err("没有可用的文案句子".into());
    }
    if items.len() >= 2 {
        if let Some(last) = items.last_mut() {
            last.ost = 1;
        }
    }
    Ok(items)
}

fn extract_json(raw: &str) -> Option<Value> {
    let trimmed = raw.trim();
    let cleaned = trimmed
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    if let Ok(value) = serde_json::from_str::<Value>(cleaned) {
        return Some(value);
    }
    let start = cleaned.find(['{', '['])?;
    let end = cleaned.rfind(['}', ']'])?;
    serde_json::from_str::<Value>(&cleaned[start..=end]).ok()
}

fn normalize_llm_items(raw: &str, project: &Project) -> Vec<ScriptItem> {
    let Some(value) = extract_json(raw) else {
        return Vec::new();
    };
    let array = value
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .or_else(|| value.as_array().cloned())
        .unwrap_or_default();

    let (video_id, video_name, video_end) = video_meta(project);
    let mut items = Vec::new();

    for entry in array {
        let narration = entry
            .get("narration")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if narration.is_empty() || narration.starts_with("播放原片") {
            continue;
        }
        let picture = entry
            .get("picture")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let ost = entry
            .get("OST")
            .or_else(|| entry.get("ost"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            .min(2) as u8;
        let raw_ts = entry
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let start = raw_ts
            .split('-')
            .next()
            .and_then(parse_ts)
            .unwrap_or(0.0);

        let cue_index = project
            .subtitles
            .iter()
            .position(|cue| start >= cue.start - 0.4 && start <= cue.end + 0.4)
            .unwrap_or_else(|| {
                project
                    .subtitles
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        (a.start - start)
                            .abs()
                            .partial_cmp(&(b.start - start).abs())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            });

        let Some(cue) = project.subtitles.get(cue_index) else {
            continue;
        };
        let min_duration = (narration.chars().count() as f64 / 5.0).max(2.0);
        let mut end = cue.end.max(cue.start + min_duration).min(cue.start + 12.0);
        if video_end > 0.0 {
            end = end.min(video_end);
        }

        items.push(ScriptItem {
            id: items.len() as u32 + 1,
            video_id,
            video_name: video_name.clone(),
            timestamp: format!("{}-{}", fmt_ts(cue.start), fmt_ts(end)),
            picture: if picture.is_empty() {
                short(&cue.text.replace('\n', " "), 40)
            } else {
                picture
            },
            narration,
            ost,
            audio_path: None,
            audio_duration: None,
        });
    }

    items.truncate(300);
    items
}

fn plot_prompt(project: &Project) -> (String, String) {
    let system = "你是影视解说策划。只输出中文结构化剧情理解，不写解说词，不输出 Markdown 代码块。".to_string();
    let user = format!(
        "作品：{}\n类型：{}\n\n字幕（含时间码）：\n{}\n\n请输出：\n一句话故事：\n人物与关系：\n主线脉络（3-8 条）：\n可用钩子（2-4 条）：\n注意事项（如有）：",
        project.name,
        project.params.genre,
        subtitle_context(project)
    );
    (system, user)
}

fn copy_prompt(project: &Project, params: &ScriptParams) -> (String, String) {
    let system = "你是短句影视解说文案写手。只输出解说正文，不要编号、时间戳、标题、解释或 Markdown。".to_string();
    let density = match params.density.as_str() {
        "dense" => "更密：每 3-5 句一个小钩子，句长 8-18 字",
        "loose" => "更疏：允许 18-26 字，留出画面呼吸",
        _ => "标准：单句 8-24 字，一句一个信息点",
    };
    let user = format!(
        "作品：{}\n类型：{}\n解说语言：{}\n目标字数：{} 字\n短句密度：{}\n\n剧情理解：\n{}\n\n原始字幕（含时间码）：\n{}\n\n写作要求：\n1. 开头前 3 句完成「人物困境 + 反常信息 + 悬念」。\n2. 一句只讲一个信息点，句子短，能一镜一句。\n3. 每 2-3 句有承接，别让观众跳场。\n4. 严格基于字幕事实，不编造人物身份与结局。\n5. 结尾留新问题或新危机，不要总结。\n6. 不要输出编号、时间戳、括号说明。",
        project.name,
        params.genre,
        params.language,
        params.target_words,
        density,
        project.plot_analysis.clone().unwrap_or_default(),
        subtitle_context(project)
    );
    (system, user)
}

fn items_prompt(project: &Project) -> (String, String) {
    let system = "你是影视解说剪辑师。只输出严格 JSON，不要 Markdown 代码块或解释。".to_string();
    let copy = project
        .narration_copy
        .clone()
        .filter(|c| !c.trim().is_empty())
        .or_else(|| demo_copy(project).ok())
        .unwrap_or_default();
    let user = format!(
        "作品：{}\n原片占比：{}%（OST=1 的镜头占比）\n\n已审核文案：\n{}\n\n字幕（含时间码）：\n{}\n\n要求：\n1. 把文案按句子顺序匹配到字幕时间轴，一句一个镜头。\n2. 只使用字幕里真实存在的时间码，timestamp 格式 HH:MM:SS,mmm-HH:MM:SS,mmm。\n3. OST=0 表示纯解说（去原声）；OST=1 表示保留原声，控制在约 {}%。\n4. 禁止选择片头、片尾、广告、预告、水印等非剧情片段。\n5. picture 用一句中文描述画面与人物状态。\n6. 输出 JSON：{{\"items\":[{{\"_id\":1,\"video_id\":1,\"video_name\":\"{}\",\"timestamp\":\"00:00:01,000-00:00:05,000\",\"picture\":\"画面描述\",\"narration\":\"解说词\",\"OST\":0}}]}}",
        project.name,
        project.params.original_ratio,
        copy,
        subtitle_context(project),
        project.params.original_ratio,
        project.videos.first().map(|v| v.name.clone()).unwrap_or_else(|| "video.mp4".into())
    );
    (system, user)
}

pub async fn analyze_plot(
    project: &Project,
    settings: &LlmSettings,
) -> Result<(String, Option<String>), String> {
    if project.subtitles.is_empty() {
        return Err("没有字幕，无法分析剧情".into());
    }
    if settings.is_demo() {
        return Ok((
            demo_plot(project),
            Some("演示模式：未配置 API Key，剧情理解为本地规则生成。".into()),
        ));
    }
    let (system, user) = plot_prompt(project);
    match llm::chat(settings, &system, &user, false, 3000).await {
        Ok(text) => Ok((text.trim().to_string(), None)),
        Err(err) => Ok((
            demo_plot(project),
            Some(format!("模型请求失败，已回退本地规则：{err}")),
        )),
    }
}

pub async fn generate_copy(
    project: &Project,
    settings: &LlmSettings,
    params: &ScriptParams,
) -> Result<(String, Option<String>), String> {
    if project.subtitles.is_empty() {
        return Err("没有字幕，无法生成文案".into());
    }
    if settings.is_demo() {
        return Ok((
            demo_copy(project)?,
            Some("演示模式：未配置 API Key，文案为本地规则生成。".into()),
        ));
    }
    let (system, user) = copy_prompt(project, params);
    match llm::chat(settings, &system, &user, false, 4096).await {
        Ok(text) => Ok((text.trim().trim_matches('"').to_string(), None)),
        Err(err) => Ok((
            demo_copy(project)?,
            Some(format!("模型请求失败，已回退本地规则：{err}")),
        )),
    }
}

pub async fn generate_items(
    project: &Project,
    settings: &LlmSettings,
) -> Result<(Vec<ScriptItem>, Option<String>), String> {
    if project.subtitles.is_empty() {
        return Err("没有字幕，无法生成分镜".into());
    }
    if settings.is_demo() {
        return Ok((
            demo_items(project)?,
            Some("演示模式：未配置 API Key，分镜为本地规则生成。".into()),
        ));
    }
    let (system, user) = items_prompt(project);
    match llm::chat(settings, &system, &user, true, 6000).await {
        Ok(raw) => {
            let items = normalize_llm_items(&raw, project);
            if items.is_empty() {
                Ok((
                    demo_items(project)?,
                    Some("模型没有返回可用的 items，已回退本地规则。".into()),
                ))
            } else {
                Ok((items, None))
            }
        }
        Err(err) => Ok((
            demo_items(project)?,
            Some(format!("模型请求失败，已回退本地规则：{err}")),
        )),
    }
}
