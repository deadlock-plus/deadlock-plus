use std::sync::atomic::{AtomicBool, Ordering};

pub mod badges;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WindowEvent};

const MAIN_WINDOW: &str = "main";
const AUTOSTART_ARG: &str = "--autostart";
const SHOW_ID: &str = "show";
const QUIT_ID: &str = "quit";

/// Whether closing the window hides it (the watchers keep running) instead of exiting.
#[derive(Default)]
pub struct CloseToTray(AtomicBool);

impl CloseToTray {
    pub fn set(&self, enabled: bool) {
        self.0.store(enabled, Ordering::Relaxed);
    }

    pub fn get(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// The autostart task passes `--autostart`; that launch stays in the tray until the user opens it.
pub fn launched_hidden<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter().any(|a| a.as_ref() == AUTOSTART_ARG)
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN_WINDOW) {
        log::debug!("showing the main window");
        for (step, result) in [("unminimize", w.unminimize()), ("show", w.show()), ("focus", w.set_focus())] {
            if let Err(e) = result {
                log::warn!("main window {step} failed: {e}");
            }
        }
    }
}

/// The tray icon is always created, even with close-to-tray off: an autostart launch has no window,
/// so it is the only way back in.
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, SHOW_ID, "Show Deadlock+", true, None::<&str>)?,
            &MenuItem::with_id(app, QUIT_ID, "Quit", true, None::<&str>)?,
        ],
    )?;
    let mut tray = TrayIconBuilder::new()
        .tooltip("Deadlock+")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            SHOW_ID => show_main(app),
            QUIT_ID => {
                log::info!("quit requested from the tray");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    let tray_icon = tray.build(app)?;
    app.manage(tray_icon);
    badges::setup(app);

    if launched_hidden(std::env::args()) {
        log::info!("autostart launch, staying in the tray");
    }
    Ok(())
}

pub fn on_window_event(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        if window.label() == MAIN_WINDOW && window.state::<CloseToTray>().get() {
            api.prevent_close();
            log::debug!("close requested, hiding to the tray");
            if let Err(e) = window.hide() {
                log::warn!("could not hide the window: {e}");
            }
        }
    }
}

pub mod commands {
    use super::*;

    #[tauri::command]
    pub fn set_close_to_tray(enabled: bool, state: tauri::State<'_, CloseToTray>) {
        state.set(enabled);
        log::info!("close to tray {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Called once the frontend has actually rendered something, so the window is only ever
    /// revealed with real content in it. WebView2 has a known slow/flashing cold start; showing
    /// the window early and painting `backgroundColor` over the gap just moves the flash from
    /// white to that color instead of removing it. Staying hidden until now, with the window built
    /// `"visible": false` in `tauri.conf.json`, means the user never sees the empty state at all.
    #[tauri::command]
    pub fn frontend_ready(app: AppHandle) {
        if !launched_hidden(std::env::args()) {
            show_main(&app);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autostart_arg_means_hidden_launch() {
        assert!(launched_hidden(["deadlock-plus.exe", "--autostart"]));
    }

    #[test]
    fn normal_launch_is_visible() {
        assert!(!launched_hidden(["deadlock-plus.exe"]));
        assert!(!launched_hidden(["deadlock-plus.exe", "--autostart-not"]));
    }

    #[test]
    fn close_to_tray_defaults_off_and_toggles() {
        let s = CloseToTray::default();
        assert!(!s.get());
        s.set(true);
        assert!(s.get());
        s.set(false);
        assert!(!s.get());
    }
}
