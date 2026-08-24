//! Wrapper untuk spawn proses FFmpeg sebagai sidecar Tauri, membaca stderr
//! per baris secara streaming (bukan menunggu proses selesai), dan
//! mendukung pembatalan (cancel) di tengah proses.
//!
//! CATATAN: modul ini sengaja TIDAK bergantung langsung pada tipe `tauri::AppHandle`
//! — fungsi progress dilewatkan sebagai closure/callback generik. Ini membuat
//! modul bisa di-unit-test tanpa runtime Tauri, dan pemanggil (commands/export.rs
//! di project sebenarnya) yang bertanggung jawab menghubungkan callback ini
//! ke `app_handle.emit(...)`.

use crate::error::{AppError, AppResult};
use crate::ffmpeg::progress_parser::{ProgressTracker, ProgressUpdate};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

/// Trait untuk mengabstraksi kemampuan `kill()` dari berbagai tipe proses.
/// Ini memungkinkan `JobRegistry` menyimpan `tokio::process::Child` (untuk test)
/// dan `tauri_plugin_shell::process::CommandChild` (untuk Tauri runtime) dalam
/// satu container yang sama.
#[async_trait]
pub trait Killable: Send + Sync {
    async fn kill(&self) -> bool;
}

/// Implementasi untuk `tokio::process::Child` — selalu tersedia, tidak perlu feature gate.
/// Target impl adalah `Mutex<Child>` (bukan `Arc<Mutex<Child>>`) supaya
/// `Arc::new(Mutex::new(child))` bisa di-coerce ke `Arc<dyn Killable + Send + Sync>`
/// (trait object di luar Arc). Caller tetap memegang `Arc<Mutex<Child>>` terpisah
/// kalau perlu memanggil `wait()`/membaca status proses setelah selesai.
#[async_trait]
impl Killable for Mutex<Child> {
    async fn kill(&self) -> bool {
        let mut child = self.lock().await;
        child.kill().await.is_ok()
    }
}

/// Implementasi untuk `tauri_plugin_shell::process::CommandChild` — hanya ada kalau
/// feature `tauri-runtime` aktif (saat build Tauri sungguhan).
///
/// API `CommandChild::kill(self)` mengonsumsi child (by value), jadi disimpan
/// sebagai `Option` di dalam Mutex supaya ownership bisa di-extract untuk
/// dipanggil `kill()` — sekali di-kill, entry tidak bisa di-kill lagi (benar,
/// karena proses sudah berakhir).
#[cfg(feature = "tauri-runtime")]
#[async_trait]
impl Killable for Mutex<Option<tauri_plugin_shell::process::CommandChild>> {
    async fn kill(&self) -> bool {
        let mut guard = self.lock().await;
        match guard.take() {
            Some(child) => child.kill().is_ok(),
            None => false,
        }
    }
}

/// Registry job yang sedang berjalan, dipakai untuk mendukung `cancel_export`.
/// Di project sebenarnya, instance ini disimpan sebagai Tauri managed state
/// (`app.manage(JobRegistry::default())`) supaya bisa diakses dari command
/// `cancel_export` secara terpisah dari command `export_audio`.
#[derive(Default, Clone)]
pub struct JobRegistry {
    inner: Arc<Mutex<HashMap<String, Arc<dyn Killable + Send + Sync>>>>,
}

impl JobRegistry {
    /// Daftarkan child ke registry. `child` harus sudah di-wrap dalam `Arc<Mutex<T>>`
    /// di mana T mengimplementasikan `Killable`.
    pub async fn register(&self, job_id: &str, child: Arc<dyn Killable + Send + Sync>) {
        self.inner.lock().await.insert(job_id.to_string(), child);
    }

    pub async fn unregister(&self, job_id: &str) {
        self.inner.lock().await.remove(job_id);
    }

    /// Dipanggil dari command `cancel_export`. Mengirim SIGKILL (via
    /// `Child::kill`) ke proses FFmpeg yang sedang berjalan untuk job ini.
    pub async fn cancel(&self, job_id: &str) -> bool {
        if let Some(handle) = self.inner.lock().await.get(job_id).cloned() {
            handle.kill().await
        } else {
            false
        }
    }
}

pub struct ExportRequest {
    pub job_id: String,
    pub ffmpeg_binary_path: PathBuf,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub filter_complex: String,
    pub codec_args: Vec<String>,
    pub total_duration_ms: u64,
}

