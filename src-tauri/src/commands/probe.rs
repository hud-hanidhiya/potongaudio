//! Command untuk mengambil metadata file audio (durasi, sample rate, channel,
//! format) sebelum proses trim/export dimulai. Dipanggil dari frontend
//! setelah file diupload, sebelum region default / durasi total dibutuhkan
//! oleh `ProgressTracker` di sisi export.
//!
//! Desain: logika PARSING (`parse_ffprobe_json`) dipisah total dari logika
//! SPAWN PROSES, supaya bagian parsing bisa diuji murni tanpa perlu binary
//! ffprobe sungguhan atau runtime Tauri. Bagian spawn (`probe_audio_file`,
//! bertanda `#[tauri::command]` di project asli) tinggal memanggil fungsi
//! yang sudah teruji ini.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub channels: u32,
    pub format_name: String,
}

// Struct internal untuk mem-parse bentuk JSON `ffprobe -print_format json
// -show_format -show_streams`. Sengaja tidak mengambil semua field yang ada
// di output ffprobe — hanya yang relevan untuk kebutuhan aplikasi ini.
#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    format: FfprobeFormat,
    streams: Vec<FfprobeStream>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    duration: Option<String>, // ffprobe menulis durasi sebagai STRING detik, mis. "123.456000"
    format_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    codec_type: Option<String>,
    sample_rate: Option<String>, // juga string, mis. "44100"
    channels: Option<u32>,
}

/// Parsing murni — tidak menyentuh filesystem atau proses eksternal sama
/// sekali. Ini fungsi yang paling penting untuk diuji ketat karena ffprobe
/// menulis banyak field sebagai STRING (bukan number) dan bisa saja tidak
/// menyertakan stream audio sama sekali (mis. file rusak atau video-only).
pub fn parse_ffprobe_json(raw: &str) -> AppResult<ProbeResult> {
    let parsed: FfprobeOutput =
        serde_json::from_str(raw).map_err(|e| AppError::InvalidAudioFile {
            path: "<unknown>".into(),
            detail: format!("output ffprobe tidak bisa di-parse: {e}"),
        })?;

    let duration_secs: f64 = parsed
        .format
        .duration
        .as_deref()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::InvalidAudioFile {
            path: "<unknown>".into(),
            detail: "field format.duration tidak ada atau tidak valid".into(),
        })?;

    let audio_stream = parsed
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("audio"))
        .ok_or_else(|| AppError::InvalidAudioFile {
            path: "<unknown>".into(),
            detail: "tidak ditemukan stream audio dalam file".into(),
        })?;

    let sample_rate: u32 = audio_stream
        .sample_rate
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(44_100); // fallback aman, bukan fatal error

    Ok(ProbeResult {
        duration_ms: (duration_secs * 1000.0).round() as u64,
        sample_rate,
        channels: audio_stream.channels.unwrap_or(2),
        format_name: parsed
            .format
            .format_name
            .unwrap_or_else(|| "unknown".into()),
    })
}

// ---------------------------------------------------------------------
// Bagian di bawah ini (spawn proses ffprobe sungguhan) TIDAK bisa diuji
// dengan cargo test murni tanpa binary ffprobe di PATH, jadi sengaja
// dipisah dari parse_ffprobe_json di atas. Di project Tauri asli, fungsi
// `probe_audio_file` ini yang diberi anotasi `#[tauri::command]`.
// ---------------------------------------------------------------------

