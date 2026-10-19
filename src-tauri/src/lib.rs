mod zip_service;
mod conversion_service;

use conversion_service::{compress_jpeg_images as engine_compress_jpeg_images, convert_images as engine_convert_images, ImageOperationResult};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    webview::WebviewWindowBuilder,
    AppHandle, Emitter, Manager, PhysicalPosition, Position, WebviewUrl, WebviewWindow,
};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Shortcut, ShortcutState,
};
use zip_service::{create_zip_in_dir, ZipResult};

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

    let monitors = window.available_monitors().unwrap_or_default();
    let selected_monitor = monitors.into_iter().find(|monitor| {
        let position = monitor.position();
        let size = monitor.size();
        cursor_x >= position.x
            && cursor_y >= position.y
            && cursor_x < position.x + size.width as i32
            && cursor_y < position.y + size.height as i32
    });
    let scale = selected_monitor
        .as_ref()
        .map(|monitor| monitor.scale_factor())
        .unwrap_or_else(|| window.scale_factor().unwrap_or(1.0));
    let width = (WHEEL_WINDOW_SIZE * scale).round() as i32;
    let height = width;
    let mut left = cursor_x - width / 2;
    let mut top = cursor_y - height / 2;

    if let Some(monitor) = selected_monitor {
        let position = monitor.position();
        let size = monitor.size();
        let max_left = (position.x + size.width as i32 - width).max(position.x);
        let max_top = (position.y + size.height as i32 - height).max(position.y);
        left = left.clamp(position.x, max_left);
        top = top.clamp(position.y, max_top);
    }

    window
        .set_position(Position::Physical(PhysicalPosition::new(left, top)))
        .map_err(|error| format!("Could not position the SHIFT wheel: {error}"))
}

fn escape_shortcut() -> Shortcut {
    Shortcut::new(None, Code::Escape)
}

