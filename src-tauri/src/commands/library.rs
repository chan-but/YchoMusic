use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use base64::Engine;
use lofty::{Accessor, AudioFile, ItemKey, ParseOptions, TaggedFile, TaggedFileExt};
use rusqlite::Connection;
use tauri::AppHandle;

use crate::db::get_connection;
use crate::models::{Track, TrackFilter};

/// Split a single artist string into multiple artist names.
///
/// Handles common separators like `&`, `/`, `;` and `feat.`/`ft.`/`featuring`
/// patterns. Duplicate entries (case-insensitive) are removed while preserving
/// the original order. The first element is considered the primary artist.
pub fn split_artists(artist: &str) -> Vec<String> {
    let separators = ['&', '/', ';'];
    let mut result: Vec<String> = Vec::new();

    for part in artist.split(|c: char| separators.contains(&c)) {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            result.push(trimmed.to_string());
        }
    }

    if result.is_empty() {
        let lower = artist.to_lowercase();
        for marker in ["feat.", "ft.", "featuring"] {
            if let Some(idx) = lower.find(marker) {
                let main = artist[..idx].trim();
                let guest = artist[idx + marker.len()..].trim();
                if !main.is_empty() {
                    result.push(main.to_string());
                }
                if !guest.is_empty() {
                    result.push(guest.to_string());
                }
                break;
            }
        }
    }

    if result.is_empty() {
        result.push(artist.to_string());
    }

    let mut seen = std::collections::HashSet::new();
    result.retain(|a| {
        let key = a.to_lowercase();
        seen.insert(key)
    });

    result
}

fn extract_metadata(path: &Path) -> (Option<String>, Option<String>, Option<String>, Option<i32>, Option<i32>, i64, Option<Vec<u8>>, Option<String>) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return (None, None, None, None, None, 0, None, None),
    };

    let mut reader = io::BufReader::new(file);

    let tagged_file = match TaggedFile::read_from(&mut reader, ParseOptions::new()) {
        Ok(tf) => tf,
        Err(_) => return (None, None, None, None, None, 0, None, None),
    };

    let tag = tagged_file.primary_tag().or(tagged_file.first_tag());
    let properties = tagged_file.properties();

    let title = tag.as_ref().and_then(|t| t.title().map(|s| s.to_string()));
    let artist = tag.as_ref().and_then(|t| t.artist().map(|s| s.to_string()));
    let album = tag.as_ref().and_then(|t| t.album().map(|s| s.to_string()));
    let year = tag.as_ref().and_then(|t| t.year().map(|y| y as i32));
    let track_number = tag.as_ref().and_then(|t| t.track().map(|t| t as i32));
    let duration = properties.duration().as_secs() as i64;

    let cover_blob = tag.as_ref().and_then(|t| {
        t.pictures().first().map(|p| p.data().to_vec())
    });

    let lyrics = tag.as_ref().and_then(|t| {
        t.get_string(&ItemKey::Lyrics).map(|s| s.to_string())
    });

    (title, artist, album, year, track_number, duration, cover_blob, lyrics)
}

fn to_base64(data: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn from_base64_or_read(raw: &rusqlite::types::Value) -> Option<String> {
    use rusqlite::types::Value;
    match raw {
        Value::Text(s) => Some(s.clone()),
        Value::Blob(b) => Some(to_base64(b)),
        _ => None,
    }
}

fn get_or_create_folder(conn: &Connection, path: &Path) -> i64 {
    let path_str = path.to_string_lossy().to_string();
    let folder_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("未命名文件夹")
        .to_string();

    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM scan_folders WHERE path = ?",
            [&path_str],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing {
        return id;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO scan_folders (path, name, added_at) VALUES (?, ?, ?)",
        (&path_str, &folder_name, now),
    ).ok();

    conn.last_insert_rowid()
}

fn collect_new_files(path: &Path, conn: &Connection) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let extensions = ["mp3", "flac", "wav", "ogg", "m4a", "aac", "ape"];

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                files.extend(collect_new_files(&entry_path, conn));
                continue;
            }

            if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext.to_lowercase().as_str()) {
                    let path_str = entry_path.to_string_lossy().to_string();

                    let exists: bool = conn
                        .query_row(
                            "SELECT EXISTS(SELECT 1 FROM tracks WHERE path = ?)",
                            [&path_str],
                            |row| row.get(0),
                        )
                        .unwrap_or(false);

                    if !exists {
                        files.push(entry_path);
                    }
                }
            }
        }
    }

    files
}

