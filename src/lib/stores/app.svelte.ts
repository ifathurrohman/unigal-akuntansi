import { invoke } from '@tauri-apps/api/core';

// Gunakan object agar bisa di-mutate tanpa reassign (Svelte 5 restriction)
export const notif = $state<{ type: 'success' | 'error'; message: string; visible: boolean }>({
  type: 'success',
  message: '',
  visible: false,
});

let _timer: ReturnType<typeof setTimeout> | null = null;

export function showNotification(type: 'success' | 'error', message: string) {
  if (_timer) clearTimeout(_timer);
  notif.type = type;
  notif.message = message;
  notif.visible = true;
  _timer = setTimeout(() => { notif.visible = false; }, 3000);
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
