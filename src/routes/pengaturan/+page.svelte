<script lang="ts">
  import { onMount } from 'svelte';
  import { Save, RotateCcw } from 'lucide-svelte';
  import { callCommand, showNotification, loadAppState, appState } from '$lib/stores/app.svelte';
  import type { Akun, Perusahaan } from '$lib/types';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import * as Card from '$lib/components/ui/card/index.js';
  import * as Select from '$lib/components/ui/select/index.js';
  import { DatePicker } from '$lib/components/ui/date-picker/index.js';

  let form = $state<Perusahaan>({
    nama: '', alamat: null, kota: null, telepon: null, email: null, npwp: null,
    akun_laba_ditahan_id: null, tanggal_kerja: null,
  });
  let akunModal = $state<Akun[]>([]);
  let tglKerja = $state('');
  let loading = $state(true);
  let saving = $state(false);

  const akunLabaDitahan = $derived(akunModal.find(a => a.id === form.akun_laba_ditahan_id));

  async function load() {
    loading = true;
    try {
      const [p, akun] = await Promise.all([
        callCommand<Perusahaan>('get_perusahaan'),
        callCommand<Akun[]>('get_akun'),
      ]);
      form = p;
      tglKerja = p.tanggal_kerja ?? '';
      akunModal = akun.filter(a => a.tipe === 'Modal' && !a.header && a.aktif);
    } finally {
      loading = false;
    }
  }

  async function save() {
    saving = true;
    try {
      const input = {
        ...form,
        nama: form.nama.trim(),
        alamat: form.alamat?.trim() || null,
        kota: form.kota?.trim() || null,
        telepon: form.telepon?.trim() || null,
        email: form.email?.trim() || null,
        npwp: form.npwp?.trim() || null,
        tanggal_kerja: tglKerja || null,
      };
      form = await callCommand<Perusahaan>('save_perusahaan', { input });
      tglKerja = form.tanggal_kerja ?? '';
      await loadAppState();
      showNotification('success', 'Pengaturan berhasil disimpan');
    } catch (e) {
      showNotification('error', String(e));
    } finally {
      saving = false;
    }
  }

  onMount(load);
</script>

<div class="p-6 max-w-3xl">
  <div class="mb-6">
    <h1 class="text-xl font-semibold text-foreground">Profil Perusahaan</h1>
    <p class="text-sm text-muted-foreground mt-0.5">Identitas yang tampil di laporan, serta pengaturan dasar pembukuan</p>
  </div>

  {#if loading}
    <div class="text-muted-foreground">Memuat data...</div>
  {:else}
    <div class="space-y-4">
      <Card.Root>
        <Card.Header>
          <Card.Title>Identitas Perusahaan</Card.Title>
        </Card.Header>
        <Card.Content class="space-y-4">
          <div>
            <Label for="p-nama">Nama Perusahaan</Label>
            <Input id="p-nama" placeholder="PT Contoh Sejahtera" bind:value={form.nama} />
          </div>
          <div>
            <Label for="p-alamat">Alamat</Label>
            <Input id="p-alamat" placeholder="Alamat lengkap" bind:value={form.alamat} />
          </div>
          <div class="grid grid-cols-2 gap-3">
            <div>
              <Label for="p-kota">Kota</Label>
              <Input id="p-kota" bind:value={form.kota} />
            </div>
            <div>
              <Label for="p-telp">Telepon</Label>
              <Input id="p-telp" bind:value={form.telepon} />
            </div>
          </div>
          <div class="grid grid-cols-2 gap-3">
            <div>
              <Label for="p-email">Email</Label>
              <Input id="p-email" type="email" bind:value={form.email} />
            </div>
            <div>
              <Label for="p-npwp">NPWP</Label>
              <Input id="p-npwp" bind:value={form.npwp} />
            </div>
          </div>
        </Card.Content>
      </Card.Root>

      <Card.Root>
        <Card.Header>
          <Card.Title>Pembukuan</Card.Title>
        </Card.Header>
        <Card.Content class="space-y-5">
          <div>
            <Label for="p-ld">Akun Laba Ditahan</Label>
            <Select.Root
              value={form.akun_laba_ditahan_id ? String(form.akun_laba_ditahan_id) : ''}
              onValueChange={(v) => { form.akun_laba_ditahan_id = v ? Number(v) : null; }}
            >
              <Select.Trigger class="w-full" id="p-ld">
                {#if akunLabaDitahan}{akunLabaDitahan.kode} - {akunLabaDitahan.nama}{:else}<span class="text-muted-foreground">-- Pilih akun Modal --</span>{/if}
              </Select.Trigger>
              <Select.Content>
                {#each akunModal as a}
                  <Select.Item value={String(a.id)}>{a.kode} - {a.nama}</Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
            <p class="text-xs text-muted-foreground mt-1">
              Dipakai saat tutup buku: laba/rugi periode dipindahkan ke akun ini. Hanya akun bertipe Modal yang muncul.
              {#if akunModal.length === 0}Buat akun Modal terlebih dahulu di Daftar Akun.{/if}
            </p>
          </div>

          <div>
            <Label for="p-tgl">Tanggal Kerja (opsional)</Label>
            <div class="flex items-center gap-2">
              <DatePicker id="p-tgl" bind:value={tglKerja} placeholder="Gunakan tanggal sistem" class="w-64" />
              {#if tglKerja}
                <Button variant="outline" size="sm" onclick={() => { tglKerja = ''; }}>
                  <RotateCcw size={14} /> Pakai tanggal sistem
                </Button>
              {/if}
            </div>
            <p class="text-xs text-muted-foreground mt-1">
              Kosongkan untuk memakai tanggal komputer. Isi untuk simulasi/pembelajaran: seluruh form transaksi akan
              memakai tanggal ini sebagai tanggal awal, sehingga tidak perlu mengubah tanggal setiap kali input.
              Transaksi tetap harus berada di dalam periode aktif.
            </p>
          </div>
        </Card.Content>
      </Card.Root>

      <div class="flex justify-end">
        <Button onclick={save} disabled={saving}>
          <Save size={16} /> {saving ? 'Menyimpan...' : 'Simpan'}
        </Button>
      </div>
    </div>
  {/if}
</div>
