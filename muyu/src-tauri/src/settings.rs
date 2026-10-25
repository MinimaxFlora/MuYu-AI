use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::Manager;

const KEYRING_SERVICE: &str = "com.muyu.studio";
const KEYRING_USER: &str = "llm-api-key";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LlmSettings {
    pub base_url: String,
    /// Runtime-only copy. It is never written to settings.json.
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

fn keyring_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("无法打开 Windows 凭据管理器：{e}"))
}

fn read_keyring() -> Result<Option<String>, String> {
    match keyring_entry()?.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("读取 Windows 凭据失败：{e}")),
    }
}

fn write_keyring(api_key: &str) -> Result<(), String> {
    let entry = keyring_entry()?;
    if api_key.trim().is_empty() {
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("删除 Windows 凭据失败：{e}")),
        }
    } else {
        entry
            .set_password(api_key)
            .map_err(|e| format!("写入 Windows 凭据失败：{e}"))
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
        let mut settings = if self.path.exists() {
            let text =
                fs::read_to_string(&self.path).map_err(|e| format!("读取设置失败：{e}"))?;
            serde_json::from_str(&text).map_err(|e| format!("解析设置失败：{e}"))?
        } else {
            LlmSettings::default()
        };

        // 旧版本把 apiKey 明文写在 settings.json。首次读取时迁移到凭据管理器，
        // 然后把明文从文件里移除。
        let legacy_key = settings.api_key.trim().to_string();
        match read_keyring()? {
            Some(key) => settings.api_key = key,
            None if !legacy_key.is_empty() => {
                write_keyring(&legacy_key)?;
                settings.api_key = legacy_key;
                self.save(&settings)?;
            }
            None => settings.api_key.clear(),
        }
        Ok(settings)
    }

    pub fn save(&self, settings: &LlmSettings) -> Result<(), String> {
        write_keyring(&settings.api_key)?;

        let mut value = serde_json::to_value(settings)
            .map_err(|e| format!("序列化设置失败：{e}"))?;
        if let Some(object) = value.as_object_mut() {
            object.remove("apiKey");
        }
        let json = serde_json::to_string_pretty(&value)
            .map_err(|e| format!("序列化设置失败：{e}"))?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, json).map_err(|e| format!("写入设置失败：{e}"))?;
        fs::rename(&tmp, &self.path).map_err(|e| format!("保存设置失败：{e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_is_not_written_to_json() {
        let settings = LlmSettings {
            api_key: "sk-not-in-file".into(),
            ..LlmSettings::default()
        };
        let mut value = serde_json::to_value(&settings).unwrap();
        value.as_object_mut().unwrap().remove("apiKey");
        assert!(!value.to_string().contains("sk-not-in-file"));
    }

    #[test]
    #[ignore]
    fn windows_credential_roundtrip() {
        let entry = keyring::Entry::new("com.muyu.studio.test", "llm-api-key").unwrap();
        entry.set_password("sk-test-roundtrip").unwrap();
        assert_eq!(entry.get_password().unwrap(), "sk-test-roundtrip");
        entry.delete_credential().unwrap();
    }
}
