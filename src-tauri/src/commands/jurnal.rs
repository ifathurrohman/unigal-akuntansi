use crate::db::Database;
use crate::models::{Jurnal, JurnalDetail, JurnalInput};
use tauri::State;

fn generate_nomor(conn: &rusqlite::Connection, tipe: &str) -> String {
    let prefix = match tipe {
        "Kas" => "KAS",
        "Bank" => "BNK",
        _ => "JU",
    };
    let now = chrono::Local::now();
    let ym = now.format("%Y%m").to_string();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM jurnal WHERE nomor LIKE ?1",
            rusqlite::params![format!("{}-{}-%", prefix, ym)],
            |r| r.get(0),
        )
        .unwrap_or(0);
    format!("{}-{}-{:04}", prefix, ym, count + 1)
}

fn fetch_jurnal(conn: &rusqlite::Connection, id: i64) -> Result<Jurnal, String> {
    let jurnal = conn
        .query_row(
            "SELECT id,nomor,tanggal,keterangan,tipe,created_at FROM jurnal WHERE id=?1",
            rusqlite::params![id],
            |row| Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            )),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT jd.id,jd.jurnal_id,jd.akun_id,a.kode,a.nama,jd.keterangan,jd.debit,jd.kredit
             FROM jurnal_detail jd JOIN akun a ON a.id=jd.akun_id WHERE jd.jurnal_id=?1 ORDER BY jd.id",
        )
        .map_err(|e| e.to_string())?;
    let detail: Vec<JurnalDetail> = stmt
        .query_map(rusqlite::params![id], |row| {
            Ok(JurnalDetail {
                id: row.get(0)?,
                jurnal_id: row.get(1)?,
                akun_id: row.get(2)?,
                akun_kode: row.get(3)?,
                akun_nama: row.get(4)?,
                keterangan: row.get(5)?,
                debit: row.get(6)?,
                kredit: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let total_debit: f64 = detail.iter().map(|d| d.debit).sum();
    let total_kredit: f64 = detail.iter().map(|d| d.kredit).sum();

    Ok(Jurnal {
        id: jurnal.0,
        nomor: jurnal.1,
        tanggal: jurnal.2,
        keterangan: jurnal.3,
        tipe: jurnal.4,
        total_debit,
        total_kredit,
        created_at: jurnal.5,
        detail,
    })
}

#[tauri::command]
pub fn get_jurnal(db: State<Database>) -> Result<Vec<Jurnal>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id FROM jurnal ORDER BY tanggal DESC, id DESC")
        .map_err(|e| e.to_string())?;
    let ids: Vec<i64> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    ids.iter()
        .map(|&id| fetch_jurnal(&conn, id))
        .collect()
}

#[tauri::command]
pub fn create_jurnal(db: State<Database>, input: JurnalInput) -> Result<Jurnal, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let total_debit: f64 = input.detail.iter().map(|d| d.debit).sum();
    let total_kredit: f64 = input.detail.iter().map(|d| d.kredit).sum();
    if (total_debit - total_kredit).abs() > 0.01 {
        return Err("Total debit harus sama dengan total kredit".to_string());
    }

    let tipe = input.tipe.as_deref().unwrap_or("Umum");
    let nomor = generate_nomor(&conn, tipe);

    conn.execute(
        "INSERT INTO jurnal (nomor,tanggal,keterangan,tipe) VALUES (?1,?2,?3,?4)",
        rusqlite::params![nomor, input.tanggal, input.keterangan, tipe],
    )
    .map_err(|e| e.to_string())?;
    let jurnal_id = conn.last_insert_rowid();

    for d in &input.detail {
        conn.execute(
            "INSERT INTO jurnal_detail (jurnal_id,akun_id,keterangan,debit,kredit) VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params![jurnal_id, d.akun_id, d.keterangan, d.debit, d.kredit],
        )
        .map_err(|e| e.to_string())?;

        // Update saldo akun
        conn.execute(
            "UPDATE akun SET saldo = saldo + ?1 - ?2 WHERE id = ?3",
            rusqlite::params![d.debit, d.kredit, d.akun_id],
        )
        .map_err(|e| e.to_string())?;
    }

    fetch_jurnal(&conn, jurnal_id)
}

#[tauri::command]
pub fn delete_jurnal(db: State<Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Reverse saldo akun
    let mut stmt = conn
        .prepare("SELECT akun_id, debit, kredit FROM jurnal_detail WHERE jurnal_id=?1")
        .map_err(|e| e.to_string())?;
    let details: Vec<(i64, f64, f64)> = stmt
        .query_map(rusqlite::params![id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for (akun_id, debit, kredit) in details {
        conn.execute(
            "UPDATE akun SET saldo = saldo - ?1 + ?2 WHERE id = ?3",
            rusqlite::params![debit, kredit, akun_id],
        )
        .map_err(|e| e.to_string())?;
    }

    conn.execute("DELETE FROM jurnal WHERE id=?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
