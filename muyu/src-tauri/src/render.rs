use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::media;
use crate::model::Project;
use crate::pipeline::fmt_ts;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RenderOptions {
    pub aspect: String,
    pub resolution: String,
    pub burn_subtitles: bool,
    pub output_name: String,
    pub bgm_path: Option<String>,
    pub bgm_volume: f32,
    pub bgm_ducking: bool,
    pub bgm_fade: bool,
    pub ducking_strength: String,
    pub encoder: String,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            aspect: "original".into(),
            resolution: "1080p".into(),
            burn_subtitles: true,
            output_name: String::new(),
            bgm_path: None,
            bgm_volume: 0.18,
            bgm_ducking: true,
            bgm_fade: true,
            ducking_strength: "medium".into(),
            encoder: "auto".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderResult {
    pub path: String,
    pub srt_path: String,
    pub duration: f64,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub segments: usize,
    pub subtitle_burned: bool,
    pub note: Option<String>,
    pub rendered_at: String,
    #[serde(default)]
    pub encoder: String,
}

fn ffmpeg_command() -> Command {
    let mut cmd = Command::new(media::ffmpeg_bin());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

fn available_encoders() -> Vec<String> {
    let output = match ffmpeg_command().args(["-hide_banner", "-encoders"]).output() {
        Ok(output) => output,
        Err(_) => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&output.stdout);
    ["h264_nvenc", "h264_qsv", "h264_amf", "libx264"]
        .iter()
        .filter(|name| text.contains(**name))
        .map(|name| (*name).to_string())
        .collect()
}

fn encoder_works(encoder: &str) -> bool {
    ffmpeg_command()
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=320x240:d=1",
            "-frames:v",
            "1",
            "-c:v",
            encoder,
            "-f",
            "null",
            "-",
        ])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn resolve_encoder(requested: &str) -> String {
    let available = available_encoders();
    let requested = requested.trim();
    if requested == "auto" {
        for candidate in ["h264_nvenc", "h264_qsv", "h264_amf"] {
            if available.iter().any(|name| name == candidate) && encoder_works(candidate) {
                return candidate.to_string();
            }
        }
    } else if requested != "libx264"
        && available.iter().any(|name| name == requested)
        && encoder_works(requested)
    {
        return requested.to_string();
    }
    "libx264".into()
}

fn encoder_args(encoder: &str) -> Vec<String> {
    let mut args = vec!["-c:v".to_string(), encoder.to_string()];
    match encoder {
        "h264_nvenc" => args.extend(
            ["-preset", "p4", "-cq", "20", "-b:v", "0"]
                .iter()
                .map(|value| value.to_string()),
        ),
        "h264_qsv" => args.extend(
            ["-preset", "veryfast", "-global_quality", "22"]
                .iter()
                .map(|value| value.to_string()),
        ),
        "h264_amf" => args.extend(
            ["-quality", "balanced", "-rc", "cqp", "-qp_i", "20", "-qp_p", "22"]
                .iter()
                .map(|value| value.to_string()),
        ),
        _ => args.extend(
            ["-preset", "veryfast", "-crf", "20"]
                .iter()
                .map(|value| value.to_string()),
        ),
    }
    args.push("-pix_fmt".into());
    args.push("yuv420p".into());
    args
}

fn run_ffmpeg(args: &[String], cwd: &Path) -> Result<(), String> {
    let output = ffmpeg_command()
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("无法执行 ffmpeg：{e}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let tail: String = stderr
        .chars()
        .rev()
        .take(600)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    Err(format!("ffmpeg 失败：{tail}"))
}

fn slot_duration(timestamp: &str) -> f64 {
    fn parse(raw: &str) -> f64 {
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
    let (start, end) = timestamp.split_once('-').unwrap_or((timestamp, timestamp));
    (parse(end) - parse(start)).max(0.0)
}

fn target_size(project: &Project, options: &RenderOptions) -> (u32, u32) {
    let info = project
        .videos
        .first()
        .map(|video| &video.info)
        .cloned()
        .unwrap_or_default();
    let even = |value: u32| (value.max(2) / 2) * 2;
    let base: u32 = if options.resolution == "720p" { 720 } else { 1080 };
    match options.aspect.as_str() {
        "16:9" => (even(base * 16 / 9), even(base)),
        "9:16" => (even(base), even(base * 16 / 9)),
        "1:1" => (even(base), even(base)),
        _ => (even(info.width), even(info.height)),
    }
}

fn video_filter(width: u32, height: u32) -> String {
    format!(
        "scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2:color=black,setsar=1,fps=25,tpad=stop_mode=clone:stop_duration=2,format=yuv420p"
    )
}

fn build_srt(project: &Project, durations: &[(u32, f64, bool)]) -> String {    let mut out = String::new();
    let mut cursor = 0.0;
    let mut index = 1;
    for (item_id, duration, is_narration) in durations {
        let start = cursor;
        let end = cursor + duration;
        cursor = end;
        if !is_narration {
            continue;
        }
        let Some(item) = project.items.iter().find(|item| item.id == *item_id) else {
            continue;
        };
        out.push_str(&format!(
            "{index}\n{} --> {}\n{}\n\n",
            fmt_ts(start),
            fmt_ts((end - 0.05).max(start + 0.2)),
            item.narration
        ));
        index += 1;
    }
    out
}

pub fn srt_for_project(project: &Project) -> String {
    let durations: Vec<(u32, f64, bool)> = project
        .items
        .iter()
        .map(|item| {
            let is_narration = item.ost == 0 || item.ost == 2;
            let duration = if is_narration {
                item.audio_duration
                    .filter(|value| *value > 0.1)
                    .unwrap_or_else(|| slot_duration(&item.timestamp))
            } else {
                slot_duration(&item.timestamp)
            };
            (item.id, duration, is_narration)
        })
        .collect();
    build_srt(project, &durations)
}

pub fn safe_name(raw: &str) -> String {
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
        "muyu-output".into()
    } else {
        trimmed.chars().take(60).collect()
    }
}

pub fn render_project(
    project: &Project,
    options: &RenderOptions,
    work_dir: &Path,
    output_dir: &Path,
    on_progress: &dyn Fn(f64, String),
    is_canceled: &dyn Fn() -> bool,
) -> Result<RenderResult, String> {
    if project.items.is_empty() {
        return Err("没有分镜，无法渲染".into());
    }
    let encoder = resolve_encoder(&options.encoder);

    let (width, height) = target_size(project, options);
    let filter = video_filter(width, height);
    let segment_dir = work_dir.join("segments");
    let _ = fs::remove_dir_all(work_dir);
    fs::create_dir_all(&segment_dir).map_err(|e| format!("创建渲染目录失败：{e}"))?;
    fs::create_dir_all(output_dir).map_err(|e| format!("创建输出目录失败：{e}"))?;

    let mut durations: Vec<(u32, f64, bool)> = Vec::new();
    let mut total = 0.0;
    let mut segment_files: Vec<PathBuf> = Vec::new();

    for (index, item) in project.items.iter().enumerate() {
        if is_canceled() {
            return Err("渲染已取消".into());
        }
        let is_narration = item.ost == 0 || item.ost == 2;
        let duration = if is_narration {
            let audio = item
                .audio_duration
                .filter(|value| *value > 0.1)
                .ok_or_else(|| format!("镜头 #{} 缺少配音，请先完成配音", item.id))?;
            audio
        } else {
            slot_duration(&item.timestamp)
        };
        if duration < 0.2 {
            continue;
        }

        let (start_raw, _) = item.timestamp.split_once('-').unwrap_or((&item.timestamp, ""));
        let start = start_raw.trim().replace(',', ".");
        let duration_text = format!("{duration:.3}");
        let out_path = segment_dir.join(format!("seg-{:03}.mp4", item.id));
        let out_arg = out_path.to_string_lossy().to_string();
        let mut args: Vec<String> = vec![
            "-y".into(),
            "-ss".into(),
            start,
            "-t".into(),
            duration_text.clone(),
            "-i".into(),
            project.videos.first().map(|v| v.path.clone()).unwrap_or_default(),
        ];

        let audio_filter;
        if item.ost == 0 {
            let audio_path = item
                .audio_path
                .clone()
                .ok_or_else(|| format!("镜头 #{} 缺少配音文件", item.id))?;
            args.extend(["-i".into(), audio_path]);
            audio_filter = format!("[1:a]apad=whole_dur={duration_text}[a]");
        } else if item.ost == 2 {
            let audio_path = item
                .audio_path
                .clone()
                .ok_or_else(|| format!("镜头 #{} 缺少配音文件", item.id))?;
            args.extend(["-i".into(), audio_path]);
            audio_filter = format!(
                "[0:a]volume=0.55,apad=whole_dur={duration_text}[a0];[1:a]volume=1.0,apad=whole_dur={duration_text}[a1];[a0][a1]amix=inputs=2:duration=first:dropout_transition=0[a]"
            );
        } else {
            audio_filter = format!("[0:a]volume=0.9,apad=whole_dur={duration_text}[a]");
        }

        args.extend([
            "-filter_complex".into(),
            format!("[0:v]{filter}[v];{audio_filter}"),
            "-map".into(),
            "[v]".into(),
            "-map".into(),
            "[a]".into(),
        ]);
        args.extend(encoder_args(&encoder));
        args.extend([
            "-c:a".into(),
            "aac".into(),
            "-ar".into(),
            "48000".into(),
            "-ac".into(),
            "2".into(),
            "-b:a".into(),
            "192k".into(),
            "-t".into(),
            duration_text,
            "-video_track_timescale".into(),
            "90000".into(),
            out_arg,
        ]);

        run_ffmpeg(&args, work_dir)?;
        segment_files.push(out_path);
        durations.push((item.id, duration, is_narration));
        total += duration;

        let percent = 5.0 + (index as f64 + 1.0) / project.items.len() as f64 * 75.0;
        on_progress(percent, format!("裁剪镜头 #{}", item.id));
    }

    if segment_files.is_empty() {
        return Err("没有可渲染的镜头".into());
    }

    if is_canceled() {
        return Err("渲染已取消".into());
    }

    let mut list = String::new();
    for file in &segment_files {
        let path = file.to_string_lossy().replace('\\', "/").replace('\'', "'\\''");
        list.push_str(&format!("file '{path}'\n"));
    }
    fs::write(work_dir.join("concat.txt"), list).map_err(|e| format!("写入拼接列表失败：{e}"))?;

    on_progress(84.0, "拼接视频".into());
    run_ffmpeg(
        &[
            "-y".into(),
            "-f".into(),
            "concat".into(),
            "-safe".into(),
            "0".into(),
            "-i".into(),
            "concat.txt".into(),
            "-c".into(),
            "copy".into(),
            "-movflags".into(),
            "+faststart".into(),
            "concat.mp4".into(),
        ],
        work_dir,
    )?;

    let srt = build_srt(project, &durations);
    fs::write(work_dir.join("subtitles.srt"), srt).map_err(|e| format!("写入字幕失败：{e}"))?;

    let mut base_video = "concat.mp4".to_string();
    if let Some(bgm) = options
        .bgm_path
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        if !Path::new(bgm).exists() {
            return Err(format!("背景音乐不存在：{bgm}"));
        }
        on_progress(87.0, "混入背景音乐".into());
        let volume = options.bgm_volume.clamp(0.0, 1.0);
        let duration_text = format!("{total:.3}");
        let fade = 1.2_f64.min(total / 4.0);
        let mut bgm_chain = format!(
            "[1:a]volume={volume:.3},atrim=0:{duration_text},apad=whole_dur={duration_text},aresample=48000"
        );
        if options.bgm_fade && fade > 0.05 {
            bgm_chain.push_str(&format!(
                ",afade=t=in:st=0:d={fade:.3},afade=t=out:st={:.3}:d={fade:.3}",
                (total - fade).max(0.0)
            ));
        }
        bgm_chain.push_str("[bg];");
        let (threshold, ratio) = match options.ducking_strength.as_str() {
            "light" => (0.03, 3.0),
            "strong" => (0.08, 10.0),
            _ => (0.05, 6.0),
        };
        let filter = if options.bgm_ducking {
            format!(
                "{bgm_chain}[0:a]asplit=2[voice][side];[bg][side]sidechaincompress=threshold={threshold:.3}:ratio={ratio:.1}:attack=15:release=300[ducked];[voice][ducked]amix=inputs=2:duration=first:dropout_transition=0[a]"
            )
        } else {
            format!(
                "{bgm_chain}[0:a]apad=whole_dur={duration_text}[voice];[voice][bg]amix=inputs=2:duration=first:dropout_transition=0[a]"
            )
        };
        run_ffmpeg(
            &[
                "-y".into(),
                "-i".into(),
                "concat.mp4".into(),
                "-stream_loop".into(),
                "-1".into(),
                "-i".into(),
                bgm.to_string(),
                "-filter_complex".into(),
                filter,
                "-map".into(),
                "0:v:0".into(),
                "-map".into(),
                "[a]".into(),
                "-c:v".into(),
                "copy".into(),
                "-c:a".into(),
                "aac".into(),
                "-ar".into(),
                "48000".into(),
                "-ac".into(),
                "2".into(),
                "-b:a".into(),
                "192k".into(),
                "-t".into(),
                duration_text,
                "mixed.mp4".into(),
            ],
            work_dir,
        )?;
        base_video = "mixed.mp4".into();
    }

    let name = {
        let raw = if options.output_name.trim().is_empty() {
            project.name.clone()
        } else {
            options.output_name.clone()
        };
        safe_name(&raw)
    };
    let final_name = format!("{name}.mp4");
    let final_path = output_dir.join(&final_name);
    let srt_path = output_dir.join(format!("{name}.srt"));

    let mut subtitle_burned = false;
    let mut note: Option<String> = None;

    if options.burn_subtitles {
        on_progress(92.0, "烧录字幕".into());
        let font_size = ((height as f64 / 34.0).round() as u32).max(14);
        let style = format!(
            "FontName=Microsoft YaHei,FontSize={font_size},PrimaryColour=&H00FFFFFF,OutlineColour=&H80000000,BorderStyle=1,Outline=1.2,Shadow=0,MarginV={}",
            (height as f64 * 0.04).round() as u32
        );
        let burn_with = |encoder_name: &str| -> Result<(), String> {
            let mut args: Vec<String> = vec![
                "-y".into(),
                "-i".into(),
                base_video.clone(),
                "-vf".into(),
                format!("subtitles=subtitles.srt:force_style='{style}'"),
            ];
            args.extend(encoder_args(encoder_name));
            args.extend([
                "-c:a".into(),
                "copy".into(),
                "-movflags".into(),
                "+faststart".into(),
                "final.mp4".into(),
            ]);
            run_ffmpeg(&args, work_dir)
        };
        let mut burn_result = burn_with(&encoder);
        if burn_result.is_err() && encoder != "libx264" {
            burn_result = burn_with("libx264");
        }
        match burn_result {
            Ok(()) => subtitle_burned = true,
            Err(err) => {
                note = Some(format!("字幕烧录失败，已输出无字幕版本：{err}"));
                fs::copy(work_dir.join(&base_video), work_dir.join("final.mp4"))
                    .map_err(|e| format!("复制成片失败：{e}"))?;
            }
        }
    } else {
        fs::copy(work_dir.join(&base_video), work_dir.join("final.mp4"))
            .map_err(|e| format!("复制成片失败：{e}"))?;
    }

    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建输出目录失败：{e}"))?;
    }
    fs::copy(work_dir.join("final.mp4"), &final_path).map_err(|e| format!("写入成片失败：{e}"))?;
    fs::copy(work_dir.join("subtitles.srt"), &srt_path).map_err(|e| format!("写入字幕失败：{e}"))?;

    let size = fs::metadata(&final_path).map(|meta| meta.len()).unwrap_or(0);
    let duration = media::probe_duration(&final_path.to_string_lossy()).unwrap_or(total);

    on_progress(100.0, "渲染完成".into());

    Ok(RenderResult {
        path: final_path.to_string_lossy().to_string(),
        srt_path: srt_path.to_string_lossy().to_string(),
        duration,
        size,
        width,
        height,
        segments: segment_files.len(),
        subtitle_burned,
        note,
        rendered_at: Utc::now().to_rfc3339(),
        encoder,
    })
}