/// Menjalankan satu proses export penuh: spawn FFmpeg, stream progress lewat
/// `on_progress`, tunggu selesai, dan kembalikan path output atau error.
///
/// `on_progress` dipanggil setiap kali persen naik cukup signifikan
/// (lihat `ProgressTracker`). Di project sebenarnya closure ini membungkus
/// `app_handle.emit("export://progress", ...)`.
pub async fn run_export<F>(
    req: ExportRequest,
    registry: &JobRegistry,
    mut on_progress: F,
) -> AppResult<PathBuf>
where
    F: FnMut(u32) + Send,
{
    let args = build_args(&req);

    let mut child = Command::new(&req.ffmpeg_binary_path)
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::SidecarSpawnFailed {
            detail: e.to_string(),
        })?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::SidecarSpawnFailed {
            detail: "tidak bisa membaca stderr proses FFmpeg".into(),
        })?;

    // Simpan sejumlah baris stderr terakhir untuk pesan error jika FFmpeg
    // gagal — supaya user dapat konteks tanpa membanjiri UI dengan log penuh.
    let mut stderr_tail: Vec<String> = Vec::with_capacity(20);
    let mut tracker = ProgressTracker::new(req.total_duration_ms);

    let mut reader = BufReader::new(stderr).lines();

    // Daftarkan child SEBELUM mulai membaca stream, supaya cancel_export
    // yang dipanggil dari command lain bisa langsung menemukan job ini.
    let child_handle: Arc<Mutex<Child>> = Arc::new(Mutex::new(child));
    let killable: Arc<dyn Killable + Send + Sync> = child_handle.clone();
    registry.register(&req.job_id, killable).await;

    let mut is_done_signal_received = false;

    while let Some(line) = reader
        .next_line()
        .await
        .map_err(|e| AppError::Io(e.to_string()))?
    {
        stderr_tail.push(line.clone());
        if stderr_tail.len() > 20 {
            stderr_tail.remove(0);
        }

        match tracker.process_line(&line) {
            ProgressUpdate::Percent(p) => on_progress(p),
            ProgressUpdate::Done => is_done_signal_received = true,
            ProgressUpdate::NoUpdate => {}
        }
    }

    let status = {
        let mut child = child_handle.lock().await;
        child
            .wait()
            .await
            .map_err(|e| AppError::Io(e.to_string()))?
    };

    registry.unregister(&req.job_id).await;

    if !status.success() {
        return Err(AppError::FfmpegExecutionFailed {
            exit_code: status.code(),
            stderr_tail: stderr_tail.join("\n"),
        });
    }

    // Jika proses exit sukses tapi tidak pernah mengirim `progress=end`,
    // itu tanda tidak biasa (mis. FFmpeg versi lama tanpa dukungan -progress
    // penuh) — bukan fatal, tapi dicatat sebagai sanity check untuk Fase 0.
    if !is_done_signal_received {
        // Sengaja tidak dijadikan Err — file output tetap valid kalau exit
        // code 0. Di implementasi nyata, ini titik yang baik untuk logging.
    }

    if !req.output_path.exists() {
        return Err(AppError::OutputWriteFailed {
            path: req.output_path.to_string_lossy().into_owned(),
            detail: "FFmpeg melaporkan sukses tapi file output tidak ditemukan".into(),
        });
    }

    Ok(req.output_path)
}

fn build_args(req: &ExportRequest) -> Vec<String> {
    let mut args = vec![
        "-y".to_string(), // overwrite tanpa prompt (kita sudah handle konfirmasi di UI)
        "-i".to_string(),
        req.input_path.to_string_lossy().into_owned(),
        "-filter_complex".to_string(),
        req.filter_complex.clone(),
        "-progress".to_string(),
        "pipe:2".to_string(),
    ];
    args.extend(req.codec_args.clone());
    args.push(req.output_path.to_string_lossy().into_owned());
    args
}

/// Mock `Killable` untuk testing tanpa binary asli.
#[derive(Default)]
pub struct FakeProcess {
    killed: Arc<Mutex<bool>>,
}