struct FileMetadata {
    path: PathBuf,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    year: Option<i32>,
    track_number: Option<i32>,
    duration: i64,
    cover_blob: Option<Vec<u8>>,
    lyrics: Option<String>,
    modified_at: i64,
}

fn extract_metadata_batch(files: Vec<PathBuf>) -> Vec<FileMetadata> {
    let mut results = Vec::with_capacity(files.len());
    for file_path in files {
        let (title, artist, album, year, track_number, duration, cover_blob, lyrics) = extract_metadata(&file_path);
        let modified_at = file_path.metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        results.push(FileMetadata {
            path: file_path,
            title,
            artist,
            album,
            year,
            track_number,
            duration,
            cover_blob,
            lyrics,
            modified_at,
        });
    }
    results
}

fn scan_directory_parallel(path: &Path, conn: &Connection, folder_id: i64) -> usize {
    let files = collect_new_files(path, conn);
    if files.is_empty() {
        return 0;
    }

    let num_threads = std::cmp::min(thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4), files.len());

    let chunk_size = (files.len() + num_threads - 1) / num_threads;
    let chunks: Vec<Vec<PathBuf>> = files
        .chunks(chunk_size)
        .map(|c| c.to_vec())
        .collect();

    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for chunk in chunks {
        let results = Arc::clone(&results);
        let handle = thread::spawn(move || {
            let batch_results = extract_metadata_batch(chunk);
            if let Ok(mut lock) = results.lock() {
                lock.extend(batch_results);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    let results = results.lock().unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let count = results.len();
    for meta in results.iter() {
        let path_str = meta.path.to_string_lossy().to_string();
        let cover_b64 = meta.cover_blob.as_ref().map(|b| to_base64(b));

        conn.execute(
            "INSERT INTO tracks (path, title, artist, album, year, track_number, duration, cover_blob, lyrics, scan_folder_id, added_at, modified_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                &path_str,
                &meta.title,
                &meta.artist,
                &meta.album,
                meta.year,
                meta.track_number,
                meta.duration,
                cover_b64,
                &meta.lyrics,
                folder_id,
                now,
                meta.modified_at,
            ),
        ).ok();
    }

    count
}

#[tauri::command]
pub async fn scan_directory(app: AppHandle, path: String) -> Result<usize, String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() || !path_buf.is_dir() {
        return Err("Directory does not exist".to_string());
    }

    let app_handle = app.clone();
    let count = tauri::async_runtime::spawn_blocking(move || -> Result<usize, String> {
        let conn = get_connection(&app_handle).map_err(|e| e.to_string())?;
        let folder_id = get_or_create_folder(&conn, &path_buf);
        Ok(scan_directory_parallel(&path_buf, &conn, folder_id))
    }).await.map_err(|e| e.to_string())??;

    Ok(count)
}

#[tauri::command]
pub async fn get_tracks(app: AppHandle, filter: TrackFilter) -> Result<Vec<Track>, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_tracks_blocking(app_handle, filter))
        .await
        .map_err(|e| e.to_string())?
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn get_tracks_blocking(app: AppHandle, filter: TrackFilter) -> Result<Vec<Track>, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    
    let mut query = String::from("SELECT id, path, title, artist, album, year, track_number, duration, bitrate, sample_rate, cover_blob, lyrics, added_at, modified_at FROM tracks WHERE 1=1");
    let mut params: Vec<String> = Vec::new();

    if let Some(ref artist) = filter.artist {
        query.push_str(" AND artist LIKE ?");
        params.push(format!("%{}%", escape_like(artist)));
    }
    if let Some(ref album) = filter.album {
        query.push_str(" AND album LIKE ?");
        params.push(format!("%{}%", escape_like(album)));
    }
    if let Some(ref search) = filter.search {
        query.push_str(" AND (title LIKE ? OR artist LIKE ? OR album LIKE ?)");
        let escaped = escape_like(search);
        let search_pattern = format!("%{}%", escaped);
        params.push(search_pattern.clone());
        params.push(search_pattern.clone());
        params.push(search_pattern);
    }

    query.push_str(" ORDER BY title ASC");

    if let Some(limit) = filter.limit {
        query.push_str(" LIMIT ?");
        params.push(limit.to_string());
    }
    if let Some(offset) = filter.offset {
        query.push_str(" OFFSET ?");
        params.push(offset.to_string());
    }

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let track_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        let cover_raw: rusqlite::types::Value = row.get(10)?;
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
            cover_blob: from_base64_or_read(&cover_raw),
            lyrics: row.get(11)?,
            added_at: row.get(12)?,
            modified_at: row.get(13)?,
        })
    }).map_err(|e| e.to_string())?;

    let tracks: Result<Vec<Track>, _> = track_iter.collect();
    tracks.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_track_detail(app: AppHandle, id: i64) -> Result<Track, String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || get_track_detail_blocking(app_handle, id))
        .await
        .map_err(|e| e.to_string())?
}

