mod platforms;

use platforms::TrendingData;

use tauri::image::Image;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

#[tauri::command]
async fn fetch_trending(
    youtube_key: Option<String>,
) -> Result<TrendingData, String> {
    platforms::fetch_all(
        youtube_key.as_deref(),
    )
    .await
    .map_err(|e| format!("{e:#}"))
}

fn toggle_window(window: &tauri::WebviewWindow) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![fetch_trending])
        .setup(|app| {
            // Load .env for API keys
            let _ = dotenvy::dotenv();

            // Set activation policy early — before tray/window setup
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            let window = app
                .get_webview_window("main")
                .expect("Window 'main' not found — check tauri.conf.json label");

            // Neumorphic design uses opaque light background — no vibrancy needed
            // Window is transparent with shadow; CSS provides the light surface

            // Auto-hide when window loses focus (click outside)
            let window_focus = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = window_focus.hide();
                }
            });

            let tray_icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
                .expect("Failed to load tray icon");

            let window_clone = window.clone();
            TrayIconBuilder::new()
                .icon(tray_icon)
                .icon_as_template(true)
                .tooltip("Light Trend")
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window(&window_clone);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
