use std::path::PathBuf;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use super::schema::*;

pub fn get_database_path(app: &AppHandle) -> PathBuf {
    let data_dir = app.path().app_data_dir()
        .unwrap_or_else(|_| {
            let home = std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".ychomusic")
        });
    std::fs::create_dir_all(&data_dir).unwrap_or_else(|e| {
        log::error!("Failed to create data directory: {}", e);
    });
    data_dir.join("ychomusic.db")
}

pub fn initialize_database(app: &AppHandle) -> Result<(), rusqlite::Error> {
    let db_path = get_database_path(app);
    let conn = Connection::open(&db_path)?;

    conn.execute(CREATE_TRACKS_TABLE, [])?;
    conn.execute(CREATE_SCAN_FOLDERS_TABLE, [])?;
    conn.execute(CREATE_PLAYLISTS_TABLE, [])?;
    conn.execute(CREATE_PLAYLIST_ITEMS_TABLE, [])?;
    conn.execute(CREATE_PLAY_HISTORY_TABLE, [])?;
    conn.execute(CREATE_FAVORITES_TABLE, [])?;
    conn.execute(CREATE_SETTINGS_TABLE, [])?;
    conn.execute(CREATE_INDEXES, [])?;

    ensure_indexes(&conn)?;
    init_default_settings(&conn)?;

    Ok(())
}

fn ensure_indexes(conn: &Connection) -> Result<(), rusqlite::Error> {
    let indexes = [
        "CREATE INDEX IF NOT EXISTS idx_tracks_title ON tracks(title)",
        "CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist)",
        "CREATE INDEX IF NOT EXISTS idx_tracks_album ON tracks(album)",
        "CREATE INDEX IF NOT EXISTS idx_playlist_items_playlist ON playlist_items(playlist_id)",
        "CREATE INDEX IF NOT EXISTS idx_play_history_track ON play_history(track_id)",
        "CREATE INDEX IF NOT EXISTS idx_play_history_played ON play_history(played_at)",
        "CREATE INDEX IF NOT EXISTS idx_tracks_artist_album ON tracks(artist, album)",
        "CREATE INDEX IF NOT EXISTS idx_tracks_duration ON tracks(duration)",
    ];
    for sql in &indexes {
        conn.execute(*sql, [])?;
    }
    Ok(())
}

fn init_default_settings(conn: &Connection) -> Result<(), rusqlite::Error> {
    let default_settings = vec![
        ("theme", "liquid-glass"),
        ("volume", "0.8"),
        ("play_mode", "normal"),
        ("auto_play", "false"),
        ("remember_position", "true"),
        ("cache_enabled", "true"),
        ("cache_max_size", "500"),
    ];

    for (key, value) in default_settings {
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)",
            (key, value),
        )?;
    }

    Ok(())
}

pub fn get_connection(app: &AppHandle) -> Result<Connection, rusqlite::Error> {
    let db_path = get_database_path(app);
    Connection::open(&db_path)
}
