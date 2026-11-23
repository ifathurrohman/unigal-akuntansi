use crate::commands::jurnal::insert_jurnal;
use crate::commands::laporan::{laba_rugi_rentang, neraca_sekarang};
use crate::db::Database;
use crate::models::{JurnalDetailInput, LaporanArsip, Perusahaan, Periode, PeriodeInput};
use rusqlite::Connection;
use tauri::State;

fn valid_tanggal(t: &str) -> bool {
    chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d").is_ok()
}

/// Tanggal kerja (jika diatur untuk simulasi) atau tanggal sistem.
pub fn tanggal_hari_ini(conn: &Connection) -> String {
    conn.query_row("SELECT tanggal_kerja FROM perusahaan WHERE id=1", [], |r| r.get::<_, Option<String>>(0))
        .ok()
        .flatten()
        .filter(|t| valid_tanggal(t))
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string())
}

fn map_periode(row: &rusqlite::Row) -> rusqlite::Result<Periode> {
    Ok(Periode {
        id: row.get(0)?,
        nama: row.get(1)?,
        tanggal_mulai: row.get(2)?,
        tanggal_selesai: row.get(3)?,
        status: row.get(4)?,
        closed_at: row.get(5)?,
    })
}

const SELECT_PERIODE: &str = "SELECT id,nama,tanggal_mulai,tanggal_selesai,status,closed_at FROM periode";

pub fn periode_aktif(conn: &Connection) -> Result<Periode, String> {
    conn.query_row(
        &format!("{} WHERE status='Aktif' ORDER BY id DESC LIMIT 1", SELECT_PERIODE),
        [],
        map_periode,
    )
    .map_err(|_| "Belum ada periode aktif. Buat periode terlebih dahulu di menu Periode.".to_string())
}

/// Transaksi hanya boleh dicatat/diubah pada tanggal di dalam periode aktif.
pub fn pastikan_periode_aktif(conn: &Connection, tanggal: &str) -> Result<(), String> {
    let p = periode_aktif(conn)?;
    if tanggal < p.tanggal_mulai.as_str() || tanggal > p.tanggal_selesai.as_str() {
        return Err(format!(
            "Tanggal {} berada di luar periode aktif \"{}\" ({} s/d {})",
            tanggal, p.nama, p.tanggal_mulai, p.tanggal_selesai
        ));
    }
    Ok(())
}

// ── Profil perusahaan ───────────────────────────────────────────────────────

fn fetch_perusahaan(conn: &Connection) -> Result<Perusahaan, String> {
    conn.query_row(
        "SELECT nama,alamat,kota,telepon,email,npwp,akun_laba_ditahan_id,tanggal_kerja FROM perusahaan WHERE id=1",
        [],
        |r| {
            Ok(Perusahaan {
                nama: r.get(0)?,
                alamat: r.get(1)?,
                kota: r.get(2)?,
                telepon: r.get(3)?,
                email: r.get(4)?,
                npwp: r.get(5)?,
                akun_laba_ditahan_id: r.get(6)?,
                tanggal_kerja: r.get(7)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_perusahaan(db: State<Database>) -> Result<Perusahaan, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    fetch_perusahaan(&conn)
}

#[tauri::command]
pub fn save_perusahaan(db: State<Database>, input: Perusahaan) -> Result<Perusahaan, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    if let Some(t) = &input.tanggal_kerja {
        if !valid_tanggal(t) {
            return Err("Format tanggal kerja tidak valid".to_string());
        }
    }
    if let Some(id) = input.akun_laba_ditahan_id {
        let ok: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM akun WHERE id=?1 AND tipe='Modal' AND header=0",
                rusqlite::params![id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if ok == 0 {
            return Err("Akun Laba Ditahan harus berupa akun tipe Modal (bukan akun induk)".to_string());
        }
    }
    conn.execute(
        "UPDATE perusahaan SET nama=?1,alamat=?2,kota=?3,telepon=?4,email=?5,npwp=?6,akun_laba_ditahan_id=?7,tanggal_kerja=?8 WHERE id=1",
        rusqlite::params![
            input.nama.trim(), input.alamat, input.kota, input.telepon, input.email, input.npwp,
            input.akun_laba_ditahan_id, input.tanggal_kerja
        ],
    )
    .map_err(|e| e.to_string())?;
    fetch_perusahaan(&conn)
}

#[tauri::command]
pub fn get_tanggal_kerja(db: State<Database>) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(tanggal_hari_ini(&conn))
}

// ── Periode ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_periode(db: State<Database>) -> Result<Vec<Periode>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(&format!("{} ORDER BY tanggal_mulai DESC", SELECT_PERIODE))
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], map_periode).map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn get_periode_aktif(db: State<Database>) -> Result<Periode, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    periode_aktif(&conn)
}

