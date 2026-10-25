use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;

use crate::model::MediaInfo;

pub fn ffmpeg_bin() -> String {
    std::env::var("MUYU_FFMPEG").unwrap_or_else(|_| "ffmpeg".to_string())
}

fn ffprobe_bin() -> String {
    std::env::var("MUYU_FFPROBE").unwrap_or_else(|_| "ffprobe".to_string())
}

fn run(mut cmd: Command) -> Result<Output, String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.output().map_err(|e| {
        format!(
            "无法执行 {}（请确认已安装 ffmpeg 并加入 PATH）：{e}",
            cmd.get_program().to_string_lossy()
        )
    })
}

fn parse_fraction(raw: &str) -> f64 {
    let raw = raw.trim();
    if raw.is_empty() {
        return 0.0;
    }
    if let Some((a, b)) = raw.split_once('/') {
        let a: f64 = a.parse().unwrap_or(0.0);
        let b: f64 = b.parse().unwrap_or(1.0);
        if b > 0.0 {
            a / b
        } else {
            0.0
        }
    } else {
        raw.parse().unwrap_or(0.0)
    }
}

fn as_f64(value: &Value) -> f64 {
    match value {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn as_u64(value: &Value) -> u64 {
    match value {
        Value::Number(n) => n.as_u64().unwrap_or(0),
        Value::String(s) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

pub fn probe(path: &str) -> Result<MediaInfo, String> {
    let output = run({
        let mut cmd = Command::new(ffprobe_bin());
        cmd.args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            path,
        ]);
        cmd
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("无法读取视频信息：{}", stderr.trim()));
    }

    let value: Value =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("解析视频信息失败：{e}"))?;

    let streams = value["streams"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let video = streams
        .iter()
        .find(|s| s["codec_type"].as_str() == Some("video"))
        .ok_or_else(|| "该文件不包含视频轨道".to_string())?;

    let format = &value["format"];
    let mut duration = as_f64(&format["duration"]);
    if duration <= 0.0 {
        duration = as_f64(&video["duration"]);
    }

    let mut fps = parse_fraction(video["avg_frame_rate"].as_str().unwrap_or(""));
    if fps <= 0.0 {
        fps = parse_fraction(video["r_frame_rate"].as_str().unwrap_or(""));
    }

    let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    Ok(MediaInfo {
        duration,
        width: as_u64(&video["width"]) as u32,
        height: as_u64(&video["height"]) as u32,
        fps,
        codec: video["codec_name"].as_str().unwrap_or("unknown").to_string(),
        bit_rate: {
            let br = as_u64(&format["bit_rate"]);
            if br > 0 {
                br
            } else {
                as_u64(&video["bit_rate"])
            }
        },
        size,
    })
}

pub fn probe_duration(path: &str) -> Result<f64, String> {
    let output = run({
        let mut cmd = Command::new(ffprobe_bin());
        cmd.args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path,
        ]);
        cmd
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("无法读取音频时长：{}", stderr.trim()));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.trim()
        .parse::<f64>()
        .map_err(|e| format!("解析音频时长失败：{e}（{text}）"))
}

pub fn extract_cover(video: &str, output: &Path, at_seconds: f64) -> Result<(), String> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("无法创建封面目录：{e}"))?;
    }
    let _ = fs::remove_file(output);

    let at = if at_seconds.is_finite() && at_seconds > 0.0 {
        at_seconds
    } else {
        1.0
    };

    let attempt = |seek: f64| -> Result<(), String> {
        let output = run({
            let mut cmd = Command::new(ffmpeg_bin());
            cmd.args([
                "-y",
                "-ss",
                &format!("{seek:.3}"),
                "-i",
                video,
                "-frames:v",
                "1",
                "-vf",
                "scale='min(960,iw)':-2",
                "-q:v",
                "3",
                output.to_string_lossy().as_ref(),
            ]);
            cmd
        })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    };

    match attempt(at) {
        Ok(()) => Ok(()),
        Err(_) => attempt(0.0).map_err(|e| format!("生成封面失败：{e}")),
    }
}

pub fn environment_status() -> (bool, bool, String) {
    fn tool_version(program: &str) -> Option<String> {
        let mut cmd = Command::new(program);
        cmd.arg("-version");
        let output = run(cmd).ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        Some(text.lines().next().unwrap_or("").trim().to_string())
    }

    let ffmpeg = tool_version(&ffmpeg_bin());
    let ffprobe = tool_version(&ffprobe_bin());
    (
        ffmpeg.is_some(),
        ffprobe.is_some(),
        ffmpeg.unwrap_or_else(|| "未检测到 FFmpeg".to_string()),
    )
}
