export interface Akun {
  id: number;
  kode: string;
  nama: string;
  tipe: 'Aset' | 'Kewajiban' | 'Modal' | 'Pendapatan' | 'Beban';
  sub_tipe: string | null;
  saldo_normal: 'Debit' | 'Kredit';
  saldo: number;
  aktif: boolean;
  header: boolean;
  kas_bank: boolean;
  created_at: string;
}

export interface AkunInput {
  kode: string;
  nama: string;
  tipe: string;
  sub_tipe: string | null;
  saldo_normal: string;
  header: boolean;
  kas_bank: boolean;
}

export interface Pelanggan {
  id: number;
  kode: string;
  nama: string;
  telepon: string | null;
  email: string | null;
  alamat: string | null;
  piutang: number;
  aktif: boolean;
  created_at: string;
}

export interface PelangganInput {
  kode: string;
  nama: string;
  telepon: string | null;
  email: string | null;
  alamat: string | null;
}

export interface Supplier {
  id: number;
  kode: string;
  nama: string;
  telepon: string | null;
  email: string | null;
  alamat: string | null;
  hutang: number;
  aktif: boolean;
  created_at: string;
}

export interface SupplierInput {
  kode: string;
  nama: string;
  telepon: string | null;
  email: string | null;
  alamat: string | null;
}

export interface Produk {
  id: number;
  kode: string;
  nama: string;
  satuan: string;
  harga_beli: number;
  harga_jual: number;
  stok: number;
  aktif: boolean;
  created_at: string;
}

export interface ProdukInput {
  kode: string;
  nama: string;
  satuan: string;
  harga_beli: number;
  harga_jual: number;
  stok?: number;
}

export interface JurnalDetail {
  id: number;
  jurnal_id: number;
  akun_id: number;
  akun_kode: string | null;
  akun_nama: string | null;
  keterangan: string | null;
  debit: number;
  kredit: number;
}

export interface Jurnal {
  id: number;
  nomor: string;
  tanggal: string;
  keterangan: string | null;
  tipe: string;
  total_debit: number;
  total_kredit: number;
  created_at: string;
  detail: JurnalDetail[];
}

export interface JurnalDetailInput {
  akun_id: number;
  keterangan: string | null;
  debit: number;
  kredit: number;
}

export interface JurnalInput {
  tanggal: string;
  keterangan: string | null;
  tipe?: string;
  detail: JurnalDetailInput[];
}

export interface KasBank {
  id: number;
  nomor: string;
  tanggal: string;
  keterangan: string | null;
  tipe: 'Masuk' | 'Keluar' | 'Transfer';
  akun_id: number;
  akun_nama: string | null;
  akun_lawan_id: number | null;
  akun_lawan_nama: string | null;
  jumlah: number;
  created_at: string;
}

export interface KasBankInput {
  tanggal: string;
  keterangan: string | null;
  tipe: string;
  akun_id: number;
  akun_lawan_id: number | null;
  jumlah: number;
}

export interface DashboardData {
  total_kas: number;
  total_piutang: number;
  total_hutang: number;
  total_pendapatan: number;
  total_beban: number;
  laba_rugi: number;
  total_aset: number;
  jumlah_pelanggan: number;
  jumlah_supplier: number;
  jumlah_produk: number;
  jurnal_bulan_ini: number;
}

export interface LabaRugiItem {
  kode: string;
  nama: string;
  tipe: string;
  saldo: number;
}

export interface NeracaItem {
  kode: string;
  nama: string;
  tipe: string;
  sub_tipe: string | null;
  saldo: number;
}

export interface Perusahaan {
  nama: string;
  alamat: string | null;
  kota: string | null;
  telepon: string | null;
  email: string | null;
  npwp: string | null;
  akun_laba_ditahan_id: number | null;
  tanggal_kerja: string | null;
}

export interface Periode {
  id: number;
  nama: string;
  tanggal_mulai: string;
  tanggal_selesai: string;
  status: 'Aktif' | 'Ditutup';
  closed_at: string | null;
}

export interface PeriodeInput {
  nama: string;
  tanggal_mulai: string;
  tanggal_selesai: string;
}

export interface LaporanArsip {
  jenis: 'LabaRugi' | 'Neraca';
  created_at: string;
  data: {
    meta: { nama: string; tanggal_mulai: string; tanggal_selesai: string };
    items: (LabaRugiItem | NeracaItem)[];
  };
}