#[cfg(not(test))]
pub async fn probe_audio_file_impl(
    ffprobe_binary_path: &std::path::Path,
    file_path: &str,
) -> AppResult<ProbeResult> {
    let output = tokio::process::Command::new(ffprobe_binary_path)
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            file_path,
        ])
        .output()
        .await
        .map_err(|e| AppError::SidecarSpawnFailed {
            detail: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(AppError::InvalidAudioFile {
            path: file_path.to_string(),
            detail: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    parse_ffprobe_json(&raw)
}

// ---------------------------------------------------------------------
// ⚠️ BELUM TERVERIFIKASI COMPILE — sama seperti catatan di commands/export.rs,
// wrapper `#[tauri::command]` di bawah butuh runtime Tauri sungguhan
// (tauri_plugin_shell untuk resolve path sidecar ffprobe) yang tidak bisa
// dicompile di sandbox pembuatan dokumen ini. `parse_ffprobe_json` di atas
// (bagian yang sesungguhnya rawan bug) SUDAH teruji penuh lewat 7 unit test.
// ---------------------------------------------------------------------

#[cfg(feature = "tauri-runtime")]
mod tauri_wiring {
    use super::*;
    use tauri_plugin_shell::ShellExt;

    /// Command yang dipanggil frontend setelah upload file, sebelum
    /// menampilkan waveform — hasilnya dipakai untuk inisialisasi
    /// `ProgressTracker` di export nanti (lihat commands/export.rs).
    #[tauri::command]
    pub async fn probe_audio_file(
        app: tauri::AppHandle,
        file_path: String,
    ) -> Result<ProbeResult, AppError> {
        let output = app
            .shell()
            .sidecar("ffprobe")
            .map_err(|e| AppError::SidecarSpawnFailed {
                detail: e.to_string(),
            })?
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                &file_path,
            ])
            .output()
            .await
            .map_err(|e| AppError::SidecarSpawnFailed {
                detail: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(AppError::InvalidAudioFile {
                path: file_path.clone(),
                detail: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        parse_ffprobe_json(&raw)
    }
}

#[cfg(feature = "tauri-runtime")]
pub use tauri_wiring::probe_audio_file;

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_VALID_JSON: &str = r#"
    {
        "streams": [
            {
                "index": 0,
                "codec_type": "audio",
                "sample_rate": "44100",
                "channels": 2
            }
        ],
        "format": {
            "filename": "input.mp3",
            "format_name": "mp3",
            "duration": "123.456000"
        }
    }
    "#;

    #[test]
    fn parse_json_valid_menghasilkan_probe_result_benar() {
        let result = parse_ffprobe_json(SAMPLE_VALID_JSON).unwrap();
        assert_eq!(result.duration_ms, 123_456);
        assert_eq!(result.sample_rate, 44_100);
        assert_eq!(result.channels, 2);
        assert_eq!(result.format_name, "mp3");
    }

    #[test]
    fn json_tanpa_stream_audio_menghasilkan_error() {
        let json = r#"
        {
            "streams": [
                { "index": 0, "codec_type": "video" }
            ],
            "format": { "duration": "10.0", "format_name": "mp4" }
        }
        "#;
        let result = parse_ffprobe_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn json_tanpa_duration_menghasilkan_error() {
        let json = r#"
        {
            "streams": [{ "codec_type": "audio", "sample_rate": "44100", "channels": 2 }],
            "format": { "format_name": "mp3" }
        }
        "#;
        let result = parse_ffprobe_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn json_sample_rate_hilang_pakai_fallback_44100() {
        let json = r#"
        {
            "streams": [{ "codec_type": "audio", "channels": 1 }],
            "format": { "duration": "5.0", "format_name": "wav" }
        }
        "#;
        let result = parse_ffprobe_json(json).unwrap();
        assert_eq!(result.sample_rate, 44_100);
        assert_eq!(result.channels, 1);
    }

    #[test]
    fn json_malformed_tidak_panic_dan_mengembalikan_error() {
        let result = parse_ffprobe_json("bukan json valid {{{");
        assert!(result.is_err());
    }

    #[test]
    fn json_kosong_mengembalikan_error() {
        let result = parse_ffprobe_json("");
        assert!(result.is_err());
    }

    #[test]
    fn duration_dengan_desimal_dibulatkan_dengan_benar() {
        let json = r#"
        {
            "streams": [{ "codec_type": "audio", "sample_rate": "48000", "channels": 2 }],
            "format": { "duration": "10.0006", "format_name": "flac" }
        }
        "#;
        let result = parse_ffprobe_json(json).unwrap();
        // 10.0006 detik -> 10000.6ms -> dibulatkan ke 10001ms
        assert_eq!(result.duration_ms, 10_001);
    }
}
