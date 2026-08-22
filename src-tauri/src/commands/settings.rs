use tauri::AppHandle;

use crate::db::get_connection;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Settings {
    pub theme: String,
    pub volume: f32,
    pub play_mode: String,
    pub auto_play: bool,
    pub remember_position: bool,
    pub cache_enabled: bool,
    pub cache_max_size: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "liquid-glass".to_string(),
            volume: 0.8,
            play_mode: "normal".to_string(),
            auto_play: false,
            remember_position: true,
            cache_enabled: true,
            cache_max_size: 500,
        }
    }
}

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_settings_blocking(app_handle))
        .await
        .map_err(|e| e.to_string())?
}

fn get_settings_blocking(app: AppHandle) -> Result<Settings, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    
    let get_string = |key: &str, default: &str| -> String {
        conn.query_row("SELECT value FROM settings WHERE key = ?", [key], |row| row.get(0))
            .unwrap_or_else(|_| default.to_string())
    };
    
    let get_bool = |key: &str, default: bool| -> bool {
        conn.query_row("SELECT value FROM settings WHERE key = ?", [key], |row| {
            let value: String = row.get(0)?;
            Ok(value == "true")
        }).unwrap_or(default)
    };
    
    let get_float = |key: &str, default: f32| -> f32 {
        conn.query_row("SELECT value FROM settings WHERE key = ?", [key], |row| {
            let value: String = row.get(0)?;
            Ok(value.parse().unwrap_or(default))
        }).unwrap_or(default)
    };

    let get_int = |key: &str, default: i32| -> i32 {
        conn.query_row("SELECT value FROM settings WHERE key = ?", [key], |row| {
            let value: String = row.get(0)?;
            Ok(value.parse().unwrap_or(default))
        }).unwrap_or(default)
    };
    
    Ok(Settings {
        theme: get_string("theme", "liquid-glass"),
        volume: get_float("volume", 0.8),
        play_mode: get_string("play_mode", "normal"),
        auto_play: get_bool("auto_play", false),
        remember_position: get_bool("remember_position", true),
        cache_enabled: get_bool("cache_enabled", true),
        cache_max_size: get_int("cache_max_size", 500),
    })
}

#[tauri::command]
pub async fn set_setting(app: AppHandle, key: String, value: String) -> Result<(), String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || set_setting_blocking(app_handle, key, value))
        .await
        .map_err(|e| e.to_string())?
}

fn set_setting_blocking(app: AppHandle, key: String, value: String) -> Result<(), String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        (&key, &value),
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}
