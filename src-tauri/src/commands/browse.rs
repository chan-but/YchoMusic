use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::commands::library::split_artists;
use crate::db::get_connection;

fn to_base64(data: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(data)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumInfo {
    pub id: String,
    pub name: String,
    pub artist: Option<String>,
    pub track_count: i64,
    pub cover_blob: Option<String>,
    pub year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistInfo {
    pub id: String,
    pub name: String,
    pub track_count: i64,
    pub cover_blob: Option<String>,
}

#[tauri::command]
pub async fn get_albums(app: AppHandle) -> Result<Vec<AlbumInfo>, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_albums_blocking(app_handle))
        .await
        .map_err(|e| e.to_string())?
}

fn get_albums_blocking(app: AppHandle) -> Result<Vec<AlbumInfo>, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT album, artist, COUNT(*) as track_count, year
             FROM tracks
             WHERE album IS NOT NULL AND album != ''
             GROUP BY album, artist
             ORDER BY album ASC",
        )
        .map_err(|e| e.to_string())?;

    let mut albums: Vec<AlbumInfo> = Vec::new();
    for row in stmt.query_map([], |row| {
        let album: String = row.get(0)?;
        let artist: Option<String> = row.get(1)?;
        let track_count: i64 = row.get(2)?;
        let year: Option<i32> = row.get(3)?;

        let cover_blob: Option<String> = conn
            .query_row(
                "SELECT cover_blob FROM tracks WHERE album = ? AND cover_blob IS NOT NULL LIMIT 1",
                [&album],
                |r| {
                    let raw: rusqlite::types::Value = r.get(0)?;
                    match raw {
                        rusqlite::types::Value::Text(s) => Ok(Some(s)),
                        rusqlite::types::Value::Blob(b) => Ok(Some(to_base64(&b))),
                        _ => Ok(None),
                    }
                },
            )
            .ok()
            .flatten();

        Ok(AlbumInfo {
            id: format!("{}|{}", album, artist.as_deref().unwrap_or("")),
            name: album,
            artist,
            track_count,
            cover_blob,
            year,
        })
    }).map_err(|e| e.to_string())? {
        if let Ok(album) = row {
            albums.push(album);
        }
    }

    Ok(albums)
}

#[tauri::command]
pub async fn get_artists(app: AppHandle) -> Result<Vec<ArtistInfo>, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_artists_blocking(app_handle))
        .await
        .map_err(|e| e.to_string())?
}

fn get_artists_blocking(app: AppHandle) -> Result<Vec<ArtistInfo>, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT artist, COUNT(*) as track_count
             FROM tracks
             WHERE artist IS NOT NULL AND artist != ''
             GROUP BY artist
             ORDER BY artist ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, i64)> = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let track_count: i64 = row.get(1)?;
            Ok((name, track_count))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    drop(stmt);

    let mut artist_map: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
    for (artist, count) in rows {
        for a in split_artists(&artist) {
            let entry = artist_map.entry(a).or_insert(0);
            *entry += count;
        }
    }

    let mut artists: Vec<ArtistInfo> = Vec::new();
    for (name, track_count) in artist_map {
        let like_pattern = format!("%{}%", escape_like(&name));
        let cover_blob: Option<String> = conn
            .query_row(
                "SELECT cover_blob FROM tracks WHERE artist LIKE ? ESCAPE '\\' AND cover_blob IS NOT NULL LIMIT 1",
                [&like_pattern],
                |r| {
                    let raw: rusqlite::types::Value = r.get(0)?;
                    match raw {
                        rusqlite::types::Value::Text(s) => Ok(Some(s)),
                        rusqlite::types::Value::Blob(b) => Ok(Some(to_base64(&b))),
                        _ => Ok(None),
                    }
                },
            )
            .ok()
            .flatten();

        artists.push(ArtistInfo {
            id: name.clone(),
            name,
            track_count,
            cover_blob,
        });
    }

    artists.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(artists)
}

