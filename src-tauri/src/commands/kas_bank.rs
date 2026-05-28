use crate::db::Database;
use crate::models::{KasBank, KasBankInput};
use tauri::State;

fn generate_nomor_kb(conn: &rusqlite::Connection, tipe: &str) -> String {
    let prefix = match tipe {
        "Masuk" => "KM",
        "Keluar" => "KK",
        _ => "TR",
    };
    let now = chrono::Local::now();
    let ym = now.format("%Y%m").to_string();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM kas_bank WHERE nomor LIKE ?1",
            rusqlite::params![format!("{}-{}-%", prefix, ym)],
            |r| r.get(0),
        )
        .unwrap_or(0);
    format!("{}-{}-{:04}", prefix, ym, count + 1)
}

#[tauri::command]
pub fn get_kas_bank(db: State<Database>) -> Result<Vec<KasBank>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT kb.id,kb.nomor,kb.tanggal,kb.keterangan,kb.tipe,kb.akun_id,a1.nama,kb.akun_lawan_id,a2.nama,kb.jumlah,kb.created_at
             FROM kas_bank kb
             JOIN akun a1 ON a1.id=kb.akun_id
             LEFT JOIN akun a2 ON a2.id=kb.akun_lawan_id
             ORDER BY kb.tanggal DESC, kb.id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(KasBank {
                id: row.get(0)?,
                nomor: row.get(1)?,
                tanggal: row.get(2)?,
                keterangan: row.get(3)?,
                tipe: row.get(4)?,
                akun_id: row.get(5)?,
                akun_nama: row.get(6)?,
                akun_lawan_id: row.get(7)?,
                akun_lawan_nama: row.get(8)?,
                jumlah: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

#[tauri::command]
pub fn create_kas_bank(db: State<Database>, input: KasBankInput) -> Result<KasBank, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let nomor = generate_nomor_kb(&conn, &input.tipe);

    conn.execute(
        "INSERT INTO kas_bank (nomor,tanggal,keterangan,tipe,akun_id,akun_lawan_id,jumlah) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        rusqlite::params![
            nomor, input.tanggal, input.keterangan, input.tipe,
            input.akun_id, input.akun_lawan_id, input.jumlah
        ],
    )
    .map_err(|e| e.to_string())?;
    let kb_id = conn.last_insert_rowid();

    // Update saldo akun utama
    match input.tipe.as_str() {
        "Masuk" => {
            conn.execute(
                "UPDATE akun SET saldo = saldo + ?1 WHERE id = ?2",
                rusqlite::params![input.jumlah, input.akun_id],
            ).map_err(|e| e.to_string())?;
            if let Some(lawan_id) = input.akun_lawan_id {
                conn.execute(
                    "UPDATE akun SET saldo = saldo - ?1 WHERE id = ?2",
                    rusqlite::params![input.jumlah, lawan_id],
                ).map_err(|e| e.to_string())?;
            }
        }
        "Keluar" => {
            conn.execute(
                "UPDATE akun SET saldo = saldo - ?1 WHERE id = ?2",
                rusqlite::params![input.jumlah, input.akun_id],
            ).map_err(|e| e.to_string())?;
            if let Some(lawan_id) = input.akun_lawan_id {
                conn.execute(
                    "UPDATE akun SET saldo = saldo + ?1 WHERE id = ?2",
                    rusqlite::params![input.jumlah, lawan_id],
                ).map_err(|e| e.to_string())?;
            }
        }
        _ => {}
    }

    let kb = conn
        .query_row(
            "SELECT kb.id,kb.nomor,kb.tanggal,kb.keterangan,kb.tipe,kb.akun_id,a1.nama,kb.akun_lawan_id,a2.nama,kb.jumlah,kb.created_at
             FROM kas_bank kb JOIN akun a1 ON a1.id=kb.akun_id LEFT JOIN akun a2 ON a2.id=kb.akun_lawan_id WHERE kb.id=?1",
            rusqlite::params![kb_id],
            |row| Ok(KasBank {
                id: row.get(0)?,
                nomor: row.get(1)?,
                tanggal: row.get(2)?,
                keterangan: row.get(3)?,
                tipe: row.get(4)?,
                akun_id: row.get(5)?,
                akun_nama: row.get(6)?,
                akun_lawan_id: row.get(7)?,
                akun_lawan_nama: row.get(8)?,
                jumlah: row.get(9)?,
                created_at: row.get(10)?,
            }),
        )
        .map_err(|e| e.to_string())?;
    Ok(kb)
}

#[tauri::command]
pub fn delete_kas_bank(db: State<Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let (tipe, akun_id, akun_lawan_id, jumlah): (String, i64, Option<i64>, f64) = conn
        .query_row(
            "SELECT tipe,akun_id,akun_lawan_id,jumlah FROM kas_bank WHERE id=?1",
            rusqlite::params![id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| e.to_string())?;

    match tipe.as_str() {
        "Masuk" => {
            conn.execute("UPDATE akun SET saldo = saldo - ?1 WHERE id = ?2", rusqlite::params![jumlah, akun_id]).map_err(|e| e.to_string())?;
            if let Some(lawan_id) = akun_lawan_id {
                conn.execute("UPDATE akun SET saldo = saldo + ?1 WHERE id = ?2", rusqlite::params![jumlah, lawan_id]).map_err(|e| e.to_string())?;
            }
        }
        "Keluar" => {
            conn.execute("UPDATE akun SET saldo = saldo + ?1 WHERE id = ?2", rusqlite::params![jumlah, akun_id]).map_err(|e| e.to_string())?;
            if let Some(lawan_id) = akun_lawan_id {
                conn.execute("UPDATE akun SET saldo = saldo - ?1 WHERE id = ?2", rusqlite::params![jumlah, lawan_id]).map_err(|e| e.to_string())?;
            }
        }
        _ => {}
    }

    conn.execute("DELETE FROM kas_bank WHERE id=?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
