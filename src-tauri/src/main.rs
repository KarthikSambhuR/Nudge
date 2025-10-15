#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri_plugin_store::StoreExt;
use tokio::time::interval;

const OVERLAY_WINDOW_LABEL: &str = "overlay";
const SETTINGS_WINDOW_LABEL: &str = "settings";
struct TimerState(pub Mutex<Option<tauri::async_runtime::JoinHandle<()>>>);

#[tauri::command]
async fn trigger_overlay(app: AppHandle) -> Result<(), String> {
    println!("✅ RUST: trigger_overlay command received!");
    show_overlay_window(&app).await;
    Ok(())
}

#[tauri::command]
async fn get_overlay_duration(app: AppHandle) -> Result<u64, String> {
    println!("✅ RUST: get_overlay_duration command received!");
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let duration = store
        .get("overlayDurationSeconds")
        .and_then(|v| v.as_u64())
        .unwrap_or(30);
    println!("✅ RUST: Returning duration: {} seconds", duration);
    Ok(duration)
}

#[tauri::command]
async fn open_settings(app: AppHandle) -> Result<(), String> {
    println!("✅ RUST: open_settings command received!");
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        println!("✅ RUST: Settings window exists, showing it");
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        println!("✅ RUST: Creating new settings window");
        let _ = WebviewWindowBuilder::new(&app, SETTINGS_WINDOW_LABEL, WebviewUrl::App("settings.html".into()))
            .title("Nudge Settings")
            .inner_size(600.0, 700.0)
            .resizable(false)
            .center()
            .build();
    }
    Ok(())
}

#[tauri::command]
fn exit_app(app: AppHandle) {
    println!("✅ RUST: exit_app command received!");
    app.exit(0);
}

#[tauri::command]
fn restart_timer(app: AppHandle) {
    println!("✅ RUST: Restarting timer due to settings change.");
    start_break_timer(app);
}


async fn show_overlay_window(app: &AppHandle) {
    println!("🎬 RUST: show_overlay_window called");
    
    if let Some(window) = app.get_webview_window(OVERLAY_WINDOW_LABEL) {
        println!("✅ RUST: Overlay window exists, showing it");
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    println!("🔨 RUST: Creating new overlay window");
    match WebviewWindowBuilder::new(
        app,
        OVERLAY_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .fullscreen(true)
    .decorations(false)
    .skip_taskbar(true)
    .center()
    .build()
    {
        Ok(window) => {
            println!("✅ RUST: Overlay window created successfully");
            
            window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::CloseRequested { .. } => {
                        println!("🚪 RUST: Overlay window close requested");
                    }
                    tauri::WindowEvent::Destroyed => {
                        println!("💥 RUST: Overlay window destroyed");
                    }
                    _ => {}
                }
            });
        }
        Err(e) => {
            println!("❌ RUST: Failed to create overlay window: {}", e);
        }
    }
}

fn start_break_timer(app: AppHandle) {
    println!("⏰ RUST: start_break_timer called");
    let timer_state = app.state::<TimerState>();

    if let Some(handle) = timer_state.0.lock().unwrap().take() {
        println!("🛑 RUST: Stopping existing timer");
        handle.abort();
    }

    let store = app.store("settings.json").expect("Failed to get store");

    let interval_minutes = store
        .get("intervalMinutes")
        .and_then(|v| v.as_u64())
        .unwrap_or(20);
    
    println!("⏰ RUST: Timer started with interval: {} minutes", interval_minutes);

    if interval_minutes == 0 { 
        println!("⚠️ RUST: Timer interval is 0, not starting timer");
        return; 
    }

    let new_handle = tauri::async_runtime::spawn({
        let app = app.clone();
        async move {
            let mut interval = interval(Duration::from_secs(interval_minutes * 60));
            println!("⏱️ RUST: Timer loop started, waiting {} minutes", interval_minutes);
            loop {
                interval.tick().await;
                println!("⏰ RUST: Timer ticked! Showing overlay window");
                show_overlay_window(&app).await;
            }
        }
    });
    
    *timer_state.0.lock().unwrap() = Some(new_handle);
    println!("✅ RUST: Timer handle stored in state");
}

// == Main Application Setup ==
fn main() {
    println!("🚀 RUST: Application starting...");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![])
        ))
        .plugin(tauri_plugin_shell::init())
        .manage(TimerState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            trigger_overlay,
            get_overlay_duration,
            open_settings,
            exit_app,
            restart_timer
        ])
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    let label = window.label();
                    println!("🚪 RUST: Window '{}' close requested", label);
                    
                    if label == "main" {
                        println!("🛡️ RUST: Preventing main window from closing");
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
                tauri::WindowEvent::Destroyed => {
                    println!("💥 RUST: Window '{}' destroyed", window.label());
                }
                _ => {}
            }
        })
        .setup(|app| {
            println!("⚙️ RUST: Running setup...");
            
            if let Some(main_window) = app.get_webview_window("main") {
                println!("👁️ RUST: Hiding main window");
                let _ = main_window.hide();
            }

            let store = app.store("settings.json").expect("Failed to get store");
            
            if store.get("intervalMinutes").is_none() {
                println!("📝 RUST: Setting default intervalMinutes: 20");
                let _ = store.set("intervalMinutes", serde_json::json!(20));
            }
            if store.get("overlayDurationSeconds").is_none() {
                println!("📝 RUST: Setting default overlayDurationSeconds: 30");
                let _ = store.set("overlayDurationSeconds", serde_json::json!(30));
            }
            if store.get("autoStart").is_none() {
                println!("📝 RUST: Setting default autoStart: false");
                let _ = store.set("autoStart", serde_json::json!(false));
            }
            let _ = store.save();

            println!("🔧 RUST: Building tray menu");
            let trigger_item = MenuItem::with_id(app, "trigger", "Start Break Now", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let exit_item = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
            
            let menu = Menu::with_items(app, &[
                &trigger_item,
                &settings_item,
                &exit_item,
            ])?;

            println!("🔧 RUST: Building tray icon");
            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(move |app, event| {
                    println!("📋 RUST: Tray menu event: {}", event.id.as_ref());
                    match event.id.as_ref() {
                        "trigger" => {
                            println!("▶️ RUST: Trigger menu item clicked");
                            let app_clone = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = trigger_overlay(app_clone).await;
                            });
                        }
                        "settings" => {
                            println!("⚙️ RUST: Settings menu item clicked");
                            let app_clone = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = open_settings(app_clone).await;
                            });
                        }
                        "exit" => {
                            println!("🚪 RUST: Exit menu item clicked - shutting down");
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            println!("⏰ RUST: Starting background timer");
            start_break_timer(app.handle().clone());
            
            println!("✅ RUST: Setup complete!");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
    println!("🛑 RUST: Application shut down");
}