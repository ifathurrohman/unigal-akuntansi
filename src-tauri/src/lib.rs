mod db;
mod models;
mod commands;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let db_path = app_dir.join("akuntansi.db");
            let database = Database::new(db_path.to_str().unwrap()).expect("failed to open database");
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::akun::get_akun,
            commands::akun::get_akun_kas_bank,
            commands::akun::create_akun,
            commands::akun::update_akun,
            commands::akun::delete_akun,
            commands::pelanggan::get_pelanggan,
            commands::pelanggan::create_pelanggan,
            commands::pelanggan::update_pelanggan,
            commands::pelanggan::delete_pelanggan,
            commands::supplier::get_supplier,
            commands::supplier::create_supplier,
            commands::supplier::update_supplier,
            commands::supplier::delete_supplier,
            commands::produk::get_produk,
            commands::produk::create_produk,
            commands::produk::update_produk,
            commands::produk::delete_produk,
            commands::jurnal::get_jurnal,
            commands::jurnal::create_jurnal,
            commands::jurnal::delete_jurnal,
            commands::kas_bank::get_kas_bank,
            commands::kas_bank::create_kas_bank,
            commands::kas_bank::delete_kas_bank,
            commands::laporan::get_dashboard,
            commands::laporan::get_laba_rugi,
            commands::laporan::get_neraca,
            commands::pengaturan::get_perusahaan,
            commands::pengaturan::save_perusahaan,
            commands::pengaturan::get_tanggal_kerja,
            commands::pengaturan::get_periode,
            commands::pengaturan::get_periode_aktif,
            commands::pengaturan::update_periode,
            commands::pengaturan::tutup_periode,
            commands::pengaturan::get_arsip_laporan,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
