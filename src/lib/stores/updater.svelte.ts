import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export const updateState = $state<{
  available: boolean;
  version: string;
  notes: string | null;
  phase: 'idle' | 'downloading' | 'done' | 'error';
  progress: number;
  errorMsg: string;
  update: Update | null;
}>({
  available: false,
  version: '',
  notes: null,
  phase: 'idle',
  progress: 0,
  errorMsg: '',
  update: null,
});

/** auto = true: langsung unduh, pasang, dan restart tanpa menunggu konfirmasi pengguna. */
export async function checkForUpdate(auto = false) {
  if (updateState.phase === 'downloading' || updateState.phase === 'done') return;
  try {
    const update = await check();
    if (update) {
      updateState.available = true;
      updateState.version = update.version;
      updateState.notes = update.body ?? null;
      updateState.update = update;
      if (auto) await downloadAndInstall();
    }
  } catch {
    // Gagal cek update (offline / endpoint belum dikonfigurasi) — abaikan saja
  }
}

export async function downloadAndInstall() {
  const update = updateState.update;
  if (!update) return;

  updateState.phase = 'downloading';
  updateState.progress = 0;

  try {
    let downloaded = 0;
    let total = 0;

    await update.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          total = event.data.contentLength ?? 0;
          break;
        case 'Progress':
          downloaded += event.data.chunkLength;
          updateState.progress = total > 0 ? Math.min((downloaded / total) * 100, 99) : 50;
          break;
        case 'Finished':
          updateState.progress = 100;
          break;
      }
    });

    updateState.phase = 'done';
    // Restart setelah 1.5 detik agar user sempat melihat pesan sukses
    setTimeout(() => relaunch(), 1500);
  } catch (e) {
    updateState.phase = 'error';
    updateState.errorMsg = String(e);
  }
}

export function dismissUpdate() {
  updateState.available = false;
  updateState.phase = 'idle';
}

/** Cek ulang berkala untuk aplikasi yang dibiarkan terbuka lama (mis. komputer lab); hanya memberi tahu, tidak memaksa restart. */
export function startPeriodicUpdateCheck(intervalMs = 30 * 60 * 1000) {
  return setInterval(() => {
    if (!updateState.available) checkForUpdate(false);
  }, intervalMs);
}
