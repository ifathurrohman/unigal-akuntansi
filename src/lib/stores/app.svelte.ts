import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';
import type { Perusahaan, Periode } from '$lib/types';

export function showNotification(type: 'success' | 'error', message: string) {
  if (type === 'success') {
    toast.success(message);
  } else {
    toast.error(message);
  }
}

export function formatCurrency(value: number): string {
  return new Intl.NumberFormat('id-ID', {
    style: 'currency',
    currency: 'IDR',
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(value);
}

export function formatDate(dateStr: string): string {
  if (!dateStr) return '-';
  const d = new Date(dateStr);
  return d.toLocaleDateString('id-ID', { day: '2-digit', month: 'short', year: 'numeric' });
}

export function formatNumber(value: number): string {
  return new Intl.NumberFormat('id-ID').format(value);
}

export async function callCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args);
}

/** Profil perusahaan, tanggal kerja, dan periode aktif — dimuat sekali dan dipakai semua halaman. */
export const appState = $state<{
  perusahaan: Perusahaan | null;
  periode: Periode | null;
  tanggalKerja: string;
}>({
  perusahaan: null,
  periode: null,
  tanggalKerja: new Date().toISOString().slice(0, 10),
});

export async function loadAppState() {
  try {
    const [perusahaan, tanggalKerja] = await Promise.all([
      invoke<Perusahaan>('get_perusahaan'),
      invoke<string>('get_tanggal_kerja'),
    ]);
    appState.perusahaan = perusahaan;
    appState.tanggalKerja = tanggalKerja;
  } catch (e) {
    console.error(e);
  }
  try {
    appState.periode = await invoke<Periode>('get_periode_aktif');
  } catch {
    appState.periode = null;
  }
}

/** Tanggal default untuk form: tanggal kerja (simulasi) atau tanggal sistem. */
export function hariIni(): string {
  return appState.tanggalKerja;
}

export function namaPerusahaan(): string {
  return appState.perusahaan?.nama?.trim() || 'Unigal Akuntansi';
}
