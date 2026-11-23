<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { Toaster } from '$lib/components/ui/sonner/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import SplashScreen from '$lib/components/SplashScreen.svelte';
  import UpdateDialog from '$lib/components/UpdateDialog.svelte';
  import {
    LayoutDashboard,
    BookOpen,
    Users,
    Truck,
    Package,
    FileText,
    Banknote,
    BarChart3,
    Building2,
    Settings,
    CalendarRange,
    CalendarClock,
  } from 'lucide-svelte';
  import { appState, loadAppState, namaPerusahaan, formatDate } from '$lib/stores/app.svelte';
  import { onMount } from 'svelte';
  import { checkForUpdate, startPeriodicUpdateCheck, downloadAndInstall, dismissUpdate, updateState } from '$lib/stores/updater.svelte';

  let { children } = $props();

  let splashDone = $state(false);
  let appReady = $state(false);

  onMount(async () => {
    await loadAppState();
    appReady = true;
  });

  async function onSplashDone() {
    splashDone = true;
    await checkForUpdate(true);
    startPeriodicUpdateCheck();
  }

  const currentPath = $derived($page.url.pathname);

  function isActive(path: string) {
    return currentPath === path || currentPath.startsWith(path + '/');
  }

  const navGroups = [
    {
      items: [
        { path: '/', label: 'Dashboard', icon: LayoutDashboard },
      ]
    },
    {
      label: 'Transaksi',
      items: [
        { path: '/jurnal', label: 'Jurnal Umum', icon: BookOpen },
        { path: '/kas-bank', label: 'Kas & Bank', icon: Banknote },
      ]
    },
    {
      label: 'Master Data',
      items: [
        { path: '/akun', label: 'Daftar Akun', icon: FileText },
        { path: '/pelanggan', label: 'Pelanggan', icon: Users },
        { path: '/supplier', label: 'Supplier', icon: Truck },
        { path: '/produk', label: 'Produk', icon: Package },
      ]
    },
    {
      label: 'Pengaturan',
      items: [
        { path: '/pengaturan', label: 'Profil Perusahaan', icon: Settings },
        { path: '/periode', label: 'Periode & Arsip', icon: CalendarRange },
      ]
    },
    {
      label: 'Laporan',
      items: [
        { path: '/laporan/laba-rugi', label: 'Laba Rugi', icon: BarChart3 },
        { path: '/laporan/neraca', label: 'Neraca', icon: BarChart3 },
      ]
    },
  ];
</script>

{#if !splashDone}
  <SplashScreen onDone={onSplashDone} />
{/if}

{#if updateState.available}
  <UpdateDialog
    version={updateState.version}
    notes={updateState.notes}
    state={updateState.phase}
    progress={updateState.progress}
    errorMsg={updateState.errorMsg}
    onConfirm={downloadAndInstall}
    onDismiss={dismissUpdate}
  />
{/if}

<div class="flex h-screen overflow-hidden bg-background">
  <!-- Sidebar -->
  <aside class="w-60 bg-sidebar border-r border-sidebar-border flex flex-col shrink-0">
    <!-- Logo -->
    <div class="px-5 py-5 border-b border-sidebar-border">
      <div class="flex items-center gap-2.5">
        <div class="w-9 h-9 rounded-xl bg-primary flex items-center justify-center shadow-sm">
          <Building2 size={18} class="text-primary-foreground" />
        </div>
        <div>
          <div class="text-sm font-bold text-sidebar-foreground leading-tight line-clamp-2">{namaPerusahaan()}</div>
          <div class="text-xs text-primary font-semibold">Akuntansi</div>
        </div>
      </div>
    </div>

    <!-- Navigation -->
    <nav class="flex-1 px-3 py-4 overflow-y-auto space-y-5">
      {#each navGroups as group}
        <div>
          {#if group.label}
            <div class="px-3 mb-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
              {group.label}
            </div>
          {/if}
          <div class="space-y-0.5">
            {#each group.items as item}
              <Button
                variant={isActive(item.path) ? 'secondary' : 'ghost'}
                class="w-full justify-start gap-3 font-medium"
                onclick={() => goto(item.path)}
              >
                <item.icon size={17} class={isActive(item.path) ? 'text-primary' : 'text-muted-foreground'} />
                <span>{item.label}</span>
              </Button>
            {/each}
          </div>
        </div>
      {/each}
    </nav>

    <!-- Periode aktif & tanggal kerja -->
    <div class="px-5 py-3 border-t border-sidebar-border space-y-1 text-xs">
      <div class="flex items-center gap-1.5 text-muted-foreground">
        <CalendarClock size={13} />
        <span>Tanggal kerja: <strong class="text-foreground">{formatDate(appState.tanggalKerja)}</strong></span>
      </div>
      {#if appState.periode}
        <div class="text-muted-foreground">Periode: <strong class="text-foreground">{appState.periode.nama}</strong></div>
      {:else}
        <div class="text-destructive">Belum ada periode aktif</div>
      {/if}
    </div>

    <!-- Footer -->
    <div class="px-5 py-4 border-t border-sidebar-border">
      <div class="text-xs text-muted-foreground">© 2026 Unigal Akuntansi</div>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="flex-1 overflow-y-auto">
    {#if appReady}
      {@render children()}
    {/if}
  </main>
</div>

<Toaster richColors position="top-right" />
