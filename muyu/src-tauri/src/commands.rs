use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::jianying::{self, DraftResult};
use crate::llm;
use crate::media;
use crate::model::{MediaInfo, Project, ProjectSummary, ScriptParams, VideoAsset};
use crate::pipeline;
use crate::render::{self, RenderOptions, RenderResult};
use crate::settings::{LlmSettings, SettingsStore};
use crate::store::{self, Store};
use crate::subtitle;
use crate::tts::{self, VoiceInfo};

static TTS_CANCEL: AtomicBool = AtomicBool::new(false);
static RENDER_CANCEL: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn create_project(
    app: tauri::AppHandle,
    paths: Vec<String>,
    name: Option<String>,
    mode: Option<String>,
) -> Result<Project, String> {
    let store = Store::new(&app)?;
    import_project(&store, &paths, name, mode)
}

pub fn import_project(
    store: &Store,
    paths: &[String],
    name: Option<String>,
    mode: Option<String>,
) -> Result<Project, String> {
    if paths.is_empty() {
        return Err("请至少选择一个视频文件".into());
    }

    let id = store::new_id();
    store.create_dir(&id)?;

    let result = (|| -> Result<Project, String> {
        let mut videos: Vec<VideoAsset> = Vec::new();
        let mut cover_path: Option<String> = None;

        for (index, raw_path) in paths.iter().enumerate() {
            let path = PathBuf::from(raw_path);
            if !path.exists() {
                return Err(format!("文件不存在：{}", path.display()));
            }
            let info = media::probe(raw_path)?;
            let display_name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "未命名素材".into());
            let trim_end = if info.duration > 0.0 { info.duration } else { 0.0 };
            let duration = info.duration;

            videos.push(VideoAsset {
                path: raw_path.clone(),
                name: display_name,
                info,
                trim_start: 0.0,
                trim_end,
            });

            if index == 0 {
                let at = if duration > 0.0 {
                    (duration * 0.1).clamp(0.0, 3.0)
                } else {
                    1.0
                };
                let out = store.project_dir(&id).join("cover.jpg");
                if media::extract_cover(raw_path, &out, at).is_ok() {
                    cover_path = Some(out.to_string_lossy().to_string());
                }
            }
        }

        let mut subtitle_path: Option<String> = None;
        let mut subtitles = Vec::new();
        if let Some(found) = subtitle::find_sibling_subtitle(&paths[0]) {
            match subtitle::read_and_parse(&found) {
                Ok(cues) => {
                    subtitle_path = Some(found);
                    subtitles = cues;
                }
                Err(err) => {
                    eprintln!("自动挂载字幕失败：{err}");
                }
            }
        }

        let project_name = name
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| {
                PathBuf::from(&paths[0])
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "未命名项目".into())
            });

        let now = store::now_iso();
        let project = Project {
            id: id.clone(),
            name: project_name,
            mode: mode.unwrap_or_else(|| "shortline".into()),
            status: "draft".into(),
            created_at: now.clone(),
            updated_at: now,
            videos,
            subtitle_path,
            subtitles,
            cover_path,
            plot_analysis: None,
            narration_copy: None,
            items: Vec::new(),
            params: ScriptParams::default(),
            pipeline_note: None,
            voice: "zh-CN-YunxiNeural".into(),
            tts_engine: "edge".into(),
            voice_rate: 1.0,
            tts_note: None,
            last_render: None,
            last_draft: None,
            bgm_path: None,
            bgm_volume: 0.18,
            bgm_ducking: true,
            bgm_fade: true,
            ducking_strength: "medium".into(),
        };

        store.save(&project)?;
        Ok(project)
    })();

    if result.is_err() {
        let _ = store.delete(&id);
    }
    result
}

#[tauri::command]
pub fn list_projects(app: tauri::AppHandle) -> Result<Vec<ProjectSummary>, String> {
    let store = Store::new(&app)?;
    Ok(store.list())
}

