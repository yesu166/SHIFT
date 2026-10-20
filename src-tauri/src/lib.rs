mod zip_service;
mod conversion_service;
mod engine_service;

const WHEEL_WINDOW_LABEL: &str = "wheel";
const WHEEL_WINDOW_SIZE: f64 = 520.0;


const GESTURE_SHORTCUT_FILE: &str = "gesture-shortcuts.conf";
const WHEEL_SIZE_FILE: &str = "wheel-size.conf";
const MIN_WHEEL_SIZE: u32 = 360;
const MAX_WHEEL_SIZE: u32 = 720;
const DEFAULT_WHEEL_SIZE: u32 = 520;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GestureShortcuts {
    convert: String,
    tools: String,
    actions: String,
}

impl Default for GestureShortcuts {
    fn default() -> Self {
        Self {
            convert: "shift".to_string(),
            tools: "altShift".to_string(),
            actions: "ctrlShift".to_string(),
        }
    }
}

#[derive(Clone)]
struct GestureShortcutStore {
    current: Arc<RwLock<GestureShortcuts>>,
    path: PathBuf,
}

#[derive(Clone)]
struct WheelSizeStore {
    current: Arc<RwLock<u32>>,
    path: PathBuf,
}

fn validate_wheel_size(size: u32) -> Result<u32, String> {
    if !(MIN_WHEEL_SIZE..=MAX_WHEEL_SIZE).contains(&size) {
        return Err(format!(
            "Wheel size must be between {MIN_WHEEL_SIZE}px and {MAX_WHEEL_SIZE}px."
        ));
    }
    Ok(size)
}

fn load_wheel_size(path: &Path) -> u32 {
    fs::read_to_string(path)
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .and_then(|value| validate_wheel_size(value).ok())
        .unwrap_or(DEFAULT_WHEEL_SIZE)
}

fn shortcut_id_is_valid(value: &str) -> bool {
    matches!(
        value,
        "shift" | "ctrl" | "alt" | "ctrlShift" | "altShift" | "ctrlAlt" | "ctrlAltShift"
    )
}

fn validate_gesture_shortcuts(shortcuts: &GestureShortcuts) -> Result<(), String> {
    if !shortcut_id_is_valid(&shortcuts.convert)
        || !shortcut_id_is_valid(&shortcuts.tools)
        || !shortcut_id_is_valid(&shortcuts.actions)
    {
        return Err("Choose a supported Windows modifier combination for every wheel mode.".into());
    }
    if shortcuts.convert == shortcuts.tools
        || shortcuts.convert == shortcuts.actions
        || shortcuts.tools == shortcuts.actions
    {
        return Err("Each wheel mode needs a different shortcut.".into());
    }
    Ok(())
}

fn load_gesture_shortcuts(path: &Path) -> GestureShortcuts {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return GestureShortcuts::default(),
    };
    let mut values = content.lines();
    let shortcuts = GestureShortcuts {
        convert: values.next().unwrap_or_default().to_string(),
        tools: values.next().unwrap_or_default().to_string(),
        actions: values.next().unwrap_or_default().to_string(),
    };
    if validate_gesture_shortcuts(&shortcuts).is_ok() {
        shortcuts
    } else {
        GestureShortcuts::default()
    }
}

fn gesture_shortcut_matches(shortcut: &str, shift: bool, control: bool, alt: bool) -> bool {
    match shortcut {
        "shift" => shift && !control && !alt,
        "ctrl" => control && !shift && !alt,
        "alt" => alt && !shift && !control,
        "ctrlShift" => control && shift && !alt,
        "altShift" => alt && shift && !control,
        "ctrlAlt" => control && alt && !shift,
        "ctrlAltShift" => control && alt && shift,
        _ => false,
    }
}

