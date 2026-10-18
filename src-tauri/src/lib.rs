mod zip_service;

use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use zip_service::{create_zip_in_dir, ZipResult};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    output_folder: String,
    app_version: String,
}

fn archives_directory(app: &AppHandle) -> Result<PathBuf, String> {
    let documents = app.path().document_dir().map_err(|error| {
        format!("Could not resolve the Documents folder: {error}")
    })?;
    Ok(documents.join("SHIFT").join("Archives"))
}

#[tauri::command]
fn get_app_info(app: AppHandle) -> Result<AppInfo, String> {
    Ok(AppInfo {
        output_folder: archives_directory(&app)?.to_string_lossy().into_owned(),
        app_version: app.package_info().version.to_string(),
    })
}

#[tauri::command]
async fn create_zip(
    app: AppHandle,
    inputs: Vec<String>,
    archive_name: Option<String>,
) -> Result<ZipResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let output_dir = archives_directory(&app)?;
        create_zip_in_dir(&inputs, &output_dir, archive_name.as_deref())
    })
    .await
    .map_err(|error| format!("The archive worker stopped unexpectedly: {error}"))?
}

#[tauri::command]
fn open_output_folder(app: AppHandle) -> Result<(), String> {
    let output_dir = archives_directory(&app)?;
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("Could not create the output folder: {error}"))?;
    open::that(&output_dir)
        .map_err(|error| format!("Could not open the output folder: {error}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            create_zip,
            open_output_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running SHIFT");
}