#[tauri::command]
pub fn get_project(app: tauri::AppHandle, id: String) -> Result<Project, String> {
    let store = Store::new(&app)?;
    store.load(&id)
}

#[tauri::command]
pub fn save_project(app: tauri::AppHandle, mut project: Project) -> Result<Project, String> {
    if project.id.trim().is_empty() {
        return Err("项目 ID 不能为空".into());
    }
    let store = Store::new(&app)?;
    if let Ok(existing) = store.load(&project.id) {
        if project.created_at.is_empty() {
            project.created_at = existing.created_at;
        }
    }
    Store::touch(&mut project);
    store.save(&project)?;
    Ok(project)
}

#[tauri::command]
pub fn attach_subtitle(
    app: tauri::AppHandle,
    id: String,
    path: String,
) -> Result<Project, String> {
    let store = Store::new(&app)?;
    let mut project = store.load(&id)?;
    let cues = subtitle::read_and_parse(&path)?;
    project.subtitle_path = Some(path);
    project.subtitles = cues;
    Store::touch(&mut project);
    store.save(&project)?;
    Ok(project)
}

#[tauri::command]
pub fn remove_subtitle(app: tauri::AppHandle, id: String) -> Result<Project, String> {
    let store = Store::new(&app)?;
    let mut project = store.load(&id)?;
    project.subtitle_path = None;
    project.subtitles = Vec::new();
    Store::touch(&mut project);
    store.save(&project)?;
    Ok(project)
}

#[tauri::command]
pub fn delete_project(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let store = Store::new(&app)?;
    store.delete(&id)
}

