//! Menyusun filter graph FFmpeg dari `EffectParams` yang dikirim frontend.
//!
//! CATATAN DESAIN — baca sebelum ubah urutan filter:
//! Urutan chain filter MEMENGARUHI hasil akhir, bukan sekadar gaya penulisan:
//!   1. atrim   — potong region dulu, supaya semua filter setelahnya bekerja
//!                pada rentang yang benar (durasi acuan untuk fade-out dihitung
//!                dari SINI, bukan dari file asli).
//!   2. atempo  — ubah speed SEBELUM fade, karena fade dihitung dalam durasi
//!                waktu-nyata hasil akhir, bukan waktu asli sebelum speed-up.
//!   3. afade   — in & out, dihitung dari durasi PASCA-trim-dan-speed.
//!   4. volume  — gain, paling akhir supaya tidak clipping filter lain
//!                (fade curve dihitung di atas sinyal yang belum di-gain).

use crate::commands::export::{EffectParams, OutputFormat};
use crate::error::{AppError, AppResult};

/// Representasi hasil build: argumen siap dipakai untuk spawn FFmpeg,
/// dipisah dari string mentah supaya gampang di-unit-test tiap bagian.
#[derive(Debug, PartialEq)]
pub struct FilterPlan {
    pub filter_complex: String,
    pub output_ext: &'static str,
    pub codec_args: Vec<String>,
}

/// Ambang batas rasio atempo per FFmpeg: filter ini hanya valid di rentang
/// 0.5–2.0. Di luar itu WAJIB di-chain berkali-kali.
const ATEMPO_MIN: f32 = 0.5;
const ATEMPO_MAX: f32 = 2.0;

pub fn build_filter_plan(params: &EffectParams, total_duration_ms: u64) -> AppResult<FilterPlan> {
    validate_params(params, total_duration_ms)?;

    let mut filters: Vec<String> = Vec::new();

    // 1. Trim — selalu ada, ini yang menentukan "durasi acuan" untuk step berikutnya.
    let trimmed_duration_ms = params.region.end_ms.saturating_sub(params.region.start_ms);
    filters.push(format!(
        "atrim=start={}ms:end={}ms,asetpts=PTS-STARTPTS",
        params.region.start_ms, params.region.end_ms
    ));

    // 2. Speed (atempo) — chain jika di luar rentang valid FFmpeg.
    //    Durasi setelah speed change dipakai untuk hitung fade-out di step 3.
    let mut duration_after_speed_ms = trimmed_duration_ms;
    if (params.speed.ratio - 1.0).abs() > f32::EPSILON {
        let chain = build_atempo_chain(params.speed.ratio)?;
        filters.extend(chain);
        duration_after_speed_ms =
            (trimmed_duration_ms as f32 / params.speed.ratio) as u64;
    }

    // 3. Fade in/out — dihitung dari duration_after_speed_ms, BUKAN trimmed_duration_ms.
    if params.fade.in_ms > 0 {
        filters.push(format!("afade=t=in:st=0:d={}ms", params.fade.in_ms));
    }
    if params.fade.out_ms > 0 {
        // start time fade-out = akhir_durasi - durasi_fade, di-clamp ke 0
        // supaya tidak underflow kalau fade_out lebih panjang dari durasi total.
        let fade_out_start_ms = duration_after_speed_ms.saturating_sub(params.fade.out_ms);
        filters.push(format!(
            "afade=t=out:st={}ms:d={}ms",
            fade_out_start_ms, params.fade.out_ms
        ));
    }

    // 4. Gain — skip sama sekali dari chain kalau 0dB (no-op tidak perlu
    //    disertakan; lebih murah dan lebih mudah didiagnosis di log FFmpeg).
    if params.gain_db.abs() > f32::EPSILON {
        filters.push(format!("volume={}dB", params.gain_db));
    }

    let filter_complex = filters.join(",");
    let (output_ext, codec_args) = codec_args_for_format(&params.output_format, params.output_bitrate_kbps);

    Ok(FilterPlan { filter_complex, output_ext, codec_args })
}

/// atempo FFmpeg hanya valid 0.5x–2.0x. Rasio di luar itu perlu di-chain,
/// misal 3.0x = atempo=2.0,atempo=1.5 (2.0 * 1.5 = 3.0).
/// Dipecah jadi fungsi terpisah supaya gampang di-unit-test.
fn build_atempo_chain(target_ratio: f32) -> AppResult<Vec<String>> {
    if target_ratio <= 0.0 {
        return Err(AppError::InvalidParams {
            detail: format!("speed ratio harus > 0, diterima: {target_ratio}"),
        });
    }

    let mut remaining = target_ratio;
    let mut steps: Vec<String> = Vec::new();

    // Pecah rasio jadi langkah-langkah dalam rentang valid.
    // Batasi iterasi untuk cegah infinite loop pada input ekstrem/aneh.
    for _ in 0..8 {
        if remaining >= ATEMPO_MIN && remaining <= ATEMPO_MAX {
            steps.push(format!("atempo={remaining:.4}"));
            return Ok(steps);
        }
        if remaining > ATEMPO_MAX {
            steps.push(format!("atempo={ATEMPO_MAX}"));
            remaining /= ATEMPO_MAX;
        } else {
            steps.push(format!("atempo={ATEMPO_MIN}"));
            remaining /= ATEMPO_MIN;
        }
    }

    Err(AppError::InvalidParams {
        detail: format!("speed ratio {target_ratio} di luar rentang yang bisa diproses"),
    })
}

