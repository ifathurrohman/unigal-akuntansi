use crate::db::Database;
use crate::commands::pengaturan::pastikan_periode_aktif;
use crate::commands::jurnal::{insert_jurnal, remove_jurnal};
use crate::models::{JurnalDetailInput, KasBank, KasBankInput};
use tauri::State;

fn generate_nomor_kb(conn: &rusqlite::Connection, tipe: &str, tanggal: &str) -> Result<String, String> {
    let prefix = match tipe {
        "Masuk" => "KM",
        "Keluar" => "KK",
        _ => "TR",
    };
    let ym = tanggal.get(..7).unwrap_or("").replace('-', "");
    let max: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(CAST(substr(nomor, -4) AS INTEGER)), 0) FROM kas_bank WHERE nomor LIKE ?1",
            rusqlite::params![format!("{}-{}-%", prefix, ym)],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(format!("{}-{}-{:04}", prefix, ym, max + 1))
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

    if !input.jumlah.is_finite() || input.jumlah <= 0.0 {
        return Err("Jumlah harus lebih dari nol".to_string());
    }
    let lawan_id = input
        .akun_lawan_id
        .ok_or_else(|| "Akun lawan wajib dipilih agar jurnal seimbang".to_string())?;
    if lawan_id == input.akun_id {
        return Err("Akun lawan tidak boleh sama dengan akun kas/bank".to_string());
    }

    // Masuk: Dr kas/bank, Kr lawan. Keluar & Transfer (akun -> lawan): Dr lawan, Kr kas/bank.
    let (debit_id, kredit_id) = match input.tipe.as_str() {
        "Masuk" => (input.akun_id, lawan_id),
        "Keluar" | "Transfer" => (lawan_id, input.akun_id),
        _ => return Err("Tipe transaksi tidak valid".to_string()),
    };
    let detail = vec![
        JurnalDetailInput { akun_id: debit_id, keterangan: input.keterangan.clone(), debit: input.jumlah, kredit: 0.0 },
        JurnalDetailInput { akun_id: kredit_id, keterangan: input.keterangan.clone(), debit: 0.0, kredit: input.jumlah },
    ];

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let nomor = generate_nomor_kb(&tx, &input.tipe, &input.tanggal)?;
    let jurnal_id = insert_jurnal(&tx, &input.tanggal, input.keterangan.as_deref(), "Kas", &detail)?;
    tx.execute(
        "INSERT INTO kas_bank (nomor,tanggal,keterangan,tipe,akun_id,akun_lawan_id,jumlah,jurnal_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        rusqlite::params![
            nomor, input.tanggal, input.keterangan, input.tipe,
            input.akun_id, lawan_id, input.jumlah, jurnal_id
        ],
    )
    .map_err(|e| e.to_string())?;
    let kb_id = tx.last_insert_rowid();
    tx.commit().map_err(|e| e.to_string())?;

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
    let (tipe, akun_id, akun_lawan_id, jumlah, jurnal_id): (String, i64, Option<i64>, f64, Option<i64>) = conn
        .query_row(
            "SELECT tipe,akun_id,akun_lawan_id,jumlah,jurnal_id FROM kas_bank WHERE id=?1",
            rusqlite::params![id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(|e| e.to_string())?;

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM kas_bank WHERE id=?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;

    match jurnal_id {
        Some(jid) => remove_jurnal(&tx, jid)?,
        None => {
            let tgl: String = tx
                .query_row("SELECT tanggal FROM kas_bank WHERE id=?1", rusqlite::params![id], |r| r.get(0))
                .unwrap_or_default();
            pastikan_periode_aktif(&tx, &tgl)?;
            // Transaksi lama (sebelum ada jurnal otomatis): balik saldo secara manual.
            let sign = if tipe == "Masuk" { 1.0 } else { -1.0 };
            tx.execute("UPDATE akun SET saldo = saldo - ?1 WHERE id = ?2", rusqlite::params![sign * jumlah, akun_id])
                .map_err(|e| e.to_string())?;
            if let Some(lawan_id) = akun_lawan_id {
                tx.execute("UPDATE akun SET saldo = saldo + ?1 WHERE id = ?2", rusqlite::params![sign * jumlah, lawan_id])
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    tx.commit().map_err(|e| e.to_string())
}
