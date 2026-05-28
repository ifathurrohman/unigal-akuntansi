<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Wallet, TrendingUp, TrendingDown, Users, Truck, Package,
    FileText, ArrowUpRight, ArrowDownRight, BookOpen
  } from 'lucide-svelte';
  import { callCommand, formatCurrency } from '$lib/stores/app.svelte';
  import type { DashboardData } from '$lib/types';

  let data = $state<DashboardData | null>(null);
  let loading = $state(true);

  onMount(async () => {
    try {
      data = await callCommand<DashboardData>('get_dashboard');
    } finally {
      loading = false;
    }
  });

  const statCards = $derived(data ? [
    {
      label: 'Total Kas & Bank',
      value: formatCurrency(data.total_kas),
      icon: Wallet,
      bg: 'bg-blue-50',
      text: 'text-blue-600',
    },
    {
      label: 'Piutang Usaha',
      value: formatCurrency(data.total_piutang),
      icon: TrendingUp,
      bg: 'bg-orange-50',
      text: 'text-orange-600',
    },
    {
      label: 'Hutang Usaha',
      value: formatCurrency(data.total_hutang),
      icon: TrendingDown,
      bg: 'bg-red-50',
      text: 'text-red-600',
    },
    {
      label: 'Laba / Rugi',
      value: formatCurrency(data.laba_rugi),
      icon: data.laba_rugi >= 0 ? ArrowUpRight : ArrowDownRight,
      bg: data.laba_rugi >= 0 ? 'bg-green-50' : 'bg-red-50',
      text: data.laba_rugi >= 0 ? 'text-green-600' : 'text-red-600',
    },
  ] : []);

  const infoCards = $derived(data ? [
    { label: 'Pelanggan', value: data.jumlah_pelanggan, icon: Users, color: 'text-blue-600', bg: 'bg-blue-50' },
    { label: 'Supplier', value: data.jumlah_supplier, icon: Truck, color: 'text-purple-600', bg: 'bg-purple-50' },
    { label: 'Produk', value: data.jumlah_produk, icon: Package, color: 'text-teal-600', bg: 'bg-teal-50' },
    { label: 'Jurnal Bulan Ini', value: data.jurnal_bulan_ini, icon: BookOpen, color: 'text-indigo-600', bg: 'bg-indigo-50' },
  ] : []);
</script>

<div class="p-6">
  <div class="mb-6">
    <h1 class="text-xl font-semibold text-gray-900">Dashboard</h1>
    <p class="text-sm text-gray-500 mt-0.5">Ringkasan keuangan perusahaan</p>
  </div>

  {#if loading}
    <div class="flex items-center justify-center h-64">
      <div class="w-8 h-8 border-2 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
    </div>
  {:else if data}
    <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4 mb-6">
      {#each statCards as card}
        <div class="stat-card">
          <div class="flex items-start justify-between">
            <div>
              <p class="text-xs font-medium text-gray-500 mb-1">{card.label}</p>
              <p class="text-lg font-bold text-gray-900">{card.value}</p>
            </div>
            <div class="w-10 h-10 rounded-xl {card.bg} flex items-center justify-center shrink-0">
              <card.icon size={20} class={card.text} />
            </div>
          </div>
        </div>
      {/each}
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-4 mb-6">
      <div class="lg:col-span-1 grid grid-cols-2 gap-4">
        {#each infoCards as card}
          <div class="stat-card">
            <div class="w-9 h-9 rounded-lg {card.bg} flex items-center justify-center mb-3">
              <card.icon size={18} class={card.color} />
            </div>
            <div class="text-xl font-bold text-gray-900">{card.value}</div>
            <div class="text-xs text-gray-500 mt-0.5">{card.label}</div>
          </div>
        {/each}
      </div>

      <div class="lg:col-span-2 card p-5">
        <h3 class="text-sm font-semibold text-gray-900 mb-4">Ringkasan Keuangan</h3>
        <div class="space-y-4">
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-xs text-gray-500">Total Pendapatan</span>
              <span class="text-sm font-semibold text-green-600">{formatCurrency(data.total_pendapatan)}</span>
            </div>
            <div class="h-2 bg-gray-100 rounded-full overflow-hidden">
              <div class="h-full bg-green-500 rounded-full" style="width: {data.total_pendapatan > 0 ? 100 : 0}%"></div>
            </div>
          </div>
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-xs text-gray-500">Total Beban</span>
              <span class="text-sm font-semibold text-red-500">{formatCurrency(data.total_beban)}</span>
            </div>
            <div class="h-2 bg-gray-100 rounded-full overflow-hidden">
              <div class="h-full bg-red-400 rounded-full"
                style="width: {data.total_pendapatan > 0 ? Math.min((data.total_beban / data.total_pendapatan) * 100, 100) : 0}%">
              </div>
            </div>
          </div>
          <div class="pt-3 border-t border-gray-100">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-gray-700">Laba Bersih</span>
              <span class="text-base font-bold {data.laba_rugi >= 0 ? 'text-green-600' : 'text-red-600'}">
                {formatCurrency(data.laba_rugi)}
              </span>
            </div>
          </div>
          <div class="pt-2 border-t border-gray-100">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-gray-700">Total Aset</span>
              <span class="text-base font-bold text-primary-600">{formatCurrency(data.total_aset)}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="card p-5">
      <h3 class="text-sm font-semibold text-gray-900 mb-3">Akses Cepat</h3>
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
        {#each [
          { href: '/jurnal', label: 'Buat Jurnal', icon: BookOpen, color: 'text-indigo-600', bg: 'bg-indigo-50' },
          { href: '/kas-bank', label: 'Kas & Bank', icon: Wallet, color: 'text-blue-600', bg: 'bg-blue-50' },
          { href: '/pelanggan', label: 'Pelanggan', icon: Users, color: 'text-green-600', bg: 'bg-green-50' },
          { href: '/laporan/laba-rugi', label: 'Laporan', icon: FileText, color: 'text-purple-600', bg: 'bg-purple-50' },
        ] as link}
          <a href={link.href}
            class="flex flex-col items-center gap-2 p-4 rounded-xl border border-gray-100 hover:border-gray-200 hover:bg-gray-50 transition-all text-center no-underline">
            <div class="w-10 h-10 rounded-xl {link.bg} flex items-center justify-center">
              <link.icon size={20} class={link.color} />
            </div>
            <span class="text-xs font-medium text-gray-700">{link.label}</span>
          </a>
        {/each}
      </div>
    </div>
  {/if}
</div>