fn cover_from_row(row: &rusqlite::Row, idx: usize) -> Result<Option<String>, rusqlite::Error> {
    let raw: rusqlite::types::Value = row.get(idx)?;
    Ok(match raw {
        rusqlite::types::Value::Text(s) => Some(s),
        rusqlite::types::Value::Blob(b) => Some(to_base64(&b)),
        _ => None,
    })
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[tauri::command]
pub async fn get_tracks_by_album(
    app: AppHandle,
    album: String,
    artist: Option<String>,
) -> Result<Vec<crate::models::Track>, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_tracks_by_album_blocking(app_handle, album, artist))
        .await
        .map_err(|e| e.to_string())?
}

fn get_tracks_by_album_blocking(
    app: AppHandle,
    album: String,
    artist: Option<String>,
) -> Result<Vec<crate::models::Track>, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;

    let album_trimmed = album.trim().to_string();
    let album_escaped = escape_like(&album_trimmed);

    let tracks = if let Some(ref a) = artist {
        let artist_trimmed = a.trim().to_string();
        let artist_escaped = escape_like(&artist_trimmed);
        let mut stmt = conn
            .prepare(
                "SELECT id, path, title, artist, album, year, track_number, duration, bitrate, sample_rate, cover_blob, lyrics, added_at, modified_at FROM tracks WHERE album LIKE ? AND artist LIKE ? ORDER BY track_number ASC, title ASC",
            )
            .map_err(|e| e.to_string())?;
        let result: Vec<crate::models::Track> = stmt.query_map(
            rusqlite::params![
                format!("%{}%", album_escaped),
                format!("%{}%", artist_escaped)
            ],
            |row| {
                Ok(crate::models::Track {
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
            }
        )
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
        drop(stmt);
        result
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT id, path, title, artist, album, year, track_number, duration, bitrate, sample_rate, cover_blob, lyrics, added_at, modified_at FROM tracks WHERE album LIKE ? ORDER BY track_number ASC, title ASC",
            )
            .map_err(|e| e.to_string())?;
        let result: Vec<crate::models::Track> = stmt.query_map(
            rusqlite::params![format!("%{}%", album_escaped)],
            |row| {
                Ok(crate::models::Track {
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
            }
        )
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
        drop(stmt);
        result
    };

    Ok(tracks)
}

#[tauri::command]
pub async fn get_tracks_by_artist(app: AppHandle, artist: String) -> Result<Vec<crate::models::Track>, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_tracks_by_artist_blocking(app_handle, artist))
        .await
        .map_err(|e| e.to_string())?
}

fn get_tracks_by_artist_blocking(app: AppHandle, artist: String) -> Result<Vec<crate::models::Track>, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    let artist_trimmed = artist.trim().to_string();
    let artist_escaped = escape_like(&artist_trimmed);
    
    let mut stmt = conn
        .prepare(
            "SELECT id, path, title, artist, album, year, track_number, duration, bitrate, sample_rate, cover_blob, lyrics, added_at, modified_at FROM tracks WHERE artist = ? ORDER BY album ASC, track_number ASC, title ASC",
        )
        .map_err(|e| e.to_string())?;

    let tracks: Vec<crate::models::Track> = stmt
        .query_map(rusqlite::params![&artist_trimmed], |row| {
            Ok(crate::models::Track {
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
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    drop(stmt);

    if !tracks.is_empty() {
        return Ok(tracks);
    }

    let mut stmt2 = conn
        .prepare(
            "SELECT id, path, title, artist, album, year, track_number, duration, bitrate, sample_rate, cover_blob, lyrics, added_at, modified_at FROM tracks WHERE artist LIKE ? ORDER BY album ASC, track_number ASC, title ASC",
        )
        .map_err(|e| e.to_string())?;

    let tracks2: Vec<crate::models::Track> = stmt2
        .query_map(rusqlite::params![format!("%{}%", artist_escaped)], |row| {
            Ok(crate::models::Track {
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
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    drop(stmt2);
    Ok(tracks2)
}