#[tauri::command]
pub fn probe_media(path: String) -> Result<MediaInfo, String> {
    media::probe(&path)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentStatus {
    pub ffmpeg: bool,
    pub ffprobe: bool,
    pub ffmpeg_version: String,
}

#[tauri::command]
pub fn check_environment() -> EnvironmentStatus {
    let (ffmpeg, ffprobe, version) = media::environment_status();
    EnvironmentStatus {
        ffmpeg,
        ffprobe,
        ffmpeg_version: version,
    }
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Result<LlmSettings, String> {
    SettingsStore::new(&app)?.load()
}

#[tauri::command]
pub fn save_settings(
    app: tauri::AppHandle,
    mut settings: LlmSettings,
) -> Result<LlmSettings, String> {
    settings.base_url = settings.base_url.trim().trim_end_matches('/').to_string();
    settings.model = settings.model.trim().to_string();
    settings.temperature = settings.temperature.clamp(0.0, 2.0);
    if settings.base_url.is_empty() {
        return Err("API Base URL 不能为空".into());
    }
    if settings.model.is_empty() {
        return Err("模型名称不能为空".into());
    }
    llm::endpoint(&settings.base_url)?;
    SettingsStore::new(&app)?.save(&settings)?;
    Ok(settings)
}

#[tauri::command]
pub async fn test_llm(settings: LlmSettings) -> Result<String, String> {
    llm::test_connection(&settings).await
}

fn normalized_params(mut params: ScriptParams) -> ScriptParams {
    params.target_words = params.target_words.clamp(120, 6000);
    params.original_ratio = params.original_ratio.min(80);
    if params.genre.trim().is_empty() {
        params.genre = "悬疑 / 犯罪".into();
    }
    if params.language.trim().is_empty() {
        params.language = "中文（简体）".into();
    }
    if !matches!(params.density.as_str(), "dense" | "standard" | "loose") {
        params.density = "standard".into();
    }
    params
}

#[tauri::command]
pub async fn analyze_plot(app: tauri::AppHandle, id: String) -> Result<Project, String> {
    let store = Store::new(&app)?;
    let mut project = store.load(&id)?;
    let settings = SettingsStore::new(&app)?.load()?;
    let (text, note) = pipeline::analyze_plot(&project, &settings).await?;
    project.plot_analysis = Some(text);
    project.pipeline_note = note;
    Store::touch(&mut project);
    store.save(&project)?;
    Ok(project)
}

#[tauri::command]
pub async fn generate_copy(
    app: tauri::AppHandle,
    id: String,
    params: ScriptParams,
) -> Result<Project, String> {
    let store = Store::new(&app)?;
    let mut project = store.load(&id)?;
    let settings = SettingsStore::new(&app)?.load()?;
    let params = normalized_params(params);
    let (text, note) = pipeline::generate_copy(&project, &settings, &params).await?;
    project.params = params;
    project.narration_copy = Some(text);
    project.items = Vec::new();
    project.pipeline_note = note;
    Store::touch(&mut project);
    store.save(&project)?;
    Ok(project)
}

#[tauri::command]
pub async fn generate_items(app: tauri::AppHandle, id: String) -> Result<Project, String> {
    let store = Store::new(&app)?;
    let mut project = store.load(&id)?;
    let settings = SettingsStore::new(&app)?.load()?;
    let (items, note) = pipeline::generate_items(&project, &settings).await?;
    project.items = items;
    project.pipeline_note = note;
    Store::touch(&mut project);
    store.save(&project)?;
    Ok(project)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsProgress {
    pub current: usize,
    pub total: usize,
    pub item_id: Option<u32>,
    pub percent: f64,
    pub message: String,
    pub done: bool,
}

#[tauri::command]
pub async fn list_tts_voices(engine: String) -> Result<Vec<VoiceInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || tts::list_voices(&engine))
        .await
        .map_err(|e| format!("音色任务失败：{e}"))?
}

#[tauri::command]
pub async fn preview_voice(
    app: tauri::AppHandle,
    engine: String,
    voice: String,
    rate: f32,
) -> Result<String, String> {
    use tauri::Manager;

    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法定位应用数据目录：{e}"))?
        .join("cache");
    std::fs::create_dir_all(&base).map_err(|e| format!("创建缓存目录失败：{e}"))?;
    let extension = if engine == "sapi" { "wav" } else { "mp3" };
    let out = base.join(format!("preview-{}.{}", store::new_id(), extension));

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut session = tts::SynthSession::new(&engine, &voice, rate, base.clone())?;
        session.synthesize("你好，我是幕语的配音测试，这是一段试听。", 0, &out)?;
        Ok::<String, String>(out.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| format!("试听任务失败：{e}"))?;
    result
}

#[tauri::command]
pub fn cancel_tts() {
    TTS_CANCEL.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub async fn synthesize_items(
    app: tauri::AppHandle,
    id: String,
    engine: String,
    voice: String,
    rate: f32,
    force: bool,
) -> Result<Project, String> {
    let store = Store::new(&app)?;
    let mut project = store.load(&id)?;
    if project.items.is_empty() {
        return Err("还没有分镜，先生成 items".into());
    }

    let rate = rate.clamp(0.5, 2.0);
    let audio_dir = store.project_dir(&id).join("audio");
    std::fs::create_dir_all(&audio_dir).map_err(|e| format!("创建音频目录失败：{e}"))?;

    let indices: Vec<usize> = project
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| (item.ost == 0 || item.ost == 2) && !item.narration.trim().is_empty())
        .map(|(index, _)| index)
        .collect();
    if indices.is_empty() {
        return Err("没有需要配音的解说镜头".into());
    }

    TTS_CANCEL.store(false, Ordering::SeqCst);
    let total = indices.len();
    let app_handle = app.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut session = tts::SynthSession::new(&engine, &voice, rate, audio_dir.clone())?;
        let mut current = 0usize;
        let mut errors: Vec<String> = Vec::new();
        let mut skipped = 0usize;

        for &index in &indices {
            if TTS_CANCEL.load(Ordering::SeqCst) {
                break;
            }
            let (item_id, narration) = {
                let item = &project.items[index];
                (item.id, item.narration.clone())
            };

            if !force {
                if let Some(path) = project.items[index].audio_path.clone() {
                    if Path::new(&path).exists() {
                        skipped += 1;
                        current += 1;
                        let _ = app_handle.emit(
                            "tts://progress",
                            TtsProgress {
                                current,
                                total,
                                item_id: Some(item_id),
                                percent: current as f64 / total as f64 * 100.0,
                                message: format!("已存在，跳过 #{item_id}"),
                                done: false,
                            },
                        );
                        continue;
                    }
                }
            }

            let out = audio_dir.join(format!("{item_id:03}.{}", session.extension()));
            match session.synthesize(&narration, item_id, &out) {
                Ok(duration) => {
                    project.items[index].audio_path =
                        Some(out.to_string_lossy().to_string());
                    project.items[index].audio_duration = Some(duration);
                    let _ = store.save(&project);
                }
                Err(err) => {
                    errors.push(format!("#{item_id} {err}"));
                }
            }

            current += 1;
            let _ = app_handle.emit(
                "tts://progress",
                TtsProgress {
                    current,
                    total,
                    item_id: Some(item_id),
                    percent: current as f64 / total as f64 * 100.0,
                    message: format!("已完成 {current} / {total}"),
                    done: false,
                },
            );
        }

        let canceled = TTS_CANCEL.load(Ordering::SeqCst);
        project.voice = voice;
        project.tts_engine = engine;
        project.voice_rate = rate;
        project.tts_note = if canceled {
            Some(format!("配音已取消（完成 {current}/{total}）"))
        } else if !errors.is_empty() {
            Some(format!(
                "配音完成 {}/{total}，失败 {} 条：{}",
                current,
                errors.len(),
                errors.first().cloned().unwrap_or_default()
            ))
        } else {
            Some(format!(
                "配音完成 {current}/{total}{}",
                if skipped > 0 {
                    format!("，跳过 {skipped} 条已存在")
                } else {
                    String::new()
                }
            ))
        };
        Store::touch(&mut project);
        store.save(&project)?;

        let _ = app_handle.emit(
            "tts://progress",
            TtsProgress {
                current,
                total,
                item_id: None,
                percent: if total > 0 {
                    current as f64 / total as f64 * 100.0
                } else {
                    100.0
                },
                message: if canceled {
                    "已取消".into()
                } else {
                    "配音完成".into()
                },
                done: true,
            },
        );

        Ok::<Project, String>(project)
    })
    .await
    .map_err(|e| format!("配音任务失败：{e}"))?;

    result
}

