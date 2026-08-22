use tauri::AppHandle;

use crate::db::get_connection;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlayStats {
    pub total_tracks_played: i64,
    pub total_duration_played: i64,
    pub top_tracks: Vec<TopTrack>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TopTrack {
    pub track_id: i64,
    pub title: String,
    pub artist: Option<String>,
    pub play_count: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlayHistoryEntry {
    pub track_id: i64,
    pub played_at: i64,
    pub duration_played: i64,
    pub completed: bool,
}

#[tauri::command]
pub async fn get_stats(app: AppHandle) -> Result<PlayStats, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_stats_blocking(app_handle))
        .await
        .map_err(|e| e.to_string())?
}

fn get_stats_blocking(app: AppHandle) -> Result<PlayStats, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    
    let total_tracks_played: i64 = conn
        .query_row("SELECT COUNT(*) FROM play_history", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    
    let total_duration_played: i64 = conn
        .query_row("SELECT COALESCE(SUM(duration_played), 0) FROM play_history", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    
    let mut stmt = conn
        .prepare(r#"
            SELECT t.id, t.title, t.artist, COUNT(*) as play_count
            FROM play_history ph
            JOIN tracks t ON ph.track_id = t.id
            GROUP BY t.id
            ORDER BY play_count DESC
            LIMIT 10
        "#)
        .map_err(|e| e.to_string())?;
    
    let top_tracks: Result<Vec<TopTrack>, _> = stmt
        .query_map([], |row| {
            Ok(TopTrack {
                track_id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                play_count: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect();
    
    Ok(PlayStats {
        total_tracks_played,
        total_duration_played,
        top_tracks: top_tracks.map_err(|e| e.to_string())?,
    })
}

#[tauri::command]
pub async fn get_play_history(app: AppHandle, days: i64) -> Result<Vec<PlayHistoryEntry>, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_play_history_blocking(app_handle, days))
        .await
        .map_err(|e| e.to_string())?
}

fn get_play_history_blocking(app: AppHandle, days: i64) -> Result<Vec<PlayHistoryEntry>, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    
    let since = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64)
        - (days * 24 * 60 * 60);
    
    let mut stmt = conn
        .prepare("SELECT track_id, played_at, duration_played, completed FROM play_history WHERE played_at >= ? ORDER BY played_at DESC")
        .map_err(|e| e.to_string())?;
    
    let history: Result<Vec<PlayHistoryEntry>, _> = stmt
        .query_map([since], |row| {
            Ok(PlayHistoryEntry {
                track_id: row.get(0)?,
                played_at: row.get(1)?,
                duration_played: row.get(2)?,
                completed: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect();
    
    history.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn record_play_history(app: AppHandle, track_id: i64, duration_played: i64, completed: bool) -> Result<(), String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || record_play_history_blocking(app_handle, track_id, duration_played, completed))
        .await
        .map_err(|e| e.to_string())?
}

fn record_play_history_blocking(app: AppHandle, track_id: i64, duration_played: i64, completed: bool) -> Result<(), String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO play_history (track_id, played_at, duration_played, completed) VALUES (?, ?, ?, ?)",
        (
            track_id,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            duration_played,
            completed,
        ),
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}
