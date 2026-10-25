use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LlmSettings {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub demo_mode: bool,
    pub update_repo: String,
    pub auto_check_update: bool,
}

impl Default for LlmSettings {
    fn default() -> Self {
        Self {
            base_url: "https://api.deepseek.com/v1".into(),
            api_key: String::new(),
            model: "deepseek-chat".into(),
            temperature: 0.8,
            demo_mode: false,
            update_repo: "MinimaxFlora/MuYu-AI".into(),
            auto_check_update: true,
        }
    }
}

impl LlmSettings {
    pub fn is_demo(&self) -> bool {
        self.demo_mode || self.api_key.trim().is_empty()
    }
}

pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(app: &tauri::AppHandle) -> Result<Self, String> {
        let base = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("无法定位应用数据目录：{e}"))?;
        fs::create_dir_all(&base).map_err(|e| format!("无法创建应用数据目录：{e}"))?;
        Ok(Self {
            path: base.join("settings.json"),
        })
    }

    pub fn load(&self) -> Result<LlmSettings, String> {
        if !self.path.exists() {
            return Ok(LlmSettings::default());
        }
        let text = fs::read_to_string(&self.path).map_err(|e| format!("读取设置失败：{e}"))?;
        serde_json::from_str(&text).map_err(|e| format!("解析设置失败：{e}"))
    }

    pub fn save(&self, settings: &LlmSettings) -> Result<(), String> {
        let tmp = self.path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("序列化设置失败：{e}"))?;
        fs::write(&tmp, json).map_err(|e| format!("写入设置失败：{e}"))?;
        fs::rename(&tmp, &self.path).map_err(|e| format!("保存设置失败：{e}"))
    }
}
