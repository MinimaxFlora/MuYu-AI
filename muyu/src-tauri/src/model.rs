use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct MediaInfo {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub codec: String,
    pub bit_rate: u64,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoAsset {
    pub path: String,
    pub name: String,
    pub info: MediaInfo,
    pub trim_start: f64,
    pub trim_end: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleCue {
    pub index: u32,
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub videos: Vec<VideoAsset>,
    pub subtitle_path: Option<String>,
    pub subtitles: Vec<SubtitleCue>,
    pub cover_path: Option<String>,
    pub plot_analysis: Option<String>,
    pub narration_copy: Option<String>,
    pub items: Vec<ScriptItem>,
    pub params: ScriptParams,
    pub pipeline_note: Option<String>,
    pub voice: String,
    pub tts_engine: String,
    pub voice_rate: f32,
    pub tts_note: Option<String>,
    pub last_render: Option<crate::render::RenderResult>,
    pub last_draft: Option<crate::jianying::DraftResult>,
    pub bgm_path: Option<String>,
    pub bgm_volume: f32,
    pub bgm_ducking: bool,
    pub bgm_fade: bool,
    pub ducking_strength: String,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            mode: "shortline".into(),
            status: "draft".into(),
            created_at: String::new(),
            updated_at: String::new(),
            videos: Vec::new(),
            subtitle_path: None,
            subtitles: Vec::new(),
            cover_path: None,
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScriptParams {
    pub genre: String,
    pub language: String,
    pub target_words: u32,
    pub density: String,
    pub original_ratio: u32,
}

impl Default for ScriptParams {
    fn default() -> Self {
        Self {
            genre: "悬疑 / 犯罪".into(),
            language: "中文（简体）".into(),
            target_words: 600,
            density: "standard".into(),
            original_ratio: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptItem {
    #[serde(rename = "_id")]
    pub id: u32,
    pub video_id: u32,
    pub video_name: String,
    pub timestamp: String,
    pub picture: String,
    pub narration: String,
    #[serde(rename = "OST")]
    pub ost: u8,
    pub audio_path: Option<String>,
    pub audio_duration: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub status: String,
    pub updated_at: String,
    pub cover_path: Option<String>,
    pub duration: f64,
    pub video_count: usize,
    pub subtitle_count: usize,
}

impl From<&Project> for ProjectSummary {
    fn from(p: &Project) -> Self {
        let duration = p
            .videos
            .iter()
            .map(|v| (v.trim_end - v.trim_start).max(0.0))
            .sum::<f64>();
        Self {
            id: p.id.clone(),
            name: p.name.clone(),
            mode: p.mode.clone(),
            status: p.status.clone(),
            updated_at: p.updated_at.clone(),
            cover_path: p.cover_path.clone(),
            duration,
            video_count: p.videos.len(),
            subtitle_count: p.subtitles.len(),
        }
    }
}