#[tauri::command]
pub fn reveal_project(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let store = Store::new(&app)?;
    let dir = store.project_dir(&id);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("打开目录失败：{e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("打开目录失败：{e}"))?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("打开目录失败：{e}"))?;
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderProgress {
    pub percent: f64,
    pub message: String,
    pub done: bool,
}

#[tauri::command]
pub async fn render_video(
    app: tauri::AppHandle,
    id: String,
    options: RenderOptions,
) -> Result<RenderResult, String> {
    let store = Store::new(&app)?;
    let mut project = store.load(&id)?;
    if project.items.is_empty() {
        return Err("还没有分镜，无法渲染".into());
    }

    let project_dir = store.project_dir(&id);
    let work_dir = project_dir.join("render").join(store::new_id());
    let output_dir = project_dir.join("output");
    RENDER_CANCEL.store(false, Ordering::SeqCst);

    let app_handle = app.clone();
    let project_clone = project.clone();
    let options_clone = options.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        render::render_project(
            &project_clone,
            &options_clone,
            &work_dir,
            &output_dir,
            &|percent, message| {
                let _ = app_handle.emit(
                    "render://progress",
                    RenderProgress {
                        percent,
                        message,
                        done: percent >= 100.0,
                    },
                );
            },
            &|| RENDER_CANCEL.load(Ordering::SeqCst),
        )
    })
    .await
    .map_err(|e| format!("渲染任务失败：{e}"))??;

    project.last_render = Some(result.clone());
    Store::touch(&mut project);
    store.save(&project)?;
    Ok(result)
}

