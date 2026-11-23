use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_updater::UpdaterExt;

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn spawn_update_check(app: &tauri::AppHandle) {
    let app = app.clone();

    tauri::async_runtime::spawn(async move {
        if let Err(error) = check_and_install_update(app).await {
            eprintln!("Voxly updater: {error}");
        }
    });
}

async fn check_and_install_update(
    app: tauri::AppHandle,
) -> tauri_plugin_updater::Result<()> {
    let updater = app.updater()?;

    let Some(update) = updater.check().await? else {
        return Ok(());
    };

    println!(
        "Voxly update available: {} -> {}",
        update.current_version, update.version
    );

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
        // Register this first so a second launch focuses the existing window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let open_item =
                MenuItem::with_id(app, "open", "Open Voxly", true, None::<&str>)?;
            let updates_item = MenuItem::with_id(
                app,
                "check_updates",
                "Check for Updates",
                true,
                None::<&str>,
            )?;
            let quit_item =
                MenuItem::with_id(app, "quit", "Quit Voxly", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_item, &updates_item, &quit_item])?;

            let icon = app
                .default_window_icon()
                .expect("Voxly application icon is configured")
                .clone();

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("Voxly")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main_window(app),
                    "check_updates" => spawn_update_check(app),
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
            spawn_update_check(app.handle());

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Keep the app alive in the system tray when the window is closed.
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Voxly Desktop");
}