fn codec_args_for_format(
    format: &OutputFormat,
    bitrate_kbps: Option<u32>,
) -> (&'static str, Vec<String>) {
    match format {
        OutputFormat::Mp3 => (
            "mp3",
            vec![
                "-codec:a".into(),
                "libmp3lame".into(),
                "-b:a".into(),
                format!("{}k", bitrate_kbps.unwrap_or(192)),
            ],
        ),
        OutputFormat::Wav => ("wav", vec!["-codec:a".into(), "pcm_s16le".into()]),
        OutputFormat::M4a => (
            "m4a",
            vec![
                "-codec:a".into(),
                "aac".into(),
                "-b:a".into(),
                format!("{}k", bitrate_kbps.unwrap_or(192)),
            ],
        ),
        OutputFormat::Flac => ("flac", vec!["-codec:a".into(), "flac".into()]),
        // M4R secara codec identik dengan M4A (AAC in MP4 container).
        // Penanganan rename ekstensi + metadata `stik` dilakukan di layer
        // caller (commands/export.rs), BUKAN di sini — filter builder hanya
        // tanggung jawab atas argumen filter+codec, bukan post-processing file.
        OutputFormat::M4r => (
            "m4a", // sengaja masih .m4a di tahap encode; rename terjadi setelahnya
            vec![
                "-codec:a".into(),
                "aac".into(),
                "-b:a".into(),
                format!("{}k", bitrate_kbps.unwrap_or(192)),
            ],
        ),
    }
}

