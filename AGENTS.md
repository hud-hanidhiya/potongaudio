# AGENTS.md — Potong Audio

## Proyek ini apa
Aplikasi desktop offline cross-platform (Windows/Linux) untuk trim, efek
audio (fade, gain, speed), dan konversi format audio.

## Stack
- Frontend: Vite + React 18 + TypeScript strict + Tailwind CSS 4 + Zustand + WaveSurfer.js
- Backend: Rust edition 2021 + Tauri v2
- Audio processing: FFmpeg NATIVE via Tauri sidecar (BUKAN FFmpeg.wasm)
- Dependency Tauri di `Cargo.toml` bersifat OPTIONAL via feature flag
  `tauri-runtime` — JANGAN hapus pola ini, ini yang membuat `cargo test`
  tetap cepat tanpa runtime GUI.

## Di mana semuanya berada
- Docs: `docs/` (workflow kit 2-tier — lihat `docs/00-guardrails.md` dulu)
- Guardrails: `docs/00-guardrails.md` (subset yang ditegakkan:
  `.kilocode/rules/guardrails.md`)
- Task plan: `docs/06-task-plan.md` — kerjakan urut dari atas, satu baris
  per waktu.
- Implementation plan aktif (kalau sedang mengerjakan task Tier 2):
  `docs/04a-implementation-plan.md`
- Histori teknis detail (opsional, kalau butuh alasan mendalam):
  `PLAN_AUDIO_CUTTER.md`, `TECH_IMPLEMENTATION_PLAN.md`, `POC_FINDINGS.md`

## Aturan
1. Kerjakan satu baris `docs/06-task-plan.md` per waktu. Berhenti setelah
   tiap baris dan laporkan status.
2. Jangan sentuh file di luar scope baris tersebut tanpa bertanya dulu.
3. Ikuti `.kilocode/rules/guardrails.md` secara ketat.
4. **Verifikasi wajib**: tiap perubahan Rust → `cargo test` (dan
   `cargo build --features tauri-runtime` kalau menyentuh command layer
   Tauri); tiap perubahan frontend → `npm run build`. Tempel output
   perintah di laporan sebelum lanjut — jangan lapor "selesai" tanpa bukti.
5. Kalau `EffectParams` berubah, WAJIB disinkronkan di
   `src/types/audio.types.ts` DAN `src-tauri/src/commands/export.rs`
   dalam commit yang sama.
6. Commit setelah tiap baris task-plan lulus, sertakan ID task di pesan
   commit (misal `git commit -m "[T1.5] Wire tauri://file-drop di Dropzone"`).
7. Kalau ternyata satu task menyentuh Risk Trigger yang tidak ditandai di
   task plan, berhenti dan tanya — jangan improvisasi sendiri.
8. Bahasa pesan error yang ditampilkan ke user: Bahasa Indonesia.
9. Jangan asumsikan struktur project dari dokumen lama (`PLAN_AUDIO_CUTTER.md`
   dkk) tanpa cross-check ke kondisi repo sebenarnya — struktur sudah
   berubah beberapa kali sejak Fase 0 (frontend pindah ke root,
   `commands/mod.rs` bertambah modul `version`, dst).

## Definition of done
Lihat `docs/02-scope-brief.md` bagian "Definition of done".
