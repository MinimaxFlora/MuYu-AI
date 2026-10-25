use std::fs;
use std::path::{Path, PathBuf};

use crate::model::SubtitleCue;

fn decode(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&bytes[3..]).into_owned();
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let (text, _, _) = encoding_rs::UTF_16LE.decode(&bytes[2..]);
        return text.into_owned();
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let (text, _, _) = encoding_rs::UTF_16BE.decode(&bytes[2..]);
        return text.into_owned();
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => {
            let (text, _, _) = encoding_rs::GBK.decode(bytes);
            text.into_owned()
        }
    }
}

fn strip_tags(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_angle = false;
    let mut in_brace = false;
    for ch in line.chars() {
        match ch {
            '<' => {
                in_angle = true;
                continue;
            }
            '>' => {
                in_angle = false;
                continue;
            }
            '{' => {
                in_brace = true;
                continue;
            }
            '}' => {
                in_brace = false;
                continue;
            }
            _ => {}
        }
        if !in_angle && !in_brace {
            out.push(ch);
        }
    }
    out
}

fn parse_time(raw: &str) -> Option<f64> {
    let cleaned = raw
        .trim()
        .split_whitespace()
        .next()
        .unwrap_or("")
        .replace(',', ".");
    if cleaned.is_empty() {
        return None;
    }
    let parts: Vec<&str> = cleaned.split(':').collect();
    let (h, m, s) = match parts.len() {
        3 => (
            parts[0].parse::<f64>().ok()?,
            parts[1].parse::<f64>().ok()?,
            parts[2].parse::<f64>().ok()?,
        ),
        2 => (
            0.0,
            parts[0].parse::<f64>().ok()?,
            parts[1].parse::<f64>().ok()?,
        ),
        _ => return None,
    };
    Some(h * 3600.0 + m * 60.0 + s)
}

fn parse_time_range(line: &str) -> Option<(f64, f64)> {
    let (left, right) = line.split_once("-->")?;
    Some((parse_time(left)?, parse_time(right)?))
}

pub fn parse_text(text: &str) -> Vec<SubtitleCue> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut cues = Vec::new();

    for block in normalized.split("\n\n") {
        let block = block.trim_matches('\n');
        if block.trim().is_empty() {
            continue;
        }
        let lines: Vec<&str> = block.lines().collect();
        let Some(arrow_index) = lines.iter().position(|line| line.contains("-->")) else {
            continue;
        };
        let Some((start, end)) = parse_time_range(lines[arrow_index]) else {
            continue;
        };

        let text_lines: Vec<String> = lines[arrow_index + 1..]
            .iter()
            .map(|line| strip_tags(line).trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        if text_lines.is_empty() {
            continue;
        }

        let index = if arrow_index > 0 {
            lines[0].trim().parse::<u32>().unwrap_or(cues.len() as u32 + 1)
        } else {
            cues.len() as u32 + 1
        };

        cues.push(SubtitleCue {
            index,
            start,
            end,
            text: text_lines.join("\n"),
        });
    }

    cues.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
    cues
}

pub fn read_and_parse(path: &str) -> Result<Vec<SubtitleCue>, String> {
    let bytes = fs::read(path).map_err(|e| format!("读取字幕失败：{e}"))?;
    let text = decode(&bytes);
    let cues = parse_text(&text);
    if cues.is_empty() {
        return Err("字幕文件里没有解析到有效条目".to_string());
    }
    Ok(cues)
}

pub fn find_sibling_subtitle(video_path: &str) -> Option<String> {    let path = Path::new(video_path);
    let stem = path.file_stem()?.to_string_lossy().to_lowercase();
    let dir = path.parent()?;
    let entries = fs::read_dir(dir).ok()?;

    for entry in entries.flatten() {
        let candidate = entry.path();
        if !candidate.is_file() {
            continue;
        }
        let ext = candidate
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if ext != "srt" {
            continue;
        }
        let candidate_stem = candidate
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if candidate_stem == stem {
            return Some(candidate.to_string_lossy().to_string());
        }
    }

    let direct = PathBuf::from(format!("{}.srt", path.with_extension("").to_string_lossy()));
    if direct.exists() {
        return Some(direct.to_string_lossy().to_string());
    }
    None
}
