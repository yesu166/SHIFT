mod zip_service;

use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    webview::{WebviewUrl, WebviewWindowBuilder},
    AppHandle, Manager, PhysicalPosition, Position, State, WebviewWindow,
};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Shortcut, ShortcutState,
};
use zip_service::{create_zip_in_dir, ZipResult};

const DEFAULT_WHEEL_KEY: &str = "F8";
const WHEEL_WINDOW_LABEL: &str = "wheel";
const WHEEL_WINDOW_SIZE: f64 = 520.0;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    output_folder: String,
    app_version: String,
}

struct WheelShortcutState(Mutex<String>);

fn shortcut_for_key(key: &str) -> Result<Shortcut, String> {
    let code = match key {
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        _ => return Err("Choose one of the supported function keys F6–F12.".to_string()),
    };
    Ok(Shortcut::new(None, code))
}

fn shortcut_settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("Could not resolve the SHIFT settings folder: {error}"))?;
    Ok(config_dir.join("wheel-shortcut.txt"))
}

fn load_saved_shortcut(app: &AppHandle) -> String {
    let saved = shortcut_settings_path(app)
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|value| value.trim().to_uppercase())
        .unwrap_or_default();
    if shortcut_for_key(&saved).is_ok() {
        saved
    } else {
        DEFAULT_WHEEL_KEY.to_string()
    }
}

fn save_shortcut(app: &AppHandle, key: &str) -> Result<(), String> {
    let path = shortcut_settings_path(app)?;
    let parent = path
        .parent()
        .ok_or_else(|| "The SHIFT settings path is invalid.".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create the SHIFT settings folder: {error}"))?;
    fs::write(path, key)
        .map_err(|error| format!("Could not save the wheel shortcut: {error}"))
}

fn system_cursor_position() -> Option<(i32, i32)> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::POINT;
        use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
        let mut point = POINT { x: 0, y: 0 };
        // SAFETY: GetCursorPos writes one POINT to the valid pointer provided here.
        let result = unsafe { GetCursorPos(&mut point) };
        if result != 0 {
            return Some((point.x, point.y));
        }
    }
    None
}

fn position_wheel_near_cursor(window: &WebviewWindow) -> Result<(), String> {
    let Some((cursor_x, cursor_y)) = system_cursor_position() else {
        return window
            .center()
            .map_err(|error| format!("Could not center the SHIFT wheel: {error}"));
    };

    let scale = window.scale_factor().unwrap_or(1.0);
    let width = (WHEEL_WINDOW_SIZE * scale).round() as i32;
    let height = width;
    let mut left = cursor_x - width / 2;
    let mut top = cursor_y - height / 2;

    if let Ok(monitors) = window.available_monitors() {
        if let Some(monitor) = monitors.into_iter().find(|monitor| {
            let position = monitor.position();
            let size = monitor.size();
            cursor_x >= position.x
                && cursor_y >= position.y
                && cursor_x < position.x + size.width as i32
                && cursor_y < position.y + size.height as i32
        }) {
            let position = monitor.position();
            let size = monitor.size();
            let max_left = (position.x + size.width as i32 - width).max(position.x);
            let max_top = (position.y + size.height as i32 - height).max(position.y);
            left = left.clamp(position.x, max_left);
            top = top.clamp(position.y, max_top);
        }
    }

    window
        .set_position(Position::Physical(PhysicalPosition::new(left, top)))
        .map_err(|error| format!("Could not position the SHIFT wheel: {error}"))
}

fn escape_shortcut() -> Shortcut {
    Shortcut::new(None, Code::Escape)
}

fn show_wheel_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(WHEEL_WINDOW_LABEL)
        .ok_or_else(|| "The SHIFT wheel window is not available.".to_string())?;
    position_wheel_near_cursor(&window)?;
    window
        .show()
        .map_err(|error| format!("Could not show the SHIFT wheel: {error}"))?;
    // Escape is registered only while the overlay is visible, so it doesn't affect normal work.
    let _ = app.global_shortcut().register(escape_shortcut());
    Ok(())
}

