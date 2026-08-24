//! Command informasi sistem: menampilkan versi FFmpeg yang di-bundle,
//! dipakai di UI sementara Fase 0 (T0.1) untuk membuktikan sidecar
//! FFmpeg bisa di-spawn dan versi binary sesuai yang diharapkan.

#[cfg(feature = "tauri-runtime")]
mod tauri_wiring {
    use crate::error::AppError;
    use tauri_plugin_shell::ShellExt;

    /// Spawn sidecar `ffmpeg -version` dan kembalikan baris pertama
    /// (string versi lengkap stdout sudah cukup untuk ditampilkan).
    #[tauri::command]
    pub async fn get_ffmpeg_version(app: tauri::AppHandle) -> Result<String, AppError> {
        let output = app
            .shell()
            .sidecar("ffmpeg")
            .map_err(|e| AppError::SidecarSpawnFailed {
                detail: e.to_string(),
            })?
            .args(["-version"])
            .output()
            .await
            .map_err(|e| AppError::SidecarSpawnFailed {
                detail: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(AppError::FfmpegExecutionFailed {
                exit_code: output.status.code(),
                stderr_tail: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

#[cfg(feature = "tauri-runtime")]
pub use tauri_wiring::get_ffmpeg_version;
