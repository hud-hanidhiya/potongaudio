Kamu membantu saya membangun POTONG AUDIO, aplikasi desktop personal untuk
trim/efek/konversi audio, offline penuh, cross-platform Windows & Linux.

CONTEXT
- Idea/problem brief: docs/01-idea-brief.md
- Scope: docs/02-scope-brief.md
- Spec: docs/03-spec.md
- Architecture notes: docs/04-architecture-notes.md
- Implementation plan (isi ulang tiap task baru): docs/04a-implementation-plan.md
- Guardrails: docs/00-guardrails.md
- Histori detail (opsional): PLAN_AUDIO_CUTTER.md, TECH_IMPLEMENTATION_PLAN.md, POC_FINDINGS.md

STACK
- Frontend: Vite + React 18 + TypeScript strict + Tailwind CSS 4 + Zustand + WaveSurfer.js
- Backend: Rust edition 2021 + Tauri v2, dependency Tauri OPTIONAL via
  feature flag `tauri-runtime` (jangan hapus pola ini)
- Audio processing: FFmpeg NATIVE via Tauri sidecar (BUKAN FFmpeg.wasm)

SCOPE
Lihat docs/02-scope-brief.md — ringkas: trim single-region, efek gain/fade/
speed, export mp3/wav/m4a/flac/m4r.

OUT OF SCOPE — JANGAN DIKERJAKAN TANPA PERSETUJUAN EKSPLISIT
- Multi-region trim, undo/redo, equalizer (resmi ditunda ke v2)
- macOS, code signing/notarization (ditunda)

IMPLEMENTATION RULES
1. Perubahan kecil dan lokal ke task yang sedang dikerjakan.
2. Jangan sentuh file tidak terkait atau ganti struktur yang sudah ada
   tanpa instruksi eksplisit.
3. Sebelum perubahan besar, jelaskan rencana dan file terdampak dulu.
4. Ikuti docs/00-guardrails.md secara ketat.
5. Jangan tambah dependency baru tanpa menjelaskan alasannya.
6. **Verifikasi wajib, bukan opsional**: tiap perubahan Rust → `cargo test`
   (dan `cargo build --features tauri-runtime` kalau menyentuh command
   layer Tauri); tiap perubahan frontend → `npm run build`. Tempel output
   perintah — jangan lapor "selesai" tanpa bukti.
7. Kalau `EffectParams` berubah, WAJIB sinkron di kedua sisi sekaligus.
8. Kalau menemukan keputusan desain yang belum ada jawabannya di
   docs/03-spec.md (Pertanyaan Terbuka) atau dokumen manapun, tandai
   sebagai pertanyaan terbuka — jangan berasumsi sendiri.
9. Bahasa pesan error ke user: Bahasa Indonesia.

WORK ORDER
Stage 1: Ulangi pemahaman, sebutkan asumsi/pertanyaan, cross-check kondisi
         REPO SEBENARNYA (jangan asumsi dari dokumen lama), buat/isi
         docs/04a-implementation-plan.md untuk task ini.
Stage 2: Scaffold/implementasi core.
Stage 3: Test untuk logika kritis, ikuti pola fake-script untuk test yang
         butuh spawn proses eksternal.
Stage 4: Polish + edge case.
Stage 5: Update docs/06-task-plan.md dan README. Catat bug nyata yang
         ditemukan di docs/07-debug-log.md.

DEFINITION OF DONE
Lihat docs/02-scope-brief.md bagian "Definition of done".