fn validate_params(params: &EffectParams, total_duration_ms: u64) -> AppResult<()> {
    if params.region.end_ms <= params.region.start_ms {
        return Err(AppError::InvalidParams {
            detail: format!(
                "region end_ms ({}) harus lebih besar dari start_ms ({})",
                params.region.end_ms, params.region.start_ms
            ),
        });
    }
    // Defense-in-depth (H3): region tidak boleh melebihi durasi file.
    // Pembanding `>` ketat — end == durasi sah (trim sampai akhir file).
    if params.region.end_ms > total_duration_ms {
        return Err(AppError::InvalidParams {
            detail: format!(
                "region end_ms ({}) melebihi durasi file ({} ms)",
                params.region.end_ms, total_duration_ms
            ),
        });
    }
    if params.speed.ratio <= 0.0 {
        return Err(AppError::InvalidParams {
            detail: "speed ratio harus positif".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::export::{Fade, Region, Speed};

    fn base_params() -> EffectParams {
        EffectParams {
            source_file_path: "/tmp/in.mp3".into(),
            region: Region { start_ms: 1000, end_ms: 5000 }, // durasi 4000ms
            gain_db: 0.0,
            fade: Fade { in_ms: 0, out_ms: 0 },
            speed: Speed { ratio: 1.0, preserve_pitch: true },
            output_format: OutputFormat::Mp3,
            output_bitrate_kbps: None,
        }
    }

    /// Durasi file acuan untuk test — region default (1000..5000) berada
    /// di dalamnya. Signature `build_filter_plan` menerima durasi file
    /// sejak H3 (validasi end_ms tidak boleh melebihi durasi).
    const DURATION_MS: u64 = 10_000;

    #[test]
    fn trim_selalu_ada_di_chain() {
        let plan = build_filter_plan(&base_params(), DURATION_MS).unwrap();
        assert!(plan.filter_complex.contains("atrim=start=1000ms:end=5000ms"));
    }

    #[test]
    fn gain_0db_di_skip_dari_chain() {
        let plan = build_filter_plan(&base_params(), DURATION_MS).unwrap();
        assert!(!plan.filter_complex.contains("volume="));
    }

    #[test]
    fn gain_nonzero_masuk_chain() {
        let mut p = base_params();
        p.gain_db = 6.0;
        let plan = build_filter_plan(&p, DURATION_MS).unwrap();
        assert!(plan.filter_complex.contains("volume=6dB"));
    }

    #[test]
    fn fade_out_dihitung_dari_durasi_setelah_trim_tanpa_speed_change() {
        let mut p = base_params();
        p.fade.out_ms = 500;
        let plan = build_filter_plan(&p, DURATION_MS).unwrap();
        // durasi trim = 4000ms, fade_out 500ms -> start di 3500ms
        assert!(plan.filter_complex.contains("afade=t=out:st=3500ms:d=500ms"));
    }

    #[test]
    fn fade_out_dihitung_ulang_setelah_speed_change() {
        let mut p = base_params();
        p.speed.ratio = 2.0; // durasi 4000ms -> jadi 2000ms setelah 2x speed
        p.fade.out_ms = 500;
        let plan = build_filter_plan(&p, DURATION_MS).unwrap();
        // durasi setelah speed = 2000ms, fade_out 500ms -> start di 1500ms
        assert!(plan.filter_complex.contains("afade=t=out:st=1500ms:d=500ms"));
    }

    #[test]
    fn fade_out_lebih_panjang_dari_durasi_di_clamp_ke_nol() {
        let mut p = base_params();
        p.fade.out_ms = 10_000; // lebih panjang dari durasi trim 4000ms
        let plan = build_filter_plan(&p, DURATION_MS).unwrap();
        assert!(plan.filter_complex.contains("afade=t=out:st=0ms:d=10000ms"));
    }

    #[test]
    fn atempo_dalam_rentang_valid_tidak_di_chain() {
        let chain = build_atempo_chain(1.5).unwrap();
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0], "atempo=1.5000");
    }

    #[test]
    fn atempo_di_atas_2x_di_chain_dua_langkah() {
        // 3.0 = 2.0 * 1.5
        let chain = build_atempo_chain(3.0).unwrap();
        assert_eq!(chain, vec!["atempo=2".to_string(), "atempo=1.5000".to_string()]);
    }

    #[test]
    fn atempo_di_bawah_half_di_chain() {
        // 0.25 = 0.5 * 0.5
        let chain = build_atempo_chain(0.25).unwrap();
        assert_eq!(chain, vec!["atempo=0.5".to_string(), "atempo=0.5000".to_string()]);
    }

    #[test]
    fn region_end_kurang_dari_start_ditolak() {
        let mut p = base_params();
        p.region = Region { start_ms: 5000, end_ms: 1000 };
        let result = build_filter_plan(&p, DURATION_MS);
        assert!(result.is_err());
    }

    #[test]
    fn speed_ratio_nol_ditolak() {
        let mut p = base_params();
        p.speed.ratio = 0.0;
        let result = build_filter_plan(&p, DURATION_MS);
        assert!(result.is_err());
    }

    #[test]
    fn m4r_pakai_codec_aac_sama_seperti_m4a() {
        let mut p = base_params();
        p.output_format = OutputFormat::M4r;
        let plan = build_filter_plan(&p, DURATION_MS).unwrap();
        assert!(plan.codec_args.contains(&"aac".to_string()));
    }

    #[test]
    fn urutan_filter_trim_sebelum_atempo_sebelum_fade_sebelum_volume() {
        let mut p = base_params();
        p.speed.ratio = 1.5;
        p.fade.in_ms = 200;
        p.gain_db = 3.0;
        let plan = build_filter_plan(&p, DURATION_MS).unwrap();

        let atrim_pos = plan.filter_complex.find("atrim").unwrap();
        let atempo_pos = plan.filter_complex.find("atempo").unwrap();
        let afade_pos = plan.filter_complex.find("afade").unwrap();
        let volume_pos = plan.filter_complex.find("volume").unwrap();

        assert!(atrim_pos < atempo_pos);
        assert!(atempo_pos < afade_pos);
        assert!(afade_pos < volume_pos);
    }

    // --- H3: region vs durasi file ---

    #[test]
    fn region_end_melebihi_durasi_file_ditolak() {
        let mut p = base_params();
        p.region = Region { start_ms: 1000, end_ms: 20_000 }; // > DURATION_MS (10_000)
        let result = build_filter_plan(&p, DURATION_MS);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(err.contains("melebihi durasi file"));
    }

    #[test]
    fn region_end_sama_dengan_durasi_file_lolos() {
        // Pembanding `>` ketat: end == durasi sah.
        let mut p = base_params();
        p.region = Region { start_ms: 1000, end_ms: 10_000 };
        let plan = build_filter_plan(&p, DURATION_MS).unwrap();
        assert!(plan.filter_complex.contains("end=10000ms"));
    }

    #[test]
    fn durasi_nol_menolak_region_positif() {
        // Dokumentasi perilaku: durasi 0 berarti tidak ada audio; region
        // apapun dengan end_ms >= 1 otomatis > durasi → ditolak oleh cek
        // `end_ms > total_duration_ms` (region 0..0 sudah ditolak cek end>start).
        let mut p = base_params();
        p.region = Region { start_ms: 0, end_ms: 1 };
        let result = build_filter_plan(&p, 0);
        assert!(result.is_err());
    }
}