fn requested_wheel_mode(
    shortcuts: &GestureShortcuts,
    shift: bool,
    control: bool,
    alt: bool,
) -> Option<&'static str> {
    if gesture_shortcut_matches(&shortcuts.convert, shift, control, alt) {
        Some("convert")
    } else if gesture_shortcut_matches(&shortcuts.tools, shift, control, alt) {
        Some("tools")
    } else if gesture_shortcut_matches(&shortcuts.actions, shift, control, alt) {
        Some("actions")
    } else {
        None
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    output_folder: String,
    app_version: String,
}

use conversion_service::{compress_jpeg_images as engine_compress_jpeg_images, convert_images as engine_convert_images, ImageOperationResult};
use engine_service::{detect_engines, execute_operation as run_engine_operation, EngineOperationResult, EngineStatus};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
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

fn position_wheel_near_cursor(window: &WebviewWindow, requested_size: f64) -> Result<(), String> {
    let cursor = system_cursor_position();
    let monitors = window.available_monitors().unwrap_or_default();
    let selected_monitor = cursor.as_ref().and_then(|(cursor_x, cursor_y)| {
        monitors.iter().find(|monitor| {
            let position = monitor.position();
            let size = monitor.size();
            *cursor_x >= position.x
                && *cursor_y >= position.y
                && *cursor_x < position.x + size.width as i32
                && *cursor_y < position.y + size.height as i32
        })
    });
    let scale = selected_monitor
        .map(|monitor| monitor.scale_factor())
        .unwrap_or_else(|| window.scale_factor().unwrap_or(1.0));

    // Shrink the wheel on small or high-DPI monitors so its full hit area remains onscreen.
    let wheel_size = selected_monitor
        .map(|monitor| {
            let monitor_size = monitor.size();
            let available_logical = (monitor_size.width.min(monitor_size.height) as f64 / scale) - 24.0;
            requested_size.min(available_logical.max(160.0))
        })
        .unwrap_or(requested_size);

    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize::new(wheel_size, wheel_size)))
        .map_err(|error| format!("Could not resize the SHIFT wheel: {error}"))?;

    let Some((cursor_x, cursor_y)) = cursor else {
        return window
            .center()
            .map_err(|error| format!("Could not center the SHIFT wheel: {error}"));
    };

    let width = (wheel_size * scale).round() as i32;
    let height = width;
    let mut left = cursor_x - width / 2;
    let mut top = cursor_y - height / 2;

    if let Some(monitor) = selected_monitor {
        let position = monitor.position();
        let monitor_size = monitor.size();
        let max_left = (position.x + monitor_size.width as i32 - width).max(position.x);
        let max_top = (position.y + monitor_size.height as i32 - height).max(position.y);
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
    let wheel_size = app
        .state::<WheelSizeStore>()
        .current
        .read()
        .map(|size| *size)
        .unwrap_or(DEFAULT_WHEEL_SIZE);
    position_wheel_near_cursor(&window, wheel_size as f64)?;
    window
        .show()
        .map_err(|error| format!("Could not show the SHIFT wheel: {error}"))?;
    // Keep the drag source focused while the contextual wheel is shown.
    let _ = app.global_shortcut().register(escape_shortcut());
    app.emit("wheel-mode", mode.to_string()).map_err(|error| format!("Could not update wheel mode: {error}"))?;
    app.emit("wheel-opened", ()).map_err(|error| format!("Could not announce wheel opening: {error}"))?;
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
fn get_gesture_shortcuts(
    state: tauri::State<'_, GestureShortcutStore>,
) -> Result<GestureShortcuts, String> {
    state
        .current
        .read()
        .map(|shortcuts| shortcuts.clone())
        .map_err(|_| "Could not read SHIFT gesture settings.".to_string())
}

#[tauri::command]
fn update_gesture_shortcuts(
    state: tauri::State<'_, GestureShortcutStore>,
    shortcuts: GestureShortcuts,
) -> Result<GestureShortcuts, String> {
    validate_gesture_shortcuts(&shortcuts)?;
    if let Some(parent) = state.path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create the SHIFT settings folder: {error}"))?;
    }
    let serialized = format!(
        "{}\n{}\n{}\n",
        shortcuts.convert, shortcuts.tools, shortcuts.actions
    );
    fs::write(&state.path, serialized)
        .map_err(|error| format!("Could not save the SHIFT shortcuts: {error}"))?;
    let mut current = state
        .current
        .write()
        .map_err(|_| "Could not update SHIFT gesture settings.".to_string())?;
    *current = shortcuts.clone();
    Ok(shortcuts)
}

