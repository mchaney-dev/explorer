pub mod commands;
pub mod db;
pub mod models;

use tauri::Manager;

use crate::db::Db;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // `setup` runs once at startup. We open the database here (rather than in
        // a `const`/`static`) because we need Tauri's path resolver to find the
        // per-user app-data directory, which only exists once the app is built.
        .setup(|app| {
            // e.g. %APPDATA%/com.explorer.app on Windows,
            // ~/Library/Application Support/... on macOS. Tauri derives it from
            // the identifier in tauri.conf.json.
            let data_dir = app.path().app_data_dir()?;
            // The directory may not exist on first launch; create it so opening
            // the database file inside it can't fail.
            std::fs::create_dir_all(&data_dir)?;

            // Open (creating if needed) and migrate the database, then hand it to
            // Tauri's managed state so commands can pull it out via `State<Db>`.
            let db = Db::new(data_dir.join("explorer.db"))?;
            app.manage(db);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::create_tag,
            commands::save_tag,
            commands::get_tag,
            commands::list_tags,
            commands::delete_tag,
            commands::create_task,
            commands::save_task,
            commands::get_task,
            commands::list_tasks,
            commands::delete_task,
            commands::create_project,
            commands::save_project,
            commands::get_project,
            commands::list_projects,
            commands::delete_project,
            commands::create_note,
            commands::save_note,
            commands::get_note,
            commands::list_notes,
            commands::delete_note,
            commands::create_graph,
            commands::save_graph,
            commands::get_graph,
            commands::list_graphs,
            commands::delete_graph,
            commands::create_calendar,
            commands::save_calendar,
            commands::get_calendar,
            commands::list_calendars,
            commands::delete_calendar,
            commands::create_recurrence,
            commands::save_recurrence,
            commands::get_recurrence,
            commands::list_recurrences,
            commands::delete_recurrence,
            commands::create_feed_source,
            commands::save_feed_source,
            commands::get_feed_source,
            commands::list_feed_sources,
            commands::delete_feed_source,
            commands::create_user,
            commands::save_user,
            commands::get_user,
            commands::list_users,
            commands::delete_user,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
