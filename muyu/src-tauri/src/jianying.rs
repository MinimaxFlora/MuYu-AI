use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::media;
use crate::model::Project;

const DRAFT_PATH_PLACEHOLDER: &str =
    "##_draftpath_placeholder_0E685133-18CE-45ED-8CB8-2904A212EC80_##";
const PLATFORM_VERSION: &str = "10.6.0";
const DRAFT_VERSION: &str = "169.0.0";

const MATERIAL_KEYS: &[&str] = &[
    "ai_translates",
    "audio_balances",
    "audio_effects",
    "audio_fades",
    "audio_pannings",
    "audio_pitch_shifts",
    "audio_track_indexes",
    "audios",
    "beats",
    "canvases",
    "chromas",
    "color_curves",
    "common_mask",
    "digital_human_model_dressing",
    "digital_humans",
    "drafts",
    "effects",
    "flowers",
    "green_screens",
    "handwrites",
    "hsl",
    "hsl_curves",
    "images",
    "log_color_wheels",
    "loudnesses",
    "manual_beautys",
    "manual_deformations",
    "material_animations",
    "material_colors",
    "multi_language_refs",
    "placeholder_infos",
    "placeholders",
    "plugin_effects",
    "primary_color_wheels",
    "realtime_denoises",
    "shapes",
    "smart_crops",
    "smart_relights",
    "sound_channel_mappings",
    "speeds",
    "stickers",
    "tail_leaders",
    "text_templates",
    "texts",
    "time_marks",
    "transitions",
    "video_effects",
    "video_radius",
    "video_shadows",
    "video_strokes",
    "video_trackings",
    "videos",
    "vocal_beautifys",
    "vocal_separations",
    "audios",
    "texts",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftResult {
    pub path: String,
    pub name: String,
    pub duration: f64,
    pub segments: usize,
    pub tracks: usize,
    pub assets: usize,
    pub warnings: Vec<String>,
}

fn uid() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

fn now_us() -> i64 {
    Utc::now().timestamp_micros()
}

fn format_uuid(raw: &str) -> String {
    let compact: String = raw.chars().filter(|ch| *ch != '-').collect();
    if compact.len() != 32 {
        return raw.to_string();
    }
    format!(
        "{}-{}-{}-{}-{}",
        &compact[0..8],
        &compact[8..12],
        &compact[12..16],
        &compact[16..20],
        &compact[20..32]
    )
    .to_uppercase()
}

fn safe_name(raw: &str, fallback: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|ch| {
            if "<>:\"/\\|?*".contains(ch) || ch.is_control() {
                '_'
            } else {
                ch
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').to_string();
    if trimmed.is_empty() {
        fallback.into()
    } else {
        trimmed.chars().take(60).collect()
    }
}

fn parse_ts(raw: &str) -> f64 {
    let cleaned = raw.trim().replace(',', ".");
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() != 3 {
        return 0.0;
    }
    let h: f64 = parts[0].parse().unwrap_or(0.0);
    let m: f64 = parts[1].parse().unwrap_or(0.0);
    let s: f64 = parts[2].parse().unwrap_or(0.0);
    h * 3600.0 + m * 60.0 + s
}

fn slot_duration(timestamp: &str) -> f64 {
    let (start, end) = timestamp.split_once('-').unwrap_or((timestamp, timestamp));
    (parse_ts(end) - parse_ts(start)).max(0.0)
}

fn floor_ms(seconds: f64) -> f64 {
    (seconds * 1000.0).floor() / 1000.0
}

fn us(seconds: f64) -> i64 {
    (floor_ms(seconds) * 1_000_000.0).round() as i64
}

fn placeholder() -> String {
    DRAFT_PATH_PLACEHOLDER.to_string()
}

fn material_path(relative: &str) -> String {
    format!("{}/{relative}", placeholder())
}

fn platform_info() -> Value {
    json!({
        "app_id": 3704,
        "app_source": "lv",
        "app_version": PLATFORM_VERSION,
        "device_id": "",
        "hard_disk_id": "",
        "mac_address": "",
        "os": "windows",
        "os_version": ""
    })
}

fn create_materials() -> Map<String, Value> {
    let mut map = Map::new();
    for key in MATERIAL_KEYS {
        map.insert(key.to_string(), json!([]));
    }
    map
}

fn draft_template(draft_id: &str, name: &str, width: u32, height: u32) -> Value {
    let now = now_us();
    json!({
        "canvas_config": {"height": height, "ratio": "original", "width": width},
        "color_space": 0,
        "config": {
            "adjust_max_index": 1,
            "attachment_info": [],
            "combination_max_index": 1,
            "export_range": null,
            "extract_audio_last_index": 1,
            "lyrics_recognition_id": "",
            "lyrics_sync": true,
            "lyrics_taskinfo": [],
            "maintrack_adsorb": true,
            "material_save_mode": 0,
            "multi_language_current": "none",
            "multi_language_list": [],
            "multi_language_main": "none",
            "multi_language_mode": "none",
            "original_sound_last_index": 1,
            "record_audio_last_index": 1,
            "sticker_max_index": 1,
            "subtitle_keywords_config": null,
            "subtitle_recognition_id": "",
            "subtitle_sync": true,
            "subtitle_taskinfo": [],
            "system_font_list": [],
            "video_mute": false,
            "zoom_info_params": null
        },
        "cover": null,
        "create_time": now,
        "duration": 0,
        "extra_info": null,
        "fps": 30.0,
        "free_render_index_mode_on": false,
        "group_container": null,
        "id": draft_id,
        "keyframe_graph_list": [],
        "keyframes": {
            "adjusts": [], "audios": [], "effects": [], "filters": [],
            "handwrites": [], "stickers": [], "texts": [], "videos": []
        },
        "last_modified_platform": platform_info(),
        "materials": create_materials(),
        "mutable_config": null,
        "name": name,
        "new_version": DRAFT_VERSION,
        "relationships": [],
        "render_index_track_mode_on": true,
        "retouch_cover": null,
        "source": "default",
        "static_cover_image_path": "",
        "time_marks": null,
        "tracks": [],
        "update_time": now,
        "version": 360000
    })
}

fn track(kind: &str, name: &str) -> Value {
    json!({
        "attribute": 0,
        "flag": 0,
        "id": uid(),
        "is_default_name": true,
        "name": name,
        "segments": [],
        "type": kind
    })
}

fn crop() -> Value {
    json!({
        "lower_left_x": 0.0, "lower_left_y": 1.0,
        "lower_right_x": 1.0, "lower_right_y": 1.0,
        "upper_left_x": 0.0, "upper_left_y": 0.0,
        "upper_right_x": 1.0, "upper_right_y": 0.0
    })
}

fn video_material(id: &str, rel: &str, duration_sec: f64, width: u32, height: u32) -> Value {
    let name = Path::new(rel)
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| "source.mp4".into());
    json!({
        "id": id,
        "path": material_path(rel),
        "type": "video",
        "duration": us(duration_sec),
        "width": width,
        "height": height,
        "material_name": name,
        "create_time": now_us(),
        "crop": crop(),
        "extra_type_option": 0,
        "source_platform": 0,
        "category_name": "local",
        "check_flag": 65535,
        "has_audio": true,
        "is_copyright": false,
        "local_material_id": "",
        "material_id": "",
        "picture_from": "none",
        "source": 0,
        "unique_id": ""
    })
}

fn audio_material(id: &str, rel: &str, duration_sec: f64) -> Value {
    let name = Path::new(rel)
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| "audio.mp3".into());
    json!({
        "id": id,
        "path": material_path(rel),
        "type": "extract_music",
        "duration": us(duration_sec),
        "name": name,
        "category_id": "",
        "category_name": "local",
        "check_flag": 1,
        "copyright_limit_type": "none",
        "effect_id": "",
        "formula_id": "",
        "local_material_id": id,
        "music_id": id,
        "source_platform": 0,
        "remote_url": "",
        "request_id": "",
        "resource_id": "",
        "team_id": "",
        "text_id": "",
        "video_id": "",
        "wave_points": []
    })
}

fn text_material(id: &str, text: &str) -> Value {
    let content = json!({
        "styles": [{
            "fill": {
                "alpha": 1.0,
                "content": {
                    "render_type": "solid",
                    "solid": { "alpha": 1.0, "color": [1.0, 1.0, 1.0] }
                }
            },
            "range": [0, text.chars().count()],
            "size": 5.0,
            "bold": false,
            "italic": false,
            "underline": false,
            "strokes": [{
                "content": { "solid": { "alpha": 1.0, "color": [0.0, 0.0, 0.0] } },
                "width": 0.02
            }]
        }],
        "text": text
    });
    json!({
        "id": id,
        "content": serde_json::to_string(&content).unwrap_or_default(),
        "typesetting": 0,
        "alignment": 1,
        "letter_spacing": 0.0,
        "line_spacing": 0.02,
        "line_feed": 1,
        "line_max_width": 0.82,
        "force_apply_line_max_width": false,
        "check_flag": 15,
        "type": "subtitle",
        "global_alpha": 1.0
    })
}

fn video_segment(
    material_id: &str,
    source_start_us: i64,
    duration_us: i64,
    target_start_us: i64,
    volume: f64,
) -> Value {
    json!({
        "id": uid(),
        "material_id": material_id,
        "target_timerange": {"start": target_start_us, "duration": duration_us},
        "source_timerange": {"start": source_start_us, "duration": duration_us},
        "speed": 1.0,
        "volume": volume,
        "enable_adjust": true,
        "enable_color_curves": true,
        "enable_color_match_adjust": false,
        "enable_color_wheels": true,
        "enable_lut": true,
        "enable_smart_color_adjust": false,
        "extra_material_refs": [],
        "hdr_settings": {"intensity": 1.0, "mode": 1, "nits": 1000},
        "uniform_scale": {"on": true, "value": 1.0},
        "clip": {
            "alpha": 1.0,
            "flip": {"horizontal": false, "vertical": false},
            "rotation": 0.0,
            "scale": {"x": 1.0, "y": 1.0},
            "transform": {"x": 0.0, "y": 0.0}
        },
        "common_keyframes": [],
        "visible": true,
        "render_index": 0,
        "source": "segmentsourcenormal"
    })
}

fn audio_segment(material_id: &str, duration_us: i64, target_start_us: i64, volume: f64) -> Value {
    json!({
        "id": uid(),
        "material_id": material_id,
        "target_timerange": {"start": target_start_us, "duration": duration_us},
        "source_timerange": {"start": 0, "duration": duration_us},
        "speed": 1.0,
        "volume": volume,
        "extra_material_refs": [],
        "clip": null,
        "hdr_settings": null,
        "uniform_scale": null,
        "common_keyframes": [],
        "visible": true,
        "render_index": 0,
        "source": "segmentsourcenormal"
    })
}

fn text_segment(material_id: &str, start_us: i64, duration_us: i64) -> Value {
    json!({
        "id": uid(),
        "material_id": material_id,
        "target_timerange": {"start": start_us, "duration": duration_us},
        "source_timerange": null,
        "speed": 1.0,
        "volume": 1.0,
        "extra_material_refs": [],
        "is_tone_modify": false,
        "clip": {
            "alpha": 1.0,
            "flip": {"horizontal": false, "vertical": false},
            "rotation": 0.0,
            "scale": {"x": 1.0, "y": 1.0},
            "transform": {"x": 0.0, "y": -0.8}
        },
        "uniform_scale": {"on": true, "value": 1.0},
        "render_index": 15000,
        "common_keyframes": [],
        "visible": true,
        "source": "segmentsourcenormal"
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败：{e}"))?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(value).map_err(|e| format!("序列化失败：{e}"))?,
    )
    .map_err(|e| format!("写入 {} 失败：{e}", path.display()))
}

fn asset_entry(material: &Value, metetype: &str, width: u32, height: u32) -> Value {
    let duration = material.get("duration").and_then(|v| v.as_i64()).unwrap_or(0);
    json!({
        "ai_group_type": "",
        "create_time": -1,
        "duration": duration,
        "enter_from": 0,
        "extra_info": material.get("material_name").or_else(|| material.get("name")).cloned().unwrap_or(json!("")),
        "file_Path": material.get("path").cloned().unwrap_or(json!("")),
        "height": height,
        "id": material.get("id").cloned().unwrap_or(json!("")),
        "import_time": -1,
        "import_time_ms": -1,
        "item_source": 1,
        "md5": "",
        "metetype": metetype,
        "roughcut_time_range": {"duration": duration, "start": 0},
        "sub_time_range": {"duration": -1, "start": -1},
        "type": 0,
        "width": width
    })
}

fn meta_info(
    draft: &Value,
    name: &str,
    draft_id: &str,
    root: &Path,
    path: &Path,
    asset_size: u64,
) -> Value {
    let materials = draft.get("materials").cloned().unwrap_or(json!({}));
    let mut index = Vec::new();
    if let Some(videos) = materials.get("videos").and_then(|v| v.as_array()) {
        for video in videos {
            let width = video.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let height = video.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            index.push(asset_entry(video, "video", width, height));
        }
    }
    if let Some(audios) = materials.get("audios").and_then(|v| v.as_array()) {
        for audio in audios {
            index.push(asset_entry(audio, "music", 0, 0));
        }
    }
    json!({
        "cloud_draft_cover": false,
        "cloud_draft_sync": false,
        "cloud_package_completed_time": "",
        "draft_cloud_capcut_purchase_info": "",
        "draft_cloud_last_action_download": false,
        "draft_cloud_package_type": "",
        "draft_cloud_purchase_info": "",
        "draft_cloud_template_id": "",
        "draft_cloud_tutorial_info": "",
        "draft_cloud_videocut_purchase_info": "",
        "draft_cover": "draft_cover.jpg",
        "draft_deeplink_url": "",
        "draft_enterprise_info": {
            "draft_enterprise_extra": "",
            "draft_enterprise_id": "",
            "draft_enterprise_name": "",
            "enterprise_material": []
        },
        "draft_fold_path": path.to_string_lossy(),
        "draft_id": format_uuid(draft_id),
        "draft_is_ae_produce": false,
        "draft_is_ai_packaging_used": false,
        "draft_is_ai_shorts": false,
        "draft_is_ai_translate": false,
        "draft_is_article_video_draft": false,
        "draft_is_cloud_temp_draft": false,
        "draft_is_from_deeplink": "false",
        "draft_is_invisible": false,
        "draft_is_pippit_draft": false,
        "draft_is_web_article_video": false,
        "draft_materials": [
            {"type": 0, "value": index},
            {"type": 1, "value": []},
            {"type": 2, "value": []},
            {"type": 3, "value": []},
            {"type": 6, "value": []},
            {"type": 7, "value": []},
            {"type": 8, "value": []}
        ],
        "draft_materials_copied_info": [],
        "draft_name": name,
        "draft_need_rename_folder": false,
        "draft_new_version": "",
        "draft_removable_storage_device": "",
        "draft_root_path": root.to_string_lossy(),
        "draft_segment_extra_info": [],
        "draft_timeline_materials_size_": asset_size,
        "draft_type": "",
        "draft_web_article_video_enter_from": "",
        "tm_draft_cloud_completed": "",
        "tm_draft_cloud_entry_id": -1,
        "tm_draft_cloud_modified": 0,
        "tm_draft_cloud_parent_entry_id": -1,
        "tm_draft_cloud_space_id": -1,
        "tm_draft_cloud_user_id": -1,
        "tm_draft_create": draft.get("create_time").cloned().unwrap_or(json!(0)),
        "tm_draft_modified": draft.get("update_time").cloned().unwrap_or(json!(0)),
        "tm_draft_removed": 0,
        "tm_duration": draft.get("duration").cloned().unwrap_or(json!(0))
    })
}

fn root_entry(draft: &Value, name: &str, draft_id: &str, root: &Path, path: &Path, asset_size: u64) -> Value {
    json!({
        "cloud_draft_cover": false,
        "cloud_draft_sync": false,
        "draft_cloud_last_action_download": false,
        "draft_cloud_purchase_info": "",
        "draft_cloud_template_id": "",
        "draft_cloud_tutorial_info": "",
        "draft_cloud_videocut_purchase_info": "",
        "draft_cover": path.join("draft_cover.jpg").to_string_lossy(),
        "draft_fold_path": path.to_string_lossy(),
        "draft_id": format_uuid(draft_id),
        "draft_is_ai_shorts": false,
        "draft_is_cloud_temp_draft": false,
        "draft_is_invisible": false,
        "draft_is_web_article_video": false,
        "draft_json_file": path.join("draft_info.json").to_string_lossy(),
        "draft_name": name,
        "draft_new_version": "",
        "draft_root_path": root.to_string_lossy(),
        "draft_timeline_materials_size": asset_size,
        "draft_type": "",
        "draft_web_article_video_enter_from": "",
        "streaming_edit_draft_ready": true,
        "tm_draft_cloud_completed": "",
        "tm_draft_cloud_entry_id": -1,
        "tm_draft_cloud_modified": 0,
        "tm_draft_cloud_parent_entry_id": -1,
        "tm_draft_cloud_space_id": -1,
        "tm_draft_cloud_user_id": -1,
        "tm_draft_create": draft.get("create_time").cloned().unwrap_or(json!(0)),
        "tm_draft_modified": draft.get("update_time").cloned().unwrap_or(json!(0)),
        "tm_draft_removed": 0,
        "tm_duration": draft.get("duration").cloned().unwrap_or(json!(0))
    })
}

fn virtual_store(draft: &Value) -> Value {
    let mut ids = Vec::new();
    if let Some(materials) = draft.get("materials") {
        for key in ["videos", "audios"] {
            if let Some(items) = materials.get(key).and_then(|v| v.as_array()) {
                for item in items {
                    ids.push(json!({"child_id": item.get("id").cloned().unwrap_or(json!("")), "parent_id": ""}));
                }
            }
        }
    }
    json!({
        "draft_materials": [],
        "draft_virtual_store": [
            {"type": 0, "value": [{
                "creation_time": 0, "display_name": "", "filter_type": 0, "id": "",
                "import_time": 0, "import_time_us": 0, "sort_sub_type": 0,
                "sort_type": 0, "subdraft_filter_type": 0
            }]},
            {"type": 1, "value": ids},
            {"type": 2, "value": []}
        ]
    })
}

pub fn export_draft(project: &Project, root: &str, name: Option<String>) -> Result<DraftResult, String> {
    if project.items.is_empty() {
        return Err("没有分镜，无法导出草稿".into());
    }
    let first_video = project.videos.first().ok_or_else(|| "没有视频素材".to_string())?;
    let width = first_video.info.width.max(2);
    let height = first_video.info.height.max(2);

    let display_name = safe_name(
        &name
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| project.name.clone()),
        "幕语草稿",
    );

    let root_path = PathBuf::from(root);
    fs::create_dir_all(&root_path).map_err(|e| format!("创建草稿根目录失败：{e}"))?;
    let mut folder = display_name.clone();
    let mut counter = 2;
    while root_path.join(&folder).exists() {
        folder = format!("{display_name}_{counter}");
        counter += 1;
    }
    let draft_path = root_path.join(&folder);
    for dir in [
        "qr_upload",
        "matting",
        "common_attachment",
        "Resources/audioAlg",
        "Resources/digitalHuman",
        "Resources/restore_lut",
        "Resources/videoAlg",
        "subdraft",
        "adjust_mask",
        "assets/audio",
        "assets/video",
        "smart_crop",
    ] {
        fs::create_dir_all(draft_path.join(dir)).map_err(|e| format!("创建目录失败：{e}"))?;
    }

    let draft_id = uid();
    let mut draft = draft_template(&draft_id, &display_name, width, height);
    let mut video_track = track("video", "视频轨道");
    let mut original_track = track("audio", "原声");
    let mut audio_track = track("audio", "配音轨道");
    let mut text_track = track("text", "字幕轨道");

    let mut video_materials: HashMap<String, (String, Value)> = HashMap::new();
    let mut video_material_values: Vec<Value> = Vec::new();
    let mut audio_material_values: Vec<Value> = Vec::new();
    let mut text_material_values: Vec<Value> = Vec::new();
    let mut video_segments: Vec<Value> = Vec::new();
    let mut original_segments: Vec<Value> = Vec::new();
    let mut audio_segments: Vec<Value> = Vec::new();
    let mut text_segments: Vec<Value> = Vec::new();

    let mut cursor_us = 0i64;
    let mut warnings: Vec<String> = Vec::new();
    let mut asset_count = 0usize;

    for item in &project.items {
        let is_narration = item.ost == 0 || item.ost == 2;
        let duration = if is_narration {
            item.audio_duration
                .filter(|value| *value > 0.1)
                .ok_or_else(|| format!("镜头 #{} 缺少配音", item.id))?
        } else {
            slot_duration(&item.timestamp)
        };
        if duration < 0.2 {
            continue;
        }

        let source_start = parse_ts(item.timestamp.split('-').next().unwrap_or(""));
        let material_key = project
            .videos
            .first()
            .map(|v| v.path.clone())
            .unwrap_or_default();
        let (material_id, _) = if let Some(existing) = video_materials.get(&material_key) {
            (existing.0.clone(), ())
        } else {
            let id = uid();
            let source_name = Path::new(&material_key)
                .file_name()
                .map(|v| v.to_string_lossy().to_string())
                .unwrap_or_else(|| "source.mp4".into());
            let rel = format!("assets/video/{}", safe_name(&source_name, "source.mp4"));
            let dest = draft_path.join(&rel);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::copy(&material_key, &dest).map_err(|e| format!("复制视频素材失败：{e}"))?;
            asset_count += 1;
            let duration_sec = first_video.info.duration;
            let value = video_material(&id, &rel, duration_sec, width, height);
            video_material_values.push(value.clone());
            video_materials.insert(material_key.clone(), (id.clone(), value));
            (id, ())
        };

        video_segments.push(video_segment(
            &material_id,
            us(source_start),
            us(duration),
            cursor_us,
            0.0,
        ));

        if item.ost == 1 || item.ost == 2 {
            let original_id = uid();
            let rel = format!("assets/audio/original_{:03}.wav", item.id);
            let dest = draft_path.join(&rel);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).ok();
            }
            let ss = format!("{source_start:.3}");
            let dur = format!("{duration:.3}");
            let output = Command::new(media::ffmpeg_bin())
                .args([
                    "-y",
                    "-ss",
                    &ss,
                    "-t",
                    &dur,
                    "-i",
                    &material_key,
                    "-map",
                    "0:a:0",
                    "-vn",
                    "-c:a",
                    "pcm_s16le",
                    "-ar",
                    "48000",
                    "-ac",
                    "2",
                ])
                .arg(&dest)
                .output()
                .map_err(|e| format!("提取原声失败：{e}"))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let tail: String = stderr.chars().rev().take(400).collect::<String>().chars().rev().collect();
                return Err(format!("镜头 #{} 原声提取失败：{tail}", item.id));
            }
            asset_count += 1;
            audio_material_values.push(audio_material(&original_id, &rel, duration));
            original_segments.push(audio_segment(&original_id, us(duration), cursor_us, 1.0));
        }

        if is_narration {
            let audio_path = item
                .audio_path
                .clone()
                .ok_or_else(|| format!("镜头 #{} 缺少配音文件", item.id))?;
            let audio_id = uid();
            let rel = format!("assets/audio/audio_{:03}.mp3", item.id);
            let dest = draft_path.join(&rel);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::copy(&audio_path, &dest).map_err(|e| format!("复制配音失败：{e}"))?;
            asset_count += 1;
            audio_material_values.push(audio_material(&audio_id, &rel, duration));
            audio_segments.push(audio_segment(&audio_id, us(duration), cursor_us, 1.0));

            let text_id = uid();
            text_material_values.push(text_material(&text_id, &item.narration));
            text_segments.push(text_segment(&text_id, cursor_us, us(duration)));
        }

        cursor_us += us(duration);
    }

    if video_segments.is_empty() {
        return Err("没有可写入草稿的镜头".into());
    }

    *video_track
        .get_mut("segments")
        .and_then(|v| v.as_array_mut())
        .unwrap() = video_segments;
    let has_original = !original_segments.is_empty();
    let has_audio = !audio_segments.is_empty();
    let has_text = !text_segments.is_empty();
    *original_track
        .get_mut("segments")
        .and_then(|v| v.as_array_mut())
        .unwrap() = original_segments;
    *audio_track
        .get_mut("segments")
        .and_then(|v| v.as_array_mut())
        .unwrap() = audio_segments;
    *text_track
        .get_mut("segments")
        .and_then(|v| v.as_array_mut())
        .unwrap() = text_segments;

    let mut bgm_track = track("audio", "BGM");
    bgm_track["is_default_name"] = json!(false);
    let mut has_bgm = false;
    if let Some(bgm) = project
        .bgm_path
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty() && Path::new(value).exists())
    {
        let extension = Path::new(bgm)
            .extension()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "mp3".into());
        let rel = format!("assets/audio/bgm.{extension}");
        let dest = draft_path.join(&rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::copy(bgm, &dest).map_err(|e| format!("复制 BGM 失败：{e}"))?;
        asset_count += 1;
        let bgm_duration = media::probe_duration(bgm).unwrap_or(0.0);
        if bgm_duration > 0.1 {
            let bgm_id = uid();
            audio_material_values.push(audio_material(&bgm_id, &rel, bgm_duration));
            let volume = project.bgm_volume.clamp(0.0, 1.0);
            let mut cursor = 0i64;
            while cursor < cursor_us {
                let segment = us(bgm_duration).min(cursor_us - cursor).max(1);
                bgm_track["segments"]
                    .as_array_mut()
                    .unwrap()
                    .push(audio_segment(&bgm_id, segment, cursor, volume as f64));
                cursor += segment;
            }
            has_bgm = !bgm_track["segments"].as_array().map(|v| v.is_empty()).unwrap_or(true);
        }
    }

    let mut tracks = vec![video_track];
    if has_original {
        original_track["is_default_name"] = json!(false);
        tracks.push(original_track);
    }
    if has_audio {
        tracks.push(audio_track);
    }
    if has_text {
        text_track["is_default_name"] = json!(false);
        tracks.push(text_track);
    }
    if has_bgm {
        tracks.push(bgm_track);
    }

    if let Some(materials) = draft.get_mut("materials").and_then(|v| v.as_object_mut()) {
        materials.insert("videos".into(), json!(video_material_values));
        materials.insert("audios".into(), json!(audio_material_values));
        materials.insert("texts".into(), json!(text_material_values));
    }
    draft["tracks"] = json!(tracks);
    draft["duration"] = json!(cursor_us);
    draft["update_time"] = json!(now_us());
    draft["canvas_config"]["width"] = json!(width);
    draft["canvas_config"]["height"] = json!(height);

    let mut draft_info = draft.clone();
    if let Some(object) = draft_info.as_object_mut() {
        object.insert("draft_type".into(), json!("video"));
        object.insert(
            "function_assistant_info".into(),
            json!({
                "auto_adjust": false, "auto_caption": false, "color_correction": false,
                "enhance_quality": false, "fps": {"den": 1, "num": 0},
                "normalize_loudness": false, "retouch": false,
                "smart_rec_applied": false, "smooth_slow_motion": false
            }),
        );
        object.insert("is_drop_frame_timecode".into(), json!(false));
        object.insert("path".into(), json!(""));
        object.insert("platform".into(), platform_info());
        object.insert("render_index_track_mode_on".into(), json!(true));
        object.insert(
            "smart_ads_info".into(),
            json!({"draft_url": "", "page_from": "", "routine": ""}),
        );
        object.insert(
            "uneven_animation_template_info".into(),
            json!({"composition": "", "content": "", "order": "", "sub_template_info_list": []}),
        );
    }

    let mut empty = draft.clone();
    empty["canvas_config"] = json!({"background": null, "height": 0, "ratio": "original", "width": 0});
    empty["color_space"] = json!(-1);
    empty["duration"] = json!(0);
    empty["materials"] = json!(create_materials());
    empty["tracks"] = json!([]);
    if let Some(object) = empty.as_object_mut() {
        object.insert("new_version".into(), json!("75.0.0"));
        object.insert("draft_type".into(), json!("video"));
        object.insert("path".into(), json!(""));
        object.insert("platform".into(), platform_info());
        object.insert("function_assistant_info".into(), json!({}));
        object.insert("is_drop_frame_timecode".into(), json!(false));
        object.insert("lyrics_effects".into(), json!([]));
        object.insert("smart_ads_info".into(), json!({"draft_url": "", "page_from": "", "routine": ""}));
        object.insert(
            "uneven_animation_template_info".into(),
            json!({"composition": "", "content": "", "order": "", "sub_template_info_list": []}),
        );
    }

    write_json(&draft_path.join("draft_info.json"), &draft_info)?;
    write_json(&draft_path.join("template-2.tmp"), &draft_info)?;
    write_json(&draft_path.join("template.tmp"), &empty)?;
    write_json(
        &draft_path.join("draft_meta_info.json"),
        &meta_info(&draft, &display_name, &draft_id, &root_path, &draft_path, 0),
    )?;
    fs::write(
        draft_path.join("draft_settings"),
        format!(
            "[General]\ncloud_last_modify_platform=windows\ndraft_create_time={}\ndraft_last_edit_time={}\nreal_edit_keys=1\nreal_edit_seconds=0\n\n",
            draft["create_time"].as_i64().unwrap_or(0) / 1_000_000,
            draft["update_time"].as_i64().unwrap_or(0) / 1_000_000
        ),
    )
    .map_err(|e| format!("写入 draft_settings 失败：{e}"))?;

    if let Some(cover) = project.cover_path.as_ref().filter(|p| Path::new(p).exists()) {
        let _ = fs::copy(cover, draft_path.join("draft_cover.jpg"));
    }
    write_json(
        &draft_path.join("common_attachment/attachment_pc_timeline.json"),
        &json!({
            "reference_lines_config": {
                "horizontal_lines": [], "is_lock": false,
                "is_visible": false, "vertical_lines": []
            },
            "safe_area_type": 0
        }),
    )?;
    for name in [
        "attachment_action_scene.json",
        "attachment_script_video.json",
        "attachment_gen_ai_info.json",
    ] {
        write_json(&draft_path.join("common_attachment").join(name), &json!({}))?;
    }
    write_json(
        &draft_path.join("attachment_editing.json"),
        &json!({
            "editing_draft": {
                "ai_remove_filter_words": {"enter_source": "", "right_id": ""},
                "ai_shorts_info": {"report_params": "", "type": 0},
                "edit_type": 0,
                "is_use_ai_shorts": false,
                "is_use_adjust": false,
                "is_use_ai_expand": false,
                "is_use_ai_remove": false,
                "is_use_ai_video": false,
                "is_use_loudness_unify": false,
                "is_use_subtitle_recognition": false,
                "version": "1.0.0"
            }
        }),
    )?;
    write_json(
        &draft_path.join("draft_agency_config.json"),
        &json!({
            "is_auto_agency_enabled": false,
            "is_auto_agency_popup": false,
            "is_single_agency_mode": false,
            "marterials": null,
            "use_converter": false,
            "video_resolution": height
        }),
    )?;
    fs::write(draft_path.join("draft_biz_config.json"), "").ok();
    write_json(
        &draft_path.join("draft_virtual_store.json"),
        &virtual_store(&draft),
    )?;
    write_json(
        &draft_path.join("performance_opt_info.json"),
        &json!({"manual_cancle_precombine_segs": null, "need_auto_precombine_segs": null}),
    )?;
    write_json(
        &draft_path.join("timeline_layout.json"),
        &json!({
            "activeTimeline": draft_id,
            "dockItems": [{"dockIndex": 0, "ratio": 1, "timelineIds": [draft_id], "timelineNames": ["时间线01"]}],
            "layoutOrientation": 1
        }),
    )?;

    // 素材总大小 + 根索引
    let mut asset_size: u64 = 0;
    for dir in ["assets/video", "assets/audio"] {
        if let Ok(entries) = fs::read_dir(draft_path.join(dir)) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    asset_size += meta.len();
                }
            }
        }
    }
    write_json(
        &draft_path.join("draft_meta_info.json"),
        &meta_info(&draft, &display_name, &draft_id, &root_path, &draft_path, asset_size),
    )?;

    let root_meta_path = root_path.join("root_meta_info.json");
    let mut root_meta: Value = fs::read_to_string(&root_meta_path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({}));
    let entry = root_entry(&draft, &display_name, &draft_id, &root_path, &draft_path, asset_size);
    let existing = root_meta
        .get("all_draft_store")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let filtered: Vec<Value> = existing
        .into_iter()
        .filter(|item| {
            item.get("draft_id") != entry.get("draft_id")
                && item.get("draft_fold_path") != entry.get("draft_fold_path")
                && item.get("draft_name") != entry.get("draft_name")
        })
        .collect();
    let mut store = vec![entry];
    store.extend(filtered);
    root_meta["all_draft_store"] = json!(store);
    root_meta["draft_ids"] = json!(root_meta.get("draft_ids").and_then(|v| v.as_i64()).unwrap_or(1));
    root_meta["root_path"] = json!(root.to_string());
    write_json(&root_meta_path, &root_meta)?;

    if !jianying_exists() {
        warnings.push("未检测到本机安装剪映；草稿按 10.6.0 / 169.0.0 结构生成，请用剪映打开验证".into());
    }

    Ok(DraftResult {
        path: draft_path.to_string_lossy().to_string(),
        name: display_name,
        duration: cursor_us as f64 / 1_000_000.0,
        segments: project.items.len(),
        tracks: 1
            + usize::from(has_original)
            + usize::from(has_audio)
            + usize::from(has_text)
            + usize::from(has_bgm),
        assets: asset_count,
        warnings,
    })
}

