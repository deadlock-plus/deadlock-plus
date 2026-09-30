mod features;
mod http;

use features::ingest::IngestService;
use features::network::{self, NetworkMonitor};
use features::server_picker::{self, ServerPickerState};
use tauri::{Manager, RunEvent};

/// When the binary was started as the root capture helper, runs it and returns its exit code. Otherwise
/// `None`, and the app should start normally.
#[cfg(unix)]
pub fn capture_helper_exit_code() -> Option<i32> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some(dp_connection::HELPER_ARG) {
        return None;
    }
    Some(args.next().map_or(2, |socket| dp_connection::run_helper(&socket)))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    features::logging::install_panic_hook();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| features::tray::show_main(app)))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(features::window_state::plugin())
        .manage(http::Http::default())
        .manage(ServerPickerState::default())
        .manage(NetworkMonitor::default())
        .manage(IngestService::default())
        .manage(features::diagnostics::frames::capture::FrameCapture::default())
        .manage(features::tray::CloseToTray::default())
        .manage(features::tray::badges::BadgeState::default())
        .manage(features::maintenance::MaintenanceState::default())
        .manage(features::alerts::AlertsState::default())
        .manage(features::notifications::NotificationsState::default())
        .manage(features::patch_notes::PatchNotesState::default())
        .manage(dp_demos::metadata::DemoMetaCache::default())
        .manage(dp_demos::pin::PinStore::default())
        .manage(dp_kv::KvStore::default())
        .manage(features::jobs::JobsState::default())
        .manage(features::diagnostics::scan_job::AddonScanState::default())
        .setup(|app| {
            let log_dir = app.path().app_log_dir()?;
            app.handle().plugin(features::logging::plugin(&log_dir)?)?;
            features::logging::log_startup(app.handle());
            dp_firewall::init(&app.path().app_data_dir()?);
            network::commands::start_monitor(app.handle(), false);
            features::tray::setup(app.handle())?;
            features::maintenance::start(app.handle());
            features::jobs::start(app.handle());
            features::alerts::start(app.handle());
            features::patch_notes::start(app.handle());
            server_picker::start(app.handle());
            Ok(())
        })
        .on_window_event(features::tray::on_window_event)
        .invoke_handler(tauri::generate_handler![
            server_picker::commands::get_game_definitions,
            server_picker::commands::fetch_server_groups,
            server_picker::commands::ping_server_groups,
            server_picker::commands::block_server_groups,
            server_picker::commands::unblock_server_groups,
            server_picker::commands::list_blocked_group_ids,
            server_picker::commands::sync_server_blocks,
            server_picker::commands::firewall_capability,
            server_picker::commands::detect_external_blocks,
            server_picker::commands::import_external_blocks,
            network::commands::start_network_monitor,
            network::commands::network_snapshot,
            network::commands::network_history,
            features::ingest::commands::set_ingest_enabled,
            features::ingest::commands::ingest_status,
            features::steam_account::commands::current_steam_account,
            features::steam_account::commands::local_steam_account_ids,
            features::voice_bans::commands::is_game_running,
            features::voice_bans::commands::read_voice_ban,
            features::voice_bans::commands::write_voice_ban,
            features::demos::commands::list_demos,
            features::demos::commands::reveal_demo,
            features::demos::commands::open_replays_dir,
            features::demos::commands::delete_preview,
            features::demos::commands::delete_demos,
            features::demos::metadata::commands::demo_metadata,
            features::demos::pin::commands::list_pinned,
            features::demos::cleanup::commands::list_cleanup_rules,
            features::demos::cleanup::commands::save_cleanup_rules,
            features::demos::cleanup::commands::cleanup_matches,
            features::demos::pin::commands::set_pinned,
            features::storage::commands::storage_entries,
            features::storage::commands::storage_entry_stats,
            features::storage::commands::storage_reveal,
            features::storage::commands::storage_clear,
            features::about::commands::app_info,
            features::about::commands::changelog,
            features::logging::commands::read_logs,
            features::logging::commands::export_logs,
            features::logging::commands::open_log_dir,
            features::autostart::commands::autostart_status,
            features::autostart::commands::set_autostart,
            features::tray::commands::set_close_to_tray,
            features::tray::commands::frontend_ready,
            features::tray::badges::commands::set_update_badge,
            features::maintenance::commands::set_maintenance_schedule,
            features::maintenance::commands::next_maintenance,
            features::alerts::commands::set_alerts_enabled,
            features::alerts::commands::list_alerts,
            features::alerts::commands::refresh_alerts,
            features::alerts::commands::mark_alerts_read,
            features::notifications::commands::list_notifications,
            features::notifications::commands::mark_notifications_read,
            features::notifications::commands::mark_notification_read,
            features::patch_notes::commands::search_patch_notes,
            features::jobs::commands::jobs_snapshot,
            features::jobs::commands::cancel_job,
            features::jobs::commands::set_job_policy,
            features::jobs::commands::set_pause_in_game,
            features::jobs::commands::set_job_enabled,
            features::jobs::commands::set_all_jobs_enabled,
            features::jobs::commands::force_run_job,
            features::patch_notes::commands::get_patch_notes,
            features::kv::commands::kv_get,
            features::kv::commands::kv_set,
            features::kv::commands::kv_delete,
            features::export::commands::save_text_file,
            features::diagnostics::commands::start_addon_scan,
            features::diagnostics::commands::addon_scan_report,
            features::diagnostics::frames::commands::start_frame_capture,
            features::diagnostics::frames::commands::frame_capture_status,
            features::diagnostics::frames::commands::stop_frame_capture,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            log::info!("Deadlock+ exiting");
            handle.state::<NetworkMonitor>().stop();
            handle.state::<IngestService>().stop();
            handle.state::<features::diagnostics::frames::capture::FrameCapture>().stop();
        }
    });
}
