use std::path::PathBuf;

use tauri::AppHandle;

use crate::db::get_connection;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LyricLine {
    pub time: f64,
    pub text: String,
}

fn parse_lrc_content(content: &str) -> Vec<LyricLine> {
    let mut lines: Vec<LyricLine> = Vec::new();
    let re = regex::Regex::new(r"\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)").unwrap();

    for cap in re.captures_iter(content) {
        let minutes: f64 = cap[1].parse().unwrap_or(0.0);
        let seconds: f64 = cap[2].parse().unwrap_or(0.0);
        let milliseconds: f64 = cap[3].parse().unwrap_or(0.0);
        let text = cap[4].trim().to_string();

        if !text.is_empty() {
            let time = minutes * 60.0 + seconds + milliseconds / 1000.0;
            lines.push(LyricLine { time, text });
        }
    }

    lines.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    lines
}

#[tauri::command]
pub async fn get_lyrics(app: AppHandle, track_id: i64) -> Result<Vec<LyricLine>, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_lyrics_blocking(app_handle, track_id))
        .await
        .map_err(|e| e.to_string())?
}

fn get_lyrics_blocking(app: AppHandle, track_id: i64) -> Result<Vec<LyricLine>, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    
    let lyrics: Option<String> = conn
        .query_row(
            "SELECT lyrics FROM tracks WHERE id = ?",
            [track_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    
    if let Some(lyrics) = lyrics {
        Ok(parse_lrc_content(&lyrics))
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
pub fn parse_lrc(content: String) -> Vec<LyricLine> {
    parse_lrc_content(&content)
}

#[tauri::command]
pub async fn load_lrc_from_file(track_path: String) -> Result<Vec<LyricLine>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut path = PathBuf::from(track_path);
        path.set_extension("lrc");
        
        if let Ok(content) = std::fs::read_to_string(&path) {
            Ok(parse_lrc_content(&content))
        } else {
            Ok(Vec::new())
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