#[async_trait]
impl Killable for FakeProcess {
    async fn kill(&self) -> bool {
        let mut killed = self.killed.lock().await;
        *killed = true;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn fake_ffmpeg_path() -> PathBuf {
        // Pilih fixture berdasarkan OS
        #[cfg(target_os = "windows")]
        {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("test-fixtures")
                .join("fake_ffmpeg.bat")
        }
        #[cfg(not(target_os = "windows"))]
        {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("test-fixtures")
                .join("fake_ffmpeg.sh")
        }
    }

    fn unique_tmp_path(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{name}_{nanos}"))
    }

    #[tokio::test]
    async fn run_export_sukses_mengembalikan_output_path_dan_progress_bertahap() {
        let output_path = unique_tmp_path("sidecar_ok.mp3");

        let req = ExportRequest {
            job_id: "job-ok".into(),
            ffmpeg_binary_path: fake_ffmpeg_path(),
            input_path: PathBuf::from("/tmp/fake_input.mp3"),
            output_path: output_path.clone(),
            filter_complex: "atrim=start=0ms:end=1000ms".into(),
            codec_args: vec!["-codec:a".into(), "libmp3lame".into()],
            total_duration_ms: 10_000,
        };

        let registry = JobRegistry::default();
        let mut progress_events: Vec<u32> = Vec::new();

        let result = run_export(req, &registry, |p| progress_events.push(p)).await;

        assert!(result.is_ok(), "run_export gagal: {:?}", result.err());
        assert_eq!(result.unwrap(), output_path);
        assert!(output_path.exists());
        assert_eq!(progress_events, vec![10, 50, 100]);

        let _ = std::fs::remove_file(&output_path);
    }

    #[tokio::test]
    async fn run_export_gagal_mengembalikan_ffmpeg_execution_failed_dengan_stderr_tail() {
        let output_path = unique_tmp_path("sidecar_fail.mp3");

        let req = ExportRequest {
            job_id: "job-fail".into(),
            ffmpeg_binary_path: fake_ffmpeg_path(),
            input_path: PathBuf::from("/tmp/fake_input.mp3"),
            output_path: output_path.clone(),
            filter_complex: "--fail".into(), // trik: fake script cek semua argumen
            codec_args: vec![],
            total_duration_ms: 10_000,
        };

        let registry = JobRegistry::default();
        let result = run_export(req, &registry, |_p| {}).await;

        assert!(result.is_err());
        match result {
            Err(AppError::FfmpegExecutionFailed {
                exit_code,
                stderr_tail,
            }) => {
                assert_eq!(exit_code, Some(1));
                assert!(stderr_tail.contains("Invalid data"));
            }
            other => panic!("expected FfmpegExecutionFailed, got {other:?}"),
        }
        assert!(!output_path.exists());
    }

    #[tokio::test]
    async fn job_dihapus_dari_registry_setelah_selesai() {
        let output_path = unique_tmp_path("sidecar_registry.mp3");
        let job_id = "job-registry-check".to_string();

        let req = ExportRequest {
            job_id: job_id.clone(),
            ffmpeg_binary_path: fake_ffmpeg_path(),
            input_path: PathBuf::from("/tmp/fake_input.mp3"),
            output_path: output_path.clone(),
            filter_complex: "atrim=start=0ms:end=1000ms".into(),
            codec_args: vec![],
            total_duration_ms: 10_000,
        };

        let registry = JobRegistry::default();
        let _ = run_export(req, &registry, |_p| {}).await;

        // Setelah selesai, cancel terhadap job_id yang sama harus gagal
        // (return false) karena entry sudah di-unregister.
        let cancel_result = registry.cancel(&job_id).await;
        assert!(!cancel_result);

        let _ = std::fs::remove_file(&output_path);
    }

    #[tokio::test]
    async fn cancel_menghentikan_proses_yang_sedang_berjalan() {
        // Jalankan perintah long-running LANGSUNG lewat shell sistem yang sudah
        // ada (sh di Unix, cmd di Windows), BUKAN menulis lalu mengeksekusi file
        // script sendiri. Write+exec file di runner CI (overlayfs/tmpfs) memicu
        // ETXTBSY "Text file busy" pada exec -> spawn().unwrap() panic
        // (sidecar.rs:378). Itu penyebab asli, BUKAN race cancel.
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", "ping -n 30 127.0.0.1 >nul"]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", "sleep 30; echo done"]);
            c
        };
        let child = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("gagal spawn shell long-running");
        let pid_before = child.id();
        assert!(pid_before.is_some(), "proses harus berhasil di-spawn");

        // Simpan handle child supaya bisa memverifikasi proses benar-benar mati
        // setelah cancel (deterministik, tanpa sleep buta).
        let child_handle: Arc<Mutex<Child>> = Arc::new(Mutex::new(child));
        let killable: Arc<dyn Killable + Send + Sync> = child_handle.clone();

        let registry = JobRegistry::default();
        let job_id = "cancel-test-job";

        registry.register(job_id, killable).await;

        let cancelled = registry.cancel(job_id).await;
        assert!(
            cancelled,
            "cancel terhadap job yang sedang berjalan harus berhasil"
        );

        // Verifikasi proses BENAR-BENAR di-terminate: tunggu reaping dgn timeout.
        let reaped = tokio::time::timeout(std::time::Duration::from_secs(10), async {
            child_handle.lock().await.wait().await
        })
        .await;
        assert!(
            reaped.is_ok(),
            "proses harus selesai (reaped) dalam 10 dtk setelah cancel"
        );
    }

    #[tokio::test]
    async fn cancel_terhadap_job_id_yang_tidak_ada_mengembalikan_false() {
        let registry = JobRegistry::default();
        let cancelled = registry.cancel("job-tidak-pernah-ada").await;
        assert!(!cancelled);
    }

    #[tokio::test]
    async fn killable_mock_bekerja() {
        let fake = FakeProcess::default();
        let killed = fake.killed.clone();
        assert!(!*killed.lock().await);
        let result = fake.kill().await;
        assert!(result);
        assert!(*killed.lock().await);
    }
}
