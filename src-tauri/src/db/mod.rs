use rusqlite::{Connection, Result};
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Tabel pencatat migrasi yang sudah dijalankan
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                id   TEXT NOT NULL PRIMARY KEY,
                run_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );",
        )?;

        for (id, sql) in MIGRATIONS {
            let already_run: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM _migrations WHERE id = ?1",
                    rusqlite::params![id],
                    |r| r.get::<_, i64>(0),
                )
                .map(|n| n > 0)?;

            if already_run {
                continue;
            }

            // Setiap migrasi dijalankan dalam satu transaksi agar atomik
            conn.execute_batch(&format!("BEGIN;\n{}\nCOMMIT;", sql))
                .map_err(|e| {
                    // Rollback jika gagal, lalu teruskan error ke caller
                    let _ = conn.execute_batch("ROLLBACK;");
                    rusqlite::Error::SqliteFailure(
                        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                        Some(format!("Migrasi '{}' gagal: {}", id, e)),
                    )
                })?;

            conn.execute(
                "INSERT INTO _migrations (id) VALUES (?1)",
                rusqlite::params![id],
            )?;
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DAFTAR MIGRASI — tambahkan entry BARU di paling bawah, JANGAN ubah yang lama
// Format: ("NNN_nama_deskriptif", r#"SQL;"#)
// ─────────────────────────────────────────────────────────────────────────────
const MIGRATIONS: &[(&str, &str)] = &[
    // ── 001 : Skema awal ────────────────────────────────────────────────────
    ("001_initial_schema", r#"
CREATE TABLE IF NOT EXISTS akun (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    kode         TEXT NOT NULL UNIQUE,
    nama         TEXT NOT NULL,
    tipe         TEXT NOT NULL CHECK(tipe IN ('Aset','Kewajiban','Modal','Pendapatan','Beban')),
    sub_tipe     TEXT,
    saldo_normal TEXT NOT NULL CHECK(saldo_normal IN ('Debit','Kredit')),
    saldo        REAL NOT NULL DEFAULT 0,
    aktif        INTEGER NOT NULL DEFAULT 1,
    created_at   TEXT NOT NULL DEFAULT (datetime('now','localtime'))
);

CREATE TABLE IF NOT EXISTS pelanggan (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    kode       TEXT NOT NULL UNIQUE,
    nama       TEXT NOT NULL,
    telepon    TEXT,
    email      TEXT,
    alamat     TEXT,
    piutang    REAL NOT NULL DEFAULT 0,
    aktif      INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
);

CREATE TABLE IF NOT EXISTS supplier (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    kode       TEXT NOT NULL UNIQUE,
    nama       TEXT NOT NULL,
    telepon    TEXT,
    email      TEXT,
    alamat     TEXT,
    hutang     REAL NOT NULL DEFAULT 0,
    aktif      INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
);

CREATE TABLE IF NOT EXISTS produk (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    kode                TEXT NOT NULL UNIQUE,
    nama                TEXT NOT NULL,
    satuan              TEXT NOT NULL DEFAULT 'pcs',
    harga_beli          REAL NOT NULL DEFAULT 0,
    harga_jual          REAL NOT NULL DEFAULT 0,
    stok                REAL NOT NULL DEFAULT 0,
    akun_persediaan_id  INTEGER,
    aktif               INTEGER NOT NULL DEFAULT 1,
    created_at          TEXT NOT NULL DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (akun_persediaan_id) REFERENCES akun(id)
);

CREATE TABLE IF NOT EXISTS jurnal (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    nomor      TEXT NOT NULL UNIQUE,
    tanggal    TEXT NOT NULL,
    keterangan TEXT,
    tipe       TEXT NOT NULL DEFAULT 'Umum',
    created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
);

CREATE TABLE IF NOT EXISTS jurnal_detail (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    jurnal_id  INTEGER NOT NULL,
    akun_id    INTEGER NOT NULL,
    keterangan TEXT,
    debit      REAL NOT NULL DEFAULT 0,
    kredit     REAL NOT NULL DEFAULT 0,
    FOREIGN KEY (jurnal_id) REFERENCES jurnal(id) ON DELETE CASCADE,
    FOREIGN KEY (akun_id)   REFERENCES akun(id)
);

CREATE TABLE IF NOT EXISTS kas_bank (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    nomor         TEXT NOT NULL UNIQUE,
    tanggal       TEXT NOT NULL,
    keterangan    TEXT,
    tipe          TEXT NOT NULL CHECK(tipe IN ('Masuk','Keluar','Transfer')),
    akun_id       INTEGER NOT NULL,
    akun_lawan_id INTEGER,
    jumlah        REAL NOT NULL DEFAULT 0,
    jurnal_id     INTEGER,
    created_at    TEXT NOT NULL DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (akun_id)       REFERENCES akun(id),
    FOREIGN KEY (akun_lawan_id) REFERENCES akun(id),
    FOREIGN KEY (jurnal_id)     REFERENCES jurnal(id)
);
"#),

    // ── 002 : Seed data akun default ────────────────────────────────────────
    ("002_seed_akun", r#"
INSERT OR IGNORE INTO akun (kode,nama,tipe,sub_tipe,saldo_normal) VALUES
('1-0000','ASET','Aset',NULL,'Debit'),
('1-1000','Aset Lancar','Aset','Lancar','Debit'),
('1-1001','Kas','Aset','Lancar','Debit'),
('1-1002','Bank BCA','Aset','Lancar','Debit'),
('1-1003','Bank Mandiri','Aset','Lancar','Debit'),
('1-1100','Piutang Usaha','Aset','Lancar','Debit'),
('1-1200','Persediaan Barang','Aset','Lancar','Debit'),
('1-1300','Uang Muka','Aset','Lancar','Debit'),
('1-2000','Aset Tetap','Aset','Tetap','Debit'),
('1-2001','Peralatan','Aset','Tetap','Debit'),
('1-2002','Kendaraan','Aset','Tetap','Debit'),
('1-2003','Gedung','Aset','Tetap','Debit'),
('1-2100','Akumulasi Penyusutan Peralatan','Aset','Tetap','Kredit'),
('2-0000','KEWAJIBAN','Kewajiban',NULL,'Kredit'),
('2-1000','Kewajiban Lancar','Kewajiban','Lancar','Kredit'),
('2-1001','Hutang Usaha','Kewajiban','Lancar','Kredit'),
('2-1002','Hutang Bank','Kewajiban','Lancar','Kredit'),
('2-1003','Hutang Pajak','Kewajiban','Lancar','Kredit'),
('2-1100','Uang Muka Pelanggan','Kewajiban','Lancar','Kredit'),
('3-0000','MODAL','Modal',NULL,'Kredit'),
('3-1001','Modal Pemilik','Modal',NULL,'Kredit'),
('3-1002','Laba Ditahan','Modal',NULL,'Kredit'),
('4-0000','PENDAPATAN','Pendapatan',NULL,'Kredit'),
('4-1001','Pendapatan Penjualan','Pendapatan',NULL,'Kredit'),
('4-1002','Pendapatan Jasa','Pendapatan',NULL,'Kredit'),
('4-1003','Pendapatan Lain-lain','Pendapatan',NULL,'Kredit'),
('5-0000','BEBAN','Beban',NULL,'Debit'),
('5-1001','Beban Pokok Penjualan','Beban',NULL,'Debit'),
('5-1002','Beban Gaji','Beban',NULL,'Debit'),
('5-1003','Beban Sewa','Beban',NULL,'Debit'),
('5-1004','Beban Listrik & Air','Beban',NULL,'Debit'),
('5-1005','Beban Telepon & Internet','Beban',NULL,'Debit'),
('5-1006','Beban Penyusutan','Beban',NULL,'Debit'),
('5-1007','Beban Pemasaran','Beban',NULL,'Debit'),
('5-1008','Beban Administrasi','Beban',NULL,'Debit'),
('5-1009','Beban Lain-lain','Beban',NULL,'Debit');
"#),

    // ─────────────────────────────────────────────────────────────────────────
    // Tambahkan migrasi baru di sini. Contoh:
    //
    // ("003_akun_add_catatan",
    //  "ALTER TABLE akun ADD COLUMN catatan TEXT;"),
    //
    // ("004_produk_add_barcode",
    //  "ALTER TABLE produk ADD COLUMN barcode TEXT;"),
    //
    // ("005_hapus_kolom_lama",
    //  r#"
    //   CREATE TABLE produk_new AS SELECT id,kode,nama,satuan,harga_beli,harga_jual,stok FROM produk;
    //   DROP TABLE produk;
    //   ALTER TABLE produk_new RENAME TO produk;
    //  "#),
    // ─────────────────────────────────────────────────────────────────────────
];