fn get_track_detail_blocking(app: AppHandle, id: i64) -> Result<Track, String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;
    let track: Track = conn
        .query_row(
            "SELECT id, path, title, artist, album, year, track_number, duration, bitrate, sample_rate, cover_blob, lyrics, added_at, modified_at FROM tracks WHERE id = ?",
            [id],
            |row| {
                let cover_raw: rusqlite::types::Value = row.get(10)?;
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
                    cover_blob: from_base64_or_read(&cover_raw),
                    lyrics: row.get(11)?,
                    added_at: row.get(12)?,
                    modified_at: row.get(13)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(track)
}

#[tauri::command]
pub async fn update_track_tags(
    app: AppHandle,
    track_id: i64,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    year: Option<i32>,
    track_number: Option<i32>,
    lyrics: Option<String>,
) -> Result<(), String> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        update_track_tags_blocking(app_handle, track_id, title, artist, album, year, track_number, lyrics)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn update_track_tags_blocking(
    app: AppHandle,
    track_id: i64,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    year: Option<i32>,
    track_number: Option<i32>,
    lyrics: Option<String>,
) -> Result<(), String> {
    let conn = get_connection(&app).map_err(|e| e.to_string())?;

    let path: String = conn
        .query_row("SELECT path FROM tracks WHERE id = ?", [track_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let path_obj = PathBuf::from(&path);

    let file = File::open(&path_obj).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut reader = io::BufReader::new(file);

    let mut tagged_file = TaggedFile::read_from(&mut reader, ParseOptions::new())
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let tag = if let Some(t) = tagged_file.primary_tag_mut() {
        Some(t)
    } else {
        tagged_file.first_tag_mut()
    };

    if let Some(tag) = tag {
        if let Some(ref t) = title {
            tag.set_title(t.clone());
        }
        if let Some(ref a) = artist {
            tag.set_artist(a.clone());
        }
        if let Some(ref al) = album {
            tag.set_album(al.clone());
        }
        if let Some(y) = year {
            tag.set_year(y as u32);
        }
        if let Some(tn) = track_number {
            tag.set_track(tn as u32);
        }
    }

    let temp_path = path_obj.with_extension("tmp");
    let mut temp_file = File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    tagged_file
        .save_to(&mut temp_file)
        .map_err(|e| format!("Failed to save tags: {}", e))?;

    std::fs::rename(&temp_path, &path_obj)
        .map_err(|e| format!("Failed to rename temp file: {}", e))?;

    conn.execute(
        "UPDATE tracks SET title = ?, artist = ?, album = ?, year = ?, track_number = ?, lyrics = ?, modified_at = ? WHERE id = ?",
        (
            title,
            artist,
            album,
            year,
            track_number,
            lyrics,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            track_id,
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