#[tauri::command]
fn get_wheel_size(state: tauri::State<'_, WheelSizeStore>) -> Result<u32, String> {
    state
        .current
        .read()
        .map(|size| *size)
        .map_err(|_| "Could not read the SHIFT wheel size.".to_string())
}

#[tauri::command]
fn update_wheel_size(
    state: tauri::State<'_, WheelSizeStore>,
    size: u32,
) -> Result<u32, String> {
    validate_wheel_size(size)?;
    if let Some(parent) = state.path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create the SHIFT settings folder: {error}"))?;
    }
    fs::write(&state.path, size.to_string())
        .map_err(|error| format!("Could not save the SHIFT wheel size: {error}"))?;
    let mut current = state
        .current
        .write()
        .map_err(|_| "Could not update the SHIFT wheel size.".to_string())?;
    *current = size;
    Ok(size)
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
async fn get_engine_status() -> Result<EngineStatus, String> {
    tauri::async_runtime::spawn_blocking(detect_engines)
        .await
        .map_err(|error| format!("The engine detection worker stopped unexpectedly: {error}"))
}

#[tauri::command]
async fn execute_engine_operation(
    app: AppHandle,
    inputs: Vec<String>,
    operation_id: String,
    target_format: Option<String>,
) -> Result<Vec<EngineOperationResult>, String> {
    let output_dir = archives_directory(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        run_engine_operation(&inputs, &operation_id, target_format.as_deref(), &output_dir)
    })
    .await
    .map_err(|error| format!("The external engine worker stopped unexpectedly: {error}"))?
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
fn start_drag_monitor(app: AppHandle, shortcut_store: GestureShortcutStore) {
    use std::{thread, time::Duration};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetAncestor, GetClassNameW, GetCursorPos, WindowFromPoint, GA_ROOT,
    };

    const VK_LBUTTON: i32 = 0x01;
    const VK_SHIFT: i32 = 0x10;
    const VK_CONTROL: i32 = 0x11;
    const VK_MENU: i32 = 0x12;
    let is_down = |key: i32| unsafe { (GetAsyncKeyState(key) as u16 & 0x8000) != 0 };
    let is_shell_window = |point: windows_sys::Win32::Foundation::POINT| {
        let child = unsafe { WindowFromPoint(point) };
        if child.is_null() {
            return false;
        }
        let root = unsafe { GetAncestor(child, GA_ROOT) };
        if root.is_null() {
            return false;
        }
        let mut class_name = [0_u16; 256];
        let length = unsafe {
            GetClassNameW(root, class_name.as_mut_ptr(), class_name.len() as i32)
        };
        let class = String::from_utf16_lossy(&class_name[..length.max(0) as usize]);
        matches!(
            class.as_str(),
            "CabinetWClass"
                | "ExploreWClass"
                | "Progman"
                | "WorkerW"
                | "Shell_TrayWnd"
                | "XamlExplorerHostIslandWindow"
        )
    };
    let mut mouse_down = false;
    let mut shell_drag = false;
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
                    // Resolve the drag origin once, while the cursor is over the
                    // Desktop/File Explorer source. Foreground HWNDs can change mid-drag.
                    shell_drag = is_shell_window(point);
                }
                let moved = (point.x - anchor.0).abs() + (point.y - anchor.1).abs() >= 9;
                let shift = is_down(VK_SHIFT);
                let control = is_down(VK_CONTROL);
                let alt = is_down(VK_MENU);
                let current_shortcuts = shortcut_store
                    .current
                    .read()
                    .map(|value| value.clone())
                    .unwrap_or_default();
                let requested_mode =
                    requested_wheel_mode(&current_shortcuts, shift, control, alt);

                // The drag origin is captured once; the wheel can then appear anywhere
                // on the virtual desktop even as the foreground HWND changes.
                if moved && !gesture_active && shell_drag {
                    if let Some(mode) = requested_mode {
                        if show_wheel_window(&app, mode).is_ok() {
                            gesture_active = true;
                            active_mode = mode.to_string();
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
                shell_drag = false;
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
fn start_drag_monitor(_app: AppHandle, _shortcut_store: GestureShortcutStore) {}

pub fn run() {
    tauri::Builder::default()
        // Register the native dialog plugin used by the React file/folder pickers.
        .plugin(tauri_plugin_dialog::init())
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
            let shortcuts_path = app.path().app_config_dir()?.join(GESTURE_SHORTCUT_FILE);
            let shortcut_store = GestureShortcutStore {
                current: Arc::new(RwLock::new(load_gesture_shortcuts(&shortcuts_path))),
                path: shortcuts_path,
            };
            app.manage(shortcut_store.clone());
            let wheel_size_path = app.path().app_config_dir()?.join(WHEEL_SIZE_FILE);
            let wheel_size_store = WheelSizeStore {
                current: Arc::new(RwLock::new(load_wheel_size(&wheel_size_path))),
                path: wheel_size_path,
            };
            app.manage(wheel_size_store);

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

            start_drag_monitor(app.handle().clone(), shortcut_store);
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
            get_engine_status,
            execute_engine_operation,
            create_zip,
            convert_images_to_format,
            compress_jpeg_images,
            open_output_folder,
            open_output_parent,
            show_wheel_overlay,
            hide_wheel_overlay,
            get_gesture_shortcuts,
            update_gesture_shortcuts,
            get_wheel_size,
            update_wheel_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running SHIFT");
}


#[cfg(test)]
mod gesture_shortcut_tests {
    use super::*;

    #[test]
    fn default_gestures_match_expected_modes() {
        let shortcuts = GestureShortcuts::default();
        assert_eq!(requested_wheel_mode(&shortcuts, true, false, false), Some("convert"));
        assert_eq!(requested_wheel_mode(&shortcuts, true, false, true), Some("tools"));
        assert_eq!(requested_wheel_mode(&shortcuts, true, true, false), Some("actions"));
        assert_eq!(requested_wheel_mode(&shortcuts, false, false, false), None);
    }

    #[test]
    fn modifier_combinations_must_match_exactly() {
        assert!(!gesture_shortcut_matches("shift", true, true, false));
        assert!(!gesture_shortcut_matches("ctrlShift", true, true, true));
        assert!(gesture_shortcut_matches("ctrlAlt", false, true, true));
    }

    #[test]
    fn wheel_size_is_bounded() {
        assert_eq!(validate_wheel_size(MIN_WHEEL_SIZE).unwrap(), MIN_WHEEL_SIZE);
        assert_eq!(validate_wheel_size(MAX_WHEEL_SIZE).unwrap(), MAX_WHEEL_SIZE);
        assert!(validate_wheel_size(MIN_WHEEL_SIZE - 1).is_err());
        assert!(validate_wheel_size(MAX_WHEEL_SIZE + 1).is_err());
    }

    #[test]
    fn invalid_or_missing_wheel_size_uses_default() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("wheel-size.conf");
        assert_eq!(load_wheel_size(&path), DEFAULT_WHEEL_SIZE);
        fs::write(&path, "9999").unwrap();
        assert_eq!(load_wheel_size(&path), DEFAULT_WHEEL_SIZE);
        fs::write(&path, "440").unwrap();
        assert_eq!(load_wheel_size(&path), 440);
    }

    #[test]
    fn shortcut_modes_must_be_distinct() {
        let shortcuts = GestureShortcuts {
            convert: "shift".into(),
            tools: "shift".into(),
            actions: "ctrlShift".into(),
        };
        assert!(validate_gesture_shortcuts(&shortcuts).is_err());
    }
}
