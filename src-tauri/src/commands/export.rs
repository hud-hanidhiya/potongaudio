//! Versi struct EffectParams sesuai kontrak di TECH_IMPLEMENTATION_PLAN.md
//! Section 2, DITAMBAH command Tauri `export_audio` dan `cancel_export`
//! (di-gate lewat feature flag, lihat catatan di bawah).
//!
//! STATUS VERIFIKASI: seluruh file ini terverifikasi — struct kontrak
//! teruji via filter_builder.rs, command layer ter-compile & clippy bersih
//! dengan `--features tauri-runtime`, dan alur export lolos CI dua OS.
//! Struct di bawah tetap di luar gate supaya `cargo test` (tanpa Tauri)
//! bisa mengunci kontrak TS↔Rust.

#[cfg(feature = "tauri-runtime")]
use crate::error::AppError;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EffectParams {
    pub source_file_path: String,
    pub region: Region,
    pub gain_db: f32,
    pub fade: Fade,
    pub speed: Speed,
    pub output_format: OutputFormat,
    pub output_bitrate_kbps: Option<u32>,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Fade {
    pub in_ms: u64,
    pub out_ms: u64,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Speed {
    pub ratio: f32,
    pub preserve_pitch: bool,
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Mp3,
    Wav,
    M4a,
    Flac,
    M4r,
}

// ---------------------------------------------------------------------
// STATUS: command di bawah terverifikasi compile + clippy `-D warnings`
// (`--features tauri-runtime`) dan dieksekusi nyata oleh CI dua OS.
// Gate `#[cfg(feature = "tauri-runtime")]` dipertahankan agar `cargo test`
// default tetap cepat tanpa dependency Tauri.
// ---------------------------------------------------------------------

#[cfg(feature = "tauri-runtime")]
mod tauri_wiring {
    use super::*;
    use crate::ffmpeg::filter_builder::build_filter_plan;
    use crate::ffmpeg::progress_parser::{ProgressTracker, ProgressUpdate};
    use crate::ffmpeg::sidecar::JobRegistry;
    use std::sync::Arc;
    use tauri::{AppHandle, Emitter, State};
    use tauri_plugin_shell::process::CommandEvent;
    use tauri_plugin_shell::ShellExt;
    use tokio::sync::Mutex;

    #[derive(serde::Serialize)]
    pub struct ExportResult {
        pub output_path: String,
    }

    #[derive(Clone, serde::Serialize)]
    struct ProgressPayload {
        job_id: String,
        percent: u32,
    }

    #[derive(Clone, serde::Serialize)]
    struct DonePayload {
        job_id: String,
        output_path: String,
    }

    #[derive(Clone, serde::Serialize)]
    struct ErrorPayload {
        job_id: String,
        message: String,
    }

    /// Command utama export. Alur:
    /// 1. Validasi & bangun filter plan lewat `build_filter_plan` (sudah teruji).
    /// 2. Frontend sudah punya `total_duration_ms` dari hasil `probe_audio_file`
    ///    sebelumnya (dikirim sebagai parameter, bukan di-probe ulang di sini,
    ///    supaya command ini tetap fokus satu tanggung jawab).
    /// 3. Spawn sidecar FFmpeg lewat tauri-plugin-shell (cara resmi Tauri v2
    ///    untuk binary yang di-bundle via `externalBin`, capability-gated
    ///    lewat tauri.conf.json / capabilities/*.json).
    /// 4. Stream CommandEvent::Stderr per baris ke ProgressTracker yang sama
    ///    persis dengan yang sudah diuji di ffmpeg::progress_parser.
    /// 5. Emit event ke frontend: export://progress, export://done, export://error.
    #[tauri::command]
    pub async fn export_audio(
        app: AppHandle,
        registry: State<'_, JobRegistry>,
        job_id: String,
        params: EffectParams,
        total_duration_ms: u64,
        output_path: String, // hasil dari native save dialog (Fase T4.5)
    ) -> Result<ExportResult, AppError> {
        let plan = build_filter_plan(&params, total_duration_ms)?;

        let mut args: Vec<String> = vec![
            "-y".into(),
            "-i".into(),
            params.source_file_path.clone(),
            "-filter_complex".into(),
            plan.filter_complex.clone(),
            "-progress".into(),
            "pipe:2".into(),
        ];
        args.extend(plan.codec_args.clone());
        args.push(output_path.clone());

        let (mut rx, child) = app
            .shell()
            .sidecar("ffmpeg")
            .map_err(|e| AppError::SidecarSpawnFailed {
                detail: e.to_string(),
            })?
            .args(&args)
            .spawn()
            .map_err(|e| AppError::SidecarSpawnFailed {
                detail: e.to_string(),
            })?;

        // Daftarkan child ke JobRegistry (dukung cancel lewat `Killable`,
        // berlaku untuk `tauri_plugin_shell::process::CommandChild` di runtime
        // maupun `tokio::process::Child` di test).
        registry
            .register(&job_id, Arc::new(Mutex::new(Some(child))))
            .await;

        let mut tracker = ProgressTracker::new(total_duration_ms);
        let mut stderr_tail: Vec<String> = Vec::with_capacity(20);

        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stderr(bytes) => {
                    let line = String::from_utf8_lossy(&bytes).into_owned();
                    stderr_tail.push(line.clone());
                    if stderr_tail.len() > 20 {
                        stderr_tail.remove(0);
                    }

                    if let ProgressUpdate::Percent(p) = tracker.process_line(&line) {
                        let _ = app.emit(
                            "export://progress",
                            ProgressPayload {
                                job_id: job_id.clone(),
                                percent: p,
                            },
                        );
                    }
                }
                CommandEvent::Terminated(payload) => {
                    registry.unregister(&job_id).await;

                    if payload.code != Some(0) {
                        let err = AppError::FfmpegExecutionFailed {
                            exit_code: payload.code,
                            stderr_tail: stderr_tail.join("\n"),
                        };
                        let _ = app.emit(
                            "export://error",
                            ErrorPayload {
                                job_id: job_id.clone(),
                                message: err.to_string(),
                            },
                        );
                        return Err(err);
                    }
                    break;
                }
                CommandEvent::Error(e) => {
                    registry.unregister(&job_id).await;
                    let err = AppError::SidecarSpawnFailed { detail: e };
                    let _ = app.emit(
                        "export://error",
                        ErrorPayload {
                            job_id: job_id.clone(),
                            message: err.to_string(),
                        },
                    );
                    return Err(err);
                }
                _ => {}
            }
        }

        if !std::path::Path::new(&output_path).exists() {
            let err = AppError::OutputWriteFailed {
                path: output_path.clone(),
                detail: "FFmpeg melaporkan sukses tapi file output tidak ditemukan".into(),
            };
            let _ = app.emit(
                "export://error",
                ErrorPayload {
                    job_id: job_id.clone(),
                    message: err.to_string(),
                },
            );
            return Err(err);
        }

        let result = ExportResult {
            output_path: output_path.clone(),
        };
        let _ = app.emit(
            "export://done",
            DonePayload {
                job_id: job_id.clone(),
                output_path,
            },
        );

        Ok(result)
    }

    #[tauri::command]
    pub async fn cancel_export(
        registry: State<'_, JobRegistry>,
        job_id: String,
    ) -> Result<bool, AppError> {
        Ok(registry.cancel(&job_id).await)
    }
}

#[cfg(feature = "tauri-runtime")]
pub use tauri_wiring::{cancel_export, export_audio};
