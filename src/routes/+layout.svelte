<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import Notification from '$lib/components/Notification.svelte';
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
  } from 'lucide-svelte';
  import { checkForUpdate, downloadAndInstall, dismissUpdate, updateState } from '$lib/stores/updater.svelte';

  let { children } = $props();

  let splashDone = $state(false);

  async function onSplashDone() {
    splashDone = true;
    // Cek update di background setelah splash selesai
    await checkForUpdate();
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

<div class="flex h-screen overflow-hidden bg-gray-50">
  <!-- Sidebar -->
  <aside class="w-60 bg-white border-r border-gray-100 flex flex-col shrink-0">
    <!-- Logo -->
    <div class="px-5 py-5 border-b border-gray-100">
      <div class="flex items-center gap-2.5">
        <div class="w-9 h-9 rounded-xl bg-primary-600 flex items-center justify-center shadow-sm">
          <Building2 size={18} class="text-white" />
        </div>
        <div>
          <div class="text-sm font-bold text-gray-900 leading-tight">Unigal</div>
          <div class="text-xs text-primary-600 font-semibold">Akuntansi</div>
        </div>
      </div>
    </div>

    <!-- Navigation -->
    <nav class="flex-1 px-3 py-4 overflow-y-auto space-y-5">
      {#each navGroups as group}
        <div>
          {#if group.label}
            <div class="px-3 mb-1.5 text-xs font-semibold text-gray-400 uppercase tracking-wider">
              {group.label}
            </div>
          {/if}
          <div class="space-y-0.5">
            {#each group.items as item}
              <button
                class={isActive(item.path) ? 'nav-item-active w-full text-left' : 'nav-item-inactive w-full text-left'}
                onclick={() => goto(item.path)}
              >
                <item.icon size={17} class={isActive(item.path) ? 'text-primary-600' : 'text-gray-400'} />
                <span>{item.label}</span>
              </button>
            {/each}
          </div>
        </div>
      {/each}
    </nav>

    <!-- Footer -->
    <div class="px-5 py-4 border-t border-gray-100">
      <div class="text-xs text-gray-400">© 2026 Unigal Akuntansi</div>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="flex-1 overflow-y-auto">
    {@render children()}
  </main>
</div>

<Notification />
