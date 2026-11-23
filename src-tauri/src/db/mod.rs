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
        db.run_migrations(Some(path))?;
        Ok(db)
    }

    /// `backup_base`: path file database; jika ada migrasi yang menunggu, salinan cadangan dibuat dulu.
    fn run_migrations(&self, backup_base: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Tabel pencatat migrasi yang sudah dijalankan
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                id   TEXT NOT NULL PRIMARY KEY,
                run_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );",
        )?;

        let mut sudah_backup = false;
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

            // Cadangkan database sekali sebelum migrasi pertama yang tertunda (lewati DB baru/kosong & in-memory)
            if !sudah_backup {
                sudah_backup = true;
                let ada_data: bool = conn
                    .query_row("SELECT COUNT(*) FROM _migrations", [], |r| r.get::<_, i64>(0))
                    .map(|n| n > 0)
                    .unwrap_or(false);
                if let (Some(base), true) = (backup_base, ada_data) {
                    if base != ":memory:" {
                        let tujuan = format!("{}.bak-sebelum-{}", base, id);
                        if !std::path::Path::new(&tujuan).exists() {
                            // Gagal membuat cadangan tidak boleh menghentikan aplikasi
                            let _ = conn.execute("VACUUM INTO ?1", rusqlite::params![tujuan]);
                        }
                    }
                }
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

    // ── 002 : (dulu seed akun default; kini dikosongkan — akun diinput sendiri) ──
    ("002_seed_akun", "SELECT 1;"),

    // ── 003 : Profil perusahaan, periode, arsip laporan, flag akun ──────────
    ("003_perusahaan_periode", r#"
CREATE TABLE IF NOT EXISTS perusahaan (
    id                   INTEGER PRIMARY KEY CHECK(id = 1),
    nama                 TEXT NOT NULL DEFAULT '',
    alamat               TEXT,
    kota                 TEXT,
    telepon              TEXT,
    email                TEXT,
    npwp                 TEXT,
    akun_laba_ditahan_id INTEGER,
    tanggal_kerja        TEXT,
    FOREIGN KEY (akun_laba_ditahan_id) REFERENCES akun(id)
);
INSERT OR IGNORE INTO perusahaan (id, nama) VALUES (1, '');

CREATE TABLE IF NOT EXISTS periode (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    nama            TEXT NOT NULL,
    tanggal_mulai   TEXT NOT NULL,
    tanggal_selesai TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'Aktif' CHECK(status IN ('Aktif','Ditutup')),
    closed_at       TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now','localtime'))
);

CREATE TABLE IF NOT EXISTS laporan_arsip (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    periode_id INTEGER NOT NULL,
    jenis      TEXT NOT NULL CHECK(jenis IN ('LabaRugi','Neraca')),
    data       TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
    UNIQUE(periode_id, jenis),
    FOREIGN KEY (periode_id) REFERENCES periode(id)
);

ALTER TABLE akun ADD COLUMN header   INTEGER NOT NULL DEFAULT 0;
ALTER TABLE akun ADD COLUMN kas_bank INTEGER NOT NULL DEFAULT 0;

-- Data lama: tandai akun induk & akun kas/bank sesuai aturan sebelumnya
UPDATE akun SET header = 1 WHERE kode GLOB '?-0000' OR kode GLOB '?-?000';
UPDATE akun SET kas_bank = 1
 WHERE header = 0 AND tipe = 'Aset' AND sub_tipe = 'Lancar'
   AND (nama LIKE '%Kas%' OR nama LIKE '%Bank%');

-- Satu periode aktif awal; mencakup seluruh transaksi lama jika ada
INSERT INTO periode (nama, tanggal_mulai, tanggal_selesai, status)
SELECT 'Tahun ' || strftime('%Y','now','localtime'),
       MIN(COALESCE((SELECT strftime('%Y-01-01', MIN(tanggal)) FROM jurnal), strftime('%Y-01-01','now','localtime')), strftime('%Y-01-01','now','localtime')),
       MAX(COALESCE((SELECT strftime('%Y-12-31', MAX(tanggal)) FROM jurnal), strftime('%Y-12-31','now','localtime')), strftime('%Y-12-31','now','localtime')),
       'Aktif';
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::jurnal::insert_jurnal;
    use crate::commands::laporan::{laba_rugi_rentang, neraca_sekarang};
    use crate::models::JurnalDetailInput;

    fn det(akun_id: i64, debit: f64, kredit: f64) -> JurnalDetailInput {
        JurnalDetailInput { akun_id, keterangan: None, debit, kredit }
    }

    #[test]
    fn instalasi_baru_kosong_dengan_satu_periode() {
        let db = Database::new(":memory:").unwrap();
        let c = db.conn.lock().unwrap();
        let akun: i64 = c.query_row("SELECT COUNT(*) FROM akun", [], |r| r.get(0)).unwrap();
        let periode: i64 = c.query_row("SELECT COUNT(*) FROM periode WHERE status='Aktif'", [], |r| r.get(0)).unwrap();
        assert_eq!(akun, 0);
        assert_eq!(periode, 1);
    }

    #[test]
    fn migrasi_db_lama_menandai_header_dan_kas() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(MIGRATIONS[0].1).unwrap();
        conn.execute_batch(
            "INSERT INTO akun (kode,nama,tipe,sub_tipe,saldo_normal) VALUES
             ('1-1000','Aset Lancar','Aset','Lancar','Debit'),
             ('1-1001','Kas','Aset','Lancar','Debit'),
             ('1-1100','Piutang Usaha','Aset','Lancar','Debit');
             INSERT INTO jurnal (nomor,tanggal,tipe) VALUES ('JU-202301-0001','2023-03-05','Umum');
             CREATE TABLE _migrations (id TEXT NOT NULL PRIMARY KEY, run_at TEXT NOT NULL DEFAULT (datetime('now')));
             INSERT INTO _migrations (id) VALUES ('001_initial_schema'),('002_seed_akun');",
        )
        .unwrap();
        let db = Database { conn: Mutex::new(conn) };
        db.run_migrations(None).unwrap();
        let c = db.conn.lock().unwrap();
        let h: Vec<(String, i64, i64)> = c
            .prepare("SELECT kode, header, kas_bank FROM akun ORDER BY kode").unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap()
            .map(|r| r.unwrap()).collect();
        assert_eq!(h, vec![("1-1000".into(), 1, 0), ("1-1001".into(), 0, 1), ("1-1100".into(), 0, 0)]);
        let mulai: String = c.query_row("SELECT tanggal_mulai FROM periode", [], |r| r.get(0)).unwrap();
        assert_eq!(mulai, "2023-01-01");
    }

    #[test]
    fn jurnal_di_luar_periode_ditolak_dan_laporan_per_periode() {
        let db = Database::new(":memory:").unwrap();
        let c = db.conn.lock().unwrap();
        c.execute_batch(
            "UPDATE periode SET tanggal_mulai='2025-01-01', tanggal_selesai='2025-12-31';
             INSERT INTO akun (kode,nama,tipe,saldo_normal) VALUES
             ('1-1','Kas','Aset','Debit'),('4-1','Jual','Pendapatan','Kredit'),('3-1','Modal','Modal','Kredit');",
        ).unwrap();
        let id = |k: &str| -> i64 { c.query_row("SELECT id FROM akun WHERE kode=?1", [k], |r| r.get(0)).unwrap() };
        let (kas, jual, modal) = (id("1-1"), id("4-1"), id("3-1"));

        assert!(insert_jurnal(&c, "2024-12-31", None, "Umum", &[det(kas, 10.0, 0.0), det(modal, 0.0, 10.0)]).is_err());
        insert_jurnal(&c, "2025-01-10", None, "Saldo Awal", &[det(kas, 100.0, 0.0), det(modal, 0.0, 100.0)]).unwrap();
        insert_jurnal(&c, "2025-02-10", None, "Umum", &[det(kas, 50.0, 0.0), det(jual, 0.0, 50.0)]).unwrap();

        let lr = laba_rugi_rentang(&c, "2025-03-01", "2025-03-31").unwrap();
        assert_eq!(lr[0].saldo, 0.0); // filter periode bekerja
        let lr = laba_rugi_rentang(&c, "2025-02-01", "2025-02-28").unwrap();
        assert_eq!(lr[0].saldo, 50.0);

        let n = neraca_sekarang(&c).unwrap();
        let aset: f64 = n.iter().filter(|i| i.tipe == "Aset").map(|i| i.saldo).sum();
        let ekuitas: f64 = n.iter().filter(|i| i.tipe == "Modal").map(|i| i.saldo).sum();
        assert_eq!(aset, 150.0);
        assert_eq!(ekuitas, 150.0); // 100 modal + 50 laba berjalan
    }

    #[test]
    fn tutup_buku_menutup_nominal_mengarsip_dan_membuka_periode_baru() {
        use crate::commands::pengaturan::{periode_aktif, tutup_periode_inner};
        use crate::models::PeriodeInput;
        let db = Database::new(":memory:").unwrap();
        let c = db.conn.lock().unwrap();
        c.execute_batch(
            "UPDATE periode SET tanggal_mulai='2025-01-01', tanggal_selesai='2025-12-31';
             INSERT INTO akun (kode,nama,tipe,saldo_normal) VALUES
             ('1-1','Kas','Aset','Debit'),('4-1','Jual','Pendapatan','Kredit'),
             ('5-1','Gaji','Beban','Debit'),('3-1','Modal','Modal','Kredit'),('3-2','Laba Ditahan','Modal','Kredit');",
        ).unwrap();
        let id = |k: &str| -> i64 { c.query_row("SELECT id FROM akun WHERE kode=?1", [k], |r| r.get(0)).unwrap() };
        let (kas, jual, gaji, modal, ld) = (id("1-1"), id("4-1"), id("5-1"), id("3-1"), id("3-2"));
        insert_jurnal(&c, "2025-01-01", None, "Saldo Awal", &[det(kas, 100.0, 0.0), det(modal, 0.0, 100.0)]).unwrap();
        insert_jurnal(&c, "2025-05-01", None, "Umum", &[det(kas, 80.0, 0.0), det(jual, 0.0, 80.0)]).unwrap();
        insert_jurnal(&c, "2025-06-01", None, "Umum", &[det(gaji, 30.0, 0.0), det(kas, 0.0, 30.0)]).unwrap();

        let baru = || PeriodeInput { nama: "2026".into(), tanggal_mulai: "2026-01-01".into(), tanggal_selesai: "2026-12-31".into() };
        assert!(tutup_periode_inner(&c, baru()).is_err()); // laba ditahan belum dipilih
        c.execute("UPDATE perusahaan SET akun_laba_ditahan_id=?1", [ld]).unwrap();
        let p = tutup_periode_inner(&c, baru()).unwrap();
        assert_eq!(p.nama, "2026");

        let saldo = |a: i64| -> f64 { c.query_row("SELECT saldo FROM akun WHERE id=?1", [a], |r| r.get(0)).unwrap() };
        assert_eq!(saldo(jual), 0.0);
        assert_eq!(saldo(gaji), 0.0);
        assert_eq!(saldo(ld), -50.0); // kredit 50 (debit-kredit)
        let n = neraca_sekarang(&c).unwrap();
        let aset: f64 = n.iter().filter(|i| i.tipe == "Aset").map(|i| i.saldo).sum();
        let ekuitas: f64 = n.iter().filter(|i| i.tipe == "Modal").map(|i| i.saldo).sum();
        assert_eq!((aset, ekuitas), (150.0, 150.0));

        // laba rugi periode lama tetap utuh (jurnal penutup tidak dihitung) dan terarsip
        let lr = laba_rugi_rentang(&c, "2025-01-01", "2025-12-31").unwrap();
        assert_eq!(lr.iter().map(|i| i.saldo).collect::<Vec<_>>(), vec![80.0, 30.0]);
        let arsip: i64 = c.query_row("SELECT COUNT(*) FROM laporan_arsip", [], |r| r.get(0)).unwrap();
        assert_eq!(arsip, 2);

        // periode lama terkunci; periode baru aktif
        assert!(insert_jurnal(&c, "2025-07-01", None, "Umum", &[det(kas, 1.0, 0.0), det(modal, 0.0, 1.0)]).is_err());
        insert_jurnal(&c, "2026-01-15", None, "Umum", &[det(kas, 1.0, 0.0), det(modal, 0.0, 1.0)]).unwrap();
        assert_eq!(periode_aktif(&c).unwrap().tanggal_mulai, "2026-01-01");
    }

    #[test]
    fn backup_dibuat_sebelum_migrasi_pada_db_lama() {
        let path = std::env::temp_dir().join(format!("akuntansi-test-{}.db", std::process::id()));
        let path_s = path.to_str().unwrap().to_string();
        let bak = format!("{}.bak-sebelum-003_perusahaan_periode", path_s);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&bak);
        {
            // DB lama: baru sampai migrasi 002
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(MIGRATIONS[0].1).unwrap();
            conn.execute_batch(
                "CREATE TABLE _migrations (id TEXT NOT NULL PRIMARY KEY, run_at TEXT NOT NULL DEFAULT (datetime('now')));
                 INSERT INTO _migrations (id) VALUES ('001_initial_schema'),('002_seed_akun');
                 INSERT INTO akun (kode,nama,tipe,saldo_normal) VALUES ('1-1','Kas','Aset','Debit');",
            ).unwrap();
        }
        let db = Database::new(&path_s).unwrap();
        drop(db);
        assert!(std::path::Path::new(&bak).exists());
        let b = Connection::open(&bak).unwrap();
        let n: i64 = b.query_row("SELECT COUNT(*) FROM akun", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
        let kolom: i64 = b.query_row("SELECT COUNT(*) FROM pragma_table_info('akun') WHERE name='header'", [], |r| r.get(0)).unwrap();
        assert_eq!(kolom, 0); // cadangan = kondisi sebelum migrasi
        drop(b);
        for f in [&path_s, &bak, &format!("{}-wal", path_s), &format!("{}-shm", path_s)] { let _ = std::fs::remove_file(f); }
    }
}
