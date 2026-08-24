pub mod commands;
pub mod error;
pub mod ffmpeg;

// ---------------------------------------------------------------------
// ⚠️ BELUM TERVERIFIKASI COMPILE dengan Tauri sungguhan — lihat catatan
// di commands/export.rs dan Cargo.toml soal keterbatasan sandbox.
// Semua yang di-import di sini hanya ada kalau feature "tauri-runtime"
// aktif (`cargo build --features tauri-runtime` atau `cargo tauri dev`).
// ---------------------------------------------------------------------
#[cfg(feature = "tauri-runtime")]
pub fn run() {
    use commands::export::{cancel_export, export_audio};
    use commands::probe::probe_audio_file;
    use commands::version::get_ffmpeg_version;
    use ffmpeg::sidecar::JobRegistry;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(JobRegistry::default())
        .invoke_handler(tauri::generate_handler![
            export_audio,
            cancel_export,
            probe_audio_file,
            get_ffmpeg_version,
        ])
        .run(tauri::generate_context!())
        .expect("gagal menjalankan aplikasi Tauri");
}
