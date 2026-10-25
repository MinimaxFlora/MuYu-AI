use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

use crate::settings::SettingsStore;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current_version: String,
    pub version: String,
    pub body: Option<String>,
    pub date: Option<String>,
    pub repo: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: f64,
    pub message: String,
}

fn endpoint(repo: &str) -> Result<tauri::Url, String> {
    let repo = repo
        .trim()
        .trim_start_matches("https://github.com/")
        .trim_end_matches('/');
    if repo.is_empty() || !repo.contains('/') {
        return Err("请先填写更新仓库，例如 muyu-studio/MuYu".into());
    }
    tauri::Url::parse(&format!(
        "https://github.com/{repo}/releases/latest/download/latest.json"
    ))
    .map_err(|e| format!("更新地址无效：{e}"))
}

async fn check_with(app: &AppHandle, repo: &str) -> Result<Option<UpdateInfo>, String> {
    let url = endpoint(repo)?;
    let updater = app
        .updater_builder()
        .endpoints(vec![url])
        .map_err(|e| format!("创建更新检查失败：{e}"))?
        .build()
        .map_err(|e| format!("创建更新器失败：{e}"))?;
    let update = updater.check().await.map_err(|e| format!("检查更新失败：{e}"))?;
    Ok(update.map(|update| UpdateInfo {
        current_version: update.current_version,
        version: update.version,
        body: update.body,
        date: update.date.map(|date| date.to_string()),
        repo: repo.to_string(),
    }))
}

pub async fn check_auto(app: &AppHandle) -> Result<Option<UpdateInfo>, String> {
    let settings = SettingsStore::new(app)?.load()?;
    if !settings.auto_check_update || settings.update_repo.trim().is_empty() {
        return Ok(None);
    }
    check_with(app, &settings.update_repo).await
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let settings = SettingsStore::new(&app)?.load()?;
    check_with(&app, &settings.update_repo).await
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let settings = SettingsStore::new(&app)?.load()?;
    let url = endpoint(&settings.update_repo)?;
    let updater = app
        .updater_builder()
        .endpoints(vec![url])
        .map_err(|e| format!("创建更新检查失败：{e}"))?
        .build()
        .map_err(|e| format!("创建更新器失败：{e}"))?;
    let update = updater
        .check()
        .await
        .map_err(|e| format!("检查更新失败：{e}"))?
        .ok_or_else(|| "当前已经是最新版本".to_string())?;

    let version = update.version.clone();
    let downloaded = AtomicU64::new(0);
    let handle = app.clone();
    update
        .download_and_install(
            move |chunk_length, content_length| {
                let current = downloaded.fetch_add(chunk_length as u64, Ordering::Relaxed)
                    + chunk_length as u64;
                let percent = content_length
                    .map(|total| (current as f64 / total.max(1) as f64 * 100.0).min(99.0))
                    .unwrap_or(0.0);
                let _ = handle.emit(
                    "updater://progress",
                    UpdateProgress {
                        downloaded: current,
                        total: content_length,
                        percent,
                        message: format!("正在下载 v{version}"),
                    },
                );
            },
            || {},
        )
        .await
        .map_err(|e| format!("下载或安装更新失败：{e}"))?;

    let _ = app.emit(
        "updater://progress",
        UpdateProgress {
            downloaded: 0,
            total: None,
            percent: 100.0,
            message: "安装程序已启动".into(),
        },
    );
    Ok(())
}
