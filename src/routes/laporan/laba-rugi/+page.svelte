<script lang="ts">
  import { onMount } from 'svelte';
  import { BarChart3, TrendingUp, TrendingDown, Printer } from 'lucide-svelte';
  import { callCommand, formatCurrency } from '$lib/stores/app.svelte';
  import type { LabaRugiItem } from '$lib/types';

  let items = $state<LabaRugiItem[]>([]);
  let loading = $state(true);

  const now = new Date();
  let tahun = $state(now.getFullYear());
  let bulan = $state<number | ''>(now.getMonth() + 1);

  const pendapatan = $derived(items.filter(i => i.tipe === 'Pendapatan'));
  const beban = $derived(items.filter(i => i.tipe === 'Beban'));
  const totalPendapatan = $derived(pendapatan.reduce((s, i) => s + i.saldo, 0));
  const totalBeban = $derived(beban.reduce((s, i) => s + i.saldo, 0));
  const laba = $derived(totalPendapatan - totalBeban);

  async function load() {
    loading = true;
    try {
      items = await callCommand<LabaRugiItem[]>('get_laba_rugi', {
        tahun,
        bulan: bulan === '' ? null : bulan,
      });
    } finally { loading = false; }
  }

  const bulanNama = ['Januari','Februari','Maret','April','Mei','Juni','Juli','Agustus','September','Oktober','November','Desember'];

  onMount(load);
</script>

<div class="p-6">
  <div class="page-header">
    <div>
      <h1 class="page-title">Laporan Laba Rugi</h1>
      <p class="page-subtitle">Income Statement</p>
    </div>
    <button class="btn-secondary" onclick={() => window.print()}>
      <Printer size={16} /> Cetak
    </button>
  </div>

  <!-- Filter -->
  <div class="card p-4 mb-6 flex items-center gap-4">
    <div>
      <label class="label" for="lr-tahun">Tahun</label>
      <select id="lr-tahun" class="input w-28" bind:value={tahun} onchange={load}>
        {#each [2024, 2025, 2026] as y}
          <option value={y}>{y}</option>
        {/each}
      </select>
    </div>
    <div>
      <label class="label" for="lr-bulan">Bulan</label>
      <select id="lr-bulan" class="input w-36" bind:value={bulan} onchange={load}>
        <option value="">Semua Bulan</option>
        {#each bulanNama as b, i}
          <option value={i + 1}>{b}</option>
        {/each}
      </select>
    </div>
    <div class="flex-1"></div>
    <div class="text-right">
      <div class="text-xs text-gray-500">Periode</div>
      <div class="text-sm font-semibold text-gray-900">
        {bulan ? bulanNama[Number(bulan) - 1] : 'Semua Bulan'} {tahun}
      </div>
    </div>
  </div>

  {#if loading}
    <div class="flex items-center justify-center h-48">
      <div class="w-8 h-8 border-2 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
    </div>
  {:else}
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-4 mb-6">
      <div class="stat-card">
        <div class="flex items-center gap-2 mb-1">
          <TrendingUp size={16} class="text-green-600" />
          <span class="text-xs text-gray-500">Total Pendapatan</span>
        </div>
        <div class="text-xl font-bold text-green-600">{formatCurrency(totalPendapatan)}</div>
      </div>
      <div class="stat-card">
        <div class="flex items-center gap-2 mb-1">
          <TrendingDown size={16} class="text-red-500" />
          <span class="text-xs text-gray-500">Total Beban</span>
        </div>
        <div class="text-xl font-bold text-red-500">{formatCurrency(totalBeban)}</div>
      </div>
      <div class="stat-card border-2 {laba >= 0 ? 'border-green-200 bg-green-50' : 'border-red-200 bg-red-50'}">
        <div class="flex items-center gap-2 mb-1">
          <BarChart3 size={16} class={laba >= 0 ? 'text-green-700' : 'text-red-700'} />
          <span class="text-xs text-gray-500">{laba >= 0 ? 'Laba Bersih' : 'Rugi Bersih'}</span>
        </div>
        <div class="text-xl font-bold {laba >= 0 ? 'text-green-700' : 'text-red-700'}">{formatCurrency(Math.abs(laba))}</div>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <!-- Pendapatan -->
      <div class="card overflow-hidden">
        <div class="px-5 py-4 border-b border-gray-100 bg-green-50">
          <h3 class="text-sm font-semibold text-green-800 flex items-center gap-2">
            <TrendingUp size={16} /> PENDAPATAN
          </h3>
        </div>
        <div class="divide-y divide-gray-50">
          {#each pendapatan as item}
            <div class="flex items-center justify-between px-5 py-3 hover:bg-gray-50">
              <div>
                <div class="text-xs text-gray-400 font-mono">{item.kode}</div>
                <div class="text-sm text-gray-700">{item.nama}</div>
              </div>
              <div class="text-sm font-semibold text-green-600">{formatCurrency(item.saldo)}</div>
            </div>
          {:else}
            <div class="px-5 py-8 text-center text-sm text-gray-400">Tidak ada data pendapatan</div>
          {/each}
          <div class="flex items-center justify-between px-5 py-3 bg-green-50">
            <span class="text-sm font-bold text-green-800">Total Pendapatan</span>
            <span class="text-sm font-bold text-green-700">{formatCurrency(totalPendapatan)}</span>
          </div>
        </div>
      </div>

      <!-- Beban -->
      <div class="card overflow-hidden">
        <div class="px-5 py-4 border-b border-gray-100 bg-red-50">
          <h3 class="text-sm font-semibold text-red-800 flex items-center gap-2">
            <TrendingDown size={16} /> BEBAN
          </h3>
        </div>
        <div class="divide-y divide-gray-50">
          {#each beban as item}
            <div class="flex items-center justify-between px-5 py-3 hover:bg-gray-50">
              <div>
                <div class="text-xs text-gray-400 font-mono">{item.kode}</div>
                <div class="text-sm text-gray-700">{item.nama}</div>
              </div>
              <div class="text-sm font-semibold text-red-500">{formatCurrency(item.saldo)}</div>
            </div>
          {:else}
            <div class="px-5 py-8 text-center text-sm text-gray-400">Tidak ada data beban</div>
          {/each}
          <div class="flex items-center justify-between px-5 py-3 bg-red-50">
            <span class="text-sm font-bold text-red-800">Total Beban</span>
            <span class="text-sm font-bold text-red-600">{formatCurrency(totalBeban)}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Laba Rugi total -->
    <div class="mt-4 card px-5 py-4 flex items-center justify-between {laba >= 0 ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'}">
      <span class="font-bold text-gray-900">{laba >= 0 ? 'LABA BERSIH' : 'RUGI BERSIH'}</span>
      <span class="text-lg font-bold {laba >= 0 ? 'text-green-700' : 'text-red-700'}">{formatCurrency(Math.abs(laba))}</span>
    </div>
  {/if}
</div>
