use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::db::get_connection;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFolder {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub track_count: i64,
    pub added_at: i64,
}

#[tauri::command]
pub async fn get_scan_folders(app: AppHandle) -> Result<Vec<ScanFolder>, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_scan_folders_blocking(app_handle))
        .await
        .map_err(|e| e.to_string())?
}

fn get_scan_folders_blocking(app: AppHandle) -> Result<Vec<ScanFolder>, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT sf.id, sf.path, sf.name, sf.added_at, COUNT(t.id) as track_count
             FROM scan_folders sf
             LEFT JOIN tracks t ON t.scan_folder_id = sf.id
             GROUP BY sf.id
             ORDER BY sf.added_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let folders = stmt
        .query_map([], |row| {
            Ok(ScanFolder {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                added_at: row.get(3)?,
                track_count: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(folders)
}

#[tauri::command]
pub async fn delete_scan_folder(app: AppHandle, folder_id: i64) -> Result<(), String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || delete_scan_folder_blocking(app_handle, folder_id))
        .await
        .map_err(|e| e.to_string())?
}

fn delete_scan_folder_blocking(app: AppHandle, folder_id: i64) -> Result<(), String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM tracks WHERE scan_folder_id = ?",
        [folder_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM scan_folders WHERE id = ?",
        [folder_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
