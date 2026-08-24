import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

// Konfigurasi Vite untuk PotongAudio — dua penyesuaian khusus Tauri
// (bukan default Vite biasa), lihat komentar inline di bawah.
export default defineConfig({
  plugins: [react(), tailwindcss()],

  // Frontend entry (index.html, src/) tetap di src/ — hanya config yang
  // dipindah ke root repo, jadi Vite perlu diarahkan ke root proyek aslinya.
  root: 'src',

  // Tauri butuh port dev server yang FIXED dan bisa gagal-cepat kalau
  // sudah dipakai (bukan auto-increment ke port lain seperti default Vite),
  // karena tauri.conf.json -> build.devUrl menunjuk ke port spesifik.
  server: {
    port: 1420,
    strictPort: true,
  },

  // Tauri v2 mem-bundle aplikasi sendiri; env var TAURI_ENV_* dipakai
  // untuk membedakan build target (desktop vs mobile) kalau nanti
  // diperluas ke Android (disebut sebagai eksplorasi di riwayat project).
  envPrefix: ['VITE_', 'TAURI_ENV_'],

  build: {
    // root diset ke 'src' di atas, jadi outDir default ikut jadi src/dist —
    // override eksplisit ke ../dist (root repo) supaya cocok dengan
    // frontendDist: "../dist" di tauri.conf.json.
    outDir: '../dist',
    // WebView2 (Windows) versi lama kadang bermasalah dengan target esnext
    // penuh — target ini kompromi aman lintas WebView2/WebKit/WebKitGTK.
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
