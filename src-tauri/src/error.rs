//! Error type terpusat untuk seluruh backend.
//!
//! Prinsip: semua error yang bisa terjadi di layer Rust (spawn proses gagal,
//! FFmpeg exit non-zero, path tidak valid, dll) dikonversi ke varian di sini,
//! lalu di-serialize sebagai string pesan yang aman ditampilkan ke user.
//! Frontend TIDAK PERNAH menerima raw stderr FFmpeg atau Rust panic message.

use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    /// File input tidak ditemukan atau tidak bisa dibaca.
    FileNotFound { path: String },

    /// File ada tapi bukan format audio yang valid / corrupt.
    InvalidAudioFile { path: String, detail: String },

    /// Sidecar FFmpeg gagal di-spawn (binary tidak ditemukan, permission, dll).
    SidecarSpawnFailed { detail: String },

    /// FFmpeg berjalan tapi exit dengan kode non-zero.
    FfmpegExecutionFailed {
        exit_code: Option<i32>,
        stderr_tail: String,
    },

    /// Parameter dari frontend tidak valid (mis. region end < start).
    InvalidParams { detail: String },

    /// Disk penuh atau tidak ada permission menulis ke output path.
    OutputWriteFailed { path: String, detail: String },

    /// Proses dibatalkan oleh user via cancel_export.
    Cancelled { job_id: String },

    /// Fallback untuk error I/O umum yang belum punya varian spesifik.
    Io(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::FileNotFound { path } => {
                write!(f, "File tidak ditemukan: {path}")
            }
            AppError::InvalidAudioFile { path, detail } => {
                write!(f, "File audio tidak valid ({path}): {detail}")
            }
            AppError::SidecarSpawnFailed { detail } => {
                write!(f, "Gagal menjalankan proses audio: {detail}")
            }
            AppError::FfmpegExecutionFailed {
                exit_code,
                stderr_tail,
            } => {
                write!(
                    f,
                    "Proses audio gagal (exit code {:?}): {}",
                    exit_code,
                    // Ambil beberapa baris terakhir stderr saja — cukup untuk
                    // diagnosis tanpa membanjiri UI dengan log mentah.
                    stderr_tail
                )
            }
            AppError::InvalidParams { detail } => {
                write!(f, "Parameter tidak valid: {detail}")
            }
            AppError::OutputWriteFailed { path, detail } => {
                write!(f, "Gagal menyimpan file ke {path}: {detail}")
            }
            AppError::Cancelled { job_id } => {
                write!(f, "Proses dibatalkan (job {job_id})")
            }
            AppError::Io(msg) => write!(f, "Terjadi kesalahan: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

/// Bentuk error yang dikirim ke frontend lewat Tauri (via Result<T, AppError>
/// yang di-serialize otomatis saat command mengembalikan Err).
///
/// Tauri v2 mengharuskan tipe error command mengimplementasikan Serialize,
/// jadi kita bungkus jadi struct sederhana { message, kind } supaya frontend
/// bisa switch UI behavior berdasarkan `kind` jika perlu (mis. tampilkan
/// tombol retry khusus untuk SidecarSpawnFailed).
#[derive(Serialize)]
pub struct SerializableError {
    pub kind: &'static str,
    pub message: String,
}

// Implementasi Serialize langsung di AppError supaya bisa dipakai sebagai
// tipe Err di signature #[tauri::command] tanpa konversi manual di tiap command.
// (Tauri v2 mensyaratkan tipe Err pada command mengimplementasikan Serialize.)
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableError::from(self).serialize(serializer)
    }
}

// &AppError -> SerializableError (dipakai internal oleh impl Serialize di atas,
// yang hanya punya akses &self, bukan ownership).
impl From<&AppError> for SerializableError {
    fn from(e: &AppError) -> Self {
        let kind = match e {
            AppError::FileNotFound { .. } => "file_not_found",
            AppError::InvalidAudioFile { .. } => "invalid_audio_file",
            AppError::SidecarSpawnFailed { .. } => "sidecar_spawn_failed",
            AppError::FfmpegExecutionFailed { .. } => "ffmpeg_execution_failed",
            AppError::InvalidParams { .. } => "invalid_params",
            AppError::OutputWriteFailed { .. } => "output_write_failed",
            AppError::Cancelled { .. } => "cancelled",
            AppError::Io(_) => "io_error",
        };
        SerializableError {
            kind,
            message: e.to_string(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_message_bahasa_indonesia_dan_tidak_kosong() {
        let e = AppError::FileNotFound {
            path: "test.mp3".into(),
        };
        assert!(e.to_string().contains("test.mp3"));
    }

    #[test]
    fn ffmpeg_execution_failed_menyertakan_exit_code() {
        let e = AppError::FfmpegExecutionFailed {
            exit_code: Some(1),
            stderr_tail: "Invalid data found".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("Invalid data found"));
    }

    #[test]
    fn serializable_error_kind_konsisten() {
        let e = AppError::InvalidParams {
            detail: "region end < start".into(),
        };
        let ser: SerializableError = (&e).into();
        assert_eq!(ser.kind, "invalid_params");
    }
}