fn show_wheel_window(app: &AppHandle, mode: &str) -> Result<(), String> {
    let window = app
        .get_webview_window(WHEEL_WINDOW_LABEL)
        .ok_or_else(|| "The SHIFT wheel window is not available.".to_string())?;
    position_wheel_near_cursor(&window)?;
    window
        .show()
        .map_err(|error| format!("Could not show the SHIFT wheel: {error}"))?;
    // Keep the drag source focused while the contextual wheel is shown.
    let _ = app.global_shortcut().register(escape_shortcut());
    app.emit("wheel-mode", mode.to_string()).map_err(|error| format!("Could not update wheel mode: {error}"))?;
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
async fn convert_images_to_format(
    inputs: Vec<String>,
    target_format: String,
) -> Result<Vec<ImageOperationResult>, String> {
    tauri::async_runtime::spawn_blocking(move || engine_convert_images(&inputs, &target_format))
        .await
        .map_err(|error| format!("The image conversion worker stopped unexpectedly: {error}"))?
}

#[tauri::command]
async fn compress_jpeg_images(inputs: Vec<String>) -> Result<Vec<ImageOperationResult>, String> {
    tauri::async_runtime::spawn_blocking(move || engine_compress_jpeg_images(&inputs))
        .await
        .map_err(|error| format!("The JPEG compression worker stopped unexpectedly: {error}"))?
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
fn open_output_parent(path: String) -> Result<(), String> {
    let file = fs::canonicalize(&path)
        .map_err(|error| format!("Could not resolve the output file {}: {error}", path))?;
    if !file.is_file() {
        return Err("The requested output path is not a file.".to_string());
    }
    let parent = file.parent()
        .ok_or_else(|| "The output file has no parent folder.".to_string())?;
    open::that(parent)
        .map_err(|error| format!("Could not open the output folder: {error}"))
}

#[tauri::command]
fn show_wheel_overlay(app: AppHandle) -> Result<(), String> {
    show_wheel_window(&app, "tools")
}

#[tauri::command]
fn hide_wheel_overlay(app: AppHandle) -> Result<(), String> {
    hide_wheel_window(&app)
}

#[cfg(target_os = "windows")]
fn start_drag_monitor(app: AppHandle) {
    use std::{thread, time::Duration};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetAsyncKeyState, GetClassNameW, GetCursorPos, GetForegroundWindow,
    };

    const VK_LBUTTON: i32 = 0x01;
    const VK_SHIFT: i32 = 0x10;
    const VK_CONTROL: i32 = 0x11;
    const VK_MENU: i32 = 0x12;
    let is_down = |key: i32| unsafe { (GetAsyncKeyState(key) as u16 & 0x8000) != 0 };
    let mut mouse_down = false;
    let mut release_samples = 0_u8;
    let mut anchor = (0_i32, 0_i32);
    let mut gesture_active = false;
    let mut active_mode = String::new();

    thread::spawn(move || loop {
        let left_down = is_down(VK_LBUTTON);
        if left_down {
            release_samples = 0;
            let mut point = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
            if unsafe { GetCursorPos(&mut point) } != 0 {
                if !mouse_down {
                    anchor = (point.x, point.y);
                    mouse_down = true;
                }
                let moved = (point.x - anchor.0).abs() + (point.y - anchor.1).abs() >= 9;
                let shift = is_down(VK_SHIFT);
                let control = is_down(VK_CONTROL);
                let alt = is_down(VK_MENU);
                let requested_mode = if shift {
                    Some(if control { "actions" } else if alt { "tools" } else { "convert" })
                } else {
                    None
                };

                if moved && !gesture_active && requested_mode.is_some() {
                    let hwnd = unsafe { GetForegroundWindow() };
                    if !hwnd.is_null() {
                        let mut class_name = [0_u16; 256];
                        let length = unsafe { GetClassNameW(hwnd, class_name.as_mut_ptr(), class_name.len() as i32) };
                        let class = String::from_utf16_lossy(&class_name[..length.max(0) as usize]);
                        let explorer_source = matches!(
                            class.as_str(),
                            "CabinetWClass" | "ExploreWClass" | "Progman" | "WorkerW"
                                | "Shell_TrayWnd" | "XamlExplorerHostIslandWindow"
                        );
                        if explorer_source {
                            let mode = requested_mode.unwrap();
                            if show_wheel_window(&app, mode).is_ok() {
                                gesture_active = true;
                                active_mode = mode.to_string();
                            }
                        }
                    }
                } else if gesture_active {
                    if let Some(mode) = requested_mode {
                        if mode != active_mode {
                            let _ = app.emit("wheel-mode", mode.to_string());
                            active_mode = mode.to_string();
                        }
                    }
                }
            }
        } else if mouse_down {
            release_samples = release_samples.saturating_add(1);
            // Explorer can briefly report the button up during OLE drag processing.
            if release_samples >= 4 {
                mouse_down = false;
                if gesture_active {
                    let _ = app.emit("wheel-gesture-end", ());
                }
                gesture_active = false;
                active_mode.clear();
                anchor = (0, 0);
            }
        }
        thread::sleep(Duration::from_millis(16));
    });
}

#[cfg(not(target_os = "windows"))]
fn start_drag_monitor(_app: AppHandle) {}

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed && shortcut == &escape_shortcut() {
                        let _ = hide_wheel_window(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let wheel_url = WebviewUrl::App("wheel.html".into());
            WebviewWindowBuilder::new(app, WHEEL_WINDOW_LABEL, wheel_url)
                .title("SHIFT Wheel")
                .inner_size(WHEEL_WINDOW_SIZE, WHEEL_WINDOW_SIZE)
                .resizable(false)
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .focusable(false)
                .skip_taskbar(true)
                .visible(false)
                .shadow(false)
                .build()?;

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

            start_drag_monitor(app.handle().clone());
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
            convert_images_to_format,
            compress_jpeg_images,
            open_output_folder,
            open_output_parent,
            show_wheel_overlay,
            hide_wheel_overlay,
        ])
        .run(tauri::generate_context!())
        .expect("error while running SHIFT");
}
