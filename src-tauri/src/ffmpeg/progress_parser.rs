//! Parser untuk output `-progress pipe:2` FFmpeg.
//!
//! CATATAN PENTING soal kuirk penamaan field FFmpeg (sumber bug yang sering
//! tidak disadari):
//!   - Field `out_time_ms` **namanya menyesatkan** — pada banyak versi FFmpeg
//!     nilainya sebenarnya dalam MIKRODETIK, bukan milidetik.
//!   - FFmpeg versi lebih baru menambahkan `out_time_us` yang eksplisit dalam
//!     mikrodetik untuk menghindari ambiguitas ini.
//!   - Field `out_time` (string, format `HH:MM:SS.microseconds`) selalu
//!     konsisten di semua versi, tapi lebih mahal untuk di-parse per baris.
//!
//! Strategi di sini: **prioritaskan `out_time_us` jika ada**, dan perlakukan
//! `out_time_ms` SEBAGAI MIKRODETIK JUGA (bukan milidetik) mengikuti perilaku
//! aktual FFmpeg, bukan nama field-nya. Ini WAJIB diverifikasi ulang di
//! Fase 0 (Section 2.2 PLAN_AUDIO_CUTTER.md) terhadap versi FFmpeg yang
//! benar-benar dipakai sebagai sidecar, karena ada laporan perilaku berbeda
//! antar versi.

use std::collections::HashMap;

/// Melacak progress satu job export dan memutuskan kapan sebuah update
/// "layak" di-emit ke frontend (supaya tidak flood event tiap baris).
pub struct ProgressTracker {
    total_duration_ms: u64,
    last_emitted_percent: i64,
    /// Ambang minimum kenaikan persen sebelum emit lagi.
    emit_threshold_percent: u32,
}

#[derive(Debug, PartialEq)]
pub enum ProgressUpdate {
    /// Persen naik cukup signifikan, layak di-emit ke frontend.
    Percent(u32),
    /// Baris ini punya `progress=end`, proses FFmpeg selesai.
    Done,
    /// Baris tidak mengandung informasi progress baru yang perlu ditindaklanjuti.
    NoUpdate,
}

impl ProgressTracker {
    pub fn new(total_duration_ms: u64) -> Self {
        Self {
            total_duration_ms: total_duration_ms.max(1), // hindari div-by-zero
            last_emitted_percent: -1,
            emit_threshold_percent: 1,
        }
    }

    /// FFmpeg dengan `-progress pipe:2` menulis banyak baris `key=value`
    /// per "frame" progress, diakhiri baris `progress=continue` atau
    /// `progress=end`. Fungsi ini dipanggil PER BARIS (bukan per frame),
    /// jadi caller perlu mengakumulasi key=value sampai ketemu baris
    /// `progress=...` sebelum menganggap satu "frame" progress lengkap.
    ///
    /// Untuk skeleton ini kita sederhanakan: setiap baris `out_time_us=`
    /// atau `out_time_ms=` langsung dihitung sebagai update, tanpa menunggu
    /// baris `progress=continue`. Ini valid karena FFmpeg selalu menulis
    /// `out_time_*` sebelum `progress=*` dalam satu frame, dan kita hanya
    /// butuh nilai waktu terbaru — bukan menunggu penanda batas frame.
    pub fn process_line(&mut self, line: &str) -> ProgressUpdate {
        let line = line.trim();

        if line == "progress=end" {
            return ProgressUpdate::Done;
        }

        if let Some((key, value)) = split_key_value(line) {
            let out_time_us: Option<u64> = match key {
                "out_time_us" => value.parse().ok(),
                // Nama field menyesatkan — lihat catatan modul di atas.
                // Nilainya diperlakukan sebagai mikrodetik, BUKAN milidetik.
                "out_time_ms" => value.parse().ok(),
                _ => None,
            };

            if let Some(us) = out_time_us {
                let elapsed_ms = us / 1000;
                let percent = ((elapsed_ms as f64 / self.total_duration_ms as f64) * 100.0)
                    .clamp(0.0, 100.0) as u32;

                if percent as i64 - self.last_emitted_percent >= self.emit_threshold_percent as i64
                {
                    self.last_emitted_percent = percent as i64;
                    return ProgressUpdate::Percent(percent);
                }
            }
        }

        ProgressUpdate::NoUpdate
    }
}

fn split_key_value(line: &str) -> Option<(&str, &str)> {
    line.split_once('=')
}

