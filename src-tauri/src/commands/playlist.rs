use tauri::AppHandle;

use crate::db::get_connection;
use crate::models::{Playlist, PlaylistItem};

#[tauri::command]
pub async fn get_playlists(app: AppHandle) -> Result<Vec<Playlist>, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_playlists_blocking(app_handle))
        .await
        .map_err(|e| e.to_string())?
}

fn get_playlists_blocking(app: AppHandle) -> Result<Vec<Playlist>, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, sort_key, sort_order, created_at FROM playlists")
        .map_err(|e| e.to_string())?;
    
    let playlists: Result<Vec<Playlist>, _> = stmt
        .query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_key: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect();
    
    playlists.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_playlist(app: AppHandle, name: String) -> Result<Playlist, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || create_playlist_blocking(app_handle, name))
        .await
        .map_err(|e| e.to_string())?
}

fn create_playlist_blocking(app: AppHandle, name: String) -> Result<Playlist, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO playlists (name, sort_key, sort_order, created_at) VALUES (?, 'title', 'asc', ?)",
        (
            &name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        ),
    ).map_err(|e| e.to_string())?;
    
    let id = conn.last_insert_rowid();
    
    let playlist: Playlist = conn
        .query_row(
            "SELECT id, name, sort_key, sort_order, created_at FROM playlists WHERE id = ?",
            [id],
            |row| {
                Ok(Playlist {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    sort_key: row.get(2)?,
                    sort_order: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;
    
    Ok(playlist)
}

#[tauri::command]
pub async fn add_to_playlist(app: AppHandle, playlist_id: i64, track_ids: Vec<i64>) -> Result<usize, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || add_to_playlist_blocking(app_handle, playlist_id, track_ids))
        .await
        .map_err(|e| e.to_string())?
}

fn add_to_playlist_blocking(app: AppHandle, playlist_id: i64, track_ids: Vec<i64>) -> Result<usize, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    
    let count = conn
        .query_row(
            "SELECT COUNT(*) FROM playlist_items WHERE playlist_id = ?",
            [playlist_id],
            |row| row.get::<_, i32>(0),
        )
        .map_err(|e| e.to_string())?;
    
    let mut added = 0;
    for (i, track_id) in track_ids.iter().enumerate() {
        conn.execute(
            "INSERT OR IGNORE INTO playlist_items (playlist_id, track_id, position) VALUES (?, ?, ?)",
            (playlist_id, track_id, count + i as i32),
        ).map_err(|e| e.to_string())?;
        added += 1;
    }
    
    Ok(added)
}

#[tauri::command]
pub async fn get_playlist_items(app: AppHandle, playlist_id: i64) -> Result<Vec<i64>, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_playlist_items_blocking(app_handle, playlist_id))
        .await
        .map_err(|e| e.to_string())?
}

fn get_playlist_items_blocking(app: AppHandle, playlist_id: i64) -> Result<Vec<i64>, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT track_id FROM playlist_items WHERE playlist_id = ? ORDER BY position")
        .map_err(|e| e.to_string())?;
    
    let track_ids: Result<Vec<i64>, _> = stmt
        .query_map([playlist_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect();
    
    track_ids.map_err(|e| e.to_string())
}
