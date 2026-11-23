<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Wallet, TrendingUp, TrendingDown, Users, Truck, Package,
    FileText, ArrowUpRight, ArrowDownRight, BookOpen
  } from 'lucide-svelte';
  import * as Card from '$lib/components/ui/card/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
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
    { label: 'Total Kas & Bank', value: formatCurrency(data.total_kas), icon: Wallet, colorClass: 'text-blue-600', bgClass: 'bg-blue-50' },
    { label: 'Piutang Usaha', value: formatCurrency(data.total_piutang), icon: TrendingUp, colorClass: 'text-orange-600', bgClass: 'bg-orange-50' },
    { label: 'Hutang Usaha', value: formatCurrency(data.total_hutang), icon: TrendingDown, colorClass: 'text-red-600', bgClass: 'bg-red-50' },
    { label: 'Laba / Rugi', value: formatCurrency(data.laba_rugi), icon: data.laba_rugi >= 0 ? ArrowUpRight : ArrowDownRight, colorClass: data.laba_rugi >= 0 ? 'text-green-600' : 'text-red-600', bgClass: data.laba_rugi >= 0 ? 'bg-green-50' : 'bg-red-50' },
  ] : []);

  const infoCards = $derived(data ? [
    { label: 'Pelanggan', value: data.jumlah_pelanggan, icon: Users, colorClass: 'text-blue-600', bgClass: 'bg-blue-50' },
    { label: 'Supplier', value: data.jumlah_supplier, icon: Truck, colorClass: 'text-purple-600', bgClass: 'bg-purple-50' },
    { label: 'Produk', value: data.jumlah_produk, icon: Package, colorClass: 'text-teal-600', bgClass: 'bg-teal-50' },
    { label: 'Jurnal Bulan Ini', value: data.jurnal_bulan_ini, icon: BookOpen, colorClass: 'text-indigo-600', bgClass: 'bg-indigo-50' },
  ] : []);

  const quickLinks = [
    { href: '/jurnal', label: 'Buat Jurnal', icon: BookOpen, colorClass: 'text-indigo-600', bgClass: 'bg-indigo-50' },
    { href: '/kas-bank', label: 'Kas & Bank', icon: Wallet, colorClass: 'text-blue-600', bgClass: 'bg-blue-50' },
    { href: '/pelanggan', label: 'Pelanggan', icon: Users, colorClass: 'text-green-600', bgClass: 'bg-green-50' },
    { href: '/laporan/laba-rugi', label: 'Laporan', icon: FileText, colorClass: 'text-purple-600', bgClass: 'bg-purple-50' },
  ];
</script>

<div class="p-6">
  <div class="mb-6">
    <h1 class="text-xl font-semibold text-foreground">Dashboard</h1>
    <p class="text-sm text-muted-foreground mt-0.5">Ringkasan keuangan perusahaan</p>
  </div>

  {#if loading}
    <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4 mb-6">
      {#each [1,2,3,4] as _}
        <Skeleton class="h-24 rounded-xl" />
      {/each}
    </div>
  {:else if data}
    <!-- Stat cards -->
    <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4 mb-6">
      {#each statCards as card}
        <Card.Root class="p-5">
          <div class="flex items-start justify-between">
            <div>
              <p class="text-xs font-medium text-muted-foreground mb-1">{card.label}</p>
              <p class="text-lg font-bold text-foreground">{card.value}</p>
            </div>
            <div class="w-10 h-10 rounded-xl {card.bgClass} flex items-center justify-center shrink-0">
              <card.icon size={20} class={card.colorClass} />
            </div>
          </div>
        </Card.Root>
      {/each}
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-4 mb-6">
      <!-- Info mini cards -->
      <div class="lg:col-span-1 grid grid-cols-2 gap-4">
        {#each infoCards as card}
          <Card.Root class="p-5">
            <div class="w-9 h-9 rounded-lg {card.bgClass} flex items-center justify-center mb-3">
              <card.icon size={18} class={card.colorClass} />
            </div>
            <div class="text-xl font-bold text-foreground">{card.value}</div>
            <div class="text-xs text-muted-foreground mt-0.5">{card.label}</div>
          </Card.Root>
        {/each}
      </div>

      <!-- Financial summary -->
      <Card.Root class="lg:col-span-2 p-5">
        <h3 class="text-sm font-semibold text-foreground mb-4">Ringkasan Keuangan</h3>
        <div class="space-y-4">
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-xs text-muted-foreground">Total Pendapatan</span>
              <span class="text-sm font-semibold text-green-600">{formatCurrency(data.total_pendapatan)}</span>
            </div>
            <div class="h-2 bg-muted rounded-full overflow-hidden">
              <div class="h-full bg-green-500 rounded-full" style="width: {data.total_pendapatan > 0 ? 100 : 0}%"></div>
            </div>
          </div>
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-xs text-muted-foreground">Total Beban</span>
              <span class="text-sm font-semibold text-red-500">{formatCurrency(data.total_beban)}</span>
            </div>
            <div class="h-2 bg-muted rounded-full overflow-hidden">
              <div class="h-full bg-red-400 rounded-full"
                style="width: {data.total_pendapatan > 0 ? Math.min((data.total_beban / data.total_pendapatan) * 100, 100) : 0}%">
              </div>
            </div>
          </div>
          <div class="pt-3 border-t border-border">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-foreground">Laba Bersih</span>
              <span class="text-base font-bold {data.laba_rugi >= 0 ? 'text-green-600' : 'text-red-600'}">
                {formatCurrency(data.laba_rugi)}
              </span>
            </div>
          </div>
          <div class="pt-2 border-t border-border">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-foreground">Total Aset</span>
              <span class="text-base font-bold text-primary">{formatCurrency(data.total_aset)}</span>
            </div>
          </div>
        </div>
      </Card.Root>
    </div>

    <!-- Quick access -->
    <Card.Root class="p-5">
      <h3 class="text-sm font-semibold text-foreground mb-3">Akses Cepat</h3>
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
        {#each quickLinks as link}
          <a href={link.href}
            class="flex flex-col items-center gap-2 p-4 rounded-xl border border-border hover:border-border/80 hover:bg-muted transition-all text-center no-underline">
            <div class="w-10 h-10 rounded-xl {link.bgClass} flex items-center justify-center">
              <link.icon size={20} class={link.colorClass} />
            </div>
            <span class="text-xs font-medium text-foreground">{link.label}</span>
          </a>
        {/each}
      </div>
    </Card.Root>
  {/if}
</div>
