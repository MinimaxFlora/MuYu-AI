use std::fs;
use std::path::PathBuf;

use chrono::{SecondsFormat, Utc};
use tauri::Manager;

use crate::model::{Project, ProjectSummary};

pub struct Store {
    projects_dir: PathBuf,
}

impl Store {
    pub fn from_projects_dir(projects_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&projects_dir).map_err(|e| format!("无法创建项目目录：{e}"))?;
        Ok(Self { projects_dir })
    }

    pub fn new(app: &tauri::AppHandle) -> Result<Self, String> {
        let base = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("无法定位应用数据目录：{e}"))?;
        Self::from_projects_dir(base.join("projects"))
    }

    pub fn project_dir(&self, id: &str) -> PathBuf {
        self.projects_dir.join(id)
    }

    pub fn project_file(&self, id: &str) -> PathBuf {
        self.project_dir(id).join("project.json")
    }

    pub fn create_dir(&self, id: &str) -> Result<PathBuf, String> {
        let dir = self.project_dir(id);
        fs::create_dir_all(&dir).map_err(|e| format!("无法创建项目目录：{e}"))?;
        Ok(dir)
    }

    pub fn save(&self, project: &Project) -> Result<(), String> {
        let dir = self.project_dir(&project.id);
        fs::create_dir_all(&dir).map_err(|e| format!("无法创建项目目录：{e}"))?;
        let path = dir.join("project.json");
        let tmp = dir.join("project.json.tmp");
        let json = serde_json::to_string_pretty(project).map_err(|e| format!("序列化项目失败：{e}"))?;
        fs::write(&tmp, json).map_err(|e| format!("写入项目失败：{e}"))?;
        if let Err(err) = fs::rename(&tmp, &path) {
            fs::copy(&tmp, &path).map_err(|e| format!("保存项目失败：{err} / {e}"))?;
            let _ = fs::remove_file(&tmp);
        }
        Ok(())
    }

    pub fn load(&self, id: &str) -> Result<Project, String> {
        let path = self.project_file(id);
        let text = fs::read_to_string(&path).map_err(|e| format!("读取项目失败：{e}"))?;
        serde_json::from_str(&text).map_err(|e| format!("解析项目失败：{e}"))
    }

    pub fn list(&self) -> Vec<ProjectSummary> {
        let mut items = Vec::new();
        let Ok(entries) = fs::read_dir(&self.projects_dir) else {
            return items;
        };
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let path = entry.path().join("project.json");
            let Ok(text) = fs::read_to_string(path) else {
                continue;
            };
            if let Ok(project) = serde_json::from_str::<Project>(&text) {
                items.push(ProjectSummary::from(&project));
            }
        }
        items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        items
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        let dir = self.project_dir(id);
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|e| format!("删除项目失败：{e}"))?;
        }
        Ok(())
    }

    pub fn touch(project: &mut Project) {
        project.updated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    }
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}