#[tauri::command]
pub fn cancel_render() {
    RENDER_CANCEL.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub fn export_srt(app: tauri::AppHandle, id: String) -> Result<String, String> {
    let store = Store::new(&app)?;
    let project = store.load(&id)?;
    let output_dir = store.project_dir(&id).join("output");
    std::fs::create_dir_all(&output_dir).map_err(|e| format!("创建输出目录失败：{e}"))?;
    let name = render::safe_name(&project.name);
    let path = output_dir.join(format!("{name}.srt"));
    std::fs::write(&path, render::srt_for_project(&project))
        .map_err(|e| format!("写入字幕失败：{e}"))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn export_items(app: tauri::AppHandle, id: String) -> Result<String, String> {
    let store = Store::new(&app)?;
    let project = store.load(&id)?;
    let output_dir = store.project_dir(&id).join("output");
    std::fs::create_dir_all(&output_dir).map_err(|e| format!("创建输出目录失败：{e}"))?;
    let name = render::safe_name(&project.name);
    let path = output_dir.join(format!("{name}.items.json"));
    let json = serde_json::to_string_pretty(&project.items)
        .map_err(|e| format!("序列化 items 失败：{e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("写入 items 失败：{e}"))?;
    Ok(path.to_string_lossy().to_string())
}
#[tauri::command]
pub async fn export_jianying(
    app: tauri::AppHandle,
    id: String,
    root: String,
    name: Option<String>,
) -> Result<DraftResult, String> {
    if root.trim().is_empty() {
        return Err("请选择剪映草稿目录".into());
    }
    let store = Store::new(&app)?;
    let mut project = store.load(&id)?;
    let project_for_thread = project.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        jianying::export_draft(&project_for_thread, &root, name)
    })
    .await
    .map_err(|e| format!("草稿导出任务失败：{e}"))??;
    project.last_draft = Some(result.clone());
    Store::touch(&mut project);
    store.save(&project)?;
    Ok(result)
}

#[tauri::command]
pub fn allow_asset_path(app: tauri::AppHandle, path: String) -> Result<String, String> {
    let canonical = std::fs::canonicalize(&path)
        .map_err(|e| format!("无法解析视频路径：{e}"))?;
    app.asset_protocol_scope()
        .allow_file(&canonical)
        .map_err(|e| format!("无法授权视频预览：{e}"))?;
    Ok(canonical.to_string_lossy().to_string())
}

#[tauri::command]
pub fn detect_jianying_root() -> Option<String> {
    crate::jianying::detect_draft_root()
}

#[tauri::command]
pub fn open_log_dir(app: tauri::AppHandle) -> Result<String, String> {
    let dir = app
        .path()
        .app_log_dir()
        .map_err(|e| format!("无法定位日志目录：{e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建日志目录失败：{e}"))?;
    open_path(dir.to_string_lossy().to_string())?;
    Ok(dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    let target = if Path::new(&path).is_dir() {
        PathBuf::from(&path)
    } else {
        Path::new(&path)
            .parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(&path))
    };
    if !target.exists() {
        return Err(format!("目录不存在：{}", target.display()));
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&target)
            .spawn()
            .map_err(|e| format!("打开目录失败：{e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&target)
            .spawn()
            .map_err(|e| format!("打开目录失败：{e}"))?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(&target)
            .spawn()
            .map_err(|e| format!("打开目录失败：{e}"))?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_sample_project_end_to_end() {
        let base = std::env::temp_dir().join(format!("muyu-test-{}", store::new_id()));
        let store = Store::from_projects_dir(base.join("projects")).expect("create store");

        let sample = format!(
            "{}/../samples/雾港_EP01.mp4",
            env!("CARGO_MANIFEST_DIR")
        );
        let project = import_project(&store, &[sample.clone()], None, Some("shortline".into()))
            .expect("import project");

        assert!(project.videos.len() == 1);
        assert!(
            project.videos[0].info.duration > 7.0,
            "duration: {}",
            project.videos[0].info.duration
        );
        assert_eq!(project.videos[0].info.width, 1280);
        assert_eq!(project.subtitles.len(), 3, "srt cues");
        assert!(project.subtitle_path.is_some());
        assert!(
            project
                .cover_path
                .as_ref()
                .map(|p| std::path::Path::new(p).exists())
                .unwrap_or(false),
            "cover should exist"
        );

        let listed = store.list();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].duration.round() as i64, 8);

        let loaded = store.load(&project.id).expect("load");
        assert_eq!(loaded.name, "雾港_EP01");
        assert_eq!(loaded.subtitles[0].text, "海雾漫过码头，一艘没有编号的货轮靠岸。");

        let _ = std::fs::remove_dir_all(base);
    }

    /// 仅供本机开发预览：把示例项目写进真实应用数据目录（含 AI 三步产物）。
    #[tokio::test]
    #[ignore]
    async fn seed_sample_project_for_dev() {
        let appdata = std::env::var("APPDATA").expect("APPDATA");
        let root = std::path::PathBuf::from(appdata)
            .join("com.muyu.studio")
            .join("projects");
        let store = Store::from_projects_dir(root).expect("store");
        let sample = format!("{}/../samples/雾港_EP01.mp4", env!("CARGO_MANIFEST_DIR"));
        let mut project = import_project(
            &store,
            &[sample],
            Some("《雾港》第 1 集".into()),
            Some("shortline".into()),
        )
        .expect("seed");

        let settings = LlmSettings::default();
        let (plot, _) = pipeline::analyze_plot(&project, &settings)
            .await
            .expect("plot");
        project.plot_analysis = Some(plot);
        let (copy, _) = pipeline::generate_copy(&project, &settings, &project.params)
            .await
            .expect("copy");
        project.narration_copy = Some(copy);
        let (items, _) = pipeline::generate_items(&project, &settings)
            .await
            .expect("items");
        project.items = items;
        project.pipeline_note = Some("演示模式：本地规则生成".into());

        let audio_dir = store.project_dir(&project.id).join("audio");
        std::fs::create_dir_all(&audio_dir).expect("audio dir");
        let mut session = tts::SynthSession::new(
            "edge",
            "zh-CN-YunxiNeural",
            1.0,
            audio_dir.clone(),
        )
        .expect("tts");
        for item in project
            .items
            .iter_mut()
            .filter(|item| item.ost == 0 || item.ost == 2)
        {
            let out = audio_dir.join(format!("{:03}.mp3", item.id));
            let duration = session
                .synthesize(&item.narration, item.id, &out)
                .expect("synth");
            item.audio_path = Some(out.to_string_lossy().to_string());
            item.audio_duration = Some(duration);
        }
        project.tts_note = Some("配音完成".into());

        Store::touch(&mut project);
        store.save(&project).expect("save");
        println!("seeded_project_id={}", project.id);
    }

    #[test]
    #[ignore]
    fn edge_tts_smoke() {
        use msedge_tts::tts::{client::connect, SpeechConfig};
        use msedge_tts::voice::get_voices_list;

        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let voices = get_voices_list().expect("voices");
        assert!(!voices.is_empty());
        let voice = voices
            .iter()
            .find(|v| v.name.contains("YunxiNeural"))
            .or_else(|| voices.iter().find(|v| v.locale.as_deref() == Some("zh-CN")))
            .expect("chinese voice");
        let config = SpeechConfig::from(voice);
        let mut client = connect().expect("connect");
        let audio = client
            .synthesize("你好，这是幕语的配音测试。", &config)
            .expect("synthesize");
        assert!(audio.audio_bytes.len() > 1000, "audio too small");
        let out = format!("{}/../samples/edge-test.mp3", env!("CARGO_MANIFEST_DIR"));
        std::fs::write(&out, &audio.audio_bytes).expect("write mp3");
        println!(
            "voice={} bytes={} out={}",
            voice.name,
            audio.audio_bytes.len(),
            out
        );
    }

    #[tokio::test]
    #[ignore]
    async fn jianying_draft_end_to_end() {
        let base = std::env::temp_dir().join(format!("muyu-p4-{}", store::new_id()));
        let store = Store::from_projects_dir(base.join("projects")).expect("store");
        let sample = format!("{}/../samples/雾港_EP01.mp4", env!("CARGO_MANIFEST_DIR"));
        let mut project = import_project(&store, &[sample], None, None).expect("import");

        let settings = LlmSettings::default();
        let (plot, _) = pipeline::analyze_plot(&project, &settings).await.expect("plot");
        project.plot_analysis = Some(plot);
        let (copy, _) = pipeline::generate_copy(&project, &settings, &project.params)
            .await
            .expect("copy");
        project.narration_copy = Some(copy);
        let (items, _) = pipeline::generate_items(&project, &settings)
            .await
            .expect("items");
        project.items = items;

        let audio_dir = base.join("audio");
        std::fs::create_dir_all(&audio_dir).expect("audio");
        let mut session = tts::SynthSession::new("edge", "zh-CN-YunxiNeural", 1.0, audio_dir.clone())
            .expect("tts");
        for item in project.items.iter_mut().filter(|item| item.ost == 0 || item.ost == 2) {
            let out = audio_dir.join(format!("{:03}.mp3", item.id));
            let duration = session.synthesize(&item.narration, item.id, &out).expect("synth");
            item.audio_path = Some(out.to_string_lossy().to_string());
            item.audio_duration = Some(duration);
        }

        let root = base.join("drafts");
        std::fs::create_dir_all(&root).expect("root");
        let bgm = base.join("bgm.wav");
        let bgm_status = std::process::Command::new(media::ffmpeg_bin())
            .args([
                "-y", "-f", "lavfi", "-i", "sine=frequency=220:duration=20",
                "-c:a", "pcm_s16le",
            ])
            .arg(&bgm)
            .status()
            .expect("bgm");
        assert!(bgm_status.success());
        project.bgm_path = Some(bgm.to_string_lossy().to_string());
        project.bgm_volume = 0.15;
        let result = jianying::export_draft(&project, &root.to_string_lossy(), None).expect("draft");
        println!(
            "draft={} duration={} tracks={} segments={} assets={} warnings={:?}",
            result.path, result.duration, result.tracks, result.segments, result.assets, result.warnings
        );

        let draft_dir = std::path::Path::new(&result.path);
        for file in ["draft_info.json", "draft_meta_info.json", "draft_settings", "template.tmp", "template-2.tmp"] {
            assert!(draft_dir.join(file).exists(), "missing {file}");
        }
        assert!(root.join("root_meta_info.json").exists());
        let info: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(draft_dir.join("draft_info.json")).unwrap()).unwrap();
        assert!(info["duration"].as_i64().unwrap_or(0) > 0);
        assert!(info["tracks"].as_array().map(|a| a.len() >= 5).unwrap_or(false));
        assert!(draft_dir.join("assets/video").read_dir().map(|mut i| i.next().is_some()).unwrap_or(false));
        assert!(draft_dir.join("assets/audio").read_dir().map(|mut i| i.next().is_some()).unwrap_or(false));
        assert!(draft_dir.join("assets/audio/bgm.wav").exists());
        assert!(draft_dir.join("assets/audio/original_003.wav").exists());
        let _ = std::fs::remove_dir_all(base);
    }
    #[tokio::test]
    #[ignore]
    async fn render_sample_end_to_end() {
        let base = std::env::temp_dir().join(format!("muyu-p3-{}", store::new_id()));
        let store = Store::from_projects_dir(base.join("projects")).expect("store");
        let sample = format!("{}/../samples/雾港_EP01.mp4", env!("CARGO_MANIFEST_DIR"));
        let mut project = import_project(&store, &[sample], None, None).expect("import");

        let settings = LlmSettings::default();
        let (plot, _) = pipeline::analyze_plot(&project, &settings).await.expect("plot");
        project.plot_analysis = Some(plot);
        let (copy, _) = pipeline::generate_copy(&project, &settings, &project.params)
            .await
            .expect("copy");
        project.narration_copy = Some(copy);
        let (items, _) = pipeline::generate_items(&project, &settings)
            .await
            .expect("items");
        project.items = items;

        let audio_dir = base.join("audio");
        std::fs::create_dir_all(&audio_dir).expect("audio dir");
        let mut session = tts::SynthSession::new(
            "edge",
            "zh-CN-YunxiNeural",
            1.0,
            audio_dir.clone(),
        )
        .expect("tts session");
        for item in project
            .items
            .iter_mut()
            .filter(|item| item.ost == 0 || item.ost == 2)
        {
            let out = audio_dir.join(format!("{:03}.mp3", item.id));
            let duration = session
                .synthesize(&item.narration, item.id, &out)
                .expect("synth");
            item.audio_path = Some(out.to_string_lossy().to_string());
            item.audio_duration = Some(duration);
        }

        let out_dir = base.join("output");
        let work_dir = base.join("work");
        let bgm = base.join("bgm.wav");
        let bgm_status = std::process::Command::new(media::ffmpeg_bin())
            .args([
                "-y", "-f", "lavfi", "-i", "sine=frequency=220:duration=20",
                "-c:a", "pcm_s16le",
            ])
            .arg(&bgm)
            .status()
            .expect("bgm");
        assert!(bgm_status.success());
        let mut render_options = RenderOptions::default();
        render_options.bgm_path = Some(bgm.to_string_lossy().to_string());
        render_options.bgm_volume = 0.15;
        let result = render::render_project(
            &project,
            &render_options,
            &work_dir,
            &out_dir,
            &|_, _| {},
            &|| false,
        )
        .expect("render");

        assert!(std::path::Path::new(&result.path).exists());
        assert!(result.size > 10_000, "size={}", result.size);
        assert!(result.duration > 5.0, "duration={}", result.duration);
        println!(
            "rendered={} size={} duration={} subs={} enc={} note={:?}",
            result.path, result.size, result.duration, result.subtitle_burned, result.encoder, result.note
        );
        if std::env::var("MUYU_KEEP_RENDER").is_err() {
            let _ = std::fs::remove_dir_all(base);
        }
    }
    #[tokio::test]
    async fn demo_pipeline_generates_items() {
        let base = std::env::temp_dir().join(format!("muyu-p1-{}", store::new_id()));
        let store = Store::from_projects_dir(base.join("projects")).expect("store");
        let sample = format!("{}/../samples/雾港_EP01.mp4", env!("CARGO_MANIFEST_DIR"));
        let project = import_project(&store, &[sample], None, None).expect("import");

        let settings = LlmSettings::default();
        assert!(settings.is_demo(), "empty key should use demo mode");

        let (plot, _) = pipeline::analyze_plot(&project, &settings).await.expect("plot");
        assert!(plot.contains("一句话故事"), "plot: {plot}");

        let mut project = project;
        project.plot_analysis = Some(plot);

        let (copy, _) = pipeline::generate_copy(&project, &settings, &project.params)
            .await
            .expect("copy");
        assert!(copy.chars().count() > 20, "copy: {copy}");
        project.narration_copy = Some(copy);

        let (items, _) = pipeline::generate_items(&project, &settings)
            .await
            .expect("items");
        assert!(!items.is_empty());
        assert_eq!(items[0].id, 1);
        assert!(items[0].timestamp.contains('-'));
        assert!(items.iter().all(|item| !item.narration.is_empty()));
        assert!(items.iter().any(|item| item.ost == 0));

        let _ = std::fs::remove_dir_all(base);
    }
}