fn hide_wheel_window(app: &AppHandle) -> Result<(), String> {
    app.get_webview_window(WHEEL_WINDOW_LABEL)
        .ok_or_else(|| "The SHIFT wheel window is not available.".to_string())?
        .hide()
        .map_err(|error| format!("Could not hide the SHIFT wheel: {error}"))?;
    let _ = app.global_shortcut().unregister(escape_shortcut());
    Ok(())
}

#[tauri::command]
fn get_app_info(app: AppHandle) -> Result<AppInfo, String> {
    let documents = app.path().document_dir().map_err(|error| {
        format!("Could not resolve the Documents folder: {error}")
    })?;
    Ok(AppInfo {
        output_folder: documents
            .join("SHIFT")
            .join("Archives")
            .to_string_lossy()
            .into_owned(),
        app_version: app.package_info().version.to_string(),
    })
}

fn archives_directory(app: &AppHandle) -> Result<PathBuf, String> {
    let documents = app.path().document_dir().map_err(|error| {
        format!("Could not resolve the Documents folder: {error}")
    })?;
    Ok(documents.join("SHIFT").join("Archives"))
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

#[tauri::command]
fn show_wheel_overlay(app: AppHandle) -> Result<(), String> {
    show_wheel_window(&app)
}

#[tauri::command]
fn hide_wheel_overlay(app: AppHandle) -> Result<(), String> {
    hide_wheel_window(&app)
}

#[tauri::command]
fn get_wheel_shortcut(state: State<'_, WheelShortcutState>) -> Result<String, String> {
    state
        .0
        .lock()
        .map(|key| key.clone())
        .map_err(|_| "The wheel shortcut settings lock is unavailable.".to_string())
}

#[tauri::command]
fn set_wheel_shortcut(
    app: AppHandle,
    state: State<'_, WheelShortcutState>,
    key: String,
) -> Result<String, String> {
    let key = key.trim().to_uppercase();
    let new_shortcut = shortcut_for_key(&key)?;
    let mut current = state
        .0
        .lock()
        .map_err(|_| "The wheel shortcut settings lock is unavailable.".to_string())?;
    if *current == key {
        return Ok(key);
    }

    let previous_key = current.clone();
    let previous_shortcut = shortcut_for_key(&previous_key)?;
    app.global_shortcut()
        .register(new_shortcut)
        .map_err(|error| format!("{key} is unavailable. Choose another key. ({error})"))?;
    let _ = app.global_shortcut().unregister(previous_shortcut);

    if let Err(error) = save_shortcut(&app, &key) {
        let _ = app.global_shortcut().unregister(new_shortcut);
        let _ = app.global_shortcut().register(previous_shortcut);
        return Err(error);
    }
    *current = key.clone();
    Ok(key)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    if shortcut == &escape_shortcut() {
                        let _ = hide_wheel_window(app);
                    } else {
                        // Do not explicitly focus the overlay: stealing focus can interrupt the Explorer drag.
                        let _ = show_wheel_window(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let wheel_url = WebviewUrl::App("index.html?wheel=1".into());
            WebviewWindowBuilder::new(app, WHEEL_WINDOW_LABEL, wheel_url)
                .title("SHIFT Wheel")
                .inner_size(WHEEL_WINDOW_SIZE, WHEEL_WINDOW_SIZE)
                .resizable(false)
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .visible(false)
                .shadow(false)
                .build()?;

            let app_handle = app.handle().clone();
            let shortcut = load_saved_shortcut(&app_handle);
            app.manage(WheelShortcutState(Mutex::new(shortcut.clone())));
            app.global_shortcut()
                .register(shortcut_for_key(&shortcut).map_err(std::io::Error::other)?)?;

            let show_item = MenuItem::with_id(app, "open", "Open SHIFT", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit SHIFT", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            let icon = app
                .default_window_icon()
                .expect("SHIFT default window icon is configured")
                .clone();
            TrayIconBuilder::new()
                .icon(icon)
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            create_zip,
            open_output_folder,
            show_wheel_overlay,
            hide_wheel_overlay,
            get_wheel_shortcut,
            set_wheel_shortcut
        ])
        .run(tauri::generate_context!())
        .expect("error while running SHIFT");
}
