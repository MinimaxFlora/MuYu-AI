use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;

use serde::Serialize;

use crate::media;

static CRYPTO_INIT: Once = Once::new();

pub fn ensure_crypto() {
    CRYPTO_INIT.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceInfo {
    pub id: String,
    pub label: String,
    pub locale: String,
    pub gender: String,
}

pub fn list_voices(engine: &str) -> Result<Vec<VoiceInfo>, String> {
    match engine {
        "sapi" => sapi_voices(),
        _ => edge_voices(),
    }
}

fn edge_voices() -> Result<Vec<VoiceInfo>, String> {
    ensure_crypto();
    let voices = msedge_tts::voice::get_voices_list()
        .map_err(|e| format!("获取 Edge 音色失败：{e}（可改用 Windows SAPI 离线引擎）"))?;

    let mut items: Vec<VoiceInfo> = voices
        .into_iter()
        .filter(|voice| {
            let locale = voice.locale.as_deref().unwrap_or("");
            locale.starts_with("zh") || locale == "en-US"
        })
        .map(|voice| {
            let id = voice.name.clone();
            let short = voice.short_name.clone().unwrap_or_else(|| id.clone());
            let locale = voice.locale.clone().unwrap_or_default();
            let gender = match voice.gender.as_deref() {
                Some("Female") => "女".to_string(),
                Some("Male") => "男".to_string(),
                other => other.unwrap_or("").to_string(),
            };
            VoiceInfo {
                id,
                label: format!("{short} · {gender}"),
                locale,
                gender,
            }
        })
        .collect();

    items.sort_by(|a, b| {
        let rank = |locale: &str| {
            if locale.starts_with("zh-CN") {
                0
            } else if locale.starts_with("zh") {
                1
            } else {
                2
            }
        };
        rank(&a.locale)
            .cmp(&rank(&b.locale))
            .then_with(|| a.label.cmp(&b.label))
    });
    Ok(items)
}

fn run_powershell(script: &str) -> Result<String, String> {
    let mut cmd = Command::new("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let output = cmd
        .output()
        .map_err(|e| format!("无法执行 PowerShell：{e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn sapi_voices() -> Result<Vec<VoiceInfo>, String> {
    let script = r#"
[Console]::OutputEncoding=[Text.Encoding]::UTF8
Add-Type -AssemblyName System.Speech
$s = New-Object System.Speech.Synthesis.SpeechSynthesizer
$s.GetInstalledVoices() | ForEach-Object { "{0}|{1}" -f $_.VoiceInfo.Name, $_.VoiceInfo.Culture.Name }
"#;
    let output = run_powershell(script)?;
    let mut items = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (name, culture) = line.split_once('|').unwrap_or((line, ""));
        items.push(VoiceInfo {
            id: name.to_string(),
            label: if culture.is_empty() {
                name.to_string()
            } else {
                format!("{name} · {culture}")
            },
            locale: culture.to_string(),
            gender: String::new(),
        });
    }
    if items.is_empty() {
        return Err("系统没有可用的 SAPI 语音".into());
    }
    Ok(items)
}

pub struct EdgeSession {
    client: msedge_tts::tts::client::MSEdgeTTSClient<std::net::TcpStream>,
    config: msedge_tts::tts::SpeechConfig,
}

impl EdgeSession {
    pub fn new(voice: &str, rate: f32) -> Result<Self, String> {
        ensure_crypto();
        let voices = msedge_tts::voice::get_voices_list()
            .map_err(|e| format!("连接 Edge TTS 失败：{e}"))?;
        let found = voices
            .iter()
            .find(|v| v.name == voice || v.short_name.as_deref() == Some(voice))
            .ok_or_else(|| format!("找不到音色：{voice}"))?;

        let mut config = msedge_tts::tts::SpeechConfig::from(found);
        config.rate = (((rate - 1.0) * 100.0).round() as i32).clamp(-50, 100);
        config.pitch = 0;
        config.volume = 0;

        let client = msedge_tts::tts::client::connect()
            .map_err(|e| format!("连接 Edge TTS 失败：{e}"))?;
        Ok(Self { client, config })
    }

    pub fn synthesize(&mut self, text: &str, out: &Path) -> Result<f64, String> {
        let audio = self
            .client
            .synthesize(text, &self.config)
            .map_err(|e| format!("Edge TTS 合成失败：{e}"))?;
        if audio.audio_bytes.is_empty() {
            return Err("Edge TTS 返回了空音频".into());
        }
        fs::write(out, &audio.audio_bytes).map_err(|e| format!("写入音频失败：{e}"))?;
        media::probe_duration(&out.to_string_lossy())
            .or_else(|_| Ok::<f64, String>((text.chars().count() as f64 / 5.0).max(1.0)))
    }
}

pub struct SapiSession {
    voice: String,
    rate: f32,
    work_dir: PathBuf,
}

impl SapiSession {
    pub fn new(voice: &str, rate: f32, work_dir: PathBuf) -> Self {
        Self {
            voice: voice.to_string(),
            rate,
            work_dir,
        }
    }

    fn synthesize(&mut self, text: &str, index: u32, out: &Path) -> Result<f64, String> {
        fs::create_dir_all(&self.work_dir).map_err(|e| format!("创建音频目录失败：{e}"))?;
        let txt = self.work_dir.join(format!("sapi-{index:03}.txt"));
        fs::write(&txt, text).map_err(|e| format!("写入配音文本失败：{e}"))?;

        let rate = (((self.rate - 1.0) * 5.0).round() as i32).clamp(-10, 10);
        let quote = |value: &str| value.replace('\'', "''");
        let script = format!(
            r#"
[Console]::OutputEncoding=[Text.Encoding]::UTF8
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Speech
$s = New-Object System.Speech.Synthesis.SpeechSynthesizer
try {{ if ('{}' -ne '') {{ $s.SelectVoice('{}') }} }} catch {{ }}
$s.Rate = {}
$s.SetOutputToWaveFile('{}')
$text = Get-Content -Raw -Encoding UTF8 '{}'
$s.Speak($text)
$s.Dispose()
"#,
            quote(&self.voice),
            quote(&self.voice),
            rate,
            quote(&out.to_string_lossy()),
            quote(&txt.to_string_lossy())
        );

        let result = run_powershell(&script);
        let _ = fs::remove_file(&txt);
        result?;
        if !out.exists() {
            return Err("SAPI 没有生成音频文件".into());
        }
        media::probe_duration(&out.to_string_lossy())
            .or_else(|_| Ok::<f64, String>((text.chars().count() as f64 / 5.0).max(1.0)))
    }
}

pub enum SynthSession {
    Edge(EdgeSession),
    Sapi(SapiSession),
}

impl SynthSession {
    pub fn new(
        engine: &str,
        voice: &str,
        rate: f32,
        work_dir: PathBuf,
    ) -> Result<Self, String> {
        if engine == "sapi" {
            Ok(Self::Sapi(SapiSession::new(voice, rate, work_dir)))
        } else {
            Ok(Self::Edge(EdgeSession::new(voice, rate)?))
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Edge(_) => "mp3",
            Self::Sapi(_) => "wav",
        }
    }

    pub fn synthesize(&mut self, text: &str, index: u32, out: &Path) -> Result<f64, String> {
        match self {
            Self::Edge(session) => session.synthesize(text, out),
            Self::Sapi(session) => session.synthesize(text, index, out),
        }
    }
}
