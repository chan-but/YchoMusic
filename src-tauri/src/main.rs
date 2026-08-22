#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

mod audio;
mod commands;
mod db;
mod models;

fn handle_action(action: &str) {
    match action {
        "play_pause" => {
            let state = crate::audio::PLAYER.get_state();
            if state.state == crate::models::PlayState::Playing {
                crate::audio::PLAYER.pause();
            } else {
                crate::audio::PLAYER.resume();
            }
        }
        "prev" => crate::audio::PLAYER.prev(),
        "next" => crate::audio::PLAYER.next(),
        "vol_up" => {
            let state = crate::audio::PLAYER.get_state();
            crate::audio::PLAYER.set_volume((state.volume + 0.05).min(1.0));
        }
        "vol_down" => {
            let state = crate::audio::PLAYER.get_state();
            crate::audio::PLAYER.set_volume((state.volume - 0.05).max(0.0));
        }
        _ => {}
    }
}

fn main() {
    env_logger::init();

    let context = tauri::generate_context!();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init());
    
    builder = builder.setup(|app| {
            db::init::initialize_database(&app.handle())?;

            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let play_pause_item = MenuItem::with_id(app, "play_pause", "播放/暂停", true, None::<&str>)?;
            let next_item = MenuItem::with_id(app, "next", "下一首", true, None::<&str>)?;
            let prev_item = MenuItem::with_id(app, "prev", "上一首", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[&show_item, &play_pause_item, &prev_item, &next_item, &quit_item],
            )?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("YchoMusic")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => app.exit(0),
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                        }
                        "play_pause" => handle_action("play_pause"),
                        "next" => handle_action("next"),
                        "prev" => handle_action("prev"),
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(true) {
                                window.hide().ok();
                            } else {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                        }
                    }
                })
                .build(app)?;

            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let w = window.clone();
                    std::thread::spawn(move || {
                        for _ in 0..30 {
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            if let Ok(resp) = std::net::TcpStream::connect("127.0.0.1:5173") {
                                drop(resp);
                                let _ = w.eval("window.location.replace('http://127.0.0.1:5173')");
                                break;
                            }
                        }
                    });
                    window.open_devtools();
                }
            }

            Ok(())
        });

        builder = builder
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Read close_behavior from settings
                let behavior = {
                    let app = window.app_handle();
                    let conn = match crate::db::init::get_connection(app) {
                        Ok(c) => c,
                        Err(_) => {
                            // Default: hide to tray
                            let _ = window.hide();
                            api.prevent_close();
                            return;
                        }
                    };
                    let result: Option<String> = conn
                        .query_row(
                            "SELECT value FROM settings WHERE key = 'close_behavior'",
                            [],
                            |row| row.get(0),
                        )
                        .ok();
                    result.unwrap_or_else(|| "ask".to_string())
                };

                match behavior.as_str() {
                    "tray" => {
                        let _ = window.hide();
                        api.prevent_close();
                    }
                    "exit" => {
                        // Let the app exit normally
                        // Don't prevent close
                    }
                    _ => {
                        // "ask" - default to hide to tray
                        let _ = window.hide();
                        api.prevent_close();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::audio::play,
            commands::audio::pause,
            commands::audio::resume,
            commands::audio::stop,
            commands::audio::prev,
            commands::audio::next,
            commands::audio::seek,
            commands::audio::set_volume,
            commands::audio::set_mode,
            commands::audio::get_player_state,
            commands::audio::set_playlist_from_tracks,
            commands::library::scan_directory,
            commands::library::get_tracks,
            commands::library::get_track_detail,
            commands::library::update_track_tags,
            commands::folders::get_scan_folders,
            commands::folders::delete_scan_folder,
            commands::browse::get_albums,
            commands::browse::get_artists,
            commands::browse::get_tracks_by_album,
            commands::browse::get_tracks_by_artist,
            commands::playlist::get_playlists,
            commands::playlist::create_playlist,
            commands::playlist::add_to_playlist,
            commands::playlist::get_playlist_items,
            commands::lyrics::get_lyrics,
            commands::lyrics::parse_lrc,
            commands::lyrics::load_lrc_from_file,
            commands::favorites::toggle_favorite,
            commands::favorites::get_favorites,
            commands::favorites::is_favorite,
            commands::stats::get_stats,
            commands::stats::get_play_history,
            commands::stats::record_play_history,
            commands::settings::get_settings,
            commands::settings::set_setting,
            show_main_window,
        ]);

    builder.run(context)
        .expect("error while running tauri application");
}

#[tauri::command]
fn show_main_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
