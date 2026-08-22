use tauri::AppHandle;

use crate::db::get_connection;

#[tauri::command]
pub async fn toggle_favorite(app: AppHandle, track_id: i64) -> Result<bool, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || toggle_favorite_blocking(app_handle, track_id))
        .await
        .map_err(|e| e.to_string())?
}

fn toggle_favorite_blocking(app: AppHandle, track_id: i64) -> Result<bool, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM favorites WHERE track_id = ?)",
            [track_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    
    if exists {
        conn.execute(
            "DELETE FROM favorites WHERE track_id = ?",
            [track_id],
        ).map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        conn.execute(
            "INSERT INTO favorites (track_id, added_at) VALUES (?, ?)",
            (
                track_id,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            ),
        ).map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub async fn get_favorites(app: AppHandle) -> Result<Vec<i64>, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_favorites_blocking(app_handle))
        .await
        .map_err(|e| e.to_string())?
}

fn get_favorites_blocking(app: AppHandle) -> Result<Vec<i64>, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT track_id FROM favorites ORDER BY added_at DESC")
        .map_err(|e| e.to_string())?;
    
    let track_ids: Result<Vec<i64>, _> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect();
    
    track_ids.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_favorite(app: AppHandle, track_id: i64) -> Result<bool, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || is_favorite_blocking(app_handle, track_id))
        .await
        .map_err(|e| e.to_string())?
}

fn is_favorite_blocking(app: AppHandle, track_id: i64) -> Result<bool, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM favorites WHERE track_id = ?)",
            [track_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    
    Ok(exists)
}
