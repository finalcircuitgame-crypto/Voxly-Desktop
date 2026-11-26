mod game_detection;

use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::{PermissionKind, PermissionResponse},
    Manager, WindowEvent,
};
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;

struct UnreadMenuItem(MenuItem<tauri::Wry>);
struct GameMenuItem(MenuItem<tauri::Wry>);

#[tauri::command]
fn get_foreground_game() -> Option<String> {
    game_detection::get_foreground_game()
}

fn spawn_game_detection(app: &tauri::AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let mut last_game: Option<String> = None;
        loop {
            let game = game_detection::get_foreground_game();
            if game != last_game {
                let label = game
                    .as_ref()
                    .map(|name| format!("🎮 Playing: {name}"))
                    .unwrap_or_else(|| "Playing: Not detected".to_string());
                if let Some(item) = app.try_state::<GameMenuItem>() {
                    let _ = item.0.set_text(&label);
                }
                last_game = game;
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

#[tauri::command]
fn set_unread_count(app: tauri::AppHandle, count: u32) -> Result<(), String> {
    let label = if count == 0 { "Unread: 0".to_string() } else { format!("Unread: {}", count.min(999)) };
    app.state::<UnreadMenuItem>().0.set_text(&label).map_err(|error| error.to_string())?;
    if let Some(tray) = app.tray_by_id("main") {
        let tooltip = if count == 0 { "Voxly".to_string() } else { format!("Voxly — {} unread", count) };
        tray.set_tooltip(Some(tooltip)).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn main_window_is_visible(app: &tauri::AppHandle) -> bool {
    app.get_webview_window("main")
        .map(|window| {
            let visible = window.is_visible().unwrap_or(true);
            let minimized = window.is_minimized().unwrap_or(false);
            visible && !minimized
        })
        .unwrap_or(true)
}

#[tauri::command]
fn is_main_window_visible(app: tauri::AppHandle) -> bool {
    main_window_is_visible(&app)
}

#[tauri::command]
fn send_desktop_notification(
    app: tauri::AppHandle,
    title: String,
    body: Option<String>,
    only_if_hidden: Option<bool>,
) -> Result<bool, String> {
    // Visibility must come from the native window: a Tauri window hidden to
    // the tray can still report document.visibilityState as "visible".
    if only_if_hidden.unwrap_or(false) && main_window_is_visible(&app) {
        return Ok(false);
    }

    let mut builder = app.notification().builder().title(title);
    if let Some(body) = body {
        builder = builder.body(body);
    }
    builder.show()
        .map(|_| true)
        .map_err(|error| error.to_string())
}

fn open_route(app: &tauri::AppHandle, route: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let target = format!("https://voxly-vc.vercel.app{}", route);
        let script = format!("window.location.assign({})", serde_json::to_string(&target).unwrap());
        let _ = window.eval(&script);
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn reload_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.eval("window.location.reload()");
        show_main_window(app);
    }
}

fn show_update_dialog(app: &tauri::AppHandle, title: &str, body: &str) {
    app.dialog()
        .message(body)
        .title(title)
        .show(|_| {});
}

fn notify(app: &tauri::AppHandle, title: &str, body: &str) {
    if let Err(error) = app.notification().builder().title(title).body(body).show() {
        eprintln!("Voxly notification failed: {error}");
        show_update_dialog(
            app,
            title,
            &format!("{body}

Windows notification error: {error}"),
        );
    }
}

fn spawn_update_check(app: &tauri::AppHandle, notify_if_current: bool) {
    let app = app.clone();

    tauri::async_runtime::spawn(async move {
        if let Err(error) = check_and_install_update(app.clone(), notify_if_current).await {
            eprintln!("Voxly updater: {error}");
            if notify_if_current {
                show_update_dialog(
                    &app,
                    "Voxly update check failed",
                    &format!("Could not check for updates: {error}"),
                );
            }
        }
    });
}

async fn check_and_install_update(
    app: tauri::AppHandle,
    notify_if_current: bool,
) -> tauri_plugin_updater::Result<()> {
    let updater = app.updater()?;

    let Some(update) = updater.check().await? else {
        if notify_if_current {
            show_update_dialog(&app, "Voxly is up to date", "You're running the latest available version.");
        }
        return Ok(());
    };

    println!(
        "Voxly update available: {} -> {}",
        update.current_version, update.version
    );
    if notify_if_current {
        show_update_dialog(
            &app,
            "Voxly update available",
            &format!("Downloading version {}. Voxly will restart when the update is ready.", update.version),
        );
    } else {
        notify(
            &app,
            "Voxly update available",
            &format!("Downloading version {}. Voxly will restart when the update is ready.", update.version),
        );
    }

    update
        .download_and_install(
            |chunk_length, content_length| {
                println!(
                    "Voxly update download: +{chunk_length} bytes (total: {content_length:?})"
                );
            },
            || println!("Voxly update download finished"),
        )
        .await?;
    // On Windows, the updater closes the running process and restarts it.
    app.restart();
}

pub fn run() {
    tauri::Builder::default()
        // Windows WebView2 requires the host app to decide media-capture
        // permissions. Only trust the production Voxly origin.
        .on_permission_request(|webview, kind| {
            let trusted_origin = webview.url().map(|url| {
                url.scheme() == "https"
                    && url.host_str() == Some("voxly-vc.vercel.app")
                    && url.port().is_none()
            }).unwrap_or(false);

            // Do not silently grant microphone or camera access. Returning
            // Default lets WebView2 show its normal permission prompt and
            // remember the user's choice for this origin.
            let response = if trusted_origin {
                PermissionResponse::Default
            } else {
                PermissionResponse::Deny
            };

            eprintln!(
                "[Voxly permissions] {:?} request from trusted_origin={}: {:?}",
                kind, trusted_origin, response
            );
            response
        })
        // Register this first so a second launch focuses the existing window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let open_item =
                MenuItem::with_id(app, "open", "Open Voxly", true, None::<&str>)?;
            let messages_item = MenuItem::with_id(app, "messages", "Open Messages", true, None::<&str>)?;
            let calls_item = MenuItem::with_id(app, "calls", "Open Calls", true, None::<&str>)?;
            let groups_item = MenuItem::with_id(app, "groups", "Open Groups", true, None::<&str>)?;
            let unread_item = MenuItem::with_id(app, "unread_count", "Unread: 0", false, None::<&str>)?;
            let game_item = MenuItem::with_id(app, "game_activity", "Playing: Not detected", false, None::<&str>)?;
            let reload_item = MenuItem::with_id(app, "reload", "Reload Voxly", true, None::<&str>)?;
            let test_notification_item = MenuItem::with_id(
                app,
                "test_notification",
                "Send Test Notification",
                true,
                None::<&str>,
            )?;
            let updates_item = MenuItem::with_id(
                app,
                "check_updates",
                "Check for Updates",
                true,
                None::<&str>,
            )?;
            app.manage(UnreadMenuItem(unread_item.clone()));
            app.manage(GameMenuItem(game_item.clone()));
            let quit_item =
                MenuItem::with_id(app, "quit", "Quit Voxly", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[
                &open_item,
                &messages_item,
                &calls_item,
                &groups_item,
                &unread_item,
                &game_item,
                &reload_item,
                &test_notification_item,
                &updates_item,
                &quit_item,
            ])?;

            let icon = app
                .default_window_icon()
                .expect("Voxly application icon is configured")
                .clone();

            let _tray = TrayIconBuilder::with_id("main")
                .icon(icon)
                .menu(&menu)
                .tooltip("Voxly")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main_window(app),
                    "messages" => open_route(app, "/chat"),
                    "calls" => open_route(app, "/call"),
                    "groups" => open_route(app, "/dashboard?view=groups"),
                    "reload" => reload_main_window(app),
                    "test_notification" => notify(
                        app,
                        "Voxly test notification",
                        "Desktop notifications are working.",
                    ),
                    "check_updates" => spawn_update_check(app, true),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Check at launch in the background. Network/update failures do not
            // prevent Voxly from opening; users can also check from the tray.
            spawn_update_check(app.handle(), false);
            spawn_game_detection(app.handle());

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Keep the app alive in the system tray when the window is closed.
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![set_unread_count, is_main_window_visible, send_desktop_notification, get_foreground_game])
        .run(tauri::generate_context!())
        .expect("error while running Voxly Desktop");
}
