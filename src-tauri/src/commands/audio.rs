use tauri::AppHandle;

use crate::audio::PLAYER;
use crate::db::get_connection;
use crate::models::{PlayMode, PlayerState, Track};

fn cover_from_row(row: &rusqlite::Row, idx: usize) -> Result<Option<String>, rusqlite::Error> {
    let raw: rusqlite::types::Value = row.get(idx)?;
    Ok(match raw {
        rusqlite::types::Value::Text(s) => Some(s),
        rusqlite::types::Value::Blob(b) => {
            use base64::Engine;
            Some(base64::engine::general_purpose::STANDARD.encode(&b))
        }
        _ => None,
    })
}

fn load_track_blocking(app: AppHandle, track_id: i64) -> Result<Track, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;

    let track_exists: bool = conn
        .query_row("SELECT COUNT(*) FROM tracks WHERE id = ?", [track_id], |row| row.get::<_, i64>(0))
        .map(|count| count > 0)
        .unwrap_or(false);

    if !track_exists {
        return Err(format!("曲目 ID {} 不存在，请尝试重新扫描文件夹", track_id));
    }

    let track: Track = conn
        .query_row(
            "SELECT id, path, title, artist, album, year, track_number, duration, bitrate, sample_rate, cover_blob, lyrics, added_at, modified_at FROM tracks WHERE id = ?",
            [track_id],
            |row| {
                Ok(Track {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    title: row.get(2)?,
                    artist: row.get(3)?,
                    album: row.get(4)?,
                    year: row.get(5)?,
                    track_number: row.get(6)?,
                    duration: row.get(7)?,
                    bitrate: row.get(8)?,
                    sample_rate: row.get(9)?,
                    cover_blob: cover_from_row(row, 10)?,
                    lyrics: row.get(11)?,
                    added_at: row.get(12)?,
                    modified_at: row.get(13)?,
                })
            },
        )
        .map_err(|e| format!("加载曲目失败: {}", e))?;

    Ok(track)
}

#[tauri::command]
pub async fn play(app: AppHandle, track_id: i64) -> Result<PlayerState, String> {
    {
        use crate::models::PlayState;
        let cur = PLAYER.get_state();
        if cur.current_track_id == Some(track_id)
            && matches!(cur.state, PlayState::Paused)
        {
            PLAYER.resume();
            std::thread::sleep(std::time::Duration::from_millis(30));
            return Ok(PLAYER.get_state());
        }
    }

    let track = {
        let app_handle = app.clone();
        tauri::async_runtime::spawn_blocking(move || load_track_blocking(app_handle, track_id))
            .await
            .map_err(|e| e.to_string())??
    };

    PLAYER.play(&track)?;
    Ok(PLAYER.get_state())
}

#[tauri::command]
pub async fn resume() -> PlayerState {
    PLAYER.resume();
    std::thread::sleep(std::time::Duration::from_millis(30));
    PLAYER.get_state()
}

#[tauri::command]
pub async fn pause() -> PlayerState {
    PLAYER.pause();
    std::thread::sleep(std::time::Duration::from_millis(30));
    PLAYER.get_state()
}

#[tauri::command]
pub fn stop() -> PlayerState {
    PLAYER.stop();
    PLAYER.get_state()
}

#[tauri::command]
pub fn prev() -> PlayerState {
    PLAYER.prev();
    PLAYER.get_state()
}

#[tauri::command]
pub fn next() -> PlayerState {
    PLAYER.next();
    PLAYER.get_state()
}

#[tauri::command]
pub async fn seek(position: f64) -> PlayerState {
    PLAYER.seek((position * 1000.0) as u64);
    std::thread::sleep(std::time::Duration::from_millis(20));
    PLAYER.get_state()
}

#[tauri::command]
pub fn set_volume(volume: f32) -> PlayerState {
    PLAYER.set_volume(volume);
    PLAYER.get_state()
}

#[tauri::command]
pub fn set_mode(mode: String) -> PlayerState {
    let play_mode = match mode.as_str() {
        "shuffle" => PlayMode::Shuffle,
        "repeat_one" => PlayMode::RepeatOne,
        "repeat_list" => PlayMode::RepeatList,
        _ => PlayMode::RepeatList,
    };
    PLAYER.set_mode(play_mode);
    PLAYER.get_state()
}

#[tauri::command]
pub fn get_player_state() -> PlayerState {
    PLAYER.get_state()
}

#[tauri::command]
pub fn set_playlist_from_tracks(tracks: Vec<Track>) {
    PLAYER.set_playlist(tracks);
}