/// Helper untuk parsing satu blok output FFmpeg utuh sekaligus (dipakai di
/// test, dan bisa juga dipakai untuk kasus non-streaming / batch parsing
/// kalau suatu saat dibutuhkan).
#[allow(dead_code)]
pub fn parse_full_output(output: &str, _total_duration_ms: u64) -> HashMap<&str, &str> {
    output.lines().filter_map(split_key_value).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_time_us_dihitung_sebagai_persen_dengan_benar() {
        let mut tracker = ProgressTracker::new(10_000); // total 10 detik
                                                        // 5_000_000 mikrodetik = 5000ms = 50% dari 10000ms
        let update = tracker.process_line("out_time_us=5000000");
        assert_eq!(update, ProgressUpdate::Percent(50));
    }

    #[test]
    fn out_time_ms_diperlakukan_sebagai_mikrodetik_bukan_milidetik() {
        let mut tracker = ProgressTracker::new(10_000);
        // Meski nama field "out_time_ms", nilainya tetap diperlakukan
        // sebagai mikrodetik mengikuti perilaku aktual FFmpeg.
        let update = tracker.process_line("out_time_ms=5000000");
        assert_eq!(update, ProgressUpdate::Percent(50));
    }

    #[test]
    fn progress_end_mengembalikan_done() {
        let mut tracker = ProgressTracker::new(10_000);
        let update = tracker.process_line("progress=end");
        assert_eq!(update, ProgressUpdate::Done);
    }

    #[test]
    fn baris_tanpa_info_relevan_menghasilkan_no_update() {
        let mut tracker = ProgressTracker::new(10_000);
        let update = tracker.process_line("bitrate=128.0kbits/s");
        assert_eq!(update, ProgressUpdate::NoUpdate);
    }

    #[test]
    fn tidak_emit_ulang_jika_kenaikan_persen_di_bawah_threshold() {
        let mut tracker = ProgressTracker::new(10_000);
        let first = tracker.process_line("out_time_us=5000000"); // 50%
        assert_eq!(first, ProgressUpdate::Percent(50));

        // Kenaikan sangat kecil (masih dibulatkan ke 50%) tidak boleh emit lagi.
        let second = tracker.process_line("out_time_us=5010000"); // 50.1% -> 50
        assert_eq!(second, ProgressUpdate::NoUpdate);
    }

    #[test]
    fn emit_lagi_setelah_kenaikan_melewati_threshold() {
        let mut tracker = ProgressTracker::new(10_000);
        tracker.process_line("out_time_us=5000000"); // 50%
        let update = tracker.process_line("out_time_us=6000000"); // 60%
        assert_eq!(update, ProgressUpdate::Percent(60));
    }

    #[test]
    fn persen_di_clamp_ke_100_walau_out_time_melebihi_durasi_total() {
        // Bisa terjadi karena floating point / estimasi durasi awal meleset tipis.
        let mut tracker = ProgressTracker::new(10_000);
        let update = tracker.process_line("out_time_us=12000000"); // 120% mentah
        assert_eq!(update, ProgressUpdate::Percent(100));
    }

    #[test]
    fn total_duration_nol_tidak_menyebabkan_panic() {
        let mut tracker = ProgressTracker::new(0);
        let update = tracker.process_line("out_time_us=1000000");
        // Tidak boleh panic (division by zero) — hasil di-clamp ke 100%.
        assert_eq!(update, ProgressUpdate::Percent(100));
    }

    #[test]
    fn baris_kosong_atau_malformed_tidak_panic() {
        let mut tracker = ProgressTracker::new(10_000);
        assert_eq!(tracker.process_line(""), ProgressUpdate::NoUpdate);
        assert_eq!(
            tracker.process_line("bukan_key_value_valid"),
            ProgressUpdate::NoUpdate
        );
        assert_eq!(
            tracker.process_line("out_time_us=bukan_angka"),
            ProgressUpdate::NoUpdate
        );
    }

    #[test]
    fn simulasi_stream_progress_realistis_end_to_end() {
        // Simulasi potongan output nyata dari `ffmpeg -progress pipe:2`.
        let raw_output = "\
frame=100
fps=25.0
out_time_us=1000000
progress=continue
frame=250
fps=25.0
out_time_us=5000000
progress=continue
frame=500
fps=25.0
out_time_us=10000000
progress=end";

        let mut tracker = ProgressTracker::new(10_000); // 10 detik
        let mut emitted: Vec<ProgressUpdate> = Vec::new();

        for line in raw_output.lines() {
            let update = tracker.process_line(line);
            if update != ProgressUpdate::NoUpdate {
                emitted.push(update);
            }
        }

        assert_eq!(
            emitted,
            vec![
                ProgressUpdate::Percent(10),
                ProgressUpdate::Percent(50),
                ProgressUpdate::Percent(100),
                ProgressUpdate::Done,
            ]
        );
    }
}