fn validasi_rentang(input: &PeriodeInput) -> Result<(), String> {
    if input.nama.trim().is_empty() {
        return Err("Nama periode wajib diisi".to_string());
    }
    if !valid_tanggal(&input.tanggal_mulai) || !valid_tanggal(&input.tanggal_selesai) {
        return Err("Format tanggal periode tidak valid".to_string());
    }
    if input.tanggal_selesai < input.tanggal_mulai {
        return Err("Tanggal selesai tidak boleh sebelum tanggal mulai".to_string());
    }
    Ok(())
}

fn selesai_periode_tertutup_terakhir(conn: &Connection) -> Option<String> {
    conn.query_row("SELECT MAX(tanggal_selesai) FROM periode WHERE status='Ditutup'", [], |r| r.get(0))
        .ok()
        .flatten()
}

/// Mengubah nama/rentang periode aktif. Seluruh transaksi periode ini harus tetap berada di dalam rentang baru.
#[tauri::command]
pub fn update_periode(db: State<Database>, id: i64, input: PeriodeInput) -> Result<Periode, String> {
    validasi_rentang(&input)?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let p = periode_aktif(&conn)?;
    if p.id != id {
        return Err("Hanya periode aktif yang dapat diubah".to_string());
    }
    let prev = selesai_periode_tertutup_terakhir(&conn).unwrap_or_default();
    if !prev.is_empty() && input.tanggal_mulai <= prev {
        return Err(format!("Tanggal mulai harus setelah periode sebelumnya berakhir ({})", prev));
    }
    let (min, max): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT MIN(tanggal), MAX(tanggal) FROM jurnal WHERE tanggal > ?1",
            rusqlite::params![prev],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    if let (Some(min), Some(max)) = (min, max) {
        if min < input.tanggal_mulai || max > input.tanggal_selesai {
            return Err(format!(
                "Sudah ada transaksi bertanggal {} s/d {}; rentang periode harus mencakup semuanya",
                min, max
            ));
        }
    }
    conn.execute(
        "UPDATE periode SET nama=?1,tanggal_mulai=?2,tanggal_selesai=?3 WHERE id=?4",
        rusqlite::params![input.nama.trim(), input.tanggal_mulai, input.tanggal_selesai, id],
    )
    .map_err(|e| e.to_string())?;
    periode_aktif(&conn)
}

fn simpan_arsip(conn: &Connection, periode_id: i64, jenis: &str, data: &serde_json::Value) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO laporan_arsip (periode_id,jenis,data) VALUES (?1,?2,?3)",
        rusqlite::params![periode_id, jenis, data.to_string()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Tutup buku: membuat jurnal penutup (pendapatan & beban → Laba Ditahan), mengarsipkan
/// Laba Rugi dan Neraca periode ini, menutup periode, lalu membuka periode baru.
#[tauri::command]
pub fn tutup_periode(db: State<Database>, periode_baru: PeriodeInput) -> Result<Periode, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    tutup_periode_inner(&conn, periode_baru)
}