fn jianying_exists() -> bool {
    let mut candidates: Vec<PathBuf> = Vec::new();

    #[cfg(windows)]
    {
        use winreg::enums::{
            HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
        };
        use winreg::RegKey;

        let uninstall_paths = [
            "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ];
        for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
            for flags in [KEY_READ | KEY_WOW64_64KEY, KEY_READ | KEY_WOW64_32KEY] {
                for uninstall_path in uninstall_paths {
                    let Ok(root) = RegKey::predef(hive).open_subkey_with_flags(uninstall_path, flags)
                    else {
                        continue;
                    };
                    for name in root.enum_keys().flatten() {
                        let Ok(item) = root.open_subkey_with_flags(&name, flags) else {
                            continue;
                        };
                        let display: String = item.get_value("DisplayName").unwrap_or_default();
                        if !display.contains("剪映") && !display.to_lowercase().contains("jianying") {
                            continue;
                        }
                        for value_name in ["InstallLocation", "DisplayIcon", "UninstallString"] {
                            let raw: String = item.get_value(value_name).unwrap_or_default();
                            push_jianying_candidates(&mut candidates, &raw);
                        }
                    }
                }
            }
        }

        for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
            for flags in [KEY_READ | KEY_WOW64_64KEY, KEY_READ | KEY_WOW64_32KEY] {
                let path = "Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\JianyingPro.exe";
                if let Ok(key) = RegKey::predef(hive).open_subkey_with_flags(path, flags) {
                    let raw: String = key.get_value("").unwrap_or_default();
                    push_jianying_candidates(&mut candidates, &raw);
                }
            }
        }
    }

    // 便携版/未写注册表的安装：只查系统登记的 LocalAppData，不扫描盘符。
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let base = Path::new(&local).join("JianyingPro");
        push_jianying_candidates(&mut candidates, &base.to_string_lossy());
    }

    candidates.iter().any(|path| path.exists())
}

fn push_jianying_candidates(candidates: &mut Vec<PathBuf>, raw: &str) {
    let raw = raw.trim().trim_matches('"');
    if raw.is_empty() {
        return;
    }
    let path = Path::new(raw);
    if path.is_file() {
        candidates.push(path.to_path_buf());
    }
    let parent = if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    };
    if let Some(parent) = parent {
        candidates.push(parent.join("JianyingPro.exe"));
        for sub in ["Apps", "JianyingPro"] {
            if let Ok(entries) = fs::read_dir(parent.join(sub)) {
                for entry in entries.flatten() {
                    let candidate = entry.path();
                    if candidate.file_name().and_then(|n| n.to_str()) == Some("JianyingPro.exe") {
                        candidates.push(candidate);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod detection_tests {
    #[test]
    #[ignore]
    fn detects_jianying_from_registry() {
        assert!(super::jianying_exists(), "JianyingPro should be discovered from registry/App Paths");
    }
}