pub fn tutup_periode_inner(conn: &Connection, periode_baru: PeriodeInput) -> Result<Periode, String> {
    validasi_rentang(&periode_baru)?;
    let p = periode_aktif(conn)?;
    if periode_baru.tanggal_mulai <= p.tanggal_selesai {
        return Err(format!(
            "Periode baru harus dimulai setelah {} (akhir periode berjalan)",
            p.tanggal_selesai
        ));
    }

    let laba_ditahan: i64 = conn
        .query_row("SELECT akun_laba_ditahan_id FROM perusahaan WHERE id=1", [], |r| r.get::<_, Option<i64>>(0))
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Pilih akun Laba Ditahan terlebih dahulu di menu Profil Perusahaan".to_string())?;

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    // Laba rugi periode (jurnal penutup tidak ikut dihitung)
    let laba_rugi = laba_rugi_rentang(&tx, &p.tanggal_mulai, &p.tanggal_selesai)?;

    // Jurnal penutup: nolkan saldo pendapatan & beban
    let akun_nominal: Vec<(i64, f64)> = {
        let mut stmt = tx
            .prepare("SELECT id, saldo FROM akun WHERE tipe IN ('Pendapatan','Beban') AND header=0 AND ABS(saldo) > 0.005")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?)))
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| e.to_string())?);
        }
        out
    };
    if !akun_nominal.is_empty() {
        let mut detail: Vec<JurnalDetailInput> = akun_nominal
            .iter()
            .map(|&(akun_id, saldo)| JurnalDetailInput {
                akun_id,
                keterangan: None,
                debit: if saldo < 0.0 { -saldo } else { 0.0 },
                kredit: if saldo > 0.0 { saldo } else { 0.0 },
            })
            .collect();
        let net: f64 = akun_nominal.iter().map(|&(_, s)| s).sum(); // < 0 laba, > 0 rugi
        if net.abs() > 0.005 {
            detail.push(JurnalDetailInput {
                akun_id: laba_ditahan,
                keterangan: Some("Laba (rugi) periode".to_string()),
                debit: if net > 0.0 { net } else { 0.0 },
                kredit: if net < 0.0 { -net } else { 0.0 },
            });
        }
        let ket = format!("Jurnal penutup {}", p.nama);
        insert_jurnal(&tx, &p.tanggal_selesai, Some(&ket), "Penutup", &detail)?;
    }

    // Arsip laporan periode ini
    let neraca = neraca_sekarang(&tx)?;
    let meta = serde_json::json!({
        "nama": p.nama, "tanggal_mulai": p.tanggal_mulai, "tanggal_selesai": p.tanggal_selesai
    });
    simpan_arsip(&tx, p.id, "LabaRugi", &serde_json::json!({ "meta": meta, "items": laba_rugi }))?;
    simpan_arsip(&tx, p.id, "Neraca", &serde_json::json!({ "meta": meta, "items": neraca }))?;

    tx.execute(
        "UPDATE periode SET status='Ditutup', closed_at=datetime('now','localtime') WHERE id=?1",
        rusqlite::params![p.id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO periode (nama,tanggal_mulai,tanggal_selesai,status) VALUES (?1,?2,?3,'Aktif')",
        rusqlite::params![periode_baru.nama.trim(), periode_baru.tanggal_mulai, periode_baru.tanggal_selesai],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    periode_aktif(conn)
}

#[tauri::command]
pub fn get_arsip_laporan(db: State<Database>, periode_id: i64) -> Result<Vec<LaporanArsip>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT jenis,created_at,data FROM laporan_arsip WHERE periode_id=?1 ORDER BY jenis")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![periode_id], |r| {
            let data: String = r.get(2)?;
            Ok(LaporanArsip {
                jenis: r.get(0)?,
                created_at: r.get(1)?,
                data: serde_json::from_str(&data).unwrap_or(serde_json::Value::Null),
            })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}
